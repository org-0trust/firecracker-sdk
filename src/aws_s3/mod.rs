use std::path::Path;

use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use tokio::fs;

pub struct AwsS3 {
    xml_base_path: String,
    prefix: String,
    download_base_path: String,
}

impl AwsS3 {
    pub fn new(prefix: &str) -> Self {
        Self {
            xml_base_path: "http://spec.ccfc.min.s3.amazonaws.com".into(),
            prefix: prefix.into(),
            download_base_path: "https://s3.amazonaws.com/spec.ccfc.min".into(),
        }
    }

    pub async fn xml(&self) -> Result<String> {
        Ok(Client::new()
            .get(format!(
                "{}/?prefix={}&list-type=2",
                self.xml_base_path, self.prefix
            ))
            .send()
            .await?
            .text()
            .await?)
    }

    pub async fn download(&self, item: &str) -> Result<Vec<u8>> {
        Ok(Client::new()
            .get(format!("{}/{}", self.download_base_path, item))
            .send()
            .await?
            .bytes()
            .await?
            .into())
    }

    pub async fn catch_latest<P: AsRef<Path>>(
        &self,
        download_dir: P,
        re: Regex,
        target_filename: &str,
    ) -> Result<()> {
        fs::create_dir_all(&download_dir).await?;

        let xml = self.xml().await?;
        let mut versions: Vec<_> = re.captures_iter(&xml).map(|m| m[1].to_string()).collect();
        if versions.is_empty() {
            anyhow::bail!("Could not find any version");
        }

        versions.sort();
        let latest = versions.last().unwrap();

        let file_name = latest.split('/').next_back().unwrap();
        let file_path = download_dir.as_ref().join(file_name);

        if file_path.exists() {
            println!("Already download: {}", file_path.display());
            return Ok(());
        }

        let bytes = self.download(latest).await?;
        fs::write(&file_path, &bytes).await?;

        println!("Download successfull: {}", file_path.display());

        let target_path = download_dir.as_ref().parent().unwrap().join("latest");
        fs::create_dir_all(&target_path).await?;

        fs::copy(&file_path, target_path.join(target_filename)).await?;
        Ok(())
    }
}
