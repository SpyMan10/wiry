# wiry

A desktop patchbay for [PipeWire](https://pipewire.org/) on Linux: see what is plugged into what, rewire it in a few clicks, and adjust every volume from one window.

Built with [Tauri 2](https://tauri.app/) (Rust backend) and React + TypeScript (frontend).

## Concept

PipeWire routes audio between applications and devices as a graph of **nodes** (apps, sound cards, virtual devices) whose **ports** are joined by **links**. Command-line tools like `pw-link` and `wpctl` can do everything, but they don't show you the graph. wiry does, live:

- **Node list** (left): every connectable node grouped by kind (apps playing or recording, audio outputs and inputs, MIDI, video), with its ports, a live level meter and its volume and mute.
- **Patch graph** (centre): only the wired part of the graph, with sources on the left and destinations on the right. Wires animate while audio flows through them.
- **Bottom panel**, with two tabs:
  - **Properties**: every PipeWire property of the selected node.
  - **ALSA**: an `alsamixer`-like view of the sound card's own hardware controls.

### Making connections

- **Port to port:** click a port, then a compatible port on another node (same media type, opposite direction).
  - The click toggles: on two ports that are already linked, it removes the link.
  - Hold Shift to wire several targets in a row.
- **Node to node:** use **Link** on a node, then **Connect here** on another one. Channels are matched automatically (FL→FL, FR→FR, mono to all).
- **Disconnecting:** click a wire in the graph.

### How it works

- **The PipeWire connection:**
  - The backend talks to PipeWire natively through the [`pipewire`](https://crates.io/crates/pipewire) crate. It doesn't shell out to `pw-link` or `wpctl`.
  - It mirrors the graph from the registry and creates or destroys links through the link factory.
  - It sets volumes the way `wpctl` and pavucontrol do: on the device's active route for hardware nodes, and on the node itself otherwise.
- **Level meters:** each audio node gets a small *passive* capture stream. It measures the signal only when audio is already running and never wakes the device up. These meter streams are hidden from the UI.
- **ALSA mixer:** it uses the [`alsa`](https://crates.io/crates/alsa) crate. Volumes use `alsamixer`'s scale, and cards that are plugged in or removed are picked up automatically.
- **Live updates:** the UI receives graph snapshots, levels and mixer state as Tauri events.

## Requirements

- **Linux** with PipeWire (and WirePlumber, or another session manager). wiry is desktop-only.
- **Rust**: stable toolchain, edition 2024. The nightly toolchain is only needed for formatting (see below).
- **[Bun](https://bun.sh/)** for the frontend.
- **System libraries** (development packages):

| | Arch Linux | Debian / Ubuntu |
|---|---|---|
| WebKitGTK (Tauri webview) | `webkit2gtk-4.1` | `libwebkit2gtk-4.1-dev` |
| PipeWire | `pipewire` | `libpipewire-0.3-dev` |
| ALSA | `alsa-lib` | `libasound2-dev` |
| libclang (bindgen, used by `pipewire-sys`) | `clang` | `libclang-dev` |
| Build tools | `base-devel`, `pkgconf` | `build-essential`, `pkg-config` |

See also Tauri's [Linux prerequisites](https://tauri.app/start/prerequisites/#linux).

## Build and run

```sh
bun install

# Development: hot-reloaded frontend, rebuilt backend
bun run tauri dev

# Release build: binary plus .deb / .rpm / AppImage bundles
bun run tauri build
```

- **Release binary:** `src-tauri/target/release/wiry`.
- **Packages:** `src-tauri/target/release/bundle/`.

### Debug mode

Start the app with `WIRY_DEBUG=1` to show an FPS counter and keep the webview's right-click menu, which gives access to *Inspect Element*:

```sh
WIRY_DEBUG=1 bun run tauri dev
```

### Contributing

- Rust code is formatted with the **nightly** rustfmt. `src-tauri/rustfmt.toml` uses nightly-only options, so stable `cargo fmt` would undo them:

  ```sh
  cd src-tauri && cargo +nightly fmt
  ```

- The code is expected to stay free of Clippy warnings:

  ```sh
  cd src-tauri && cargo clippy --all-targets
  ```

- UI strings live in `src/locales/en.json`.

## Where settings are stored

Only UI layout preferences are saved:
- panel sizes,
- which panels and groups are folded,
- the active tab.

They live in the webview's `localStorage`, under `~/.local/share/net.spyman.wiry/localstorage/`. Volumes, links and mixer settings are not stored by wiry. PipeWire, WirePlumber and ALSA persist them themselves.

## AI disclaimer

A large part of this project, Rust backend and React frontend alike, was **written with the help of an AI model: Claude Opus 5.5** by Anthropic, used through Claude Code. The author steered the design and reviewed and tested the result, but the code has not been audited. It may contain bugs.

wiry controls real audio hardware. Use it at your own risk, and **take care with the ALSA tab**: hardware gains can go much louder than PipeWire's software volume. Lower the volume before experimenting, especially with headphones on.
