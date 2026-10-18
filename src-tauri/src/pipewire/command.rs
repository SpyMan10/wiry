//! Requests sent to the PipeWire thread.

#[derive(Debug)]
pub enum Command {
  /// Connects an output port to an input port.
  Link {
    output_port: u32,
    input_port: u32
  },
  Unlink {
    link_id: u32
  },
  /// Linear per-channel volumes.
  SetVolume {
    node_id: u32,
    channels: Vec<f32>
  },
  SetMute {
    node_id: u32,
    mute: bool
  },
  /// Drops everything and reconnects to PipeWire: every object is announced again.
  Resync
}
