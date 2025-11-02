use std::path::Path;

use anyhow::Result;
use tokio::process::Command;

use crate::vm::firercracker_process::FirecrackerProcess;

/// A structure for configuring the launch of FirecrackerVM. Helps to preconfigure and start the virtual machine.
///
/// Note: Firecracker must be installed globally.
///
/// Exemple:
/// ```no_run
/// let startup = FirecrackerStartup::new()
///     .api_socket("/tmp/some.socket");
/// startup.start().unwrap();
/// ```
pub struct FirecrackerStartup {
    command: Command,
}

impl FirecrackerStartup {
    /// Creates a new instance of FirecrackerStartup
    pub fn new() -> Self {
        Self {
            command: Command::new("firecracker"),
        }
    }

    /// Adds the --api-sock startup argument with the path to the unix socket
    ///
    /// Note: For the best documentation, please refer to [here](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md).
    pub fn api_socket<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.command
            .args(["--api-sock", &path.as_ref().to_string_lossy()]);
        self
    }

    /// Starts a VM with specified parameters
    /// Returns a structure for working with the Firecracker process
    pub fn start(mut self) -> Result<FirecrackerProcess> {
        Ok(FirecrackerProcess::new(self.command.spawn()?))
    }
}

impl Default for FirecrackerStartup {
    fn default() -> Self {
        Self::new()
    }
}
