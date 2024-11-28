use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_bar(size: u64) -> Result<ProgressBar> {
    Ok(ProgressBar::new(size).with_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?
        .progress_chars("##-"),
    ))
}
