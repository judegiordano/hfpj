use anyhow::Result;
use clap::Parser;
use futures::{future, StreamExt};
use parquet::file::reader::{FileReader, SerializedFileReader};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use serde_json::Value;
use std::{io::Write, sync::mpsc, time::Instant};
use tempfile::tempfile;

use crate::{args::Arguments, flush, hugging_face::HuggingFace, progress::progress_bar};

async fn spawn_worker(client: &Client, parquet_url: &str) -> Result<Vec<Value>> {
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
    let pb = progress_bar(iter_size.try_into()?)?;
    let par_iter = iter.into_par_iter().map(|row| {
        let row = match row {
            Ok(row) => row,
            Err(err) => {
                eprintln!("[ERROR PARSING PARQUET] {:?}", err);
                std::process::exit(1);
            }
        };
        pb.inc(1);
        row.to_json_value()
    });
    pb.finish_with_message("all rows parsed");
    Ok(par_iter.collect::<Vec<_>>())
}

pub async fn run() -> Result<()> {
    let args = Arguments::parse();
    let file_name = args.out.unwrap_or(args.dataset.to_string());
    // new hugging face rest client
    let hf = HuggingFace::new()?;
    let start = Instant::now();
    let parquet_links = hf.get_parquets(&args.username, &args.dataset).await?;
    println!("-> matched {:?} parquet files", parquet_links.len());
    let mut handlers = vec![];
    let (tx, rx) = mpsc::channel();
    for link in parquet_links {
        let client = hf.client.clone();
        let tx = tx.clone();
        let task = tokio::task::spawn(async move {
            let values = match spawn_worker(&client, &link).await {
                Ok(values) => values,
                Err(_) => todo!(),
            };
            match tx.send(values) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("[ERROR SENDING ACROSS THREADS] {:?}", err);
                }
            }
        });
        handlers.push(task);
    }
    let mut received_data = vec![];
    for _ in 0..handlers.len() {
        let received = rx.recv()?;
        println!("-> received {} rows", received.len());
        received_data.push(received);
    }
    future::try_join_all(handlers).await?;
    let concat_data = received_data.concat();
    println!("-> total items received {}", concat_data.len());
    println!("-> writing to '{file_name}.json'...");
    flush::to_local_file(&concat_data, &file_name)?;
    println!("-> operation complete in {:?}", start.elapsed());
    println!("-> cleaning up remaining resources...");
    Ok(())
}
