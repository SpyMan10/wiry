//! One connection to PipeWire: owns the main loop, context, core and registry, announces the
//! graph to the listeners and executes commands until a resync is requested or the connection
//! is lost.

use std::cell::RefCell;
use std::collections::{
  HashMap,
  HashSet
};
use std::rc::{
  Rc,
  Weak
};
use std::sync::Arc;
use std::time::Duration;

use pipewire as pw;
use pw::channel::{
  AttachedReceiver,
  Receiver
};
use pw::context::ContextRc;
use pw::core::CoreRc;
use pw::device::{
  Device,
  DeviceListener
};
use pw::link::Link as LinkProxy;
use pw::loop_::TimerSource;
use pw::main_loop::{
  MainLoopRc,
  MainLoopWeak
};
use pw::node::{
  Node as NodeProxy,
  NodeChangeMask,
  NodeListener
};
use pw::properties::properties;
use pw::registry::{
  GlobalObject,
  RegistryRc,
  RegistryWeak
};
use pw::spa::param::ParamType;
use pw::spa::utils::dict::DictRef;
use pw::types::ObjectType;
use tracing::{
  debug,
  info,
  trace,
  warn
};

use super::command::Command;
use super::event::{
  Event,
  Listeners
};
use super::meter::{
  self,
  Meter,
  Peaks
};
use super::parse;
use super::types::{
  Node,
  Volume
};
use super::volume::{
  self,
  Change,
  Routes,
  Target
};

/// Period of the `Levels` event.
const LEVELS_PERIOD: Duration = Duration::from_millis(33);

/// Why a session ended.
pub(super) enum Exit {
  Resync,
  /// Connection lost, with PipeWire's error message if it sent one.
  Disconnected(Option<String>)
}

/// Kind of an announced object, to send the right "removed" event (removals only carry an id).
enum Kind {
  Node,
  Port,
  Link
}

struct BoundNode {
  proxy: NodeProxy,
  _listener: NodeListener
}

struct BoundDevice {
  device: Device,
  routes: Routes,
  _listener: DeviceListener
}

/// What the session needs to know about the graph to do its job (not the UI's graph: that one
/// is built by the listeners).
#[derive(Default)]
struct State {
  kinds: HashMap<u32, Kind>,
  /// Our own meter nodes: neither they nor their ports and links are announced.
  hidden: HashSet<u32>,
  /// Port -> node, to create links from port ids.
  port_nodes: HashMap<u32, u32>,
  /// Last announced version of each node, to only send real changes.
  nodes: HashMap<u32, Node>,
  volumes: HashMap<u32, Volume>,
  /// `(device id, route device)` of hardware nodes.
  node_devices: HashMap<u32, (u32, i32)>,
  bound_nodes: HashMap<u32, BoundNode>,
  devices: HashMap<u32, BoundDevice>,
  meters: HashMap<u32, Meter>,
  /// Links we created; their proxies are kept so the server doesn't tear them down.
  created_links: Vec<LinkProxy>,
  exit: Option<Exit>
}

pub(super) struct Session {
  main_loop: MainLoopRc,
  _context: ContextRc,
  core: CoreRc,
  registry: RegistryRc,
  listeners: Arc<Listeners>,
  state: Rc<RefCell<State>>,
  peaks: Peaks
}

impl Session {
  fn connect(listeners: Arc<Listeners>) -> Result<Self, pw::Error> {
    let main_loop = MainLoopRc::new(None)?;
    let context = ContextRc::new(&main_loop, None)?;
    let core = context.connect_rc(None)?;
    let registry = core.get_registry_rc()?;
    Ok(Self {
      main_loop,
      _context: context,
      core,
      registry,
      listeners,
      state: Default::default(),
      peaks: Default::default()
    })
  }

