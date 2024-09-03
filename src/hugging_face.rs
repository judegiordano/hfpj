use anyhow::Result;
use reqwest::{header::HeaderMap, Client as HttpCLient};
use std::sync::Arc;

const API_URL: &str = "https://huggingface.co/api";

pub struct Client {
    pub http_client: Arc<HttpCLient>,
}

impl Client {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0"
                .try_into()?,
        );
        Ok(Self {
            http_client: Arc::new(HttpCLient::builder().default_headers(headers).build()?),
        })
    }

    pub async fn get_parquet_links(
        &self,
        username: &str,
        dataset_name: &str,
        split: &str,
    ) -> Result<Vec<String>> {
        let url = format!("{API_URL}/datasets/{username}/{dataset_name}/parquet/{split}");
        let response = self.http_client.get(url).send().await?;
        Ok(response.json().await?)
    }
}