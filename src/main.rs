mod args;
mod flush;
mod handler;
mod hugging_face;
mod progress;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    handler::run().await
}
