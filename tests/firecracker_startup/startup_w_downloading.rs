use std::env;

use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;
use tempfile::tempdir;

#[tokio::test]
async fn startup_with_downloading() -> Result<()> {
    let dir = tempdir()?;
    unsafe {
        env::set_var("FIRECRACKER_KERNEL", dir.path());
        env::set_var("FIRECRACKER_ROOTFS", dir.path());
    }
    let startup = FirecrackerStartup::new()
        .download_kernel(true)
        .download_rootfs(true);
    let process = startup.start().await?;
    assert!(process.config().kernel_image_path().exists());
    assert!(process.config().drive_path().exists());
    process.stop().await?;
    Ok(())
}
