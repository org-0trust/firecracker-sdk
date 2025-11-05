use anyhow::Result;
use regex::Regex;
use reqwest::Client;

pub enum S3Item {
    Rootfs,
    Kernel,
}

pub struct S3Downloader {
    xml_path: String,
    download_path: String,
    kernel_prefix: (String, Regex),
    rootfs_prefix: (String, Regex),
}

impl Default for S3Downloader {
    fn default() -> Self {
        Self {
            kernel_prefix: (
                "firecracker-ci/v1.10/x86_64/vmlinux-5.10".into(),
                Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/vmlinux-5\.10\.\d{3})</Key>")
                    .unwrap(),
            ),
            rootfs_prefix: (
                "firecracker-ci/v1.10/x86_64/ubuntu-22.04.ext4".into(),
                Regex::new(r"<Key>(firecracker-ci/v1\.10/x86_64/ubuntu-22\.04\.ext4)</Key>")
                    .unwrap(),
            ),
            xml_path: "http://spec.ccfc.min.s3.amazonaws.com".into(),
            download_path: "https://s3.amazonaws.com/spec.ccfc.min".into(),
        }
    }
}

impl S3Downloader {
    pub fn set_ketnel_prefix(mut self, kernel_prefix: &str, kernel_regex: Regex) -> Self {
        self.kernel_prefix = (kernel_prefix.to_string(), kernel_regex);
        self
    }

    pub fn set_rootfs_prefix(mut self, rootfs_prefix: &str, rootfs_regex: Regex) -> Self {
        self.rootfs_prefix = (rootfs_prefix.to_string(), rootfs_regex);
        self
    }

    async fn download_item(&self, item: &str) -> Result<Vec<u8>> {
        Ok(Client::new()
            .get(format!("{}/{}", self.download_path, item))
            .send()
            .await?
            .bytes()
            .await?
            .into())
    }

    async fn xml(&self, prefix: &str) -> Result<String> {
        Ok(Client::new()
            .get(format!("{}/?prefix={}&list-type=2", self.xml_path, prefix))
            .send()
            .await?
            .text()
            .await?)
    }

    pub async fn download(&self, s3_item: S3Item) -> Result<Vec<u8>> {
        let mut versions: Vec<_> = match s3_item {
            S3Item::Rootfs => {
                let xml = self.xml(&self.rootfs_prefix.0).await?;
                self.rootfs_prefix
                    .1
                    .captures_iter(&xml)
                    .map(|m| m[1].to_string())
                    .collect()
            }
            S3Item::Kernel => {
                let xml = self.xml(&self.kernel_prefix.0).await?;
                self.kernel_prefix
                    .1
                    .captures_iter(&xml)
                    .map(|m| m[1].to_string())
                    .collect()
            }
        };
        if versions.is_empty() {
            anyhow::bail!("Could not find any version");
        }
        versions.sort();
        let latest = versions.last().unwrap();
        self.download_item(latest).await
    }
}
