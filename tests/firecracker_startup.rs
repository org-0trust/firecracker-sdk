use std::env;

use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;
use tempfile::tempdir;

#[tokio::test]
async fn startup() -> Result<()> {
    let startup = FirecrackerStartup::new();
    println!("{}", startup.get_api_socket().display());
    let process = startup.start().await?;
    process.stop().await?;
    Ok(())
}

#[tokio::test]
async fn startup_with_args() -> Result<()> {
    let dir = tempdir()?;
    let startup = FirecrackerStartup::new().set_api_socket(dir.path().join("test.socket"));
    let process = startup.start().await?;
    process.stop().await?;
    dir.close()?;
    Ok(())
}

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
    assert!(process.get_config().kernel_image_path().exists());
    assert!(process.get_config().drive_path().exists());
    process.stop().await?;
    Ok(())
}
