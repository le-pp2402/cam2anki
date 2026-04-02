use std::{
    fmt::format,
    fs,
    io::{self, Write},
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use tokio::sync::Semaphore;

mod models;

use models::{CrawlJob, }

use crate::models::{Audio, Definition, Entry, Phonetic};
fn build_jobs_from_args() -> Vec<CrawlJob> {
    std::env::args()
        .skip(1)
        .map(|arg| CrawlJob {
            url: input_to_url(&arg),
            original_input: arg,
        })
        .collect()
}







//
async fn fetch_html(url: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RustCrawler/1.0)")
        .build()?;

    let response = client.get(url).send().await?;
    println!("Fetched: {}, with status code {}", &url, response.status());

    let html = response.text().await?;
    Ok(html)
}

fn ensure_out_dir() -> Result<()> {
    fs::create_dir_all("audio_out")?;
    Ok(())
}

// TODO: bổ sung thêm progress và hiển thị song song progress của các file
// TODO: Cải thiểu log và enhance log output
async fn download_file(url: &str, output_path: &str) -> Result<()> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RustCrawler/1.0)")
        .build()?;

    let response = client.get(url).send().await?;
    let response = response.error_for_status()?;
    println!("Download: {}, with status code {}", &url, response.status());
    let bytes = response.bytes().await?;
    fs::write(output_path, &bytes)?;
    Ok(())
}

fn sel(query: &str) -> Selector {
    Selector::parse(query).unwrap()
}

fn element_text(element: scraper::element_ref::ElementRef) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn to_absolute_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    format!("https://dictionary.cambridge.org{}", path)
}

fn parse_entry(html: &str) -> Entry {
    let document = Html::parse_document(html);

    // definition & example
    let def_block_selector = sel(".def-block");
    let def_selector = sel(".def");
    let example_selector = sel(".examp .eg");
    let mut definitions = Vec::new();

    for block in document.select(&def_block_selector) {
        let meaning = block
            .select(&def_selector)
            .next()
            .map(element_text)
            .unwrap_or_default();

        if meaning.is_empty() {
            continue;
        }

        let examples = block
            .select(&example_selector)
            .map(element_text)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        definitions.push(Definition { meaning, examples });
    }

    // word
    let dhw_selector = sel(".dhw");
    let word = document
        .select(&dhw_selector)
        .next()
        .map(element_text)
        .unwrap_or_else(|| "".to_string());

    // part of speech
    let pos_selector = sel(".pos");
    let part_of_speech = document.select(&pos_selector).next().map(element_text);

    // ipa
    let ipa_selector = sel(".ipa");
    let mut ipa_iter = document.select(&ipa_selector);
    let uk_phonetic = ipa_iter.next().map(element_text);
    let us_phonetic = ipa_iter.next().map(element_text);

    // audio
    let audio_mp3_selector = sel("audio source[type=\"audio/mpeg\"]");
    let mut audio_iter = document.select(&audio_mp3_selector);

    let uk_mp3 = audio_iter
        .next()
        .and_then(|e| e.value().attr("src"))
        .map(to_absolute_url);

    let us_mp3 = audio_iter
        .next()
        .and_then(|e| e.value().attr("src"))
        .map(to_absolute_url);

    Entry {
        word,
        part_of_speech,
        phonetic: Phonetic {
            uk: uk_phonetic,
            us: us_phonetic,
        },
        definitions,
        audio: Audio {
            uk: uk_mp3,
            us: us_mp3,
        },
    }
}

fn build_audio_output_path(word: &str, region: &str) -> String {
    let safe_word = word
        .to_lowercase()
        .replace(' ', "_")
        .replace('/', "_")
        .replace('-', "_");
    format!("audio_out/{}_{}.mp3", safe_word, region)
}

async fn download_audio_files(entry: &mut Entry) -> Result<()> {
    if let Some(ref uk_url) = entry.audio.uk {
        let uk_path = build_audio_output_path(&entry.word, "uk");

        if !Path::new(&uk_path).exists() {
            download_file(uk_url, &uk_path).await?;
        }
    }

    if let Some(ref us_url) = entry.audio.us {
        let us_path = build_audio_output_path(&entry.word, "us");

        if !Path::new(&us_path).exists() {
            download_file(us_url, &us_path).await?;
        }
    }

    Ok(())
}

async fn process_job(job: CrawlJob) -> Result<Entry> {
    let html = fetch_html(&job.url).await?;
    let mut word = parse_entry(&html);

    download_audio_files(&mut word).await?;

    // println!("{:?}", serde_json::to_string(&word));
    Ok(word)
}

#[tokio::main]
async fn main() -> Result<()> {
    ensure_out_dir()?;

    let jobs = collect_jobs()?;

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

            let result = process_job(job).await;
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
