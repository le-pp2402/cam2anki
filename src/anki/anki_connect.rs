use reqwest::Client;

use crate::anki::model::{AnkiRequest, AnkiResponse};

pub async fn get_decks(url: &str) -> Vec<String> {
    let client = Client::new();

    let payload = AnkiRequest {
        action: "deckNames".to_string(),
        version: 6,
    };

    let response = client.post(url).json(&payload).send().await;

    match response {
        Ok(res) => match res.json::<AnkiResponse<Vec<String>>>().await {
            Ok(anki_res) => anki_res.result.unwrap_or_default(),
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}
