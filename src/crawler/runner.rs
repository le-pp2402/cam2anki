use anyhow::Result;

use crate::{
    cam::{
        downloader::download_audio_files,
        model::{CrawlJob, Entry},
        scraper,
    },
};

pub async fn process_job(job: CrawlJob) -> Result<Entry> {
    let html = scraper::fetch_html(&job.url).await?;
    let mut word = scraper::parse_entry(&html);
    download_audio_files(&mut word).await?;
    Ok(word)
}