//! Event subscription: a [`Listener`] wraps the callback to run for one kind of event.
//!
//! Callbacks run on the ALSA thread: keep them short and non-blocking, and don't call
//! [`Alsa::listen`](super::Alsa::listen) from inside one.

use std::sync::Mutex;

use tracing::{
  debug,
  info
};

use super::types::{
  Card,
  Control
};

pub type OnCard = Box<dyn Fn(&Card) + Send>;
/// Receives the index of the removed card.
pub type OnCardRemoved = Box<dyn Fn(u32) + Send>;
/// Receives the card index and the control with its new values.
pub type OnControl = Box<dyn Fn(u32, &Control) + Send>;

pub enum Listener {
  /// A card appeared (also sent again when the list of its controls changes).
  CardAdded(OnCard),
  CardRemoved(OnCardRemoved),
  /// A control's value changed (from us or from elsewhere: alsamixer, media keys…).
  ControlChanged(OnControl)
}

/// Shorthands so callers can write `Listener::card_added(|card| …)` without boxing.
impl Listener {
  pub fn card_added(f: impl Fn(&Card) + Send + 'static) -> Self {
    Self::CardAdded(Box::new(f))
  }

  pub fn card_removed(f: impl Fn(u32) + Send + 'static) -> Self {
    Self::CardRemoved(Box::new(f))
  }

  pub fn control_changed(f: impl Fn(u32, &Control) + Send + 'static) -> Self {
    Self::ControlChanged(Box::new(f))
  }
}

/// An event as produced by the ALSA thread, dispatched to the matching listeners.
pub(super) enum Event<'a> {
  CardAdded(&'a Card),
  CardRemoved(u32),
  ControlChanged(u32, &'a Control)
}

impl Event<'_> {
  fn log(&self) {
    match self {
      Event::CardAdded(c) => info!(card = c.index, name = %c.name, controls = c.controls.len(), "card added"),
      Event::CardRemoved(index) => info!(card = index, "card removed"),
      Event::ControlChanged(card, c) => debug!(
        card,
        control = %c.name,
        index = c.index,
        direction = ?c.direction,
        volume = ?c.volume.as_ref().map(|v| &v.channels),
        switch = ?c.switch,
        item = ?c.enumerated.as_ref().map(|e| e.current),
        "control changed"
      )
    }
  }
}

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
        (Listener::CardAdded(f), Event::CardAdded(card)) => f(card),
        (Listener::CardRemoved(f), Event::CardRemoved(index)) => f(*index),
        (Listener::ControlChanged(f), Event::ControlChanged(card, control)) => f(*card, control),
        _ => {}
      }
    }
  }
}
