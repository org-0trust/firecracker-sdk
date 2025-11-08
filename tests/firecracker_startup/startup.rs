use anyhow::Result;
use firecracker_sdk::api::startup::FirecrackerStartup;

#[tokio::test]
async fn startup() -> Result<()> {
    let startup = FirecrackerStartup::new();
    println!("{}", startup.get_api_socket().display());
    let process = startup.start().await?;
    process.stop().await?;
    Ok(())
}
