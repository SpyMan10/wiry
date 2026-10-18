//! Requests sent to the ALSA thread.

use serde::Deserialize;

use super::types::MixerDirection;

/// Identifies one direction of a simple mixer element on a card.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlId {
  pub card: u32,
  pub name: String,
  pub index: u32,
  pub direction: MixerDirection
}

/// A value to set on a control.
#[derive(Debug)]
pub enum Command {
  /// Volume on alsamixer's mapped scale (0..1), applied to every channel.
  Volume {
    control: ControlId,
    value: f32
  },
  Switch {
    control: ControlId,
    on: bool
  },
  /// Index of the item to select in an enumerated control.
  Enum {
    control: ControlId,
    item: u32
  }
}

impl Command {
  pub(super) fn control(&self) -> &ControlId {
    match self {
      Command::Volume { control, .. } | Command::Switch { control, .. } | Command::Enum { control, .. } => control
    }
  }
}