  /// Runs one session to its end. The command receiver is handed back for the next session.
  pub(super) fn run(
    listeners: &Arc<Listeners>,
    receiver: Receiver<Command>
  ) -> (Receiver<Command>, Result<Exit, pw::Error>) {
    info!("connecting to PipeWire");
    let session = match Self::connect(listeners.clone()) {
      Ok(session) => session,
      Err(e) => return (receiver, Err(e))
    };
    session.listeners.emit(Event::Connected);

    let _core_listener = session.watch_core();
    let _registry_listener = session.watch_registry();
    let _levels_timer = session.emit_levels();
    let commands = session.handle_commands(receiver);

    session.main_loop.run();

    let exit = session
      .state
      .borrow_mut()
      .exit
      .take()
      .unwrap_or(Exit::Disconnected(None));
    match &exit {
      Exit::Resync => info!("session ended: resync requested"),
      Exit::Disconnected(error) => warn!(?error, "session ended: connection lost")
    }
    (commands.deattach(), Ok(exit))
  }

  /// What the registry/node/command callbacks need; holds the state weakly so that the
  /// listeners stored in the state don't keep it alive.
  fn ctx(&self) -> Ctx {
    Ctx {
      core: self.core.clone(),
      registry: self.registry.downgrade(),
      main_loop: self.main_loop.downgrade(),
      listeners: self.listeners.clone(),
      state: Rc::downgrade(&self.state),
      peaks: self.peaks.clone()
    }
  }

  /// A fatal core error ends the session.
  fn watch_core(&self) -> pw::core::Listener {
    let ctx = self.ctx();
    self
      .core
      .add_listener_local()
      .error(move |id, _seq, res, message| {
        warn!(id, res, message, "PipeWire error");
        if id == pw::core::PW_ID_CORE {
          ctx.quit(Exit::Disconnected(Some(format!("PipeWire: {message}"))));
        }
      })
      .register()
  }

  fn watch_registry(&self) -> pw::registry::Listener {
    let on_global = self.ctx();
    let on_remove = self.ctx();
    self
      .registry
      .add_listener_local()
      .global(move |obj| on_global.add(obj))
      .global_remove(move |id| on_remove.remove(id))
      .register()
  }

  /// Sends the peak of each meter since the previous frame, every LEVELS_PERIOD.
  fn emit_levels(&self) -> TimerSource<'_> {
    let peaks = self.peaks.clone();
    let listeners = self.listeners.clone();
    let timer = self.main_loop.loop_().add_timer(move |_| {
      let frame: HashMap<u32, f32> = peaks
        .borrow_mut()
        .iter_mut()
        .map(|(id, v)| (*id, std::mem::take(v)))
        .collect();
      listeners.emit(Event::Levels(&frame));
    });
    timer.update_timer(Some(LEVELS_PERIOD), Some(LEVELS_PERIOD));
    timer
  }

  fn handle_commands(&self, receiver: Receiver<Command>) -> AttachedReceiver<'_, Command> {
    let ctx = self.ctx();
    receiver.attach(self.main_loop.loop_(), move |cmd| ctx.execute(cmd))
  }
}

struct Ctx {
  core: CoreRc,
  registry: RegistryWeak,
  main_loop: MainLoopWeak,
  listeners: Arc<Listeners>,
  state: Weak<RefCell<State>>,
  peaks: Peaks
}

impl Ctx {
  fn quit(&self, exit: Exit) {
    if let Some(state) = self.state.upgrade() {
      state.borrow_mut().exit = Some(exit);
    }
    if let Some(main_loop) = self.main_loop.upgrade() {
      main_loop.quit();
    }
  }

  // ---- Registry ----

  fn add(&self, obj: &GlobalObject<&DictRef>) {
    let Some(props) = obj.props else { return };
    match obj.type_ {
      ObjectType::Node => self.add_node(obj, props),
      ObjectType::Device => self.add_device(obj),
      ObjectType::Port => self.add_port(obj.id, props),
      ObjectType::Link => self.add_link(obj.id, props),
      _ => {}
    }
  }

