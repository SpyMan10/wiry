mod alsa_mixer;
mod pw;

use std::collections::BTreeMap;

use alsa_mixer::{
  AlsaCard,
  AlsaHandle,
  ControlId,
  MixerCommand
};
use pw::{
  Command,
  GraphSnapshot,
  PwHandle
};
use tauri::{
  Manager,
  State
};

/// Debug overlays (FPS counter…) are enabled by running the app with `WIRY_DEBUG=1`.
#[tauri::command]
fn debug_enabled() -> bool {
  std::env::var("WIRY_DEBUG").is_ok_and(|v| v == "1")
}

#[tauri::command]
fn pw_graph(handle: State<'_, PwHandle>) -> GraphSnapshot {
  handle.snapshot()
}

#[tauri::command]
fn pw_node_props(handle: State<'_, PwHandle>, node_id: u32) -> Option<BTreeMap<String, String>> {
  handle.node_props(node_id)
}

/// Reconnects to PipeWire and rebuilds the graph from scratch.
#[tauri::command]
fn pw_resync(handle: State<'_, PwHandle>) -> Result<(), String> {
  handle.send(Command::Resync)
}

#[tauri::command]
fn pw_link(handle: State<'_, PwHandle>, output_port: u32, input_port: u32) -> Result<(), String> {
  handle.send(Command::Link {
    output_port,
    input_port
  })
}

#[tauri::command]
fn pw_unlink(handle: State<'_, PwHandle>, link_id: u32) -> Result<(), String> {
  handle.send(Command::Unlink { link_id })
}

#[tauri::command]
fn pw_set_volume(handle: State<'_, PwHandle>, node_id: u32, channels: Vec<f32>) -> Result<(), String> {
  handle.send(Command::SetVolume { node_id, channels })
}

#[tauri::command]
fn pw_set_mute(handle: State<'_, PwHandle>, node_id: u32, mute: bool) -> Result<(), String> {
  handle.send(Command::SetMute { node_id, mute })
}

#[tauri::command]
fn alsa_mixers(handle: State<'_, AlsaHandle>) -> Vec<AlsaCard> {
  handle.snapshot()
}

#[tauri::command]
fn alsa_set_volume(handle: State<'_, AlsaHandle>, control: ControlId, value: f32) -> Result<(), String> {
  handle.send(MixerCommand::Volume { id: control, value })
}

#[tauri::command]
fn alsa_set_switch(handle: State<'_, AlsaHandle>, control: ControlId, on: bool) -> Result<(), String> {
  handle.send(MixerCommand::Switch { id: control, on })
}

#[tauri::command]
fn alsa_set_enum(handle: State<'_, AlsaHandle>, control: ControlId, item: u32) -> Result<(), String> {
  handle.send(MixerCommand::Enum { id: control, item })
}

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      app.manage(pw::start(app.handle().clone()));
      app.manage(alsa_mixer::start(app.handle().clone()));
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      debug_enabled,
      pw_graph,
      pw_node_props,
      pw_resync,
      pw_link,
      pw_unlink,
      pw_set_volume,
      pw_set_mute,
      alsa_mixers,
      alsa_set_volume,
      alsa_set_switch,
      alsa_set_enum
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
