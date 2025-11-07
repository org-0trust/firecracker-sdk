//! A ***Non-official*** library for easy work with FirecrackerVM from Rust.
//!
//! Exemple:
//! ```no_compile
//! let process = FirecrackerStartup::new()
//!     .set_api_socket("/tmp/some.socket")
//!     .download_rootfs(true)
//!     .download_kernel(true)
//!     .start().await.unwrap();
//! ```
//! Before starting work, we recommend that you familiarize yourself with the official [documentation](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md).

pub mod api;
pub mod domain;
pub mod infrastructure;
