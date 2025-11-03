use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;
use regex::Regex;
use tokio::{fs, process::Command};

use crate::{
    aws_s3::AwsS3,
    firecracker::{
        firecracker_configuration::{BootSource, Drives, FirecrackerConfiguration},
        firercracker_process::FirecrackerProcess,
    },
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
    download_rootfs: bool,
}

impl FirecrackerStartup {
    /// Creates a new instance of FirecrackerStartup
    pub fn new() -> Self {
        Self {
            command: Command::new("firecracker"),
            download_kernel: false,
            download_rootfs: false,
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
    /// Flag to download the ubuntu-22.04.ext4 for microVM
    pub fn download_rootfs(mut self, flag: bool) -> Self {
        self.download_rootfs = flag;
        self
    }

    async fn setup_config(&self) -> Result<FirecrackerConfiguration> {
        Ok(FirecrackerConfiguration {
            boot_source: BootSource {
                kernel_image_path: self.get_kernel().await?,
                boot_args: HashMap::new(),
            },
            drives: Drives {
                drive_id: "rootfs".into(),
                path_on_host: self.get_rootfs().await?,
                is_root_device: true,
                is_read_only: false,
            },
        })
    }

    async fn get_kernel(&self) -> Result<PathBuf> {
        let env_path = env::var("FIRECRACKER_KERNEL");
        let env_download_path = env::var("FIRECRACKER_KERNEL_DOWNLOAD");
        let path = if env_path.is_ok() {
            PathBuf::from(env_path?)
        } else {
            env::home_dir()
                .unwrap_or(
                    env::current_exe()?
                        .parent()
                        .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                        .to_path_buf(),
                )
                .join(".firecracker_kernel/latest/vmlinux.bin")
        };

        let download_path = if env_download_path.is_ok() {
            PathBuf::from(env_download_path?)
        } else {
            env::home_dir()
                .unwrap_or(
                    env::current_exe()?
                        .parent()
                        .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                        .to_path_buf(),
                )
                .join(".firecracker_kernel/download")
        };

        if self.download_kernel {
            Self::catch_kernel(download_path).await?;
        }

        Ok(path)
    }

    async fn catch_kernel<P: AsRef<Path>>(path: P) -> Result<()> {
        let prefix = "firecracker-ci/v1.10/x86_64/vmlinux-5.10";

        let aws_s3 = AwsS3::new(prefix);

        let download_dir = path.as_ref();
        fs::create_dir_all(download_dir).await?;

        let xml = aws_s3.xml().await?;

        let re = Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/vmlinux-5\.10\.\d{3})</Key>")?;
        let mut versions: Vec<_> = re.captures_iter(&xml).map(|m| m[1].to_string()).collect();

        if versions.is_empty() {
            anyhow::bail!("Could not find any version of vmlinux");
        }

        versions.sort();
        let latest = versions.last().unwrap();

        let file_name = latest.split('/').next_back().unwrap();
        let file_path = download_dir.join(file_name);

        if file_path.exists() {
            println!("Already download: {}", file_path.display());
            return Ok(());
        }

        let bytes = aws_s3.download(latest).await?;
        fs::write(&file_path, &bytes).await?;

        println!("Download successfull: {}", file_path.display());

        let target_path = path.as_ref().parent().unwrap().join("latest");
        fs::create_dir_all(&target_path).await?;

        fs::copy(&file_path, target_path.join("vmlinux.bin")).await?;

        Ok(())
    }

    async fn get_rootfs(&self) -> Result<PathBuf> {
        let env_path = env::var("FIRECRACKER_ROOTFS");
        let env_download_path = env::var("FIRECRACKER_ROOTFS_DOWNLOAD");
        let path = if env_path.is_ok() {
            PathBuf::from(env_path?)
        } else {
            env::home_dir()
                .unwrap_or(
                    env::current_exe()?
                        .parent()
                        .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                        .to_path_buf(),
                )
                .join(".firecracker_kernel/latest/vmlinux.bin")
        };

        let download_path = if env_download_path.is_ok() {
            PathBuf::from(env_download_path?)
        } else {
            env::home_dir()
                .unwrap_or(
                    env::current_exe()?
                        .parent()
                        .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                        .to_path_buf(),
                )
                .join(".firecracker_kernel/download")
        };

        if self.download_rootfs {
            Self::catch_rootfs(download_path).await?;
        }

        Ok(path)
    }

    async fn catch_rootfs<P: AsRef<Path>>(path: P) -> Result<()> {
        let prefix = "firecracker-ci/v1.10/x86_64/ubuntu-22.04.ext4";

        let aws_s3 = AwsS3::new(prefix);

        let download_dir = path.as_ref();
        fs::create_dir_all(download_dir).await?;

        let xml = aws_s3.xml().await?;

        let re = Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/ubuntu-22\.04\.ext4)</Key>")?;
        let mut versions: Vec<_> = re.captures_iter(&xml).map(|m| m[1].to_string()).collect();

        if versions.is_empty() {
            anyhow::bail!("Could not find any version of ubuntu-22.04.ext4");
        }

        versions.sort();
        let latest = versions.last().unwrap();

        let file_name = latest.split('/').next_back().unwrap();
        let file_path = download_dir.join(file_name);

        if file_path.exists() {
            println!("Already download: {}", file_path.display());
            return Ok(());
        }

        let bytes = aws_s3.download(latest).await?;
        fs::write(&file_path, &bytes).await?;

        println!("Download successfull: {}", file_path.display());

        let target_path = path.as_ref().parent().unwrap().join("latest");
        fs::create_dir_all(&target_path).await?;

        fs::copy(&file_path, target_path.join("ubuntu-22.04.ext4")).await?;

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
