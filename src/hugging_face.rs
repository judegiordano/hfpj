use anyhow::Result;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use std::sync::Arc;

const API_URL: &str = "https://huggingface.co/api";
const DATASETS_API_URL: &str = "https://datasets-server.huggingface.co";

pub struct HuggingFace {
    pub client: Arc<Client>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SplitDataSet {
    #[allow(unused)]
    pub dataset: String,
    pub config: String,
    pub split: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SplitResponse {
    pub splits: Vec<SplitDataSet>,
}

impl HuggingFace {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0"
                .try_into()?,
        );
        Ok(Self {
            client: Arc::new(Client::builder().default_headers(headers).build()?),
        })
    }

    pub async fn get_parquets(&self, username: &str, dataset_name: &str) -> Result<Vec<String>> {
        let default_split = self.get_split_names(username, dataset_name).await?;
        let split = format!("{}/{}", default_split.config, default_split.split);
        println!("-> training off of {:?} split files", split);
        let url = format!("{API_URL}/datasets/{username}/{dataset_name}/parquet/{split}");
        let response = self.client.get(url).send().await?;
        Ok(response.json().await?)
    }

    pub async fn get_split_names(
        &self,
        username: &str,
        dataset_name: &str,
    ) -> Result<SplitDataSet> {
        let url = format!("{DATASETS_API_URL}/splits?dataset={username}/{dataset_name}");
        let response = self.client.get(url).send().await?;
        let SplitResponse { splits } = response.json().await?;
        Ok(splits.first().unwrap().to_owned())
    }
}
