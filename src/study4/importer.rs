use std::{env, path::Path, sync::Arc};

use anyhow::Result;
use futures::StreamExt;

use crate::{
    anki::anki_connect::{self, Study4Note},
    study4::db,
};

pub struct ImportConfig {
    pub anki_url: String,
    pub db_path: String,
}

pub struct ImportSummary {
    pub decks_created: usize,
    pub notes_added: usize,
    pub notes_skipped: usize,
    pub notes_failed: usize,
}

/// Deck name: "Complete TOEIC::{topic}"
fn deck_name_from_topic(topic: &str) -> String {
    format!("Complete TOEIC::{topic}")
}

/// Strip leading "data/" prefix the DB stores and build an absolute path.
fn resolve_asset_path(db_path_field: &str) -> String {
    let rel = db_path_field.trim_start_matches("data/");
    env::current_dir()
        .map(|cwd| cwd.join(rel).to_string_lossy().to_string())
        .unwrap_or_else(|_| rel.to_string())
}

/// Return Some(abs_path) only when the file actually exists on disk.
fn existing_asset(db_path_field: &str) -> Option<String> {
    let abs = resolve_asset_path(db_path_field);
    if Path::new(&abs).exists() {
        Some(abs)
    } else {
        None
    }
}

/// Skip .bin files — they are HTML error pages, not real audio.
fn existing_audio(db_path_field: &str) -> Option<String> {
    let abs = resolve_asset_path(db_path_field);
    let path = Path::new(&abs);
    if path.extension().and_then(|e| e.to_str()) == Some("bin") {
        return None;
    }
    if path.exists() { Some(abs) } else { None }
}

