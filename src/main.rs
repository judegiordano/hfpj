use anyhow::Result;
use args::Arguments;
use clap::Parser;

mod args;
mod hugging_face;

fn main() -> Result<()> {
    let args = Arguments::parse();
    Ok(())
}
