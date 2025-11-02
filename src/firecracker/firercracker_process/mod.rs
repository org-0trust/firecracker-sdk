use anyhow::Result;
use tokio::process::Child;

use crate::firecracker::firecracker_configuration::FirecrackerConfiguration;

pub mod firecracker_startup;

/// Structure for managing the Firecracker process created using `FirecrackerStartup`
pub struct FirecrackerProcess {
    process: Child,
    configuration: FirecrackerConfiguration,
}

impl FirecrackerProcess {
    pub(crate) fn new(child: Child, configuration: FirecrackerConfiguration) -> Self {
        Self {
            process: child,
            configuration,
        }
    }

    pub fn get_config(&self) -> &FirecrackerConfiguration {
        &self.configuration
    }

    /// Correctly starts the process stop and waits for it to complete
    pub async fn stop(&mut self) -> Result<()> {
        self.process.kill().await?;
        Ok(())
    }
}
