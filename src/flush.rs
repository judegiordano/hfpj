use anyhow::Result;
use serde::Serialize;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

pub fn to_local_file<T: Serialize>(data: &T, file_name: &str) -> Result<()> {
    let name = format!("{file_name}.json");
    let file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open(name)?;
    let serialized = serde_json::to_string_pretty(&data)?;
    let mut writer = BufWriter::new(&file);
    writeln!(writer, "{}", serialized)?;
    Ok(())
}
