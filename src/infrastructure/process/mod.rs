use std::{env, process::Stdio, time::Duration};

use anyhow::Result;
use http::Method;
use serde_json::to_string;
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
};

use crate::{
    domain::{
        config::{Action, ActionType, FirecrackerConfiguration},
        http::Http,
    },
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

    pub async fn start_vm(&mut self) -> Result<Http> {
        self.stream
            .send_user_request(
                Http::new_request("/boot-source", Method::PUT)
                    .add_header("Host", "localhost")
                    .add_header("Content-Type", "application/json")
                    .body(to_string(&self.configuration.boot_source)?),
            )
            .await?;
        self.stream
            .send_user_request(
                Http::new_request("/drives/rootfs", Method::PUT)
                    .add_header("Host", "localhost")
                    .add_header("Content-Type", "application/json")
                    .body(to_string(&self.configuration.drives)?),
            )
            .await?;
        for inet in &self.configuration.network_interfaces {
            self.stream
                .send_user_request(
                    Http::new_request(
                        format!("/network-interfaces/{}", inet.iface_id),
                        Method::PUT,
                    )
                    .add_header("Host", "localhost")
                    .add_header("Content-Type", "application/json")
                    .body(to_string(&inet)?),
                )
                .await?;
        }
        tokio::time::sleep(Duration::from_millis(15)).await;

        self.stream
            .send_user_request(
                Http::new_request("/actions", Method::PUT)
                    .add_header("Host", "localhost")
                    .add_header("Content-Type", "application/json")
                    .body(to_string(&Action {
                        action_type: ActionType::InstanceStart,
                    })?),
            )
            .await?;

        Ok(self.stream.read_req().await?)
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
