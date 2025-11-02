use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;

#[derive(Serialize)]
pub struct FirecrackerConfiguration {
    pub(crate) boot_source: BootSource,
}

impl FirecrackerConfiguration {
    pub fn kernel_image_path(&self) -> PathBuf {
        self.boot_source.kernel_image_path.clone()
    }
}

#[derive(Serialize)]
pub(crate) struct BootSource {
    pub(crate) kernel_image_path: PathBuf,
    pub(crate) boot_args: HashMap<String, String>,
}
