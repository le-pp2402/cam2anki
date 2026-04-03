use std::sync::Arc;

use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

use tokio::sync::Semaphore;

mod downloader;
mod input;
mod models;
mod scraper;
mod utils;

async fn crawl() -> Result<()> {
    downloader::ensure_out_dir()?;

    let jobs = input::collect_jobs()?;

    if jobs.is_empty() {
        println!("No input provided!");
        return Ok(());
    }

    let total = jobs.len() as u64;
    let pb = ProgressBar::new(total);

    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("@@-"),
    );

    pb.set_message("starting");

    let semaphore = Arc::new(Semaphore::new(5));
    let mut tasks = FuturesUnordered::new();

    for job in jobs {
        let semaphore = Arc::clone(&semaphore);
        let pb = pb.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await?;
            let label = job.original_input.clone();

            let result = input::process_job(job).await;
            pb.inc(1);

            match &result {
                Ok(entry) => {
                    pb.println(format!("OK   {:<20} -> {}", label, entry.word));
                }
                Err(err) => {
                    pb.println(format!("FAIL {:<20} -> {}", label, err));
                }
            }

            result
        }));
    }

    let mut success_count = 0usize;
    let mut failure_count = 0usize;

    while let Some(join_result) = tasks.next().await {
        match join_result {
            Ok(Ok(_entry)) => {
                success_count += 1;
            }
            Ok(Err(_err)) => {
                failure_count += 1;
            }
            Err(err) => {
                failure_count += 1;
                pb.println(format!("Task join error: {}", err));
            }
        }
    }

    pb.finish_with_message("done");
    println!();
    println!("Summary:");
    println!("  Success: {}", success_count);
    println!("  Failed : {}", failure_count);

    Ok(())
}

async fn anki_connect() {}

async fn get_deck() {}

#[tokio::main]
async fn main() -> Result<()> {
    crawl().await?;
    Ok(())
}
