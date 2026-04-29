use anyhow::{Context, Result, anyhow};

use crate::{
    cli::{
        prompt::{ask_anki_connect_url, ask_words},
        selector,
    },
    crawler::{job::build_jobs, pipeline},
};

mod anki;
mod cam;
mod cli;
mod crawler;

const CUSTOM_MODEL_NAME: &str = "Cam2Anki Cloze v2";

fn choose_deck(decks: &[String]) -> Result<String> {
    if decks.is_empty() {
        return Err(anyhow!(
            "No decks were returned by AnkiConnect. Check that Anki is open and that your profile has at least one deck."
        ));
    }

    selector::select_desk(decks)?.ok_or_else(|| anyhow!("Deck selection cancelled"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = ask_anki_connect_url().context("Failed to read AnkiConnect URL")?;
    let decks = anki::anki_connect::get_decks(&url).await?;
    let selected_deck = choose_deck(&decks)?;
    anki::anki_connect::ensure_custom_model(&url, CUSTOM_MODEL_NAME).await?;

    let words = ask_words();
    let jobs = build_jobs(words);

    if jobs.is_empty() {
        println!("No input provided!");
        return Ok(());
    }

    let summary = pipeline::run_pipeline(
        jobs,
        pipeline::PipelineConfig {
            anki_url: url,
            deck_name: selected_deck.clone(),
            model_name: CUSTOM_MODEL_NAME.to_string(),
            crawl_concurrency: 5,
            import_concurrency: 3,
            import_queue_capacity: 64,
        },
    )
    .await?;

    println!();
    println!("Pipeline summary:");
    println!("  Deck    : {}", selected_deck);
    println!("  Model   : {}", CUSTOM_MODEL_NAME);
    println!("  Total   : {}", summary.total_words);
    println!(
        "  Crawl   : ok={}, failed={}",
        summary.crawl_success, summary.crawl_failed
    );
    println!(
        "  Import  : ok={}, failed={}, skipped={}",
        summary.import_success, summary.import_failed, summary.import_skipped
    );

    if !summary.imported_words.is_empty() {
        println!();
        println!("Imported words:");
        for item in &summary.imported_words {
            println!("  - {} (note_id={})", item.word, item.note_id);
        }
    }

    if !summary.failed_words.is_empty() {
        println!();
        println!("Failed words:");
        for item in &summary.failed_words {
            println!("  - [{}] {} -> {}", item.stage, item.input, item.error);
        }
    }

    Ok(())
}
