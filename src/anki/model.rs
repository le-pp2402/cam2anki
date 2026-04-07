use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Deck {
    deck_name: String,
    deck_id: u64,
}

#[derive(Serialize)]
pub struct AnkiRequest {
    pub action: String,
    pub version: u8,
}

#[derive(Deserialize)]
pub struct AnkiResponse<T> {
    pub result: Option<T>,
    pub error: Option<String>,
}

// "Learnable",
// "Definition",
// "Audio",
// "Mems",
// "Attributes",
// "Extra",
// "Extra 2",
// "Choices"
#[derive(Serialize)]
struct AddWordRequest {}
