use anyhow::Result;
use args::Arguments;
use clap::Parser;

mod args;

fn main() -> Result<()> {
    let args = Arguments::parse();
    Ok(())
}
