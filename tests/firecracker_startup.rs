use std::{env, time::Duration};

use anyhow::Result;
use firecracker_sdk::firecracker::{
    firecracker_socket::FirecrackerSocket,
    firercracker_process::firecracker_startup::FirecrackerStartup,
};
use tempfile::tempdir;

#[tokio::test]
async fn startup() -> Result<()> {
    let startup = FirecrackerStartup::new();
    let mut process = startup.start().await?;
    process.stop().await?;
    Ok(())
}

#[tokio::test]
async fn startup_with_args() -> Result<()> {
    let dir = tempdir()?;
    let startup = FirecrackerStartup::new().api_socket(dir.path().join("test.socket"));
    let mut process = startup.start().await?;
    process.stop().await?;
    dir.close()?;
    Ok(())
}

#[tokio::test]
async fn startup_with_connection() -> Result<()> {
    let dir = tempdir()?;
    let socket_path = dir.path().join("test.socket");
    let startup = FirecrackerStartup::new().api_socket(&socket_path);
    let mut process = startup.start().await?;

    tokio::time::sleep(Duration::from_millis(150)).await;
    let socket = FirecrackerSocket::new()?;

    let stream = socket.connect(&socket_path).await?;
    stream.close().await?;

    process.stop().await?;
    dir.close()?;
    Ok(())
}

#[tokio::test]
async fn startup_with_downloading() -> Result<()> {
    let dir = tempdir()?;
    unsafe {
        env::set_var("FIRECRACKER_KERNEL_DOWNLOAD", dir.path().join("download"));
        env::set_var("FIRECRACKER_KERNEL", dir.path().join("latest/vmlinux.bin"));
        env::set_var("FIRECRACKER_ROOTFS_DOWNLOAD", dir.path().join("download"));
        env::set_var(
            "FIRECRACKER_ROOTFS",
            dir.path().join("latest/ubuntu-22.04.ext4"),
        );
    }
    let startup = FirecrackerStartup::new()
        .download_kernel(true)
        .download_rootfs(true);
    let mut process = startup.start().await?;
    process.stop().await?;

    assert!(process.get_config().kernel_image_path().exists());
    assert!(process.get_config().drive_path().exists());
    Ok(())
}
