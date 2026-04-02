use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Phonetic {
    pub uk: Option<String>,
    pub us: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Audio {
    pub uk: Option<String>,
    pub us: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Definition {
    pub meaning: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Entry {
    pub word: String,
    pub part_of_speech: Option<String>,
    pub(crate) phonetic: Phonetic,
    pub definitions: Vec<Definition>,
    pub audio: Audio,
}

#[derive(Debug, Clone)]
pub struct CrawlJob {
    pub original_input: String,
    pub url: String,
}
