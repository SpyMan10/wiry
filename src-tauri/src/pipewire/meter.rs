//! Level meters: a passive capture stream per audio node, measuring the peak of each buffer.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pipewire as pw;
use pw::properties::properties;
use pw::spa::param::ParamType;
use pw::spa::param::audio::{
  AudioFormat,
  AudioInfoRaw
};
use pw::spa::pod::{
  Object,
  Pod,
  Value
};
use pw::spa::utils::{
  Direction,
  SpaTypes
};
use pw::stream::{
  StreamFlags,
  StreamListener,
  StreamRc
};

use super::volume::serialize;

/// Name prefix of our own meter streams: they are hidden from the announced graph.
const METER_PREFIX: &str = "wiry-meter-";

/// Peak level per node since the last `Levels` event (linear).
pub(super) type Peaks = Rc<RefCell<HashMap<u32, f32>>>;

pub(super) fn is_meter(node_name: Option<&str>) -> bool {
  node_name.is_some_and(|n| n.starts_with(METER_PREFIX))
}

/// Whether a node gets a meter, and whether it must capture the sink's monitor.
pub(super) fn target(media_class: &str) -> Option<bool> {
  if media_class.starts_with("Audio/Sink") || media_class.starts_with("Audio/Duplex") {
    Some(true)
  } else if media_class.starts_with("Audio/Source") || media_class.starts_with("Stream/Output/Audio") {
    Some(false)
  } else {
    None
  }
}

/// A passive capture stream attached to a node; dropping it removes the stream.
pub(super) struct Meter {
  _stream: StreamRc,
  _listener: StreamListener<()>
}

impl Meter {
  pub(super) fn attach(
    core: &pw::core::CoreRc,
    node_id: u32,
    target_serial: &str,
    capture_sink: bool,
    peaks: &Peaks
  ) -> Result<Self, pw::Error> {
    let props = properties! {
        "media.type" => "Audio",
        "media.category" => "Monitor",
        "media.role" => "DSP",
        "media.name" => "level meter",
        "node.name" => format!("{METER_PREFIX}{node_id}"),
        "target.object" => target_serial.to_owned(),
        "stream.capture.sink" => if capture_sink { "true" } else { "false" },
        // Don't wake up / drive the target: only measure when it's already running.
        "node.passive" => "true",
        "node.dont-reconnect" => "true",
        "node.dont-move" => "true",
    };
    let stream = StreamRc::new(core.clone(), &format!("{METER_PREFIX}{node_id}"), props)?;

    let peaks = peaks.clone();
    let listener = stream
      .add_local_listener_with_user_data(())
      .process(move |stream, _| {
        let Some(mut buffer) = stream.dequeue_buffer() else {
          return
        };
        let Some(d) = buffer.datas_mut().first_mut() else {
          return
        };
        let chunk = d.chunk();
        let (offset, size) = (chunk.offset() as usize, chunk.size() as usize);
        let Some(bytes) = d.data() else { return };
        let end = (offset + size).min(bytes.len());
        let peak = bytes[offset.min(end)..end]
          .as_chunks::<4>()
          .0
          .iter()
          .map(|b| f32::from_le_bytes(*b).abs())
          .fold(0.0f32, f32::max);
        let mut peaks = peaks.borrow_mut();
        let v = peaks.entry(node_id).or_insert(0.0);
        *v = v.max(peak);
      })
      .register()?;

    let mut info = AudioInfoRaw::new();
    info.set_format(AudioFormat::F32LE);
    let format = serialize(&Value::Object(Object {
      type_: SpaTypes::ObjectParamFormat.as_raw(),
      id: ParamType::EnumFormat.as_raw(),
      properties: info.into()
    }))
    .ok_or(pw::Error::CreationFailed)?;
    let mut params = [Pod::from_bytes(&format).unwrap()];
    stream.connect(
      Direction::Input,
      None,
      StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
      &mut params
    )?;

    Ok(Self {
      _stream: stream,
      _listener: listener
    })
  }
}
