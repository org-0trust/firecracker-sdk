use anyhow::Result;
use firecracker_sdk::vm::{
    firercracker_process::firecracker_startup::FirecrackerStartup, vm_socket::VMSocket,
};
use tempfile::tempdir;

#[tokio::test]
async fn startup() -> Result<()> {
    let startup = FirecrackerStartup::new();
    let mut process = startup.start()?;
    process.stop().await?;
    Ok(())
}

#[tokio::test]
async fn startup_with_args() -> Result<()> {
    let dir = tempdir()?;
    let startup = FirecrackerStartup::new().api_socket(dir.path().join("test.socket"));
    let mut process = startup.start()?;
    process.stop().await?;
    dir.close()?;
    Ok(())
}

#[tokio::test]
async fn startup_with_connection() -> Result<()> {
    let dir = tempdir()?;
    let socket_path = dir.path().join("test.socket");
    let startup = FirecrackerStartup::new().api_socket(&socket_path);
    let mut process = startup.start()?;

    let vm_socket = VMSocket::new()?;
    let vm = vm_socket.connect(&socket_path).await?;
    vm.close().await?;

    process.stop().await?;
    dir.close()?;
    Ok(())
}