  fn add_node(&self, obj: &GlobalObject<&DictRef>, props: &DictRef) {
    let Some(state) = self.state.upgrade() else { return };
    if meter::is_meter(props.get("node.name")) {
      state.borrow_mut().hidden.insert(obj.id);
      return;
    }
    let node = parse::node(obj.id, props);
    {
      let mut s = state.borrow_mut();
      s.kinds.insert(obj.id, Kind::Node);
      s.nodes.insert(obj.id, node.clone());
      if let Some(device) = parse::node_device(props) {
        s.node_devices.insert(obj.id, device);
      }
    }
    self.listeners.emit(Event::NodeAdded(&node));

    // Every node is bound for its full properties; audio ones also for their volume.
    if let Some(registry) = self.registry.upgrade()
      && let Ok(proxy) = registry.bind::<NodeProxy, _>(obj)
    {
      let listener = self.watch_node(&proxy, obj.id);
      if node.media_class.contains("Audio") {
        proxy.subscribe_params(&[ParamType::Props]);
      }
      state.borrow_mut().bound_nodes.insert(
        obj.id,
        BoundNode {
          proxy,
          _listener: listener
        }
      );
    }

    if let Some((capture_sink, serial)) = meter::target(&node.media_class).zip(props.get("object.serial")) {
      match Meter::attach(&self.core, obj.id, serial, capture_sink, &self.peaks) {
        Ok(m) => {
          state.borrow_mut().meters.insert(obj.id, m);
        }
        Err(e) => warn!(node = obj.id, error = %e, "cannot attach level meter")
      }
    }
  }

  /// Full properties (registry globals only carry a subset) and volume of a node.
  fn watch_node(&self, proxy: &NodeProxy, id: u32) -> NodeListener {
    let (state_info, listeners_info) = (self.state.clone(), self.listeners.clone());
    let (state_param, listeners_param) = (self.state.clone(), self.listeners.clone());
    proxy
      .add_listener_local()
      .info(move |info| {
        // Info events also come for state changes (running, idle…) with empty properties:
        // only read them when they were actually sent.
        if !info.change_mask().contains(NodeChangeMask::PROPS) {
          return;
        }
        let (Some(props), Some(state)) = (info.props(), state_info.upgrade()) else {
          return
        };
        let node = parse::node(id, props);
        let changed = {
          let mut s = state.borrow_mut();
          if let Some(device) = parse::node_device(props) {
            s.node_devices.insert(id, device);
          }
          // Only nodes still known (not removed meanwhile) whose properties really changed.
          let changed = s.nodes.get(&id).is_some_and(|previous| *previous != node);
          if changed {
            s.nodes.insert(id, node.clone());
          }
          changed
        };
        if changed {
          listeners_info.emit(Event::NodeChanged(&node));
        }
      })
      .param(move |_seq, param_type, _index, _next, param| {
        if param_type != ParamType::Props {
          return;
        }
        let (Some(param), Some(state)) = (param, state_param.upgrade()) else {
          return
        };
        let volume = {
          let mut s = state.borrow_mut();
          let Some(volume) = volume::parse_props(param, s.volumes.get(&id)) else {
            return
          };
          if s.volumes.get(&id) == Some(&volume) {
            return;
          }
          s.volumes.insert(id, volume.clone());
          volume
        };
        listeners_param.emit(Event::VolumeChanged(id, &volume));
      })
      .register()
  }

  fn add_device(&self, obj: &GlobalObject<&DictRef>) {
    let (Some(state), Some(registry)) = (self.state.upgrade(), self.registry.upgrade()) else {
      return
    };
    let Ok(device) = registry.bind::<Device, _>(obj) else {
      return
    };
    let routes = Routes::default();
    let listener = volume::watch_routes(&device, &routes);
    device.subscribe_params(&[ParamType::Route]);
    state.borrow_mut().devices.insert(
      obj.id,
      BoundDevice {
        device,
        routes,
        _listener: listener
      }
    );
  }

  fn add_port(&self, id: u32, props: &DictRef) {
    let (Some(state), Some(port)) = (self.state.upgrade(), parse::port(id, props)) else {
      return
    };
    {
      let mut s = state.borrow_mut();
      if s.hidden.contains(&port.node_id) {
        return;
      }
      s.kinds.insert(id, Kind::Port);
      s.port_nodes.insert(id, port.node_id);
    }
    self.listeners.emit(Event::PortAdded(&port));
  }

