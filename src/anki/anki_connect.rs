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
        sync_custom_model(url, model_name).await?;
        return Ok(());
    }

    let client = Client::new();
    let mut params = Map::new();

    params.insert(
        "modelName".to_string(),
        Value::String(model_name.to_string()),
    );
    params.insert(
        "inOrderFields".to_string(),
        serde_json::json!([
            "Text",
            "Word",
            "IPA",
            "PartOfSpeech",
            "Definition",
            "Examples",
            "Audio",
            "Source"
        ]),
    );
    params.insert("isCloze".to_string(), Value::Bool(true));
    params.insert(
        "css".to_string(),
        Value::String(custom_model_css().to_string()),
    );
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

    sync_custom_model(url, model_name).await?;

    Ok(())
}

async fn sync_custom_model(url: &str, model_name: &str) -> Result<()> {
    update_model_styling(url, model_name, custom_model_css()).await?;
    update_model_templates(
        url,
        model_name,
        custom_front_template(),
        custom_back_template(),
    )
    .await
}

async fn update_model_styling(url: &str, model_name: &str, css: &str) -> Result<()> {
    let client = Client::new();
    let payload = AnkiRequest::<Value> {
        action: "updateModelStyling".to_string(),
        version: 6,
        params: Some(serde_json::json!({
            "model": {
                "name": model_name,
                "css": css,
            }
        })),
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
        bail!("AnkiConnect returned an error while updating model styling: {error}");
    }

    Ok(())
}

