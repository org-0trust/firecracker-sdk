use std::{env, time::Duration};

use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;
use tempfile::tempdir;

#[tokio::test]
async fn startup_w_start_vm() -> Result<()> {
    let dir = tempdir()?;
    unsafe {
        env::set_var("FIRECRACKER_KERNEL", dir.path());
        env::set_var("FIRECRACKER_ROOTFS", dir.path());
    }
    let startup = FirecrackerStartup::new()
        .download_kernel(true)
        .download_rootfs(true)
        .stdout(true);
    let mut process = startup.start().await?;
    assert!(process.config().kernel_image_path().exists());
    assert!(process.config().drive_path().exists());

    println!("Start...");
    let res = process.start_vm().await?;
    println!("{res:?}");
    println!("Delay...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("{}", process.stdout().await?);

    process.stop().await?;
    Ok(())
}
