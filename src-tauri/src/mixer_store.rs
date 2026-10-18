//! The UI's view of the sound cards' mixers: built from the [`Alsa`] events, pushed to the
//! frontend as coalesced `alsa-mixers` snapshots.

use std::collections::BTreeMap;
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

use tauri::{
  AppHandle,
  Emitter
};
use tracing::trace;

use crate::alsa::{
  Alsa,
  Card,
  Listener
};

/// At most one `alsa-mixers` event per period: bursts of changes are coalesced.
const EMIT_PERIOD: Duration = Duration::from_millis(40);

#[derive(Default)]
pub struct MixerStore {
  cards: Mutex<BTreeMap<u32, Card>>,
  dirty: AtomicBool
}

impl MixerStore {
  pub fn new() -> Arc<Self> {
    Arc::new(Self::default())
  }

  /// Keeps the cards up to date from the ALSA events.
  pub fn subscribe(self: &Arc<Self>, alsa: &Alsa) {
    let store = self.clone();
    alsa.listen(Listener::card_added(move |card| {
      store.update(|cards| {
        cards.insert(card.index, card.clone());
      })
    }));

    let store = self.clone();
    alsa.listen(Listener::card_removed(move |index| {
      store.update(|cards| {
        cards.remove(&index);
      })
    }));

    let store = self.clone();
    alsa.listen(Listener::control_changed(move |index, control| {
      store.update(|cards| {
        if let Some(slot) = cards
          .get_mut(&index)
          .and_then(|card| card.controls.iter_mut().find(|c| c.same_control(control)))
        {
          *slot = control.clone();
        }
      })
    }));
  }

  fn update(&self, f: impl FnOnce(&mut BTreeMap<u32, Card>)) {
    f(&mut self.cards.lock().unwrap());
    self.dirty.store(true, Ordering::SeqCst);
  }

  pub fn snapshot(&self) -> Vec<Card> {
    self.cards.lock().unwrap().values().cloned().collect()
  }

  /// Pushes an `alsa-mixers` snapshot to the frontend whenever a card changed (coalesced).
  pub fn emit_changes(self: &Arc<Self>, app: AppHandle) {
    let store = self.clone();
    thread::Builder::new()
      .name("mixer-emitter".into())
      .spawn(move || {
        loop {
          thread::sleep(EMIT_PERIOD);
          if store.dirty.swap(false, Ordering::SeqCst) {
            let snapshot = store.snapshot();
            trace!(cards = snapshot.len(), "alsa-mixers emitted");
            let _ = app.emit("alsa-mixers", snapshot);
          }
        }
      })
      .expect("failed to spawn mixer emitter thread");
  }
}
