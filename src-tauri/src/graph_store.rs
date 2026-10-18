//! The UI's view of the PipeWire graph: built from the [`PipeWire`] events, pushed to the
//! frontend as coalesced `pw-graph` snapshots.

use std::collections::{
  BTreeMap,
  HashMap
};
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

use serde::Serialize;
use tauri::{
  AppHandle,
  Emitter
};
use tracing::trace;

use crate::pipewire::{
  Link,
  Listener,
  Node,
  PipeWire,
  Port,
  Volume
};

/// At most one `pw-graph` event per period: bursts of changes are coalesced.
const EMIT_PERIOD: Duration = Duration::from_millis(40);

#[derive(Default)]
struct Graph {
  connected: bool,
  error: Option<String>,
  nodes: BTreeMap<u32, Node>,
  volumes: HashMap<u32, Volume>,
  ports: BTreeMap<u32, Port>,
  links: BTreeMap<u32, Link>
}

/// A node as the frontend sees it: the node and its volume.
#[derive(Clone, Serialize)]
pub struct NodeView {
  #[serde(flatten)]
  node: Node,
  volume: Option<Volume>
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphSnapshot {
  connected: bool,
  error: Option<String>,
  nodes: Vec<NodeView>,
  ports: Vec<Port>,
  links: Vec<Link>
}

#[derive(Default)]
pub struct GraphStore {
  graph: Mutex<Graph>,
  dirty: AtomicBool
}

impl GraphStore {
  pub fn new() -> Arc<Self> {
    Arc::new(Self::default())
  }

  /// Keeps the graph up to date from the PipeWire events.
  pub fn subscribe(self: &Arc<Self>, pw: &PipeWire) {
    let on = |f: fn(&mut Graph)| {
      let store = self.clone();
      move || store.update(f)
    };
    pw.listen(Listener::connected(on(|g| {
      g.connected = true;
      g.error = None;
    })));

    let store = self.clone();
    pw.listen(Listener::disconnected(move |error| {
      store.update(|g| {
        *g = Graph {
          error: error.map(str::to_owned),
          ..Default::default()
        }
      })
    }));

    let store = self.clone();
    pw.listen(Listener::node_added(move |node| {
      store.update(|g| {
        g.nodes.insert(node.id, node.clone());
      })
    }));
    let store = self.clone();
    pw.listen(Listener::node_changed(move |node| {
      store.update(|g| {
        g.nodes.insert(node.id, node.clone());
      })
    }));
    let store = self.clone();
    pw.listen(Listener::node_removed(move |id| {
      store.update(|g| {
        g.nodes.remove(&id);
        g.volumes.remove(&id);
      })
    }));
    let store = self.clone();
    pw.listen(Listener::volume_changed(move |id, volume| {
      store.update(|g| {
        g.volumes.insert(id, volume.clone());
      })
    }));

    let store = self.clone();
    pw.listen(Listener::port_added(move |port| {
      store.update(|g| {
        g.ports.insert(port.id, port.clone());
      })
    }));
    let store = self.clone();
    pw.listen(Listener::port_removed(move |id| {
      store.update(|g| {
        g.ports.remove(&id);
      })
    }));

    let store = self.clone();
    pw.listen(Listener::link_added(move |link| {
      store.update(|g| {
        g.links.insert(link.id, link.clone());
      })
    }));
    let store = self.clone();
    pw.listen(Listener::link_removed(move |id| {
      store.update(|g| {
        g.links.remove(&id);
      })
    }));
  }

  fn update(&self, f: impl FnOnce(&mut Graph)) {
    f(&mut self.graph.lock().unwrap());
    self.dirty.store(true, Ordering::SeqCst);
  }

  pub fn snapshot(&self) -> GraphSnapshot {
    let g = self.graph.lock().unwrap();
    GraphSnapshot {
      connected: g.connected,
      error: g.error.clone(),
      nodes: g
        .nodes
        .values()
        .map(|node| NodeView {
          node: node.clone(),
          volume: g.volumes.get(&node.id).cloned()
        })
        .collect(),
      ports: g.ports.values().cloned().collect(),
      links: g.links.values().cloned().collect()
    }
  }

  /// Every PipeWire property of a node.
  pub fn node_props(&self, node_id: u32) -> Option<BTreeMap<String, String>> {
    self.graph.lock().unwrap().nodes.get(&node_id).map(|n| n.props.clone())
  }

  /// Pushes a `pw-graph` snapshot to the frontend whenever the graph changed (coalesced).
  pub fn emit_changes(self: &Arc<Self>, app: AppHandle) {
    let store = self.clone();
    thread::Builder::new()
      .name("graph-emitter".into())
      .spawn(move || {
        loop {
          thread::sleep(EMIT_PERIOD);
          if store.dirty.swap(false, Ordering::SeqCst) {
            let snapshot = store.snapshot();
            trace!(
              connected = snapshot.connected,
              nodes = snapshot.nodes.len(),
              ports = snapshot.ports.len(),
              links = snapshot.links.len(),
              "pw-graph emitted"
            );
            let _ = app.emit("pw-graph", snapshot);
          }
        }
      })
      .expect("failed to spawn graph emitter thread");
  }
}
