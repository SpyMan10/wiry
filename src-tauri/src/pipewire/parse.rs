//! Building our types from PipeWire property dictionaries.

use std::collections::BTreeMap;

use pipewire::spa::utils::dict::DictRef;

use super::types::{
  Link,
  MediaType,
  Node,
  Port,
  PortDirection
};

fn get(props: &DictRef, key: &str) -> Option<String> {
  props.get(key).map(str::to_owned)
}

pub(super) fn get_u32(props: &DictRef, key: &str) -> Option<u32> {
  props.get(key).and_then(|v| v.parse().ok())
}

fn get_bool(props: &DictRef, key: &str) -> bool {
  matches!(props.get(key), Some("true" | "1"))
}

pub(super) fn node(id: u32, props: &DictRef) -> Node {
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
  Node {
    id,
    name,
    description,
    media_class: get(props, "media.class").unwrap_or_default(),
    app_name,
    icon_name: get(props, "application.icon-name").or_else(|| get(props, "device.icon-name")),
    alsa_card: get_u32(props, "alsa.card"),
    props: props
      .iter()
      .map(|(k, v)| (k.to_owned(), v.to_owned()))
      .collect::<BTreeMap<_, _>>()
  }
}

/// `(device.id, card.profile.device)` of a node backed by a hardware device.
pub(super) fn node_device(props: &DictRef) -> Option<(u32, i32)> {
  get_u32(props, "device.id").zip(props.get("card.profile.device").and_then(|v| v.parse().ok()))
}

pub(super) fn port(id: u32, props: &DictRef) -> Option<Port> {
  Some(Port {
    id,
    node_id: get_u32(props, "node.id")?,
    index: get_u32(props, "port.id").unwrap_or(id),
    name: get(props, "port.name").unwrap_or_else(|| format!("port-{id}")),
    alias: get(props, "port.alias"),
    direction: PortDirection::from_prop(props.get("port.direction")?)?,
    media_type: MediaType::from_dsp_format(props.get("format.dsp").unwrap_or_default()),
    channel: get(props, "audio.channel"),
    monitor: get_bool(props, "port.monitor"),
    physical: get_bool(props, "port.physical")
  })
}

pub(super) fn link(id: u32, props: &DictRef) -> Option<Link> {
  Some(Link {
    id,
    output_node: get_u32(props, "link.output.node")?,
    output_port: get_u32(props, "link.output.port")?,
    input_node: get_u32(props, "link.input.node")?,
    input_port: get_u32(props, "link.input.port")?
  })
}
