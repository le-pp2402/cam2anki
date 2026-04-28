use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde_json::{Map, Value};

use crate::{
    anki::model::{AnkiRequest, AnkiResponse},
    cam::model::Entry,
};

pub async fn get_decks(url: &str) -> Result<Vec<String>> {
    let client = Client::new();

    let payload = AnkiRequest::<Map<String, Value>> {
        action: "deckNames".to_string(),
        version: 6,
        params: Some(Map::new()),
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

pub async fn get_models(url: &str) -> Result<Vec<String>> {
    let client = Client::new();

    let payload = AnkiRequest::<Map<String, Value>> {
        action: "modelNames".to_string(),
        version: 6,
        params: Some(Map::new()),
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

    anki_res
        .result
        .context("AnkiConnect returned no model list")
}

pub async fn ensure_custom_model(url: &str, model_name: &str) -> Result<()> {
    let models = get_models(url).await?;
    if models.iter().any(|m| m == model_name) {
        return Ok(());
    }

    let client = Client::new();
    let mut params = Map::new();

    params.insert("modelName".to_string(), Value::String(model_name.to_string()));
    params.insert(
        "inOrderFields".to_string(),
        serde_json::json!(["Word", "IPA", "PartOfSpeech", "Definition", "Examples", "Source"]),
    );
    params.insert("isCloze".to_string(), Value::Bool(false));
    params.insert("css".to_string(), Value::String(custom_model_css().to_string()));
    params.insert(
        "cardTemplates".to_string(),
        serde_json::json!([
            {
                "Name": "Meaning",
                "Front": custom_front_template(),
                "Back": custom_back_template()
            }
        ]),
    );

    let payload = AnkiRequest::<Map<String, Value>> {
        action: "createModel".to_string(),
        version: 6,
        params: Some(params),
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
        .json::<AnkiResponse<Value>>()
        .await
        .context("Received an invalid response from AnkiConnect")?;

    if let Some(error) = anki_res.error {
        bail!("AnkiConnect returned an error: {error}");
    }

    if anki_res.result.is_none() {
        bail!("AnkiConnect returned no result when creating model");
    }

    Ok(())
}

pub async fn insert_word(url: &str, word: &Entry, deck: &str, model: &str) -> Result<i64> {
    let client = Client::new();

    let payload = crate::anki::model::create_add_word_request(word, deck, model);

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
        .json::<AnkiResponse<i64>>()
        .await
        .context("Received an invalid response from AnkiConnect")?;

    if let Some(error) = anki_res.error {
        bail!("AnkiConnect returned an error: {error}");
    }

    anki_res
        .result
        .context("AnkiConnect returned no note id for addNote")
}

fn custom_model_css() -> &'static str {
        r#".card {
    font-family: 'Georgia', 'Times New Roman', serif;
    font-size: 20px;
    text-align: left;
    color: #1f2937;
    background: linear-gradient(160deg, #f8fafc 0%, #eef2ff 100%);
    padding: 18px;
}

.wrap {
    max-width: 760px;
    margin: 0 auto;
    background: #ffffff;
    border: 1px solid #dbeafe;
    border-radius: 14px;
    box-shadow: 0 10px 24px rgba(30, 41, 59, 0.08);
    padding: 20px;
}

.word {
    font-size: 2em;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: #0f172a;
}

.meta {
    margin-top: 8px;
    color: #334155;
    font-size: 0.95em;
}

.chip {
    display: inline-block;
    margin-right: 8px;
    margin-top: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: #e0f2fe;
    color: #0c4a6e;
}

.section {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid #e2e8f0;
}

.title {
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-size: 0.72em;
    color: #64748b;
    margin-bottom: 8px;
}

.source {
    margin-top: 16px;
    font-size: 0.78em;
    color: #94a3b8;
}"#
}

fn custom_front_template() -> &'static str {
        r#"<div class="wrap"> 
    <div class="word">{{Word}}</div>
    <div class="meta"> 
        <span class="chip">{{IPA}}</span>
        <span class="chip">{{PartOfSpeech}}</span>
    </div>
</div>"#
}

fn custom_back_template() -> &'static str {
        r#"{{FrontSide}}
<div class="wrap"> 
    <div class="section">
        <div class="title">Definition</div>
        <div>{{Definition}}</div>
    </div>
    <div class="section">
        <div class="title">Examples</div>
        <div>{{Examples}}</div>
    </div>
    <div class="source">{{Source}}</div>
</div>"#
}
