//! Native ALSA mixer, independent of the UI layer: the hardware controls of every sound card
//! (what `alsamixer` shows): volumes, switches and enumerated controls.
//!
//! [`Alsa`] runs on its own thread: register [`Listener`]s to be told about cards and control
//! changes (from us or from elsewhere), then [`start`](Alsa::start) it and drive it with
//! [`Command`]s. Hot-plugged cards are picked up automatically.

mod command;
mod event;
mod mixer;
mod selem;
mod types;

use std::collections::BTreeMap;
use std::sync::mpsc::{
  self,
  RecvTimeoutError
};
use std::sync::{
  Arc,
  Mutex
};
use std::thread;
use std::time::{
  Duration,
  Instant
};

pub use command::{
  Command,
  ControlId
};
pub use event::Listener;
use event::{
  Event,
  Listeners
};
use mixer::CardMixer;
use tracing::{
  debug,
  warn
};
pub use types::Card;

/// Refresh period of the control values (and of command batching).
const TICK: Duration = Duration::from_millis(100);
/// How often cards are re-enumerated to pick up hot-plugged devices.
const RESCAN: Duration = Duration::from_secs(2);

pub struct Alsa {
  listeners: Arc<Listeners>,
  sender: mpsc::Sender<Command>,
  /// Taken by the ALSA thread on start.
  receiver: Mutex<Option<mpsc::Receiver<Command>>>
}

impl Default for Alsa {
  fn default() -> Self {
    Self::new()
  }
}

impl Alsa {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel();
    Self {
      listeners: Default::default(),
      sender,
      receiver: Mutex::new(Some(receiver))
    }
  }

  /// Registers a callback. Register before [`start`](Self::start) to see the initial cards.
  pub fn listen(&self, listener: Listener) {
    self.listeners.add(listener);
  }

  /// Spawns the ALSA thread.
  pub fn start(&self) -> Result<(), String> {
    let receiver = self.receiver.lock().unwrap().take().ok_or("ALSA is already started")?;
    let listeners = self.listeners.clone();
    thread::Builder::new()
      .name("alsa-mixer".into())
      .spawn(move || Watcher::default().run(&listeners, receiver))
      .map(drop)
      .map_err(|e| e.to_string())
  }

  pub fn send(&self, command: Command) -> Result<(), String> {
    debug!(?command, "command sent");
    self.sender.send(command).map_err(|_| "ALSA thread is gone".to_string())
  }
}

/// The ALSA thread's state: open mixers and the last state announced for each card.
#[derive(Default)]
struct Watcher {
  mixers: BTreeMap<u32, CardMixer>,
  announced: BTreeMap<u32, Card>
}

impl Watcher {
  fn run(mut self, listeners: &Listeners, receiver: mpsc::Receiver<Command>) {
    let mut last_scan: Option<Instant> = None;
    loop {
      if last_scan.is_none_or(|t| t.elapsed() >= RESCAN) {
        self.rescan(listeners);
        last_scan = Some(Instant::now());
      }

      // Wait for a command (or the next tick), then drain whatever else is queued.
      match receiver.recv_timeout(TICK) {
        Ok(cmd) => {
          self.apply(cmd);
          while let Ok(cmd) = receiver.try_recv() {
            self.apply(cmd);
          }
        }
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => return
      }

      self.announce_changes(listeners);
    }
  }

  /// Opens mixers of new cards and forgets removed ones.
  fn rescan(&mut self, listeners: &Listeners) {
    let present: Vec<alsa::Card> = alsa::card::Iter::new().filter_map(Result::ok).collect();
    let gone: Vec<u32> = self
      .mixers
      .keys()
      .copied()
      .filter(|index| !present.iter().any(|c| c.get_index() as u32 == *index))
      .collect();
    for index in gone {
      self.mixers.remove(&index);
      self.announced.remove(&index);
      listeners.emit(Event::CardRemoved(index));
    }
    for card in present {
      let index = card.get_index() as u32;
      if self.mixers.contains_key(&index) {
        continue;
      }
      match CardMixer::open(&card) {
        Ok(mixer) => {
          let state = mixer.read();
          listeners.emit(Event::CardAdded(&state));
          self.announced.insert(index, state);
          self.mixers.insert(index, mixer);
        }
        Err(e) => warn!(card = index, error = %e, "cannot open mixer")
      }
    }
  }

  fn apply(&self, cmd: Command) {
    match self.mixers.get(&cmd.control().card) {
      Some(mixer) => mixer.apply(cmd),
      None => warn!(card = cmd.control().card, "unknown card")
    }
  }

  /// Re-reads every card and announces what changed since last time.
  fn announce_changes(&mut self, listeners: &Listeners) {
    for (index, mixer) in &self.mixers {
      mixer.refresh();
      let current = mixer.read();
      let Some(previous) = self.announced.get(index) else {
        continue;
      };
      if current == *previous {
        continue;
      }
      let same_controls = current.controls.len() == previous.controls.len()
        && current
          .controls
          .iter()
          .zip(&previous.controls)
          .all(|(a, b)| a.same_control(b));
      if same_controls {
        for (now, before) in current.controls.iter().zip(&previous.controls) {
          if now != before {
            listeners.emit(Event::ControlChanged(*index, now));
          }
        }
      } else {
        // The card's control list itself changed: announce it again as a whole.
        listeners.emit(Event::CardAdded(&current));
      }
      self.announced.insert(*index, current);
    }
  }
}
