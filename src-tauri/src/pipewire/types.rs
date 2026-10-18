//! Data describing the PipeWire graph, as announced to listeners.

use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
  pub id: u32,
  pub name: String,
  pub description: String,
  pub media_class: String,
  pub app_name: Option<String>,
  pub icon_name: Option<String>,
  /// ALSA card backing this node (hardware nodes only).
  pub alsa_card: Option<u32>,
  /// Every PipeWire property of the node.
  #[serde(skip)]
  pub props: BTreeMap<String, String>
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
  /// Linear per-channel volumes (as stored by PipeWire).
  pub channels: Vec<f32>,
  pub channel_map: Vec<String>,
  pub mute: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PortDirection {
  In,
  Out
}

impl PortDirection {
  /// Parses PipeWire's `port.direction` property.
  pub(super) fn from_prop(value: &str) -> Option<Self> {
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
  pub(super) fn from_dsp_format(format: &str) -> Self {
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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Port {
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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
  pub id: u32,
  pub output_node: u32,
  pub output_port: u32,
  pub input_node: u32,
  pub input_port: u32
}
