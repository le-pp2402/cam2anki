use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Deck {
    deck_name: String,
    deck_id: u64,
}

#[derive(Deserialize)]
pub struct AnkiRequest {
    pub action: String,
    pub version: u8,
}

#[derive(Deserialize)]
pub struct AnkiResponse<T> {
    pub result: Option<T>,
    pub error: Option<String>,
}
