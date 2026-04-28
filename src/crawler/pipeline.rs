use std::sync::Arc;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::{Mutex, mpsc};

use crate::{
	anki::anki_connect,
	cam::{
		downloader::ensure_out_dir,
		model::{CrawlJob, Entry},
	},
	crawler::runner,
};

#[derive(Clone)]
pub struct PipelineConfig {
	pub anki_url: String,
	pub deck_name: String,
	pub model_name: String,
	pub crawl_concurrency: usize,
	pub import_concurrency: usize,
	pub import_queue_capacity: usize,
}

#[derive(Debug, Clone)]
pub struct ImportedWord {
	pub word: String,
	pub note_id: i64,
}

#[derive(Debug, Clone)]
pub struct FailedWord {
	pub input: String,
	pub stage: String,
	pub error: String,
}

#[derive(Debug, Clone)]
pub struct PipelineSummary {
	pub total_words: usize,
	pub crawl_success: usize,
	pub crawl_failed: usize,
	pub import_success: usize,
	pub import_failed: usize,
	pub import_skipped: usize,
	pub imported_words: Vec<ImportedWord>,
	pub failed_words: Vec<FailedWord>,
}

impl PipelineSummary {
	fn empty() -> Self {
		Self {
			total_words: 0,
			crawl_success: 0,
			crawl_failed: 0,
			import_success: 0,
			import_failed: 0,
			import_skipped: 0,
			imported_words: Vec::new(),
			failed_words: Vec::new(),
		}
	}
}

pub async fn run_pipeline(jobs: Vec<CrawlJob>, config: PipelineConfig) -> Result<PipelineSummary> {
	ensure_out_dir()?;

	if jobs.is_empty() {
		return Ok(PipelineSummary::empty());
	}

	let total_words = jobs.len();
	let total_units = (total_words * 2) as u64;

	let pb = ProgressBar::new(total_units);
	pb.set_style(
		ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
			.progress_chars("##-"),
	);
	pb.set_message("crawling + importing");

	let summary = Arc::new(Mutex::new(PipelineSummary {
		total_words,
		..PipelineSummary::empty()
	}));

	let (crawl_tx, crawl_rx) = mpsc::channel::<CrawlJob>(total_words);
	let (import_tx, import_rx) = mpsc::channel::<(String, Entry)>(config.import_queue_capacity.max(1));

	for job in jobs {
		let _ = crawl_tx.send(job).await;
	}
	drop(crawl_tx);

	let crawl_rx = Arc::new(Mutex::new(crawl_rx));
	let import_rx = Arc::new(Mutex::new(import_rx));

	let mut crawl_workers = Vec::new();
	for _ in 0..config.crawl_concurrency.max(1) {
		let crawl_rx = Arc::clone(&crawl_rx);
		let import_tx = import_tx.clone();
		let pb = pb.clone();
		let summary = Arc::clone(&summary);

		crawl_workers.push(tokio::spawn(async move {
			loop {
				let next_job = {
					let mut rx = crawl_rx.lock().await;
					rx.recv().await
				};

				let Some(job) = next_job else {
					break;
				};

				let label = job.original_input.clone();
				match runner::process_job(job).await {
					Ok(entry) => {
						{
							let mut state = summary.lock().await;
							state.crawl_success += 1;
						}
						pb.println(format!("CRAWL OK   {:<20} -> {}", label, entry.word));
						pb.inc(1);

						if import_tx.send((label, entry)).await.is_err() {
							let mut state = summary.lock().await;
							state.import_failed += 1;
							state.failed_words.push(FailedWord {
								input: "<pipeline>".to_string(),
								stage: "import".to_string(),
								error: "Import queue closed unexpectedly".to_string(),
							});
							pb.inc(1);
						}
					}
					Err(err) => {
						{
							let mut state = summary.lock().await;
							state.crawl_failed += 1;
							state.import_skipped += 1;
							state.failed_words.push(FailedWord {
								input: label.clone(),
								stage: "crawl".to_string(),
								error: err.to_string(),
							});
						}

						pb.println(format!("CRAWL FAIL {:<20} -> {}", label, err));
						pb.inc(1);
						// Crawl failed means import stage is skipped but still counted as a completed unit.
						pb.inc(1);
					}
				}
			}
		}));
	}

	let mut import_workers = Vec::new();
	for _ in 0..config.import_concurrency.max(1) {
		let import_rx = Arc::clone(&import_rx);
		let pb = pb.clone();
		let summary = Arc::clone(&summary);
		let anki_url = config.anki_url.clone();
		let deck_name = config.deck_name.clone();
		let model_name = config.model_name.clone();

		import_workers.push(tokio::spawn(async move {
			loop {
				let next_entry = {
					let mut rx = import_rx.lock().await;
					rx.recv().await
				};

				let Some((input, entry)) = next_entry else {
					break;
				};

				match anki_connect::insert_word(&anki_url, &entry, &deck_name, &model_name).await {
					Ok(note_id) => {
						{
							let mut state = summary.lock().await;
							state.import_success += 1;
							state.imported_words.push(ImportedWord {
								word: entry.word.clone(),
								note_id,
							});
						}
						pb.println(format!(
							"IMPORT OK  {:<20} -> {} (note_id={})",
							input, entry.word, note_id
						));
					}
					Err(err) => {
						{
							let mut state = summary.lock().await;
							state.import_failed += 1;
							state.failed_words.push(FailedWord {
								input,
								stage: "import".to_string(),
								error: err.to_string(),
							});
						}
						pb.println(format!("IMPORT FAIL {:<20} -> {}", entry.word, err));
					}
				}

				pb.inc(1);
			}
		}));
	}

	for worker in crawl_workers {
		worker.await?;
	}
	drop(import_tx);

	for worker in import_workers {
		worker.await?;
	}

	pb.finish_with_message("done");

	let state = summary.lock().await;
	Ok(state.clone())
}
