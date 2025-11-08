use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;
use tempfile::tempdir;

#[tokio::test]
async fn startup_with_args() -> Result<()> {
    let dir = tempdir()?;
    let startup = FirecrackerStartup::new().set_api_socket(dir.path().join("test.socket"));
    let process = startup.start().await?;
    process.stop().await?;
    dir.close()?;
    Ok(())
}