  fn add_link(&self, id: u32, props: &DictRef) {
    let (Some(state), Some(link)) = (self.state.upgrade(), parse::link(id, props)) else {
      return
    };
    {
      let mut s = state.borrow_mut();
      if s.hidden.contains(&link.output_node) || s.hidden.contains(&link.input_node) {
        return;
      }
      s.kinds.insert(id, Kind::Link);
    }
    self.listeners.emit(Event::LinkAdded(&link));
  }

  fn remove(&self, id: u32) {
    let Some(state) = self.state.upgrade() else { return };
    let kind = {
      let mut s = state.borrow_mut();
      s.hidden.remove(&id);
      s.port_nodes.remove(&id);
      s.nodes.remove(&id);
      s.volumes.remove(&id);
      s.node_devices.remove(&id);
      s.bound_nodes.remove(&id);
      s.devices.remove(&id);
      s.meters.remove(&id);
      s.kinds.remove(&id)
    };
    self.peaks.borrow_mut().remove(&id);
    match kind {
      Some(Kind::Node) => self.listeners.emit(Event::NodeRemoved(id)),
      Some(Kind::Port) => self.listeners.emit(Event::PortRemoved(id)),
      Some(Kind::Link) => self.listeners.emit(Event::LinkRemoved(id)),
      None => {}
    }
  }

  // ---- Commands ----

  fn execute(&self, cmd: Command) {
    debug!(?cmd, "executing command");
    match cmd {
      Command::Link {
        output_port,
        input_port
      } => self.link(output_port, input_port),
      Command::Unlink { link_id } => {
        if let Some(registry) = self.registry.upgrade() {
          registry.destroy_global(link_id);
        }
      }
      Command::SetVolume { node_id, channels } => self.change(node_id, Change::Volume(channels)),
      Command::SetMute { node_id, mute } => self.change(node_id, Change::Mute(mute)),
      Command::Resync => self.quit(Exit::Resync)
    }
  }

  fn link(&self, output_port: u32, input_port: u32) {
    let Some(state) = self.state.upgrade() else { return };
    let nodes = {
      let s = state.borrow();
      s.port_nodes
        .get(&output_port)
        .copied()
        .zip(s.port_nodes.get(&input_port).copied())
    };
    let Some((out_node, in_node)) = nodes else {
      warn!(output_port, input_port, "link: unknown port");
      return;
    };
    let props = properties! {
        "link.output.node" => out_node.to_string(),
        "link.output.port" => output_port.to_string(),
        "link.input.node" => in_node.to_string(),
        "link.input.port" => input_port.to_string(),
        "object.linger" => "true",
    };
    match self.core.create_object::<LinkProxy>("link-factory", &props) {
      Ok(link) => {
        debug!(out_node, output_port, in_node, input_port, "link requested");
        state.borrow_mut().created_links.push(link)
      }
      Err(e) => warn!(output_port, input_port, error = %e, "link: failed to create link")
    }
  }

  /// Volume / mute change, through the device route for hardware nodes.
  fn change(&self, node_id: u32, change: Change) {
    let Some(state) = self.state.upgrade() else { return };
    let s = state.borrow();
    let route = s.node_devices.get(&node_id).and_then(|&(device_id, route_device)| {
      let device = s.devices.get(&device_id)?;
      let route_index = *device.routes.borrow().get(&route_device)?;
      Some(Target::Route {
        device: &device.device,
        route_index,
        route_device
      })
    });
    let target = match (route, s.bound_nodes.get(&node_id)) {
      (Some(route), _) => {
        trace!(node_id, "volume change through the device route");
        route
      }
      (None, Some(bound)) => {
        trace!(node_id, "volume change on the node");
        Target::Node(&bound.proxy)
      }
      (None, None) => {
        warn!(node_id, "volume: node is not bound");
        return;
      }
    };
    volume::apply(target, change);
  }
}
