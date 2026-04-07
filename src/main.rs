use anyhow::{Context, Result, anyhow};

use crate::{
    cam::{
        crawler::{build_jobs, crawl},
        model::Entry,
    },
    cli::{
        prompt::{ask_anki_connect_url, ask_words},
        selector,
    },
};

mod anki;
mod cam;
mod cli;

async fn ask_and_crawl() -> Vec<Entry> {
    let words = ask_words();

    let jobs = build_jobs(words);
    let res = crawl(jobs).await;

    match res {
        Ok(res) => res,
        Err(msg) => {
            println!("{:?}", msg);
            Vec::new()
        }
    }
}

async fn select_deck() -> Result<String> {
    let url = ask_anki_connect_url().context("Failed to read AnkiConnect URL")?;
    let decks = anki::anki_connect::get_decks(&url).await?;

    if decks.is_empty() {
        return Err(anyhow!(
            "No decks were returned by AnkiConnect at {url}. Check that Anki is open and that your profile has at least one deck."
        ));
    }

    selector::select_desk(&decks)?.ok_or_else(|| anyhow!("Deck selection cancelled"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let selected_deck = select_deck().await?;
    let words = ask_and_crawl().await;
    Ok(())
}
