mod alsa;
mod graph_store;
mod logging;
mod mixer_store;
mod pipewire;

use std::collections::BTreeMap;
use std::sync::Arc;

use graph_store::{
  GraphSnapshot,
  GraphStore
};
use mixer_store::MixerStore;
use tauri::{
  Emitter,
  Manager,
  State
};

use crate::alsa::{
  Alsa,
  Card,
  Command as MixerCommand,
  ControlId
};
use crate::pipewire::{
  Command,
  Listener,
  PipeWire
};

/// Debug overlays (FPS counter…) are enabled by running the app with `WIRY_DEBUG=1`.
#[tauri::command]
fn debug_enabled() -> bool {
  std::env::var("WIRY_DEBUG").is_ok_and(|v| v == "1")
}

#[tauri::command]
fn pw_graph(store: State<'_, Arc<GraphStore>>) -> GraphSnapshot {
  store.snapshot()
}

#[tauri::command]
fn pw_node_props(store: State<'_, Arc<GraphStore>>, node_id: u32) -> Option<BTreeMap<String, String>> {
  store.node_props(node_id)
}

/// Reconnects to PipeWire and rebuilds the graph from scratch.
#[tauri::command]
fn pw_resync(pw: State<'_, PipeWire>) -> Result<(), String> {
  pw.send(Command::Resync)
}

#[tauri::command]
fn pw_link(pw: State<'_, PipeWire>, output_port: u32, input_port: u32) -> Result<(), String> {
  pw.send(Command::Link {
    output_port,
    input_port
  })
}

#[tauri::command]
fn pw_unlink(pw: State<'_, PipeWire>, link_id: u32) -> Result<(), String> {
  pw.send(Command::Unlink { link_id })
}

#[tauri::command]
fn pw_set_volume(pw: State<'_, PipeWire>, node_id: u32, channels: Vec<f32>) -> Result<(), String> {
  pw.send(Command::SetVolume { node_id, channels })
}

#[tauri::command]
fn pw_set_mute(pw: State<'_, PipeWire>, node_id: u32, mute: bool) -> Result<(), String> {
  pw.send(Command::SetMute { node_id, mute })
}

#[tauri::command]
fn alsa_mixers(store: State<'_, Arc<MixerStore>>) -> Vec<Card> {
  store.snapshot()
}

#[tauri::command]
fn alsa_set_volume(alsa: State<'_, Alsa>, control: ControlId, value: f32) -> Result<(), String> {
  alsa.send(MixerCommand::Volume { control, value })
}

#[tauri::command]
fn alsa_set_switch(alsa: State<'_, Alsa>, control: ControlId, on: bool) -> Result<(), String> {
  alsa.send(MixerCommand::Switch { control, on })
}

#[tauri::command]
fn alsa_set_enum(alsa: State<'_, Alsa>, control: ControlId, item: u32) -> Result<(), String> {
  alsa.send(MixerCommand::Enum { control, item })
}

pub fn run() {
  logging::init(debug_enabled());
  tracing::info!("starting wiry {}", env!("CARGO_PKG_VERSION"));
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
      let pw = PipeWire::new();
      let store = GraphStore::new();
      store.subscribe(&pw);
      let handle = app.handle().clone();
      pw.listen(Listener::levels(move |levels| {
        let _ = handle.emit("pw-levels", levels);
      }));
      pw.start()?;
      store.emit_changes(app.handle().clone());
      app.manage(pw);
      app.manage(store);

      let alsa = Alsa::new();
      let mixers = MixerStore::new();
      mixers.subscribe(&alsa);
      alsa.start()?;
      mixers.emit_changes(app.handle().clone());
      app.manage(alsa);
      app.manage(mixers);
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
