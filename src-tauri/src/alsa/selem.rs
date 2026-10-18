//! Direction-aware access to a simple mixer element, and alsamixer's volume scale.

use alsa::Round;
use alsa::mixer::{
  MilliBel,
  Selem,
  SelemChannelId
};

use super::types::MixerDirection;

/// Above this dB span, volumes are mapped on a perceptual (exponential) scale, like alsamixer.
const MAX_LINEAR_DB_SCALE: i64 = 24 * 100;

/// Playback/capture dispatch over the direction-specific `Selem` methods.
pub(super) struct Ops<'a>(pub(super) &'a Selem<'a>, pub(super) MixerDirection);

impl Ops<'_> {
  pub(super) fn has_volume(&self) -> bool {
    match self.1 {
      MixerDirection::Playback => self.0.has_playback_volume(),
      MixerDirection::Capture => self.0.has_capture_volume()
    }
  }

  pub(super) fn has_switch(&self) -> bool {
    match self.1 {
      MixerDirection::Playback => self.0.has_playback_switch(),
      MixerDirection::Capture => self.0.has_capture_switch()
    }
  }

  pub(super) fn has_channel(&self, c: SelemChannelId) -> bool {
    match self.1 {
      MixerDirection::Playback => self.0.has_playback_channel(c),
      MixerDirection::Capture => self.0.has_capture_channel(c)
    }
  }

  pub(super) fn raw_range(&self) -> (i64, i64) {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_volume_range(),
      MixerDirection::Capture => self.0.get_capture_volume_range()
    }
  }

  /// dB range in millibels; `(0, 0)` when the control has no dB information.
  pub(super) fn db_range(&self) -> (i64, i64) {
    let (min, max) = match self.1 {
      MixerDirection::Playback => self.0.get_playback_db_range(),
      MixerDirection::Capture => self.0.get_capture_db_range()
    };
    (min.0, max.0)
  }

  pub(super) fn raw(&self, c: SelemChannelId) -> Option<i64> {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_volume(c),
      MixerDirection::Capture => self.0.get_capture_volume(c)
    }
    .ok()
  }

  pub(super) fn db(&self, c: SelemChannelId) -> Option<i64> {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_vol_db(c),
      MixerDirection::Capture => self.0.get_capture_vol_db(c)
    }
    .ok()
    .map(|mb| mb.0)
  }

  pub(super) fn switch(&self, c: SelemChannelId) -> Option<bool> {
    match self.1 {
      MixerDirection::Playback => self.0.get_playback_switch(c),
      MixerDirection::Capture => self.0.get_capture_switch(c)
    }
    .ok()
    .map(|v| v != 0)
  }

  pub(super) fn set_raw_all(&self, value: i64) -> alsa::Result<()> {
    match self.1 {
      MixerDirection::Playback => self.0.set_playback_volume_all(value),
      MixerDirection::Capture => self.0.set_capture_volume_all(value)
    }
  }

  pub(super) fn set_db_all(&self, mb: i64, dir: Round) -> alsa::Result<()> {
    match self.1 {
      MixerDirection::Playback => self.0.set_playback_db_all(MilliBel(mb), dir),
      MixerDirection::Capture => self.0.set_capture_db_all(MilliBel(mb), dir)
    }
  }

  pub(super) fn set_switch(&self, on: bool) -> alsa::Result<()> {
    match self.1 {
      MixerDirection::Playback => self.0.set_playback_switch_all(on as i32),
      MixerDirection::Capture => self.0.set_capture_switch_all(on as i32)
    }
  }
}

/// alsamixer's mapped volume (alsa-utils `volume_mapping.c`): linear in dB for small ranges,
/// perceptual (cubic-like) otherwise, and linear in raw steps without dB information.
pub(super) fn normalized_volume(ops: &Ops, c: SelemChannelId) -> f32 {
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

pub(super) fn set_normalized_volume(ops: &Ops, volume: f64) -> alsa::Result<()> {
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
