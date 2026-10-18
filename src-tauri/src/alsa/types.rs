//! Data describing the sound cards' mixers, as announced to listeners.

use serde::{
  Deserialize,
  Serialize
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MixerDirection {
  Playback,
  Capture
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlVolume {
  /// Per-channel volume on alsamixer's mapped scale (0..1).
  pub channels: Vec<f32>,
  pub channel_names: Vec<String>,
  /// Loudest channel in dB, when the control has dB information.
  pub db: Option<f32>
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlEnum {
  pub items: Vec<String>,
  pub current: u32
}

/// One direction of a simple mixer element (what alsamixer shows as one control).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Control {
  pub name: String,
  pub index: u32,
  pub direction: MixerDirection,
  pub volume: Option<ControlVolume>,
  /// On/off switch (`true` = on, i.e. not muted).
  pub switch: Option<bool>,
  pub enumerated: Option<ControlEnum>
}

impl Control {
  /// Whether `other` is the same control (possibly with other values).
  pub fn same_control(&self, other: &Control) -> bool {
    self.name == other.name && self.index == other.index && self.direction == other.direction
  }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
  pub index: u32,
  pub name: String,
  pub long_name: String,
  pub controls: Vec<Control>
}
