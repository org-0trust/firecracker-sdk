use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;

#[tokio::test]
async fn startup_wo_stdout() -> Result<()> {
    let mut process = FirecrackerStartup::new().stdout(false).start().await?;
    assert_eq!(process.stdout().await?, String::new());
    Ok(())
}
