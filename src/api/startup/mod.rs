use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Serialize;
use tempfile::tempdir;

use crate::{
    domain::config::{BootSource, Drives, FirecrackerConfiguration, VSock},
    infrastructure::{fs::FileManager, process::FirecrackerProcess, s3::S3Downloader},
};

/// A structure for configuring the launch of FirecrackerVM. Helps to preconfigure and start the virtual machine.
///
/// Note: Firecracker must be installed globally.
///
/// Exemple:
/// ```no_compile
/// let process = FirecrackerStartup::new()
///     .set_api_socket("/tmp/some.socket")
///     .download_rootfs(true)
///     .download_kernel(true)
///     .start().await.unwrap();
/// ```
#[derive(Serialize)]
pub struct FirecrackerStartup {
    api_socket: PathBuf,
    vsock: PathBuf,
    stdout: bool,
    download_kernel: bool,
    download_rootfs: bool,
}

impl FirecrackerStartup {
    /// Creates a new instance of FirecrackerStartup
    pub fn new() -> Self {
        let mut tempdir = tempdir().unwrap();
        tempdir.disable_cleanup(true);
        Self {
            api_socket: tempdir.path().join("firecracker.socket"),
            download_kernel: false,
            download_rootfs: false,
            stdout: false,
            vsock: tempdir.path().join("vsock.socket"),
        }
    }

    /// Set the --api-sock startup argument with the path to the unix socket
    ///
    /// Note: For the best documentation, please refer to [here](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md).
    pub fn set_api_socket<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.api_socket = path.as_ref().to_path_buf();
        self
    }

    /// Returns the --api-sock startup argument with the path to the unix socket
    ///
    /// Note: For the best documentation, please refer to [here](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md).
    pub fn get_api_socket(&self) -> &PathBuf {
        &self.api_socket
    }

    /// Flag to enable/disable vm's stdout
    pub fn stdout(mut self, flag: bool) -> Self {
        self.stdout = flag;
        self
    }

    /// Set vsock path for vm
    pub fn vsocket<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.vsock = path.as_ref().to_path_buf();
        self
    }

    /// Returns current flag of stdout
    pub fn current_stdout(&self) -> bool {
        self.stdout
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

    /// Starts a VM with specified parameters
    /// Returns a structure for working with the Firecracker process
    pub async fn start(self) -> Result<FirecrackerProcess> {
        let fs = FileManager::default();
        let s3 = S3Downloader::default();
        let kernel_path = fs.resolve_kernel_path(self.download_kernel, &s3).await?;
        let rootfs_path = fs.resolve_rootfs_path(self.download_rootfs, &s3).await?;

        FirecrackerProcess::new(FirecrackerConfiguration {
            boot_source: BootSource {
                kernel_image_path: kernel_path,
                boot_args: HashMap::new(),
            },
            drives: Drives {
                drive_id: "rootfs".into(),
                path_on_host: rootfs_path,
                is_root_device: true,
                is_read_only: false,
            },
            vsock: VSock {
                vsock_id: "vsock0".into(),
                guest_cid: 3,
                uds_path: self.vsock.to_string_lossy().to_string(),
            },
            startup_config: self,
        })
        .await
    }
}

impl Default for FirecrackerStartup {
    fn default() -> Self {
        Self::new()
    }
}
