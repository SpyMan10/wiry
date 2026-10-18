//! Reading and writing node volumes (the SPA `Props` param), and device routes.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;

use pipewire::device::{
  Device,
  DeviceListener
};
use pipewire::node::Node as NodeProxy;
use pipewire::spa;
use pipewire::spa::param::ParamType;
use pipewire::spa::pod::deserialize::PodDeserializer;
use pipewire::spa::pod::serialize::PodSerializer;
use pipewire::spa::pod::{
  Object,
  Pod,
  Property,
  Value,
  ValueArray
};
use tracing::error;

use super::types::Volume;

const SPA_TYPE_OBJECT_PROPS: u32 = spa::sys::SPA_TYPE_OBJECT_Props;
const SPA_PROP_MUTE: u32 = spa::sys::SPA_PROP_mute;
const SPA_PROP_CHANNEL_VOLUMES: u32 = spa::sys::SPA_PROP_channelVolumes;
const SPA_PROP_CHANNEL_MAP: u32 = spa::sys::SPA_PROP_channelMap;
const SPA_TYPE_OBJECT_PARAM_ROUTE: u32 = spa::sys::SPA_TYPE_OBJECT_ParamRoute;
const SPA_PARAM_ROUTE_INDEX: u32 = spa::sys::SPA_PARAM_ROUTE_index;
const SPA_PARAM_ROUTE_DEVICE: u32 = spa::sys::SPA_PARAM_ROUTE_device;
const SPA_PARAM_ROUTE_PROPS: u32 = spa::sys::SPA_PARAM_ROUTE_props;
const SPA_PARAM_ROUTE_SAVE: u32 = spa::sys::SPA_PARAM_ROUTE_save;

/// Active routes of a device: route device -> route index.
pub(super) type Routes = Rc<RefCell<HashMap<i32, i32>>>;

/// A property change to apply to a node.
pub(super) enum Change {
  Volume(Vec<f32>),
  Mute(bool)
}

impl Change {
  fn into_property(self) -> Property {
    match self {
      Change::Volume(channels) => {
        Property::new(SPA_PROP_CHANNEL_VOLUMES, Value::ValueArray(ValueArray::Float(channels)))
      }
      Change::Mute(mute) => Property::new(SPA_PROP_MUTE, Value::Bool(mute))
    }
  }
}

/// Where a change must be written: the device's active route for hardware nodes (persisted
/// and seen by other mixers, like wpctl/pavucontrol do), the node itself otherwise.
pub(super) enum Target<'a> {
  Route {
    device: &'a Device,
    route_index: i32,
    route_device: i32
  },
  Node(&'a NodeProxy)
}

pub(super) fn apply(target: Target, change: Change) {
  let props = Value::Object(Object {
    type_: SPA_TYPE_OBJECT_PROPS,
    id: ParamType::Props.as_raw(),
    properties: vec![change.into_property()]
  });
  match target {
    Target::Route {
      device,
      route_index,
      route_device
    } => {
      let route = Value::Object(Object {
        type_: SPA_TYPE_OBJECT_PARAM_ROUTE,
        id: ParamType::Route.as_raw(),
        properties: vec![
          Property::new(SPA_PARAM_ROUTE_INDEX, Value::Int(route_index)),
          Property::new(SPA_PARAM_ROUTE_DEVICE, Value::Int(route_device)),
          Property::new(SPA_PARAM_ROUTE_PROPS, props),
          Property::new(SPA_PARAM_ROUTE_SAVE, Value::Bool(true)),
        ]
      });
      if let Some(pod) = serialize(&route) {
        device.set_param(ParamType::Route, 0, Pod::from_bytes(&pod).unwrap());
      }
    }
    Target::Node(node) => {
      if let Some(pod) = serialize(&props) {
        node.set_param(ParamType::Props, 0, Pod::from_bytes(&pod).unwrap());
      }
    }
  }
}

pub(super) fn serialize(value: &Value) -> Option<Vec<u8>> {
  match PodSerializer::serialize(Cursor::new(Vec::new()), value) {
    Ok((cursor, _)) => Some(cursor.into_inner()),
    Err(e) => {
      error!(error = ?e, "pod serialization failed");
      None
    }
  }
}

fn object(param: &Pod) -> Option<Object> {
  match PodDeserializer::deserialize_any_from(param.as_bytes()) {
    Ok((_, Value::Object(obj))) => Some(obj),
    _ => None
  }
}

/// Reads a node's `Props` param. The adapter emits several Props objects: only the one
/// carrying channelVolumes is relevant. Missing mute / channel map are taken from `previous`.
pub(super) fn parse_props(param: &Pod, previous: Option<&Volume>) -> Option<Volume> {
  let mut channels = None;
  let mut mute = None;
  let mut channel_map = None;
  for p in object(param)?.properties {
    match (p.key, p.value) {
      (SPA_PROP_CHANNEL_VOLUMES, Value::ValueArray(ValueArray::Float(v))) => channels = Some(v),
      (SPA_PROP_MUTE, Value::Bool(m)) => mute = Some(m),
      (SPA_PROP_CHANNEL_MAP, Value::ValueArray(ValueArray::Id(ids))) => {
        channel_map = Some(ids.iter().map(|i| channel_name(i.0)).collect())
      }
      _ => {}
    }
  }
  Some(Volume {
    channels: channels?,
    channel_map: channel_map
      .or_else(|| previous.map(|p| p.channel_map.clone()))
      .unwrap_or_default(),
    mute: mute.or(previous.map(|p| p.mute)).unwrap_or(false)
  })
}

/// Keeps `routes` in sync with the device's active routes.
pub(super) fn watch_routes(device: &Device, routes: &Routes) -> DeviceListener {
  let routes = routes.clone();
  device
    .add_listener_local()
    .param(move |_seq, id, _index, _next, param| {
      if id != ParamType::Route {
        return;
      }
      let Some(obj) = param.and_then(object) else { return };
      let (mut index, mut dev) = (None, None);
      for p in obj.properties {
        match (p.key, p.value) {
          (SPA_PARAM_ROUTE_INDEX, Value::Int(i)) => index = Some(i),
          (SPA_PARAM_ROUTE_DEVICE, Value::Int(d)) => dev = Some(d),
          _ => {}
        }
      }
      if let (Some(index), Some(dev)) = (index, dev) {
        routes.borrow_mut().insert(dev, index);
      }
    })
    .register()
}

fn channel_name(id: u32) -> String {
  match id {
    2 => "MONO",
    3 => "FL",
    4 => "FR",
    5 => "FC",
    6 => "LFE",
    7 => "SL",
    8 => "SR",
    9 => "FLC",
    10 => "FRC",
    11 => "RC",
    12 => "RL",
    13 => "RR",
    _ => return format!("CH{id}")
  }
  .to_string()
}
