use anyhow::Result;

use crate::{
    cam::{
        crawler::{build_jobs, crawl},
        model::Entry,
    },
    cli::{prompt::ask_words, selector},
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
    let words = anki::anki_connect::get_decks("http://localhost:8765").await;
    let selected_decks = selector::select_desk(&words);
    Ok("".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    select_deck().await?;
    ask_and_crawl().await;
    Ok(())
}
