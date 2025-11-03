use anyhow::Result;
use reqwest::Client;

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
}
