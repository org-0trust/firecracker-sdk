use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;

#[tokio::test]
async fn startup_w_stdout() -> Result<()> {
    let mut process = FirecrackerStartup::new().stdout(true).start().await?;
    let out = process.stdout().await?;
    assert!(!out.is_empty());
    println!("{out}");
    Ok(())
}
