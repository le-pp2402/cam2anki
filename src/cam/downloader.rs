use crate::cam::model::Entry;
use anyhow::Result;
use reqwest::Client;
use std::{fs, path::Path};

/*
*   ensure /audio_out dir created
*/
pub fn ensure_out_dir() -> Result<()> {
    fs::create_dir_all("audio_out")?;
    Ok(())
}

/*
*  normalize auto output file
*/
fn build_audio_basename(word: &str, region: &str) -> String {
    let safe_word = word
        .to_lowercase()
        .replace(' ', "_")
        .replace('/', "_")
        .replace('-', "_");
    format!("{}_{}.mp3", safe_word, region)
}

pub fn build_audio_output_path(word: &str, region: &str) -> String {
    format!("audio_out/{}", build_audio_basename(word, region))
}

pub fn build_audio_filename(word: &str, region: &str) -> String {
    build_audio_basename(word, region)
}

/*
*  download audio.mp3 file
*/
// TODO: Cải thiểu log và enhance log output
pub async fn download_file(url: &str, output_path: &str) -> Result<()> {
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

pub async fn download_audio_files(entry: &mut Entry) -> Result<()> {
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
