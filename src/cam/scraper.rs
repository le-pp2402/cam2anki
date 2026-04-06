use crate::cam::model::{Audio, Definition, Entry, Phonetic};
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

pub async fn fetch_html(url: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RustCrawler/1.0)")
        .build()?;

    let response = client.get(url).send().await?;
    println!("Fetched: {}, with status code {}", &url, response.status());

    let html = response.text().await?;
    Ok(html)
}

pub fn parse_entry(html: &str) -> Entry {
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
