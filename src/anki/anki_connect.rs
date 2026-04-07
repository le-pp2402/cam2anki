use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::anki::model::{AnkiRequest, AnkiResponse};

pub async fn get_decks(url: &str) -> Result<Vec<String>> {
    let client = Client::new();

    let payload = AnkiRequest {
        action: "deckNames".to_string(),
        version: 6,
    };

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .with_context(|| {
            format!(
                "Could not reach AnkiConnect at {url}. Make sure Anki Desktop is open and the AnkiConnect add-on is running."
            )
        })?
        .error_for_status()
        .with_context(|| format!("AnkiConnect at {url} returned an HTTP error"))?;

    let anki_res = response
        .json::<AnkiResponse<Vec<String>>>()
        .await
        .context("Received an invalid response from AnkiConnect")?;

    if let Some(error) = anki_res.error {
        bail!("AnkiConnect returned an error: {error}");
    }

    anki_res.result.context("AnkiConnect returned no deck list")
}
