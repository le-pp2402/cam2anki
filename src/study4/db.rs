use anyhow::{Context, Result};
use rusqlite::{Connection, params};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Study4Term {
    pub id: i64,
    pub source_url: String,
    pub word: String,
    pub part_of_speech: Option<String>,
    pub phonetic: Option<String>,
    pub definition_vi: Option<String>,
    pub definition_en: Option<String>,
    pub image_paths: Option<String>,
    pub audio_uk_path: Option<String>,
    pub audio_us_path: Option<String>,
    pub topic: Option<String>,
    pub topic_no: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Study4Example {
    pub term_id: i64,
    pub example_order: i64,
    pub text: String,
    pub translation: Option<String>,
    pub audio_path: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Study4Source {
    pub id: i64,
    pub source_url: String,
    pub page_count: i64,
}

pub fn load_sources(db_path: &str) -> Result<Vec<Study4Source>> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Cannot open SQLite database: {db_path}"))?;

    let mut stmt = conn
        .prepare("SELECT id, source_url, page_count FROM crawl_sources ORDER BY id")
        .context("Failed to prepare crawl_sources query")?;

    let sources = stmt
        .query_map([], |row| {
            Ok(Study4Source {
                id: row.get(0)?,
                source_url: row.get(1)?,
                page_count: row.get(2)?,
            })
        })
        .context("Failed to query crawl_sources")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect crawl_sources rows")?;

    Ok(sources)
}

pub fn load_topics(db_path: &str) -> Result<Vec<(i64, String)>> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Cannot open SQLite database: {db_path}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT topic_no, topic FROM terms
             WHERE topic IS NOT NULL
             GROUP BY topic_no, topic
             ORDER BY topic_no",
        )
        .context("Failed to prepare topics query")?;

    let topics = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .context("Failed to query topics")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect topics")?;

    Ok(topics)
}

pub fn load_terms_for_topic(db_path: &str, topic: &str) -> Result<Vec<Study4Term>> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Cannot open SQLite database: {db_path}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, source_url, word, part_of_speech, phonetic,
                    definition_vi, definition_en, image_paths, audio_uk_path, audio_us_path,
                    topic, topic_no
             FROM terms WHERE topic = ?1
             ORDER BY topic_no, no",
        )
        .context("Failed to prepare terms query")?;

    let terms = stmt
        .query_map(params![topic], |row| {
            Ok(Study4Term {
                id: row.get(0)?,
                source_url: row.get(1)?,
                word: row.get(2)?,
                part_of_speech: row.get(3)?,
                phonetic: row.get(4)?,
                definition_vi: row.get(5)?,
                definition_en: row.get(6)?,
                image_paths: row.get(7)?,
                audio_uk_path: row.get(8)?,
                audio_us_path: row.get(9)?,
                topic: row.get(10)?,
                topic_no: row.get(11)?,
            })
        })
        .context("Failed to query terms")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect terms rows")?;

    Ok(terms)
}

pub fn load_examples_for_term(db_path: &str, term_id: i64) -> Result<Vec<Study4Example>> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Cannot open SQLite database: {db_path}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT term_id, example_order, text, translation, audio_path
             FROM examples WHERE term_id = ?1
             ORDER BY example_order",
        )
        .context("Failed to prepare examples query")?;

    let examples = stmt
        .query_map(params![term_id], |row| {
            Ok(Study4Example {
                term_id: row.get(0)?,
                example_order: row.get(1)?,
                text: row.get(2)?,
                translation: row.get(3)?,
                audio_path: row.get(4)?,
            })
        })
        .context("Failed to query examples")?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to collect examples rows")?;

    Ok(examples)
}
