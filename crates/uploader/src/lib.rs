use std::{env, path::Path};

use anyhow::{Context, Ok, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{Client, config::Credentials};
use tokio::fs;

pub struct S3Uploader {
    bucket: String,
    client: Client,
}

impl S3Uploader {
    pub async fn new() -> Result<Self> {
        let bucket = env::var("S3_BUCKET")?;
        let endpoint = env::var("S3_ENDPOINT")?;
        let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let access = env::var("S3_ACCESS_KEY")?;
        let secret = env::var("S3_SECRET_KEY")?;

        let creds = Credentials::new(&access, &secret, None, None, "uploader");

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(creds)
            .endpoint_url(&endpoint)
            .region(aws_sdk_s3::config::Region::new(region))
            .load()
            .await;

        let client = Client::new(&config);

        Ok(Self { bucket, client })
    }

    pub async fn upload(&self, file_path: &str, key: Option<&str>) -> Result<()> {
        let content = fs::read(file_path)
            .await
            .with_context(|| format!("failed to read file: {}", file_path))?;

        let s3_key = key.map(|k| k.to_string()).unwrap_or_else(|| {
            Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&s3_key)
            .body(content.into());

        request
            .send()
            .await
            .with_context(|| format!("failed to upload file: {}", s3_key))?;

        Ok(())
    }
}