async fn update_model_templates(
    url: &str,
    model_name: &str,
    front: &str,
    back: &str,
) -> Result<()> {
    let client = Client::new();
    let payload = AnkiRequest::<Value> {
        action: "updateModelTemplates".to_string(),
        version: 6,
        params: Some(serde_json::json!({
            "model": {
                "name": model_name,
                "templates": {
                    "Meaning": {
                        "Front": front,
                        "Back": back,
                    }
                }
            }
        })),
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
        bail!("AnkiConnect returned an error while updating model templates: {error}");
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
    font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    font-size: 16px;
    text-align: left;
    color: #0f172a;
    background: #f8fafc;
    padding: 24px 16px;
}

.mx-auto { margin-left: auto; margin-right: auto; }
.block { display: block; }
.flex { display: flex; }
.grid { display: grid; }
.inline-flex { display: inline-flex; }
.w-full { width: 100%; }
.max-w-2xl { max-width: 42rem; }
.flex-wrap { flex-wrap: wrap; }
.items-center { align-items: center; }
.gap-2 { gap: 0.5rem; }
.gap-3 { gap: 0.75rem; }
.gap-4 { gap: 1rem; }
.space-y-3 > * + * { margin-top: 0.75rem; }
.space-y-4 > * + * { margin-top: 1rem; }
.space-y-5 > * + * { margin-top: 1.25rem; }
.rounded-2xl { border-radius: 1rem; }
.rounded-full { border-radius: 9999px; }
.border { border-width: 1px; border-style: solid; }
.border-l-2 { border-left-width: 2px; border-left-style: solid; }
.border-slate-200 { border-color: #e2e8f0; }
.bg-white { background: #ffffff; }
.bg-slate-50 { background: #f8fafc; }
.px-3 { padding-left: 0.75rem; padding-right: 0.75rem; }
.py-1\\.5 { padding-top: 0.375rem; padding-bottom: 0.375rem; }
.p-5 { padding: 1.25rem; }
.p-6 { padding: 1.5rem; }
.pl-4 { padding-left: 1rem; }
.pt-1 { padding-top: 0.25rem; }
.pt-2 { padding-top: 0.5rem; }
.shadow-sm { box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08); }
.text-left { text-align: left; }
.text-xs { font-size: 0.75rem; line-height: 1rem; }
.text-sm { font-size: 0.875rem; line-height: 1.25rem; }
.text-base { font-size: 1rem; line-height: 1.5rem; }
.text-2xl { font-size: 1.5rem; line-height: 2rem; }
.text-3xl { font-size: 1.875rem; line-height: 2.25rem; }
.font-medium { font-weight: 500; }
.font-semibold { font-weight: 600; }
.font-bold { font-weight: 700; }
.italic { font-style: italic; }
.leading-7 { line-height: 1.75rem; }
.leading-8 { line-height: 2rem; }
.text-slate-400 { color: #94a3b8; }
.text-slate-500 { color: #64748b; }
.text-slate-600 { color: #475569; }
.text-slate-700 { color: #334155; }
.text-slate-900 { color: #0f172a; }
.text-blue-600 { color: #2563eb; }
.text-pretty { text-wrap: pretty; }
.text-balance { text-wrap: balance; }

.cloze {
    color: #2563eb;
    font-weight: 700;
}

#typeans {
    box-sizing: border-box;
    width: 100%;
    margin-top: 0.75rem;
    padding: 0.875rem 1rem;
    border: 1px solid #cbd5e1;
    border-radius: 0.75rem;
    background: #ffffff;
    color: #0f172a;
    font-size: 1rem;
    line-height: 1.5rem;
}

.typeGood {
    color: #0f766e;
}

.typeBad {
    color: #dc2626;
}

.typeMissed {
    color: #2563eb;
}

#answer {
    margin: 0;
    border: 0;
}

audio {
    width: 100%;
    max-width: 16rem;
}"#
}

fn custom_front_template() -> &'static str {
    r#"<div class="mx-auto max-w-2xl rounded-2xl border border-slate-200 bg-white p-6 shadow-sm text-left">
    <div class="space-y-5">
        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Cloze</div>
            <div class="text-2xl font-semibold leading-8 text-slate-900 text-pretty text-balance">{{cloze:Text}}</div>
            {{type:cloze:Text}}
        </div>

        {{#Definition}}
        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Definition</div>
            <div>{{Definition}}</div>
        </div>
        {{/Definition}}

        <div class="flex flex-wrap gap-2">
            {{#IPA}}<span class="inline-flex items-center rounded-full bg-slate-50 px-3 py-1.5 text-sm font-medium text-blue-600">{{IPA}}</span>{{/IPA}}
            {{#PartOfSpeech}}<span class="inline-flex items-center rounded-full bg-slate-50 px-3 py-1.5 text-sm font-medium text-slate-600">{{PartOfSpeech}}</span>{{/PartOfSpeech}}
        </div>

        {{#Audio}}
        <div class="space-y-3 pt-1">
            <div class="text-xs font-semibold text-slate-500">Audio</div>
            <div>{{Audio}}</div>
        </div>
        {{/Audio}}
    </div>
</div>"#
}

fn custom_back_template() -> &'static str {
    r#"<div class="mx-auto max-w-2xl rounded-2xl border border-slate-200 bg-white p-6 shadow-sm text-left">
    <div class="space-y-5">
        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Sentence</div>
            <div class="text-2xl font-semibold leading-8 text-slate-900 text-pretty text-balance">{{cloze:Text}}</div>
            {{type:cloze:Text}}
        </div>

        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Answer</div>
            <div class="text-3xl font-bold text-slate-900">{{Word}}</div>
        </div>

        <div class="flex flex-wrap gap-2">
            {{#IPA}}<span class="inline-flex items-center rounded-full bg-slate-50 px-3 py-1.5 text-sm font-medium text-blue-600">{{IPA}}</span>{{/IPA}}
            {{#PartOfSpeech}}<span class="inline-flex items-center rounded-full bg-slate-50 px-3 py-1.5 text-sm font-medium text-slate-600">{{PartOfSpeech}}</span>{{/PartOfSpeech}}
        </div>

        {{#Examples}}
        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Examples</div>
            <div class="grid gap-3">{{Examples}}</div>
        </div>
        {{/Examples}}

        <div class="text-sm text-slate-400">{{Source}}</div>
    </div>
</div>"#
}
