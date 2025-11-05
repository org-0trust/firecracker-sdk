use std::{env, path::PathBuf};

use anyhow::Result;
use tokio::fs;

use crate::domain::s3::{S3Downloader, S3Item};

pub struct FileManager {
    kernel_path: PathBuf,
    rootfs_path: PathBuf,
}

impl FileManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            kernel_path: PathBuf::from(env::var("FIRECRACKER_KERNEL")?),
            rootfs_path: PathBuf::from(env::var("FIRECRACKER_ROOTFS")?),
        })
    }
    pub async fn resolve_kernel_path(
        &self,
        download_kernel: bool,
        s3: &S3Downloader,
    ) -> Result<PathBuf> {
        let target = self.kernel_path.join("vmlinux.bin");

        if download_kernel {
            let bytes = s3.download(S3Item::Kernel).await?;
            fs::write(&target, bytes).await?
        }

        Ok(target)
    }
    pub async fn resolve_rootfs_path(
        &self,
        download_rootfs: bool,
        s3: &S3Downloader,
    ) -> Result<PathBuf> {
        let target = self.rootfs_path.join("vmrootfs.ext4");

        if download_rootfs {
            let bytes = s3.download(S3Item::Kernel).await?;
            fs::write(&target, bytes).await?
        }

        Ok(target)
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self {
            kernel_path: env::home_dir()
                .unwrap_or(env::current_exe().unwrap().parent().unwrap().to_path_buf())
                .join(".firecracker/kernel/target/"),
            rootfs_path: env::home_dir()
                .unwrap_or(env::current_exe().unwrap().parent().unwrap().to_path_buf())
                .join(".firecracker/rootfs/target/"),
        }
    }
}
