use anyhow::Result;
use args::Arguments;
use clap::Parser;
use futures::{future, StreamExt};
use hugging_face::HuggingFace;
use indicatif::ProgressBar;
use parquet::file::reader::{FileReader, SerializedFileReader};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use serde_json::Value;
use std::{
    io::Write,
    sync::mpsc::{self},
    time::Instant,
};
use tempfile::tempfile;

mod args;
mod flush;
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
    let iter_size = iter.len();
    let pb = ProgressBar::new(iter_size as u64);
    let par_iter = iter.into_par_iter().map(|row| {
        let row = match row {
            Ok(row) => row,
            Err(_) => todo!(),
        };
        pb.inc(1);
        row.to_json_value()
    });
    pb.finish_with_message("all rows parsed");
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
            let values = match spawn_handler(&client, &link).await {
                Ok(values) => values,
                Err(_) => todo!(),
            };
            tx.send(values);
        });
        handlers.push(task);
    }
    let mut received_data = vec![];
    for _ in 0..handlers.len() {
        let received = rx.recv()?;
        println!("✅ received {} rows", received.len());
        received_data.push(received);
    }
    future::try_join_all(handlers).await?;
    let concat_data = received_data.concat();
    println!("✅ total items received {}", concat_data.len());
    println!("✍  writing to {file_name}.json");
    flush::to_local_file(&concat_data, &file_name)?;
    println!("operation complete in {:?}", start.elapsed());
    println!("cleaning up remaining resources...");
    Ok(())
}
