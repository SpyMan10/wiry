//! Native ALSA mixer: exposes the hardware controls of every sound card (what `alsamixer`
//! shows): volumes, switches and enumerated controls.
//!
//! A dedicated thread keeps one mixer handle per card, applies commands from the UI and
//! pushes an `alsa-mixers` event whenever a control changes (from us or from elsewhere).

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

use alsa::mixer::{
  MilliBel,
  Selem,
  SelemChannelId,
  SelemId
};
use alsa::{
  Mixer,
  Round
};
use serde::{
  Deserialize,
  Serialize
};
use tauri::{
  AppHandle,
  Emitter
};

/// Above this dB span, volumes are mapped on a perceptual (exponential) scale, like alsamixer.
const MAX_LINEAR_DB_SCALE: i64 = 24 * 100;
/// Refresh period of the control values (and of command batching).
const TICK: Duration = Duration::from_millis(100);
/// How often cards are re-enumerated to pick up hot-plugged devices.
const RESCAN: Duration = Duration::from_secs(2);

const CHANNELS: [SelemChannelId; 9] = [
  SelemChannelId::FrontLeft,
  SelemChannelId::FrontRight,
  SelemChannelId::RearLeft,
  SelemChannelId::RearRight,
  SelemChannelId::FrontCenter,
  SelemChannelId::Woofer,
  SelemChannelId::SideLeft,
  SelemChannelId::SideRight,
  SelemChannelId::RearCenter
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MixerDirection {
  Playback,
  Capture
}

/// Identifies one direction of a simple mixer element on a card.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlId {
  pub card: u32,
  pub name: String,
  pub index: u32,
  pub direction: MixerDirection
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlsaVolume {
  /// Per-channel volume on alsamixer's mapped scale (0..1).
  pub channels: Vec<f32>,
  pub channel_names: Vec<String>,
  /// Loudest channel in dB, when the control has dB information.
  pub db: Option<f32>
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlsaEnum {
  pub items: Vec<String>,
  pub current: u32
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlsaControl {
  pub name: String,
  pub index: u32,
  pub direction: MixerDirection,
  pub volume: Option<AlsaVolume>,
  /// On/off switch (`true` = on, i.e. not muted).
  pub switch: Option<bool>,
  pub enumerated: Option<AlsaEnum>
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlsaCard {
  pub index: u32,
  pub name: String,
  pub long_name: String,
  pub controls: Vec<AlsaControl>
}

/// A value to set on a control.
#[derive(Debug)]
pub enum MixerCommand {
  Volume { id: ControlId, value: f32 },
  Switch { id: ControlId, on: bool },
  Enum { id: ControlId, item: u32 }
}

/// Handle stored in Tauri state.
pub struct AlsaHandle {
  cards: Arc<Mutex<Vec<AlsaCard>>>,
  sender: Mutex<mpsc::Sender<MixerCommand>>
}

impl AlsaHandle {
  pub fn snapshot(&self) -> Vec<AlsaCard> {
    self.cards.lock().unwrap().clone()
  }

  pub fn send(&self, cmd: MixerCommand) -> Result<(), String> {
    self
      .sender
      .lock()
      .unwrap()
      .send(cmd)
      .map_err(|_| "ALSA mixer thread is gone".to_string())
  }
}

pub fn start(app: AppHandle) -> AlsaHandle {
  let cards = Arc::new(Mutex::new(Vec::new()));
  let (sender, receiver) = mpsc::channel();

  {
    let cards = cards.clone();
    thread::Builder::new()
      .name("alsa-mixer".into())
      .spawn(move || run(app, cards, receiver))
      .expect("failed to spawn alsa mixer thread");
  }

  AlsaHandle {
    cards,
    sender: Mutex::new(sender)
  }
}

struct OpenCard {
  name: String,
  long_name: String,
  mixer: Mixer
}

fn run(app: AppHandle, shared: Arc<Mutex<Vec<AlsaCard>>>, receiver: mpsc::Receiver<MixerCommand>) {
  let mut open: BTreeMap<u32, OpenCard> = BTreeMap::new();
  let mut last_scan: Option<Instant> = None;
  let mut last_emitted: Option<Vec<AlsaCard>> = None;

  loop {
    if last_scan.is_none_or(|t| t.elapsed() >= RESCAN) {
      rescan(&mut open);
      last_scan = Some(Instant::now());
    }

    // Wait for a command (or the next tick), then drain whatever else is queued.
    match receiver.recv_timeout(TICK) {
      Ok(cmd) => {
        apply(&open, cmd);
        while let Ok(cmd) = receiver.try_recv() {
          apply(&open, cmd);
        }
      }
      Err(RecvTimeoutError::Timeout) => {}
      Err(RecvTimeoutError::Disconnected) => return
    }

    for card in open.values() {
      let _ = card.mixer.handle_events();
    }
    let snapshot: Vec<AlsaCard> = open.iter().map(|(index, card)| read_card(*index, card)).collect();
    if last_emitted.as_ref() != Some(&snapshot) {
      *shared.lock().unwrap() = snapshot.clone();
      let _ = app.emit("alsa-mixers", &snapshot);
      last_emitted = Some(snapshot);
    }
  }
}

/// Opens mixers of new cards and drops those of removed ones.
fn rescan(open: &mut BTreeMap<u32, OpenCard>) {
  let present: Vec<alsa::Card> = alsa::card::Iter::new().filter_map(Result::ok).collect();
  open.retain(|index, _| present.iter().any(|c| c.get_index() as u32 == *index));
  for card in present {
    let index = card.get_index() as u32;
    if open.contains_key(&index) {
      continue;
    }
    match Mixer::new(&format!("hw:{index}"), true) {
      Ok(mixer) => {
        let name = card.get_name().unwrap_or_else(|_| format!("Card {index}"));
        let long_name = card.get_longname().unwrap_or_else(|_| name.clone());
        open.insert(index, OpenCard { name, long_name, mixer });
      }
      Err(e) => eprintln!("alsa: cannot open mixer of card {index}: {e}")
    }
  }
}

fn read_card(index: u32, card: &OpenCard) -> AlsaCard {
  let mut controls = Vec::new();
  for elem in card.mixer.iter() {
    let Some(selem) = Selem::new(elem) else { continue };
    let id = selem.get_id();
    let Ok(name) = id.get_name() else { continue };
    if selem.is_enumerated() {
      controls.extend(read_enum(&selem, name, id.get_index()));
      continue;
    }
    for direction in [MixerDirection::Playback, MixerDirection::Capture] {
      controls.extend(read_control(&selem, name, id.get_index(), direction));
    }
  }
  AlsaCard {
    index,
    name: card.name.clone(),
    long_name: card.long_name.clone(),
    controls
  }
}

/// Reads one direction of a simple element (volume and/or switch).
fn read_control(selem: &Selem, name: &str, index: u32, direction: MixerDirection) -> Option<AlsaControl> {
  let ops = Ops(selem, direction);
  if !ops.has_volume() && !ops.has_switch() {
    return None;
  }
  let volume = ops.has_volume().then(|| {
    let channels: Vec<SelemChannelId> = CHANNELS.into_iter().filter(|c| ops.has_channel(*c)).collect();
    AlsaVolume {
      channels: channels.iter().map(|c| normalized_volume(&ops, *c)).collect(),
      channel_names: channels.iter().map(|c| c.to_string()).collect(),
      db: channels
        .iter()
        .filter_map(|c| ops.db(*c))
        .max()
        .map(|mb| mb as f32 / 100.0)
    }
  });
  Some(AlsaControl {
    name: name.to_owned(),
    index,
    direction,
    volume,
    switch: ops
      .has_switch()
      .then(|| ops.switch(SelemChannelId::mono()).unwrap_or(false)),
    enumerated: None
  })
}

fn read_enum(selem: &Selem, name: &str, index: u32) -> Option<AlsaControl> {
  let items: Vec<String> = selem.iter_enum().ok()?.filter_map(Result::ok).collect();
  let current = selem.get_enum_item(SelemChannelId::mono()).ok()?;
  let direction = if selem.is_enum_capture() && !selem.is_enum_playback() {
    MixerDirection::Capture
  } else {
    MixerDirection::Playback
  };
  Some(AlsaControl {
    name: name.to_owned(),
    index,
    direction,
    volume: None,
    switch: None,
    enumerated: Some(AlsaEnum { items, current })
  })
}

fn apply(open: &BTreeMap<u32, OpenCard>, cmd: MixerCommand) {
  let id = match &cmd {
    MixerCommand::Volume { id, .. } | MixerCommand::Switch { id, .. } | MixerCommand::Enum { id, .. } => id.clone()
  };
  let Some(card) = open.get(&id.card) else {
    eprintln!("alsa: unknown card {}", id.card);
    return;
  };
  let Some(selem) = card.mixer.find_selem(&SelemId::new(&id.name, id.index)) else {
    eprintln!("alsa: no control '{}',{} on card {}", id.name, id.index, id.card);
    return;
  };
  let ops = Ops(&selem, id.direction);
  let result = match cmd {
    MixerCommand::Volume { value, .. } => set_normalized_volume(&ops, value.clamp(0.0, 1.0) as f64),
    MixerCommand::Switch { on, .. } => ops.set_switch(on),
    MixerCommand::Enum { item, .. } => selem.set_enum_item(SelemChannelId::mono(), item)
  };
  if let Err(e) = result {
    eprintln!(
      "alsa: failed to update '{}',{} on card {}: {e}",
      id.name, id.index, id.card
    );
  }
}

/// Playback/capture dispatch over the direction-specific `Selem` methods.
struct Ops<'a>(&'a Selem<'a>, MixerDirection);

impl Ops<'_> {
  fn has_volume(&self) -> bool {
    match self.1 {
      MixerDirection::Playback => self.0.has_playback_volume(),
      MixerDirection::Capture => self.0.has_capture_volume()
    }
  }

  fn has_switch(&self) -> bool {
    match self.1 {
      MixerDirection::Playback => self.0.has_playback_switch(),
      MixerDirection::Capture => self.0.has_capture_switch()
    }
  }

  fn has_channel(&self, c: SelemChannelId) -> bool {
    match self.1 {
      MixerDirection::Playback => self.0.has_playback_channel(c),
      MixerDirection::Capture => self.0.has_capture_channel(c)
    }
  }

  fn raw_range(&self) -> (i64, i64) {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_volume_range(),
      MixerDirection::Capture => self.0.get_capture_volume_range()
    }
  }

  /// dB range in millibels; `(0, 0)` when the control has no dB information.
  fn db_range(&self) -> (i64, i64) {
    let (min, max) = match self.1 {
      MixerDirection::Playback => self.0.get_playback_db_range(),
      MixerDirection::Capture => self.0.get_capture_db_range()
    };
    (min.0, max.0)
  }

  fn raw(&self, c: SelemChannelId) -> Option<i64> {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_volume(c),
      MixerDirection::Capture => self.0.get_capture_volume(c)
    }
    .ok()
  }

  fn db(&self, c: SelemChannelId) -> Option<i64> {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_vol_db(c),
      MixerDirection::Capture => self.0.get_capture_vol_db(c)
    }
    .ok()
    .map(|mb| mb.0)
  }

  fn switch(&self, c: SelemChannelId) -> Option<bool> {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_switch(c),
      MixerDirection::Capture => self.0.get_capture_switch(c)
    }
    .ok()
    .map(|v| v != 0)
  }

  fn set_raw_all(&self, value: i64) -> alsa::Result<()> {
    match self.1 {
      MixerDirection::Playback => self.0.set_playback_volume_all(value),
      MixerDirection::Capture => self.0.set_capture_volume_all(value)
    }
  }

  fn set_db_all(&self, mb: i64, dir: Round) -> alsa::Result<()> {
    match self.1 {
      MixerDirection::Playback => self.0.set_playback_db_all(MilliBel(mb), dir),
      MixerDirection::Capture => self.0.set_capture_db_all(MilliBel(mb), dir)
    }
  }

  fn set_switch(&self, on: bool) -> alsa::Result<()> {
    match self.1 {
      MixerDirection::Playback => self.0.set_playback_switch_all(on as i32),
      MixerDirection::Capture => self.0.set_capture_switch_all(on as i32)
    }
  }
}

