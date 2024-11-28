use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_bar(size: u64) -> Result<ProgressBar> {
    Ok(ProgressBar::new(size).with_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {msg}",
        )?
        .progress_chars("##-"),
    ))
}
