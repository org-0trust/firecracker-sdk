use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;

use crate::api::startup::FirecrackerStartup;

#[derive(Serialize)]
pub struct FirecrackerConfiguration {
    pub(crate) startup_config: FirecrackerStartup,
    pub(crate) boot_source: BootSource,
    pub(crate) drives: Drives,
    pub(crate) vsock: VSock,
}

impl FirecrackerConfiguration {
    pub fn kernel_image_path(&self) -> PathBuf {
        self.boot_source.kernel_image_path.clone()
    }

    pub fn drive_path(&self) -> PathBuf {
        self.drives.path_on_host.clone()
    }
}

#[derive(Serialize)]
pub(crate) struct BootSource {
    pub(crate) kernel_image_path: PathBuf,
    pub(crate) boot_args: HashMap<String, String>,
}

#[derive(Serialize)]
pub(crate) struct Drives {
    pub(crate) drive_id: String,
    pub(crate) path_on_host: PathBuf,
    pub(crate) is_root_device: bool,
    pub(crate) is_read_only: bool,
}

#[derive(Serialize)]
pub(crate) struct VSock {
    pub(crate) vsock_id: String,
    pub(crate) guest_cid: usize,
    pub(crate) uds_path: String,
}
