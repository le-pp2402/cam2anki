use std::sync::Arc;

use crate::cam::{
    self,
    downloader::{download_audio_files, ensure_out_dir},
    model::{CrawlJob, Entry},
    util,
};
use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::Semaphore;

// Get list word
fn build_jobs(words: Vec<String>) -> Result<Vec<CrawlJob>> {
    let mut jobs = Vec::new();

    for word in words {
        jobs.push(CrawlJob {
            original_input: word.clone(),
            url: util::input_to_url(&word),
        });
    }

    Ok(jobs)
}

// crawl & download audio file
async fn process_job(job: CrawlJob) -> Result<Entry> {
    let html = cam::scraper::fetch_html(&job.url).await?;
    let mut word = cam::scraper::parse_entry(&html);

    download_audio_files(&mut word).await?;

    Ok(word)
}

// handle progress and execute job
async fn crawl(jobs: Vec<CrawlJob>) -> Result<()> {
    ensure_out_dir()?;

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

            let result = cam::clawler::process_job(job).await;
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
