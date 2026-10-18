//! The mixer of one sound card: reading its controls and applying commands.

use alsa::Mixer;
use alsa::mixer::{
  Selem,
  SelemChannelId,
  SelemId
};
use tracing::{
  debug,
  warn
};

use super::command::Command;
use super::selem::{
  Ops,
  normalized_volume,
  set_normalized_volume
};
use super::types::{
  Card,
  Control,
  ControlEnum,
  ControlVolume,
  MixerDirection
};

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

pub(super) struct CardMixer {
  index: u32,
  name: String,
  long_name: String,
  mixer: Mixer
}

impl CardMixer {
  pub(super) fn open(card: &alsa::Card) -> alsa::Result<Self> {
    let index = card.get_index() as u32;
    let name = card.get_name().unwrap_or_else(|_| format!("Card {index}"));
    Ok(Self {
      index,
      long_name: card.get_longname().unwrap_or_else(|_| name.clone()),
      name,
      mixer: Mixer::new(&format!("hw:{index}"), true)?
    })
  }

  /// Processes pending mixer events so that the next `read` sees fresh values.
  pub(super) fn refresh(&self) {
    let _ = self.mixer.handle_events();
  }

  pub(super) fn read(&self) -> Card {
    let mut controls = Vec::new();
    for elem in self.mixer.iter() {
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
    Card {
      index: self.index,
      name: self.name.clone(),
      long_name: self.long_name.clone(),
      controls
    }
  }

  pub(super) fn apply(&self, cmd: Command) {
    let id = cmd.control();
    let Some(selem) = self.mixer.find_selem(&SelemId::new(&id.name, id.index)) else {
      warn!(card = id.card, control = %id.name, index = id.index, "no such control");
      return;
    };
    let ops = Ops(&selem, id.direction);
    let result = match &cmd {
      Command::Volume { value, .. } => set_normalized_volume(&ops, value.clamp(0.0, 1.0) as f64),
      Command::Switch { on, .. } => ops.set_switch(*on),
      Command::Enum { item, .. } => selem.set_enum_item(SelemChannelId::mono(), *item)
    };
    match result {
      Ok(()) => debug!(card = id.card, control = %id.name, index = id.index, "control updated"),
      Err(e) => warn!(card = id.card, control = %id.name, index = id.index, error = %e, "failed to update control")
    }
  }
}

/// Reads one direction of a simple element (volume and/or switch).
fn read_control(selem: &Selem, name: &str, index: u32, direction: MixerDirection) -> Option<Control> {
  let ops = Ops(selem, direction);
  if !ops.has_volume() && !ops.has_switch() {
    return None;
  }
  let volume = ops.has_volume().then(|| {
    let channels: Vec<SelemChannelId> = CHANNELS.into_iter().filter(|c| ops.has_channel(*c)).collect();
    ControlVolume {
      channels: channels.iter().map(|c| normalized_volume(&ops, *c)).collect(),
      channel_names: channels.iter().map(|c| c.to_string()).collect(),
      db: channels
        .iter()
        .filter_map(|c| ops.db(*c))
        .max()
        .map(|mb| mb as f32 / 100.0)
    }
  });
  Some(Control {
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

fn read_enum(selem: &Selem, name: &str, index: u32) -> Option<Control> {
  let items: Vec<String> = selem.iter_enum().ok()?.filter_map(Result::ok).collect();
  let current = selem.get_enum_item(SelemChannelId::mono()).ok()?;
  let direction = if selem.is_enum_capture() && !selem.is_enum_playback() {
    MixerDirection::Capture
  } else {
    MixerDirection::Playback
  };
  Some(Control {
    name: name.to_owned(),
    index,
    direction,
    volume: None,
    switch: None,
    enumerated: Some(ControlEnum { items, current })
  })
}
