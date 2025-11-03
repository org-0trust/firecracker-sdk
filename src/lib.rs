//! A ***Non-official*** library for easy work with FirecrackerVM from Rust.
//!
//! Exemple:
//! ```no_run
//! let vm_process = FirecrackerStartup::new()
//!     .api_sock("/tmp/some.socket")
//!     .start().unwrap();
//!
//! let firecracker_socket = FirecrackerSocket::new().unwrap();
//! let firecracker_stream = firecracker_socket.connect("/tmp/some.socket");
//! ```
//! Before starting work, we recommend that you familiarize yourself with the official [documentation](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md).

pub(crate) mod aws_s3;
pub mod firecracker;