/// alsamixer's mapped volume (alsa-utils `volume_mapping.c`): linear in dB for small ranges,
/// perceptual (cubic-like) otherwise, and linear in raw steps without dB information.
fn normalized_volume(ops: &Ops, c: SelemChannelId) -> f32 {
  let (min, max) = ops.db_range();
  let mapped = match ops.db(c) {
    Some(value) if min < max => {
      if max - min <= MAX_LINEAR_DB_SCALE {
        (value - min) as f64 / (max - min) as f64
      } else {
        let normalized = 10f64.powf((value - max) as f64 / 6000.0);
        let min_norm = 10f64.powf((min - max) as f64 / 6000.0);
        (normalized - min_norm) / (1.0 - min_norm)
      }
    }
    _ => {
      let (rmin, rmax) = ops.raw_range();
      match ops.raw(c) {
        Some(v) if rmin < rmax => (v - rmin) as f64 / (rmax - rmin) as f64,
        _ => 0.0
      }
    }
  };
  mapped.clamp(0.0, 1.0) as f32
}

fn set_normalized_volume(ops: &Ops, volume: f64) -> alsa::Result<()> {
  // Round towards the requested direction so small slider moves always change the value.
  let current = normalized_volume(ops, SelemChannelId::mono()) as f64;
  let dir = if volume >= current { Round::Ceil } else { Round::Floor };

  let (min, max) = ops.db_range();
  if min >= max {
    let (rmin, rmax) = ops.raw_range();
    return ops.set_raw_all(rmin + (volume * (rmax - rmin) as f64).round() as i64);
  }
  if max - min <= MAX_LINEAR_DB_SCALE {
    return ops.set_db_all(min + (volume * (max - min) as f64).round() as i64, dir);
  }
  let min_norm = 10f64.powf((min - max) as f64 / 6000.0);
  let scaled = volume * (1.0 - min_norm) + min_norm;
  if scaled <= 0.0 {
    // Bottom of a range that ends in "mute" (-inf dB): use the lowest raw step.
    return ops.set_raw_all(ops.raw_range().0);
  }
  ops.set_db_all((6000.0 * scaled.log10()).round() as i64 + max, dir)
}
