use std::path::Path;

use anyhow::Result;
use tokio::process::Command;

use crate::vm::firercracker_process::FirecrackerProcess;

pub struct FirecrackerStartup {
    command: Command,
}

impl FirecrackerStartup {
    pub fn new() -> Self {
        Self {
            command: Command::new("firecracker"),
        }
    }

    pub fn api_socket<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.command
            .args(&["--api-sock", &path.as_ref().to_string_lossy()]);
        self
    }

    pub fn start(mut self) -> Result<FirecrackerProcess> {
        Ok(FirecrackerProcess::new(self.command.spawn()?))
    }
}
