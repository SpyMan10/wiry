//! PipeWire integration, independent of the UI layer.
//!
//! [`PipeWire`] runs the connection on its own thread: register [`Listener`]s to be told about
//! the graph (nodes, ports, links, volumes, levels), then [`start`](PipeWire::start) it and
//! drive it with [`Command`]s. A resync request or a lost connection ends the current session
//! and starts a new one, which announces the whole graph again.

mod command;
mod event;
mod meter;
mod parse;
mod session;
mod types;
mod volume;

use std::sync::{
  Arc,
  Mutex
};
use std::thread;
use std::time::Duration;

pub use command::Command;
pub use event::Listener;
use event::{
  Event,
  Listeners
};
use pipewire as pw;
use session::{
  Exit,
  Session
};
use tracing::{
  debug,
  info
};
pub use types::{
  Link,
  Node,
  Port,
  Volume
};

/// Delay before reconnecting after PipeWire went away (e.g. restarted).
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

pub struct PipeWire {
  listeners: Arc<Listeners>,
  sender: pw::channel::Sender<Command>,
  /// Taken by the PipeWire thread on start.
  receiver: Mutex<Option<pw::channel::Receiver<Command>>>
}

impl Default for PipeWire {
  fn default() -> Self {
    Self::new()
  }
}

impl PipeWire {
  pub fn new() -> Self {
    let (sender, receiver) = pw::channel::channel();
    Self {
      listeners: Default::default(),
      sender,
      receiver: Mutex::new(Some(receiver))
    }
  }

  /// Registers a callback. Register before [`start`](Self::start) to see the initial graph
  /// (or send [`Command::Resync`] to have it announced again).
  pub fn listen(&self, listener: Listener) {
    self.listeners.add(listener);
  }

  /// Spawns the PipeWire thread. Sessions follow one another until the app exits.
  pub fn start(&self) -> Result<(), String> {
    let mut receiver = self
      .receiver
      .lock()
      .unwrap()
      .take()
      .ok_or("PipeWire is already started")?;
    let listeners = self.listeners.clone();
    thread::Builder::new()
      .name("pipewire".into())
      .spawn(move || {
        pw::init();
        loop {
          let (rx, exit) = Session::run(&listeners, receiver);
          receiver = rx;
          let error = match &exit {
            Ok(Exit::Resync) => None,
            Ok(Exit::Disconnected(message)) => message.clone(),
            Err(e) => Some(format!("PipeWire: {e}"))
          };
          listeners.emit(Event::Disconnected(error.as_deref()));
          if !matches!(exit, Ok(Exit::Resync)) {
            info!("reconnecting in {RECONNECT_DELAY:?}");
            thread::sleep(RECONNECT_DELAY);
          }
        }
      })
      .map(drop)
      .map_err(|e| e.to_string())
  }

  pub fn send(&self, command: Command) -> Result<(), String> {
    debug!(?command, "command sent");
    self
      .sender
      .send(command)
      .map_err(|_| "PipeWire thread is gone".to_string())
  }
}
