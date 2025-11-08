use std::{env, process::Stdio, time::Duration};

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, Stdout},
    process::{Child, Command},
};

use crate::{
    domain::config::FirecrackerConfiguration,
    infrastructure::connection::{socket::Socket, stream::Stream},
};

/// Structure for managing the Firecracker process created using `FirecrackerStartup`
pub struct FirecrackerProcess {
    process: Child,
    stream: Stream,
    configuration: FirecrackerConfiguration,
}

impl FirecrackerProcess {
    pub(crate) async fn new(configuration: FirecrackerConfiguration) -> Result<Self> {
        Ok(Self {
            process: {
                let child = Command::new(env::var("FIRECRACKER").unwrap_or("firecracker".into()))
                    .args([
                        "--api-sock",
                        configuration
                            .startup_config
                            .get_api_socket()
                            .to_str()
                            .unwrap(),
                    ])
                    .stdout(match configuration.startup_config.current_stdout() {
                        true => Stdio::piped(),
                        false => Stdio::null(),
                    })
                    .spawn()?;
                tokio::time::sleep(Duration::from_millis(2)).await;
                child
            },
            stream: Socket::new()?
                .connect(configuration.startup_config.get_api_socket())
                .await?,
            configuration,
        })
    }

    pub async fn stdout(&mut self) -> Result<String> {
        let mut out = String::new();
        if let Some(mut stdout) = self.process.stdout.take() {
            let mut buf = vec![];
            stdout.read_buf(&mut buf).await?;
            out = String::from_utf8(buf)?.to_string();
        }
        Ok(out)
    }

    pub fn config(&self) -> &FirecrackerConfiguration {
        &self.configuration
    }

    /// Correctly starts the process stop and waits for it to complete
    pub async fn stop(mut self) -> Result<()> {
        self.stream.close().await?;
        self.process.kill().await?;
        Ok(())
    }
}
