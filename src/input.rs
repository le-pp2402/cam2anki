use crate::downloader;
use crate::models::{CrawlJob, Entry};
use crate::scraper;
use crate::utils;
use anyhow::Result;
use std::io::{self, Write};

fn build_jobs_from_stdin() -> Result<Vec<CrawlJob>> {
    println!("Enter words or URLs, one per line. Submit empty line to finish.");
    print!("> ");
    io::stdout().flush()?;

    let mut jobs = Vec::new();

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        jobs.push(CrawlJob {
            original_input: line.to_string(),
            url: utils::input_to_url(line),
        });

        print!("> ");
        io::stdout().flush()?;
    }

    Ok(jobs)
}

fn build_jobs_from_args() -> Vec<CrawlJob> {
    std::env::args()
        .skip(1)
        .map(|arg| CrawlJob {
            url: utils::input_to_url(&arg),
            original_input: arg,
        })
        .collect()
}

pub fn collect_jobs() -> Result<Vec<CrawlJob>> {
    let jobs = build_jobs_from_args();

    if !jobs.is_empty() {
        return Ok(jobs);
    }

    build_jobs_from_stdin()
}

pub async fn process_job(job: CrawlJob) -> Result<Entry> {
    let html = scraper::fetch_html(&job.url).await?;
    let mut word = scraper::parse_entry(&html);

    downloader::download_audio_files(&mut word).await?;

    Ok(word)
}
