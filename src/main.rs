use anyhow::Result;
use args::Arguments;
use clap::Parser;
use futures::{future, StreamExt};
use hugging_face::HuggingFace;
use parquet::file::reader::{FileReader, SerializedFileReader};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelBridge,
    ParallelIterator,
};
use reqwest::Client;
use serde_json::Value;
use std::{
    io::Write,
    sync::mpsc::{self, Sender},
    time::Instant,
};
use tempfile::tempfile;

mod args;
mod hugging_face;

async fn spawn_handler(client: &Client, parquet_url: &str) -> Result<Vec<Value>> {
    let mut file = tempfile()?;
    // stream parquet bytes to temp file
    let response = client.get(parquet_url).send().await?;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write(&chunk)?;
    }
    // read from temp file
    let reader = SerializedFileReader::new(file)?;
    let rows = reader.get_row_iter(None)?;
    let iter = rows.collect::<Vec<_>>();
    let _ = iter.len();
    let par_iter = iter.into_par_iter().map(|row| {
        let row = match row {
            Ok(row) => row,
            Err(_) => todo!(),
        };
        row.to_json_value()
    });
    Ok(par_iter.collect::<Vec<_>>())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    let file_name = args.out.unwrap_or(args.dataset.to_string());
    let hugging_face = HuggingFace::new()?;
    let start = Instant::now();
    let parquet_links = hugging_face
        .get_parquet_links(&args.username, &args.dataset, &args.split)
        .await?;
    let mut handlers = vec![];
    let (tx, rx) = mpsc::channel();
    for link in parquet_links {
        let client = hugging_face.client.clone();
        let tx = tx.clone();
        let task = tokio::task::spawn(async move {
            match spawn_handler(&client, &link).await {
                Ok(values) => tx.send(values).unwrap(),
                Err(_) => todo!(),
            };
        });
        handlers.push(task);
    }
    let mut received_data = vec![];
    for _ in 0..handlers.len() {
        let received = rx.recv()?;
        // println!("✅ received {} rows", received.len());
        received_data.push(received);
    }
    future::try_join_all(handlers).await?;
    let concat_data = received_data.concat();
    println!("✅ total items received {}", concat_data.len());
    println!("operation complete in {:?}", start.elapsed());
    Ok(())
}
