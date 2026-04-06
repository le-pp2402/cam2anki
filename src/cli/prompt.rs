use crate::anki::util::norm_anki_base_url;
use anyhow::Result;
use dialoguer::{Input, theme::ColorfulTheme};

const DEFAULT_ANKI_CONNECT_URL: &str = "http://127.0.0.1:8765";

pub fn ask_anki_connect_url() -> Result<String> {
    let input = Input::new()
        .with_prompt("Enter AnkiConnect URL (default: http://localhost:8765):")
        .default(DEFAULT_ANKI_CONNECT_URL.to_string())
        .interact_text()?;

    Ok(norm_anki_base_url(input.trim()).to_string())
}

pub fn ask_words() -> Vec<String> {
    let mut words = Vec::new();
    println!("Enter words or URLs, one per line. Submit empty line to finish");

    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(">")
            .allow_empty(true)
            .interact_text()
            .unwrap();

        let trimmed_word = input.trim();

        if trimmed_word.is_empty() {
            break;
        }

        words.push(trimmed_word.to_string());
    }

    words
}
