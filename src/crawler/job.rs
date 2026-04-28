use crate::cam::{model::CrawlJob, util};

pub fn build_jobs(words: Vec<String>) -> Vec<CrawlJob> {
    words
        .into_iter()
        .map(|word| CrawlJob {
            original_input: word.clone(),
            url: util::input_to_url(&word),
        })
        .collect()
}