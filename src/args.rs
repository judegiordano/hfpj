use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// HuggingFace Username
    #[arg(short, long)]
    pub username: String,
    /// HuggingFace dataset name
    #[arg(short, long)]
    pub dataset: String,
    /// Output name of data json file
    #[arg(short, long)]
    pub out: Option<String>,
}
