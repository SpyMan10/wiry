//! PipeWire backend: runs the PipeWire main loop on a dedicated thread, mirrors the
//! graph (nodes, ports, links, volumes) into a shared snapshot and executes commands
//! coming from the UI (link / unlink / volume / mute).

use std::cell::RefCell;
use std::collections::{
  BTreeMap,
  HashMap,
  HashSet
};
use std::io::Cursor;
use std::rc::Rc;
use std::sync::atomic::{
  AtomicBool,
  Ordering
};
use std::sync::{
  Arc,
  Mutex
};
use std::thread;
use std::time::Duration;

use pipewire as pw;
use pw::device::{
  Device,
  DeviceListener
};
use pw::link::Link;
use pw::node::{
  Node,
  NodeListener
};
use pw::properties::properties;
use pw::spa::param::ParamType;
use pw::spa::param::audio::{
  AudioFormat,
  AudioInfoRaw
};
use pw::spa::pod::deserialize::PodDeserializer;
use pw::spa::pod::serialize::PodSerializer;
use pw::spa::pod::{
  Object,
  Pod,
  Property,
  Value,
  ValueArray
};
use pw::spa::utils::dict::DictRef;
use pw::spa::utils::{
  Direction,
  SpaTypes
};
use pw::spa::{
  self
};
use pw::stream::{
  StreamFlags,
  StreamListener,
  StreamRc
};
use pw::types::ObjectType;
use serde::Serialize;
use tauri::{
  AppHandle,
  Emitter
};

const SPA_TYPE_OBJECT_PROPS: u32 = spa::sys::SPA_TYPE_OBJECT_Props;
const SPA_PROP_MUTE: u32 = spa::sys::SPA_PROP_mute;
const SPA_PROP_CHANNEL_VOLUMES: u32 = spa::sys::SPA_PROP_channelVolumes;
const SPA_PROP_CHANNEL_MAP: u32 = spa::sys::SPA_PROP_channelMap;
const SPA_TYPE_OBJECT_PARAM_ROUTE: u32 = spa::sys::SPA_TYPE_OBJECT_ParamRoute;
const SPA_PARAM_ROUTE_INDEX: u32 = spa::sys::SPA_PARAM_ROUTE_index;
const SPA_PARAM_ROUTE_DEVICE: u32 = spa::sys::SPA_PARAM_ROUTE_device;
const SPA_PARAM_ROUTE_PROPS: u32 = spa::sys::SPA_PARAM_ROUTE_props;
const SPA_PARAM_ROUTE_SAVE: u32 = spa::sys::SPA_PARAM_ROUTE_save;

/// Name prefix of our own level-meter streams; they are hidden from the graph.
const METER_PREFIX: &str = "wiry-meter-";