/// Derive the Anki media filename from an asset path.
/// "assets/images/aisle_image_1.jpg" → "aisle_image_1.jpg"
fn media_filename(abs_path: &str) -> String {
    Path::new(abs_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Audio files are stored as .bin on disk; treat them as mp3 for Anki.
fn audio_anki_filename(abs_path: &str) -> String {
    let base = Path::new(abs_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    format!("{base}.mp3")
}

fn build_cloze(word: &str, examples: &[db::Study4Example]) -> String {
    for ex in examples {
        if let Some(cloze) = try_wrap_cloze(&ex.text, word) {
            return cloze;
        }
    }
    format!("{{{{c1::{word}}}}}")
}

fn try_wrap_cloze(sentence: &str, word: &str) -> Option<String> {
    if word.trim().is_empty() {
        return None;
    }
    let lower = sentence.to_lowercase();
    let lower_word = word.to_lowercase();
    let mut pos = 0usize;

    while let Some(rel) = lower[pos..].find(&lower_word) {
        let start = pos + rel;
        let end = start + lower_word.len();
        let before = lower[..start].chars().next_back();
        let after = lower[end..].chars().next();
        let boundary = |c: Option<char>| c.is_none_or(|ch| !ch.is_alphanumeric());

        if boundary(before) && boundary(after) {
            let matched = &sentence[start..end];
            return Some(format!(
                "{}{{{{c1::{}}}}}{}",
                &sentence[..start],
                matched,
                &sentence[end..]
            ));
        }
        pos = end;
    }
    None
}

fn build_examples_html(examples: &[db::Study4Example]) -> Option<String> {
    let html: String = examples
        .iter()
        .take(3)
        .map(|ex| {
            let trans = ex
                .translation
                .as_deref()
                .map(|t| format!(r#"<div class="example-trans">{t}</div>"#))
                .unwrap_or_default();
            format!(
                r#"<div class="example-item"><div class="example-text">{}</div>{trans}</div>"#,
                ex.text
            )
        })
        .collect();

    if html.is_empty() { None } else { Some(html) }
}

const MAX_DECK_SIZE: usize = 30;
const IMPORT_CONCURRENCY: usize = 8;

/// Build a Study4Note from a term + its examples (pure, no I/O).
fn build_note(term: &db::Study4Term, examples: &[db::Study4Example], deck: &str) -> Study4Note {
    let cloze_text = build_cloze(&term.word, examples);
    let examples_html = build_examples_html(examples);

    let (image_abs_path, image_filename) = term
        .image_paths
        .as_deref()
        .and_then(|p| {
            let first = p.split(',').next()?.trim();
            let abs = existing_asset(first)?;
            let fname = media_filename(&abs);
            Some((abs, fname))
        })
        .map(|(a, f)| (Some(a), Some(f)))
        .unwrap_or((None, None));

    let (audio_abs_path, audio_filename) = term
        .audio_uk_path
        .as_deref()
        .or(term.audio_us_path.as_deref())
        .and_then(|p| {
            let abs = existing_audio(p)?;
            let fname = audio_anki_filename(&abs);
            Some((abs, fname))
        })
        .map(|(a, f)| (Some(a), Some(f)))
        .unwrap_or((None, None));

    Study4Note {
        deck: deck.to_string(),
        word: term.word.clone(),
        ipa: term.phonetic.clone(),
        part_of_speech: term.part_of_speech.clone(),
        definition_en: term.definition_en.clone(),
        definition_vi: term.definition_vi.clone(),
        image_abs_path,
        image_filename,
        audio_abs_path,
        audio_filename,
        examples_html,
        cloze_text,
        source: term.topic.clone(),
    }
}

pub async fn run_import(cfg: ImportConfig) -> Result<ImportSummary> {
    let topics = db::load_topics(&cfg.db_path)?;

    anki_connect::ensure_study4_model(&cfg.anki_url).await?;

    let url = Arc::new(cfg.anki_url.clone());

    let mut summary = ImportSummary {
        decks_created: 0,
        notes_added: 0,
        notes_skipped: 0,
        notes_failed: 0,
    };

    for (_, topic) in &topics {
        let terms = db::load_terms_for_topic(&cfg.db_path, topic)?;
        let chunks: Vec<Vec<db::Study4Term>> = terms
            .into_iter()
            .collect::<Vec<_>>()
            .chunks(MAX_DECK_SIZE)
            .map(|c| c.to_vec())
            .collect();
        let total_parts = chunks.len();

        for (part_idx, chunk) in chunks.into_iter().enumerate() {
            let deck = if total_parts == 1 {
                deck_name_from_topic(topic)
            } else {
                format!("Complete TOEIC::{topic} ({}/{})", part_idx + 1, total_parts)
            };

            println!("  Deck: {deck}  ({} words)", chunk.len());
            anki_connect::create_deck(&cfg.anki_url, &deck).await?;
            summary.decks_created += 1;

            // Pre-build all notes (reads files from disk — sync, fast)
            let mut notes: Vec<Study4Note> = Vec::with_capacity(chunk.len());
            for term in &chunk {
                let examples = db::load_examples_for_term(&cfg.db_path, term.id)?;
                notes.push(build_note(term, &examples, &deck));
            }

            // Import notes concurrently
            let results: Vec<(String, Result<i64, String>)> =
                futures::stream::iter(notes.into_iter().map(|note| {
                    let url = Arc::clone(&url);
                    async move {
                        let word = note.word.clone();
                        let res = anki_connect::insert_study4_note(&url, &note)
                            .await
                            .map_err(|e| e.to_string());
                        (word, res)
                    }
                }))
                .buffer_unordered(IMPORT_CONCURRENCY)
                .collect()
                .await;

            for (word, res) in results {
                match res {
                    Ok(_) => summary.notes_added += 1,
                    Err(msg) if msg.contains("duplicate") || msg.contains("already") => {
                        summary.notes_skipped += 1;
                    }
                    Err(msg) => {
                        eprintln!("    WARN [{word}]: {msg}");
                        summary.notes_failed += 1;
                    }
                }
            }
        }
    }

    Ok(summary)
}
