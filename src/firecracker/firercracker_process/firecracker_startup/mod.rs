use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use tokio::{fs, process::Command};

use crate::firecracker::{
    firecracker_configuration::{BootSource, FirecrackerConfiguration},
    firercracker_process::FirecrackerProcess,
};

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
    download_kernel: bool,
}

impl FirecrackerStartup {
    /// Creates a new instance of FirecrackerStartup
    pub fn new() -> Self {
        Self {
            command: Command::new("firecracker"),
            download_kernel: false,
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

    /// Flag to download the latest kernel version for microVM
    pub fn download_kernel(mut self, flag: bool) -> Self {
        self.download_kernel = flag;
        self
    }

    async fn setup_config(&self) -> Result<FirecrackerConfiguration> {
        Ok(FirecrackerConfiguration {
            boot_source: BootSource {
                kernel_image_path: self.get_kernel().await?,
                boot_args: HashMap::new(),
            },
        })
    }

    async fn get_kernel(&self) -> Result<PathBuf> {
        let env_path = env::var("FIRECRACKER_KERNEL");
        let env_download_path = env::var("FIRECRACKER_KERNEL_DOWNLOAD");
        let path = if env_path.is_ok() {
            PathBuf::from(env_path?)
        } else {
            PathBuf::from(
                env::home_dir()
                    .unwrap_or(
                        env::current_exe()?
                            .parent()
                            .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                            .to_path_buf(),
                    )
                    .join(".firecracker_kernel/latest/vmlinux.bin"),
            )
        };

        let download_path = if env_download_path.is_ok() {
            PathBuf::from(env_download_path?)
        } else {
            PathBuf::from(
                env::home_dir()
                    .unwrap_or(
                        env::current_exe()?
                            .parent()
                            .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                            .to_path_buf(),
                    )
                    .join(".firecracker_kernel/download"),
            )
        };

        if self.download_kernel == true {
            Self::catch_kernel(download_path).await?;
        }

        Ok(path)
    }

    async fn catch_kernel<P: AsRef<Path>>(path: P) -> Result<()> {
        let base_url = "http://spec.ccfc.min.s3.amazonaws.com";
        let prefix = "firecracker-ci/v1.10/x86_64/vmlinux-5.10";

        let list_url = format!("{base_url}/?prefix={prefix}&list-type=2");

        let download_dir = path.as_ref();
        fs::create_dir_all(download_dir).await?;

        let xml = Client::new().get(&list_url).send().await?.text().await?;

        let re = Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/vmlinux-5\.10\.\d{3})</Key>")?;
        let mut versions: Vec<_> = re.captures_iter(&xml).map(|m| m[1].to_string()).collect();

        if versions.is_empty() {
            anyhow::bail!("Could not find any version of vmlinux");
        }

        versions.sort();
        let latest = versions.last().unwrap();

        let file_name = latest.split('/').last().unwrap();
        let file_path = download_dir.join(file_name);

        if file_path.exists() {
            println!("Already download: {}", file_name);
            return Ok(());
        }

        let download_url = format!("https://s3.amazonaws.com/spec.ccfc.min/{latest}");
        println!("Downloading: {download_url}");

        let bytes = Client::new()
            .get(&download_url)
            .send()
            .await?
            .bytes()
            .await?;
        fs::write(&file_path, &bytes).await?;

        println!("Download successfull: {}", file_path.display());

        let target_path = path.as_ref().parent().unwrap().join("latest");
        fs::create_dir_all(&target_path).await?;

        fs::copy(&file_path, target_path.join("vmlinux.bin")).await?;

        Ok(())
    }

    /// Starts a VM with specified parameters
    /// Returns a structure for working with the Firecracker process
    pub async fn start(mut self) -> Result<FirecrackerProcess> {
        let config = self.setup_config().await?;
        Ok(FirecrackerProcess::new(self.command.spawn()?, config))
    }
}

impl Default for FirecrackerStartup {
    fn default() -> Self {
        Self::new()
    }
}
