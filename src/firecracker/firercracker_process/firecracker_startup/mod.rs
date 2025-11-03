use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use anyhow::Result;
use regex::Regex;
use tokio::{join, process::Command};

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
/// ```no_compile
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
        let download_kernel_flag = self.download_kernel;
        let download_rootfs_flag = self.download_kernel;
        let kernel_path_task =
            tokio::spawn(async move { Self::get_kernel(download_kernel_flag).await });
        let rootfs_path_task =
            tokio::spawn(async move { Self::get_rootfs(download_rootfs_flag).await });
        let (kernel_path, rootfs_path) = join!(kernel_path_task, rootfs_path_task);
        Ok(FirecrackerConfiguration {
            boot_source: BootSource {
                kernel_image_path: kernel_path??,
                boot_args: HashMap::new(),
            },
            drives: Drives {
                drive_id: "rootfs".into(),
                path_on_host: rootfs_path??,
                is_root_device: true,
                is_read_only: false,
            },
        })
    }

    async fn get_kernel(download_kernel: bool) -> Result<PathBuf> {
        let env_path = env::var("FIRECRACKER_KERNEL");
        let env_download_path = env::var("FIRECRACKER_KERNEL_DOWNLOAD");
        let path = if env_path.is_ok() {
            PathBuf::from(env_path?)
        } else {
            Self::get_target_path(".firecracker/kernel", "vmlinux.bin")?
        };

        let download_path = if env_download_path.is_ok() {
            PathBuf::from(env_download_path?)
        } else {
            Self::get_download_path(".firecracker/rootfs")?
        };

        if download_kernel {
            let prefix = "firecracker-ci/v1.10/x86_64/vmlinux-5.10";

            let aws_s3 = AwsS3::new(prefix);
            let re = Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/vmlinux-5\.10\.\d{3})</Key>")?;
            let filename = path
                .to_string_lossy()
                .split('/')
                .next_back()
                .unwrap()
                .to_string();

            aws_s3.catch_latest(&download_path, re, &filename).await?;
        }

        Ok(path)
    }

    async fn get_rootfs(download_rootfs: bool) -> Result<PathBuf> {
        let env_path = env::var("FIRECRACKER_ROOTFS");
        let env_download_path = env::var("FIRECRACKER_ROOTFS_DOWNLOAD");
        let path = if env_path.is_ok() {
            PathBuf::from(env_path?)
        } else {
            Self::get_target_path(".firecracker/rootfs", "vmrootfs.ext4")?
        };

        let download_path = if env_download_path.is_ok() {
            PathBuf::from(env_download_path?)
        } else {
            Self::get_download_path(".firecracker/rootfs")?
        };

        if download_rootfs {
            let prefix = "firecracker-ci/v1.10/x86_64/ubuntu-22.04.ext4";

            let aws_s3 = AwsS3::new(prefix);
            let re = Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/ubuntu-22\.04\.ext4)</Key>")?;

            let filename = path
                .to_string_lossy()
                .split('/')
                .next_back()
                .unwrap()
                .to_string();

            aws_s3.catch_latest(&download_path, re, &filename).await?;
        }

        Ok(path)
    }

    fn get_target_path<P: AsRef<Path>>(base_path: P, item: &str) -> Result<PathBuf> {
        Ok(env::home_dir()
            .unwrap_or(
                env::current_exe()?
                    .parent()
                    .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                    .to_path_buf(),
            )
            .join(format!("{}/latest/{item}", base_path.as_ref().display())))
    }

    fn get_download_path<P: AsRef<Path>>(base_path: P) -> Result<PathBuf> {
        Ok(env::home_dir()
            .unwrap_or(
                env::current_exe()?
                    .parent()
                    .ok_or(anyhow::anyhow!("Current app is higher then root"))?
                    .to_path_buf(),
            )
            .join(format!("{}/download/", base_path.as_ref().display())))
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