/// Peak level per node since the last emission (linear, 0..1+).
type Levels = Arc<Mutex<HashMap<u32, f32>>>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PwVolume {
  /// Linear per-channel volumes (as stored by PipeWire).
  pub channels: Vec<f32>,
  pub channel_map: Vec<String>,
  pub mute: bool
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PwNode {
  pub id: u32,
  pub name: String,
  pub description: String,
  pub media_class: String,
  pub app_name: Option<String>,
  pub icon_name: Option<String>,
  pub volume: Option<PwVolume>,
  /// `(device.id, card.profile.device)` for nodes backed by a hardware device.
  /// Their volume lives on the device's active Route, like wpctl/pavucontrol do.
  #[serde(skip)]
  pub device: Option<(u32, i32)>,
  /// ALSA card backing this node (hardware nodes only).
  pub alsa_card: Option<u32>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PortDirection {
  In,
  Out
}

impl PortDirection {
  /// Parses PipeWire's `port.direction` property.
  fn from_prop(value: &str) -> Option<Self> {
    match value {
      "in" => Some(Self::In),
      "out" => Some(Self::Out),
      _ => None
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
  Audio,
  Midi,
  Video,
  Other
}

impl MediaType {
  /// Derives the media type from a port's `format.dsp` property (e.g. "32 bit float mono audio").
  fn from_dsp_format(format: &str) -> Self {
    if format.contains("audio") {
      Self::Audio
    } else if format.contains("midi") {
      Self::Midi
    } else if format.contains("video") {
      Self::Video
    } else {
      Self::Other
    }
  }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PwPort {
  pub id: u32,
  pub node_id: u32,
  pub index: u32,
  pub name: String,
  pub alias: Option<String>,
  pub direction: PortDirection,
  pub media_type: MediaType,
  pub channel: Option<String>,
  pub monitor: bool,
  pub physical: bool
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PwLink {
  pub id: u32,
  pub output_node: u32,
  pub output_port: u32,
  pub input_node: u32,
  pub input_port: u32
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphSnapshot {
  pub connected: bool,
  pub error: Option<String>,
  pub nodes: Vec<PwNode>,
  pub ports: Vec<PwPort>,
  pub links: Vec<PwLink>
}

#[derive(Default)]
struct Graph {
  connected: bool,
  error: Option<String>,
  nodes: BTreeMap<u32, PwNode>,
  ports: BTreeMap<u32, PwPort>,
  links: BTreeMap<u32, PwLink>,
  /// Our own meter nodes, filtered out of the graph with their ports and links.
  hidden: HashSet<u32>,
  /// Full property set of each node (kept out of the snapshot, fetched on demand).
  node_props: HashMap<u32, BTreeMap<String, String>>
}

impl Graph {
  fn snapshot(&self) -> GraphSnapshot {
    GraphSnapshot {
      connected: self.connected,
      error: self.error.clone(),
      nodes: self.nodes.values().cloned().collect(),
      ports: self.ports.values().cloned().collect(),
      links: self.links.values().cloned().collect()
    }
  }
}

#[derive(Debug)]
pub enum Command {
  Link { output_port: u32, input_port: u32 },
  Unlink { link_id: u32 },
  SetVolume { node_id: u32, channels: Vec<f32> },
  SetMute { node_id: u32, mute: bool }
}

/// Handle stored in Tauri state.
pub struct PwHandle {
  graph: Arc<Mutex<Graph>>,
  sender: Mutex<Option<pw::channel::Sender<Command>>>
}

impl PwHandle {
  pub fn snapshot(&self) -> GraphSnapshot {
    self.graph.lock().unwrap().snapshot()
  }

  /// All PipeWire properties of a node (from its info, or its registry global until then).
  pub fn node_props(&self, node_id: u32) -> Option<BTreeMap<String, String>> {
    self.graph.lock().unwrap().node_props.get(&node_id).cloned()
  }

  pub fn send(&self, cmd: Command) -> Result<(), String> {
    let guard = self.sender.lock().unwrap();
    let sender = guard.as_ref().ok_or("PipeWire is not connected")?;
    sender.send(cmd).map_err(|_| "PipeWire thread is gone".to_string())
  }
}

/// Spawns the PipeWire thread plus a small emitter thread that pushes coalesced
/// graph snapshots to the frontend (`pw-graph` event).
pub fn start(app: AppHandle) -> PwHandle {
  let graph = Arc::new(Mutex::new(Graph::default()));
  let dirty = Arc::new(AtomicBool::new(true));
  let levels: Levels = Default::default();
  let (sender, receiver) = pw::channel::channel::<Command>();

  {
    let graph = graph.clone();
    let dirty = dirty.clone();
    let levels = levels.clone();
    thread::Builder::new()
      .name("pipewire".into())
      .spawn(move || {
        if let Err(e) = run_loop(graph.clone(), dirty.clone(), levels, receiver) {
          let mut g = graph.lock().unwrap();
          g.connected = false;
          g.error = Some(format!("PipeWire: {e}"));
          dirty.store(true, Ordering::SeqCst);
        }
      })
      .expect("failed to spawn pipewire thread");
  }

  {
    let graph = graph.clone();
    let app = app.clone();
    thread::Builder::new()
      .name("pipewire-emitter".into())
      .spawn(move || {
        loop {
          thread::sleep(Duration::from_millis(40));
          if dirty.swap(false, Ordering::SeqCst) {
            let snap = graph.lock().unwrap().snapshot();
            let _ = app.emit("pw-graph", snap);
          }
        }
      })
      .expect("failed to spawn emitter thread");
  }

  // Level meters: ~30 fps `pw-levels` event with the peak of each node since the last frame.
  thread::Builder::new()
    .name("pipewire-levels".into())
    // Emitted even when silent: the UI uses each frame as its clock to scroll the sonographs.
    .spawn(move || {
      loop {
        thread::sleep(Duration::from_millis(33));
        let frame: HashMap<u32, f32> = {
          let mut l = levels.lock().unwrap();
          l.iter_mut().map(|(id, v)| (*id, std::mem::take(v))).collect()
        };
        let _ = app.emit("pw-levels", frame);
      }
    })
    .expect("failed to spawn levels thread");

  PwHandle {
    graph,
    sender: Mutex::new(Some(sender))
  }
}

/// Proxies bound for audio nodes (to read / write their `Props` param).
struct BoundNode {
  node: Node,
  _listener: NodeListener
}

/// Proxies bound for devices, with their active routes (route device -> route index).
struct BoundDevice {
  device: Device,
  routes: Rc<RefCell<HashMap<i32, i32>>>,
  _listener: DeviceListener
}

/// A passive capture stream attached to a node to measure its audio level.
struct Meter {
  _stream: StreamRc,
  _listener: StreamListener<()>
}

#[derive(Default)]
struct Bound {
  nodes: HashMap<u32, BoundNode>,
  devices: HashMap<u32, BoundDevice>,
  meters: HashMap<u32, Meter>
}

fn run_loop(
  graph: Arc<Mutex<Graph>>,
  dirty: Arc<AtomicBool>,
  levels: Levels,
  receiver: pw::channel::Receiver<Command>
) -> Result<(), pw::Error> {
  pw::init();

  let main_loop = pw::main_loop::MainLoopRc::new(None)?;
  let context = pw::context::ContextRc::new(&main_loop, None)?;
  let core = context.connect_rc(None)?;
  let registry = core.get_registry_rc()?;

  {
    let mut g = graph.lock().unwrap();
    g.connected = true;
    g.error = None;
  }
  dirty.store(true, Ordering::SeqCst);

  let bound: Rc<RefCell<Bound>> = Rc::new(RefCell::new(Bound::default()));
  // Link proxies we created; kept alive so the server does not tear them down.
  let created_links: Rc<RefCell<Vec<Link>>> = Rc::new(RefCell::new(Vec::new()));

  let _core_listener = {
    let graph = graph.clone();
    let dirty = dirty.clone();
    let main_loop = main_loop.downgrade();
    core
      .add_listener_local()
      .error(move |id, _seq, res, message| {
        eprintln!("pipewire error id:{id} res:{res}: {message}");
        if id == pw::core::PW_ID_CORE {
          let mut g = graph.lock().unwrap();
          g.connected = false;
          g.error = Some(format!("PipeWire: {message}"));
          dirty.store(true, Ordering::SeqCst);
          if let Some(ml) = main_loop.upgrade() {
            ml.quit();
          }
        }
      })
      .register()
  };

  let _registry_listener = {
    let graph = graph.clone();
    let dirty = dirty.clone();
    let graph_rm = graph.clone();
    let dirty_rm = dirty.clone();
    let bound = bound.clone();
    let bound_rm = bound.clone();
    let registry_weak = registry.downgrade();
    let core = core.clone();
    let levels_rm = levels.clone();

    registry
      .add_listener_local()
      .global(move |obj| {
        let Some(props) = obj.props else { return };
        match obj.type_ {
          ObjectType::Node => {
            if props.get("node.name").is_some_and(|n| n.starts_with(METER_PREFIX)) {
              graph.lock().unwrap().hidden.insert(obj.id);
              return;
            }
            let node = parse_node(obj.id, props);
            let meter_target = meter_target(&node.media_class).zip(props.get("object.serial").map(str::to_owned));
            let is_audio = node.media_class.contains("Audio");
            {
              let mut g = graph.lock().unwrap();
              g.nodes.insert(obj.id, node);
              g.node_props.insert(obj.id, dict_to_map(props));
            }
            dirty.store(true, Ordering::SeqCst);

            // Every node is bound for its full properties; audio ones also for their volume.
            if let Some(registry) = registry_weak.upgrade()
              && let Ok(proxy) = registry.bind::<Node, _>(obj)
            {
              let listener = node_param_listener(&proxy, obj.id, &graph, &dirty);
              if is_audio {
                proxy.subscribe_params(&[ParamType::Props]);
              }
              bound.borrow_mut().nodes.insert(
                obj.id,
                BoundNode {
                  node: proxy,
                  _listener: listener
                }
              );
            }

            if let Some((capture_sink, serial)) = meter_target {
              match create_meter(&core, obj.id, &serial, capture_sink, &levels) {
                Ok(m) => {
                  bound.borrow_mut().meters.insert(obj.id, m);
                }
                Err(e) => eprintln!("meter for node {}: {e}", obj.id)
              }
            }
          }
          ObjectType::Device => {
            let Some(registry) = registry_weak.upgrade() else {
              return
            };
            let Ok(device) = registry.bind::<Device, _>(obj) else {
              return
            };
            let routes = Rc::new(RefCell::new(HashMap::new()));
            let listener = device_route_listener(&device, &routes);
            device.subscribe_params(&[ParamType::Route]);
            bound.borrow_mut().devices.insert(
              obj.id,
              BoundDevice {
                device,
                routes,
                _listener: listener
              }
            );
          }
          ObjectType::Port => {
            if let Some(port) = parse_port(obj.id, props) {
              let mut g = graph.lock().unwrap();
              if g.hidden.contains(&port.node_id) {
                return;
              }
              g.ports.insert(obj.id, port);
              dirty.store(true, Ordering::SeqCst);
            }
          }
          ObjectType::Link => {
            if let Some(link) = parse_link(obj.id, props) {
              let mut g = graph.lock().unwrap();
              if g.hidden.contains(&link.output_node) || g.hidden.contains(&link.input_node) {
                return;
              }
              g.links.insert(obj.id, link);
              dirty.store(true, Ordering::SeqCst);
            }
          }
          _ => {}
        }
      })
      .global_remove(move |id| {
        let mut g = graph_rm.lock().unwrap();
        let removed = g.nodes.remove(&id).is_some() | g.ports.remove(&id).is_some() | g.links.remove(&id).is_some();
        g.hidden.remove(&id);
        g.node_props.remove(&id);
        drop(g);
        levels_rm.lock().unwrap().remove(&id);
        let mut b = bound_rm.borrow_mut();
        b.nodes.remove(&id);
        b.devices.remove(&id);
        b.meters.remove(&id);
        if removed {
          dirty_rm.store(true, Ordering::SeqCst);
        }
      })
      .register()
  };

  let _receiver = {
    let core = core.clone();
    let registry = registry.clone();
    let graph = graph.clone();
    let bound = bound.clone();
    let created_links = created_links.clone();
    receiver.attach(main_loop.loop_(), move |cmd| match cmd {
      Command::Link {
        output_port,
        input_port
      } => {
        let (out_node, in_node) = {
          let g = graph.lock().unwrap();
          (
            g.ports.get(&output_port).map(|p| p.node_id),
            g.ports.get(&input_port).map(|p| p.node_id)
          )
        };
        let (Some(out_node), Some(in_node)) = (out_node, in_node) else {
          eprintln!("link: unknown port {output_port} -> {input_port}");
          return;
        };
        let props = properties! {
            "link.output.node" => out_node.to_string(),
            "link.output.port" => output_port.to_string(),
            "link.input.node" => in_node.to_string(),
            "link.input.port" => input_port.to_string(),
            "object.linger" => "true",
        };
        match core.create_object::<Link>("link-factory", &props) {
          Ok(link) => created_links.borrow_mut().push(link),
          Err(e) => eprintln!("link: failed to create link: {e}")
        }
      }
      Command::Unlink { link_id } => {
        registry.destroy_global(link_id);
      }
      Command::SetVolume { node_id, channels } => {
        let props = vec![Property::new(
          SPA_PROP_CHANNEL_VOLUMES,
          Value::ValueArray(ValueArray::Float(channels))
        )];
        set_props(&graph, &bound.borrow(), node_id, props);
      }
      Command::SetMute { node_id, mute } => {
        let props = vec![Property::new(SPA_PROP_MUTE, Value::Bool(mute))];
        set_props(&graph, &bound.borrow(), node_id, props);
      }
    })
  };

  main_loop.run();

  let mut g = graph.lock().unwrap();
  g.connected = false;
  dirty.store(true, Ordering::SeqCst);
  Ok(())
}

/// Which nodes get a level meter, and whether it must capture the sink's monitor.
fn meter_target(media_class: &str) -> Option<bool> {
  if media_class.starts_with("Audio/Sink") || media_class.starts_with("Audio/Duplex") {
    Some(true)
  } else if media_class.starts_with("Audio/Source") || media_class.starts_with("Stream/Output/Audio") {
    Some(false)
  } else {
    None
  }
}

fn create_meter(
  core: &pw::core::CoreRc,
  node_id: u32,
  target_serial: &str,
  capture_sink: bool,
  levels: &Levels
) -> Result<Meter, pw::Error> {
  let props = properties! {
      "media.type" => "Audio",
      "media.category" => "Monitor",
      "media.role" => "DSP",
      "media.name" => "level meter",
      "node.name" => format!("{METER_PREFIX}{node_id}"),
      "target.object" => target_serial.to_owned(),
      "stream.capture.sink" => if capture_sink { "true" } else { "false" },
      // Don't wake up / drive the target: only measure when it's already running.
      "node.passive" => "true",
      "node.dont-reconnect" => "true",
      "node.dont-move" => "true",
  };
  let stream = StreamRc::new(core.clone(), &format!("{METER_PREFIX}{node_id}"), props)?;

  let levels = levels.clone();
  let listener = stream
    .add_local_listener_with_user_data(())
    .process(move |stream, _| {
      let Some(mut buffer) = stream.dequeue_buffer() else {
        return
      };
      let Some(d) = buffer.datas_mut().first_mut() else {
        return
      };
      let chunk = d.chunk();
      let (offset, size) = (chunk.offset() as usize, chunk.size() as usize);
      let Some(bytes) = d.data() else { return };
      let end = (offset + size).min(bytes.len());
      let peak = bytes[offset.min(end)..end]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b).abs())
        .fold(0.0f32, f32::max);
      let mut l = levels.lock().unwrap();
      let v = l.entry(node_id).or_insert(0.0);
      *v = v.max(peak);
    })
    .register()?;

  let mut info = AudioInfoRaw::new();
  info.set_format(AudioFormat::F32LE);
  let format = serialize(&Value::Object(Object {
    type_: SpaTypes::ObjectParamFormat.as_raw(),
    id: ParamType::EnumFormat.as_raw(),
    properties: info.into()
  }))
  .ok_or(pw::Error::CreationFailed)?;
  let mut params = [Pod::from_bytes(&format).unwrap()];
  stream.connect(
    Direction::Input,
    None,
    StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
    &mut params
  )?;

  Ok(Meter {
    _stream: stream,
    _listener: listener
  })
}

fn serialize(value: &Value) -> Option<Vec<u8>> {
  match PodSerializer::serialize(Cursor::new(Vec::new()), value) {
    Ok((cursor, _)) => Some(cursor.into_inner()),
    Err(e) => {
      eprintln!("pod serialize failed: {e:?}");
      None
    }
  }
}

/// Applies volume/mute properties to a node. Hardware nodes go through their device's
/// active Route (so the change is persisted and seen by other mixers); everything
/// else gets the node's `Props` param directly.
fn set_props(graph: &Arc<Mutex<Graph>>, bound: &Bound, node_id: u32, properties: Vec<Property>) {
  let props = Value::Object(Object {
    type_: SPA_TYPE_OBJECT_PROPS,
    id: ParamType::Props.as_raw(),
    properties
  });

  let device = graph.lock().unwrap().nodes.get(&node_id).and_then(|n| n.device);
  if let Some((device_id, route_device)) = device
    && let Some(d) = bound.devices.get(&device_id)
    && let Some(&route_index) = d.routes.borrow().get(&route_device)
  {
    let route = Value::Object(Object {
      type_: SPA_TYPE_OBJECT_PARAM_ROUTE,
      id: ParamType::Route.as_raw(),
      properties: vec![
        Property::new(SPA_PARAM_ROUTE_INDEX, Value::Int(route_index)),
        Property::new(SPA_PARAM_ROUTE_DEVICE, Value::Int(route_device)),
        Property::new(SPA_PARAM_ROUTE_PROPS, props),
        Property::new(SPA_PARAM_ROUTE_SAVE, Value::Bool(true)),
      ]
    });
    if let Some(pod) = serialize(&route) {
      d.device.set_param(ParamType::Route, 0, Pod::from_bytes(&pod).unwrap());
    }
    return;
  }

  let Some(b) = bound.nodes.get(&node_id) else {
    eprintln!("set_props: node {node_id} is not bound");
    return;
  };
  if let Some(pod) = serialize(&props) {
    b.node.set_param(ParamType::Props, 0, Pod::from_bytes(&pod).unwrap());
  }
}

fn device_route_listener(device: &Device, routes: &Rc<RefCell<HashMap<i32, i32>>>) -> DeviceListener {
  let routes = routes.clone();
  device
    .add_listener_local()
    .param(move |_seq, id, _index, _next, param| {
      if id != ParamType::Route {
        return;
      }
      let Some(param) = param else { return };
      let Ok((_, Value::Object(obj))) = PodDeserializer::deserialize_any_from(param.as_bytes()) else {
        return;
      };
      let (mut index, mut dev) = (None, None);
      for p in obj.properties {
        match (p.key, p.value) {
          (SPA_PARAM_ROUTE_INDEX, Value::Int(i)) => index = Some(i),
          (SPA_PARAM_ROUTE_DEVICE, Value::Int(d)) => dev = Some(d),
          _ => {}
        }
      }
      if let (Some(index), Some(dev)) = (index, dev) {
        routes.borrow_mut().insert(dev, index);
      }
    })
    .register()
}

fn node_param_listener(node: &Node, node_id: u32, graph: &Arc<Mutex<Graph>>, dirty: &Arc<AtomicBool>) -> NodeListener {
  let graph_info = graph.clone();
  let dirty_info = dirty.clone();
  let graph = graph.clone();
  let dirty = dirty.clone();
  node
    .add_listener_local()
    // Registry globals only carry a subset of the properties; the full set
    // (with device.id / card.profile.device) comes with the node info.
    .info(move |info| {
      let Some(props) = info.props() else { return };
      let mut g = graph_info.lock().unwrap();
      if let Some(n) = g.nodes.get_mut(&node_id) {
        if let Some(device) = node_device(props) {
          n.device = Some(device);
        }
        let alsa_card = get_u32(props, "alsa.card");
        if n.alsa_card != alsa_card {
          n.alsa_card = alsa_card;
          dirty_info.store(true, Ordering::SeqCst);
        }
      }
      let props = dict_to_map(props);
      if g.node_props.get(&node_id) != Some(&props) {
        g.node_props.insert(node_id, props);
        // Lets the UI know it should refetch the properties it shows.
        dirty_info.store(true, Ordering::SeqCst);
      }
    })
    .param(move |_seq, id, _index, _next, param| {
      if id != ParamType::Props {
        return;
      }
      let Some(param) = param else { return };
      let Ok((_, Value::Object(obj))) = PodDeserializer::deserialize_any_from(param.as_bytes()) else {
        return;
      };

      let mut channels = None;
      let mut mute = None;
      let mut channel_map = None;
      for p in obj.properties {
        match (p.key, p.value) {
          (SPA_PROP_CHANNEL_VOLUMES, Value::ValueArray(ValueArray::Float(v))) => channels = Some(v),
          (SPA_PROP_MUTE, Value::Bool(m)) => mute = Some(m),
          (SPA_PROP_CHANNEL_MAP, Value::ValueArray(ValueArray::Id(ids))) => {
            channel_map = Some(ids.iter().map(|i| channel_name(i.0)).collect())
          }
          _ => {}
        }
      }
      // The adapter emits several Props objects; only the one carrying
      // channelVolumes is relevant for us.
      let Some(channels) = channels else { return };

      let mut g = graph.lock().unwrap();
      if let Some(n) = g.nodes.get_mut(&node_id) {
        let prev = n.volume.take();
        n.volume = Some(PwVolume {
          channel_map: channel_map
            .or_else(|| prev.as_ref().map(|p| p.channel_map.clone()))
            .unwrap_or_default(),
          mute: mute.or(prev.map(|p| p.mute)).unwrap_or(false),
          channels
        });
        dirty.store(true, Ordering::SeqCst);
      }
    })
    .register()
}

fn channel_name(id: u32) -> String {
  match id {
    2 => "MONO",
    3 => "FL",
    4 => "FR",
    5 => "FC",
    6 => "LFE",
    7 => "SL",
    8 => "SR",
    9 => "FLC",
    10 => "FRC",
    11 => "RC",
    12 => "RL",
    13 => "RR",
    _ => return format!("CH{id}")
  }
  .to_string()
}

fn dict_to_map(props: &DictRef) -> BTreeMap<String, String> {
  props.iter().map(|(k, v)| (k.to_owned(), v.to_owned())).collect()
}

fn get(props: &DictRef, key: &str) -> Option<String> {
  props.get(key).map(str::to_owned)
}

fn get_u32(props: &DictRef, key: &str) -> Option<u32> {
  props.get(key).and_then(|v| v.parse().ok())
}

fn get_bool(props: &DictRef, key: &str) -> bool {
  matches!(props.get(key), Some("true" | "1"))
}

fn parse_node(id: u32, props: &DictRef) -> PwNode {
  let name = get(props, "node.name").unwrap_or_else(|| format!("node-{id}"));
  let app_name = get(props, "application.name");
  let description = get(props, "node.description")
    .or_else(|| get(props, "node.nick"))
    .or_else(|| {
      // Streams: "App: media name" is the most useful label.
      app_name.as_ref().map(|app| match props.get("media.name") {
        Some(media) => format!("{app}: {media}"),
        None => app.clone()
      })
    })
    .unwrap_or_else(|| name.clone());
  PwNode {
    id,
    name,
    description,
    media_class: get(props, "media.class").unwrap_or_default(),
    app_name,
    icon_name: get(props, "application.icon-name").or_else(|| get(props, "device.icon-name")),
    volume: None,
    device: node_device(props),
    alsa_card: get_u32(props, "alsa.card")
  }
}

fn node_device(props: &DictRef) -> Option<(u32, i32)> {
  get_u32(props, "device.id").zip(props.get("card.profile.device").and_then(|v| v.parse().ok()))
}

fn parse_port(id: u32, props: &DictRef) -> Option<PwPort> {
  let node_id = get_u32(props, "node.id")?;
  let direction = PortDirection::from_prop(props.get("port.direction")?)?;
  let media_type = MediaType::from_dsp_format(props.get("format.dsp").unwrap_or_default());
  Some(PwPort {
    id,
    node_id,
    index: get_u32(props, "port.id").unwrap_or(id),
    name: get(props, "port.name").unwrap_or_else(|| format!("port-{id}")),
    alias: get(props, "port.alias"),
    direction,
    media_type,
    channel: get(props, "audio.channel"),
    monitor: get_bool(props, "port.monitor"),
    physical: get_bool(props, "port.physical")
  })
}

fn parse_link(id: u32, props: &DictRef) -> Option<PwLink> {
  Some(PwLink {
    id,
    output_node: get_u32(props, "link.output.node")?,
    output_port: get_u32(props, "link.output.port")?,
    input_node: get_u32(props, "link.input.node")?,
    input_port: get_u32(props, "link.input.port")?
  })
}
