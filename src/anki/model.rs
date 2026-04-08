use dialoguer::console::Attribute;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Deck {
    deck_name: String,
    deck_id: u64,
}

#[derive(Serialize)]
pub struct AnkiRequest<T> {
    pub action: String,
    pub version: u8,
    pub params: Option<T>,
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
pub struct ModelField {
    #[serde(rename = "Learnable")]
    pub learnable: String,
    #[serde(rename = "Definition")]
    pub definition: Option<String>,
    pub audio: Option<String>,
    pub mems: Option<String>,
    pub attributes: Option<String>,
    pub extra: Option<String>,
    pub extra_2: Option<String>,
    pub choices: Option<String>,
}

#[derive(Serialize)]
pub struct Note {
    pub deck_name: String,
    pub model_name: String,
    pub field: ModelField,
}
