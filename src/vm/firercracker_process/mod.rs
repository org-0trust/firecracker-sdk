use anyhow::Result;
use tokio::process::Child;

pub mod firecracker_startup;

/// Structure for managing the Firecracker process created using `FirecrackerStartup`
pub struct FirecrackerProcess {
    process: Child,
}

impl FirecrackerProcess {
    pub(crate) fn new(child: Child) -> Self {
        Self { process: child }
    }

    /// Correctly starts the process stop and waits for it to complete
    pub async fn stop(&mut self) -> Result<()> {
        self.process.kill().await?;
        Ok(())
    }
}
