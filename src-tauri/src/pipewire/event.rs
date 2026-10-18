//! Event subscription: a [`Listener`] wraps the callback to run for one kind of event.
//!
//! Callbacks run on the PipeWire thread, as soon as the event happens: keep them short and
//! non-blocking, and don't call [`PipeWire::listen`](super::PipeWire::listen) from inside one.

use std::collections::HashMap;
use std::sync::Mutex;

use tracing::{
  debug,
  info,
  warn
};

use super::types::{
  Link,
  Node,
  Port,
  Volume
};

pub type OnConnected = Box<dyn Fn() + Send>;
/// Receives the error that ended the session, if any.
pub type OnDisconnected = Box<dyn Fn(Option<&str>) + Send>;
pub type OnNode = Box<dyn Fn(&Node) + Send>;
pub type OnPort = Box<dyn Fn(&Port) + Send>;
pub type OnLink = Box<dyn Fn(&Link) + Send>;
/// Receives the id of the removed object.
pub type OnRemoved = Box<dyn Fn(u32) + Send>;
/// Receives the node id and its new volume.
pub type OnVolume = Box<dyn Fn(u32, &Volume) + Send>;
/// Receives the peak level of each metered node, by node id.
pub type OnLevels = Box<dyn Fn(&HashMap<u32, f32>) + Send>;

pub enum Listener {
  /// A session with PipeWire started; every existing object is announced right after.
  Connected(OnConnected),
  /// The session ended (resync, PipeWire gone…): everything announced so far is gone.
  /// Carries the error, if any. A new session is started automatically.
  Disconnected(OnDisconnected),
  NodeAdded(OnNode),
  /// A node's properties changed (its full property set only comes after it was added).
  NodeChanged(OnNode),
  NodeRemoved(OnRemoved),
  /// Volume / mute of a node (node id, new volume).
  VolumeChanged(OnVolume),
  PortAdded(OnPort),
  PortRemoved(OnRemoved),
  LinkAdded(OnLink),
  LinkRemoved(OnRemoved),
  /// ~30 times per second, even when silent: peak level (linear) of each metered node since
  /// the previous call.
  Levels(OnLevels)
}

/// Shorthands so callers can write `Listener::node_added(|node| …)` without boxing.
impl Listener {
  pub fn connected(f: impl Fn() + Send + 'static) -> Self {
    Self::Connected(Box::new(f))
  }

  pub fn disconnected(f: impl Fn(Option<&str>) + Send + 'static) -> Self {
    Self::Disconnected(Box::new(f))
  }

  pub fn node_added(f: impl Fn(&Node) + Send + 'static) -> Self {
    Self::NodeAdded(Box::new(f))
  }

  pub fn node_changed(f: impl Fn(&Node) + Send + 'static) -> Self {
    Self::NodeChanged(Box::new(f))
  }

  pub fn node_removed(f: impl Fn(u32) + Send + 'static) -> Self {
    Self::NodeRemoved(Box::new(f))
  }

  pub fn volume_changed(f: impl Fn(u32, &Volume) + Send + 'static) -> Self {
    Self::VolumeChanged(Box::new(f))
  }

  pub fn port_added(f: impl Fn(&Port) + Send + 'static) -> Self {
    Self::PortAdded(Box::new(f))
  }

  pub fn port_removed(f: impl Fn(u32) + Send + 'static) -> Self {
    Self::PortRemoved(Box::new(f))
  }

  pub fn link_added(f: impl Fn(&Link) + Send + 'static) -> Self {
    Self::LinkAdded(Box::new(f))
  }

  pub fn link_removed(f: impl Fn(u32) + Send + 'static) -> Self {
    Self::LinkRemoved(Box::new(f))
  }

  pub fn levels(f: impl Fn(&HashMap<u32, f32>) + Send + 'static) -> Self {
    Self::Levels(Box::new(f))
  }
}

/// An event as produced by the session, dispatched to the matching listeners.
pub(super) enum Event<'a> {
  Connected,
  Disconnected(Option<&'a str>),
  NodeAdded(&'a Node),
  NodeChanged(&'a Node),
  NodeRemoved(u32),
  VolumeChanged(u32, &'a Volume),
  PortAdded(&'a Port),
  PortRemoved(u32),
  LinkAdded(&'a Link),
  LinkRemoved(u32),
  Levels(&'a HashMap<u32, f32>)
}

impl Event<'_> {
  fn log(&self) {
    match self {
      Event::Connected => info!("connected to PipeWire"),
      Event::Disconnected(None) => info!("disconnected"),
      Event::Disconnected(Some(error)) => warn!(error, "disconnected"),
      Event::NodeAdded(n) => debug!(id = n.id, name = %n.name, class = %n.media_class, "node added"),
      Event::NodeChanged(n) => debug!(id = n.id, name = %n.name, description = %n.description, "node changed"),
      Event::NodeRemoved(id) => debug!(id, "node removed"),
      Event::VolumeChanged(id, v) => debug!(id, channels = ?v.channels, mute = v.mute, "volume changed"),
      Event::PortAdded(p) => {
        debug!(id = p.id, node = p.node_id, name = %p.name, direction = ?p.direction, "port added")
      }
      Event::PortRemoved(id) => debug!(id, "port removed"),
      Event::LinkAdded(l) => debug!(
        id = l.id,
        from = format!("{}:{}", l.output_node, l.output_port),
        to = format!("{}:{}", l.input_node, l.input_port),
        "link added"
      ),
      Event::LinkRemoved(id) => debug!(id, "link removed"),
      // Level meters: ~30 frames per second, not logged.
      Event::Levels(_) => {}
    }
  }
}

/// Registered listeners, shared between the public handle and the PipeWire thread.
#[derive(Default)]
pub(super) struct Listeners(Mutex<Vec<Listener>>);

impl Listeners {
  pub(super) fn add(&self, listener: Listener) {
    self.0.lock().unwrap().push(listener);
  }

  pub(super) fn emit(&self, event: Event) {
    event.log();
    for listener in self.0.lock().unwrap().iter() {
      match (listener, &event) {
        (Listener::Connected(f), Event::Connected) => f(),
        (Listener::Disconnected(f), Event::Disconnected(error)) => f(*error),
        (Listener::NodeAdded(f), Event::NodeAdded(node)) => f(node),
        (Listener::NodeChanged(f), Event::NodeChanged(node)) => f(node),
        (Listener::NodeRemoved(f), Event::NodeRemoved(id)) => f(*id),
        (Listener::VolumeChanged(f), Event::VolumeChanged(id, volume)) => f(*id, volume),
        (Listener::PortAdded(f), Event::PortAdded(port)) => f(port),
        (Listener::PortRemoved(f), Event::PortRemoved(id)) => f(*id),
        (Listener::LinkAdded(f), Event::LinkAdded(link)) => f(link),
        (Listener::LinkRemoved(f), Event::LinkRemoved(id)) => f(*id),
        (Listener::Levels(f), Event::Levels(levels)) => f(levels),
        _ => {}
      }
    }
  }
}
