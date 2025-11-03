use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;

#[derive(Serialize)]
pub struct FirecrackerConfiguration {
    pub(crate) boot_source: BootSource,
    pub(crate) drives: Drives,
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
