//! Backend logging (`tracing`), written to stderr.
//!
//! The level is set with `WIRY_LOG`, using the `RUST_LOG` syntax (e.g. `WIRY_LOG=debug`,
//! `WIRY_LOG=wiry_lib::pipewire=trace,info`). Without it: `info`, or `debug` with `WIRY_DEBUG=1`.

use tracing_subscriber::EnvFilter;

pub fn init(debug: bool) {
  let default = if debug {
    "wiry_lib=debug,info"
  } else {
    "wiry_lib=info,warn"
  };
  let filter = EnvFilter::try_from_env("WIRY_LOG").unwrap_or_else(|_| EnvFilter::new(default));
  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_thread_names(true)
    .init();
}
