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
        "Meaning",
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
    template_name: &str,
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
                    template_name: {
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
.max-w-3xl { max-width: 48rem; }
.max-w-4xl { max-width: 56rem; }
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
    r#"<div class="mx-auto max-w-4xl rounded-2xl border border-slate-200 bg-white p-6 shadow-sm text-left">
    <div class="space-y-5">
        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Cloze</div>
            <div class="text-3xl font-semibold leading-8 text-slate-900 text-pretty text-balance">{{cloze:Text}}</div>
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
    r#"<div class="mx-auto max-w-4xl rounded-2xl border border-slate-200 bg-white p-6 shadow-sm text-left">
    <div class="space-y-5">
        <div class="space-y-3">
            <div class="text-xs font-semibold text-slate-500">Sentence</div>
            <div class="text-3xl font-semibold leading-8 text-slate-900 text-pretty text-balance">{{cloze:Text}}</div>
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

// ── Study4 deck + model helpers ─────────────────────────────────────────────

pub async fn create_deck(url: &str, deck_name: &str) -> Result<()> {
    let client = Client::new();
    let payload = AnkiRequest::<Value> {
        action: "createDeck".to_string(),
        version: 6,
        params: Some(serde_json::json!({ "deck": deck_name })),
    };

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .with_context(|| format!("Cannot reach AnkiConnect at {url}"))?
        .error_for_status()
        .with_context(|| format!("AnkiConnect at {url} returned HTTP error"))?;

    let anki_res = response
        .json::<AnkiResponse<Value>>()
        .await
        .context("Invalid response from AnkiConnect (createDeck)")?;

    if let Some(error) = anki_res.error {
        bail!("AnkiConnect createDeck error: {error}");
    }

    Ok(())
}

pub const STUDY4_MODEL_NAME: &str = "Study4 TOEIC";

pub async fn ensure_study4_model(url: &str) -> Result<()> {
    let models = get_models(url).await?;
    if models.iter().any(|m| m == STUDY4_MODEL_NAME) {
        sync_study4_model(url).await?;
        return Ok(());
    }

    let client = Client::new();
    let mut params = Map::new();

    params.insert(
        "modelName".to_string(),
        Value::String(STUDY4_MODEL_NAME.to_string()),
    );
    params.insert(
        "inOrderFields".to_string(),
        serde_json::json!([
            "Text",
            "Word",
            "IPA",
            "PartOfSpeech",
            "DefinitionEN",
            "DefinitionVI",
            "Image",
            "Audio",
            "Examples",
            "Source"
        ]),
    );
    params.insert("isCloze".to_string(), Value::Bool(true));
    params.insert("css".to_string(), Value::String(study4_css().to_string()));
    params.insert(
        "cardTemplates".to_string(),
        serde_json::json!([{
            "Name": "Vocabulary",
            "Front": study4_front(),
            "Back": study4_back()
        }]),
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
        .with_context(|| format!("Cannot reach AnkiConnect at {url}"))?
        .error_for_status()
        .with_context(|| format!("AnkiConnect at {url} returned HTTP error"))?;

    let anki_res = response
        .json::<AnkiResponse<Value>>()
        .await
        .context("Invalid response from AnkiConnect (createModel)")?;

    if let Some(error) = anki_res.error {
        bail!("AnkiConnect createModel error: {error}");
    }

    Ok(())
}

async fn sync_study4_model(url: &str) -> Result<()> {
    update_model_styling(url, STUDY4_MODEL_NAME, study4_css()).await?;
    update_model_templates(
        url,
        STUDY4_MODEL_NAME,
        "Vocabulary",
        study4_front(),
        study4_back(),
    )
    .await
}

pub struct Study4Note {
    pub deck: String,
    pub word: String,
    pub ipa: Option<String>,
    pub part_of_speech: Option<String>,
    pub definition_en: Option<String>,
    pub definition_vi: Option<String>,
    /// Absolute path to image file on disk
    pub image_abs_path: Option<String>,
    pub image_filename: Option<String>,
    /// Absolute path to audio file on disk
    pub audio_abs_path: Option<String>,
    pub audio_filename: Option<String>,
    /// Pre-built HTML for example sentences
    pub examples_html: Option<String>,
    /// Cloze text e.g. "She walked down the {{c1::aisle}}."
    pub cloze_text: String,
    pub source: Option<String>,
}

pub async fn insert_study4_note(url: &str, note: &Study4Note) -> Result<i64> {
    let client = Client::new();

    // Encode files as base64 so AnkiConnect can receive them regardless of
    // whether Anki Desktop is on the same machine or not.
    let audio: Option<Vec<Value>> = note
        .audio_abs_path
        .as_ref()
        .zip(note.audio_filename.as_ref())
        .and_then(|(path, filename)| {
            let bytes = std::fs::read(path).ok()?;
            let b64 = base64_encode(&bytes);
            Some(vec![serde_json::json!({
                "data": b64,
                "filename": filename,
                "fields": ["Audio"]
            })])
        });

    let picture: Option<Vec<Value>> = note
        .image_abs_path
        .as_ref()
        .zip(note.image_filename.as_ref())
        .and_then(|(path, filename)| {
            let bytes = std::fs::read(path).ok()?;
            let b64 = base64_encode(&bytes);
            Some(vec![serde_json::json!({
                "data": b64,
                "filename": filename,
                "fields": ["Image"]
            })])
        });

    let mut note_obj = serde_json::json!({
        "deckName": note.deck,
        "modelName": STUDY4_MODEL_NAME,
        "fields": {
            "Text": note.cloze_text,
            "Word": note.word,
            "IPA": note.ipa.as_deref().unwrap_or(""),
            "PartOfSpeech": note.part_of_speech.as_deref().unwrap_or(""),
            "DefinitionEN": note.definition_en.as_deref().unwrap_or(""),
            "DefinitionVI": note.definition_vi.as_deref().unwrap_or(""),
            "Image": "",
            "Audio": "",
            "Examples": note.examples_html.as_deref().unwrap_or(""),
            "Source": note.source.as_deref().unwrap_or("")
        },
        "options": { "allowDuplicate": false, "duplicateScope": "deck" }
    });

    if let Some(a) = audio {
        note_obj["audio"] = serde_json::json!(a);
    }
    if let Some(p) = picture {
        note_obj["picture"] = serde_json::json!(p);
    }

    let payload = serde_json::json!({
        "action": "addNote",
        "version": 6,
        "params": { "note": note_obj }
    });

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .with_context(|| format!("Cannot reach AnkiConnect at {url}"))?
        .error_for_status()
        .with_context(|| format!("AnkiConnect at {url} returned HTTP error"))?;

    let anki_res = response
        .json::<AnkiResponse<i64>>()
        .await
        .context("Invalid response from AnkiConnect (addNote)")?;

    if let Some(error) = anki_res.error {
        bail!("AnkiConnect addNote error: {error}");
    }

    anki_res.result.context("AnkiConnect returned no note id")
}

/// Simple base64 encoder — avoids adding a crate dependency.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[(n >> 18) & 0x3f] as char);
        out.push(CHARS[(n >> 12) & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            CHARS[(n >> 6) & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[n & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

fn study4_css() -> &'static str {
    r#".card {
    font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    font-size: 16px;
    color: #0f172a;
    background: #f1f5f9;
    padding: 24px;
    margin: 0;
}
.card-wrap {
    max-width: 780px;
    margin: 0 auto;
    background: #ffffff;
    border-radius: 16px;
    box-shadow: 0 2px 16px rgba(15,23,42,0.08);
    padding: 28px 32px;
}

/* ── Header row: thumbnail + word info ── */
.header {
    display: flex;
    align-items: flex-start;
    gap: 20px;
    margin-bottom: 20px;
}
.thumb {
    flex-shrink: 0;
    width: 120px;
    height: 120px;
    border-radius: 12px;
    overflow: hidden;
    background: #f1f5f9;
    border: 1px solid #e2e8f0;
}
.thumb img {
    width: 120px;
    height: 120px;
    object-fit: cover;
    display: block;
}
.word-block { flex: 1; min-width: 0; }
.word-row {
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-wrap: wrap;
    margin-bottom: 4px;
}
.word {
    font-size: 2.1rem;
    font-weight: 700;
    color: #0f172a;
    line-height: 1.15;
}
.pos {
    font-size: 0.78rem;
    font-weight: 600;
    color: #64748b;
    background: #f1f5f9;
    border-radius: 6px;
    padding: 3px 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border: 1px solid #e2e8f0;
}
.ipa {
    font-size: 1.05rem;
    color: #2563eb;
    margin-top: 2px;
    margin-bottom: 10px;
}
.def-en {
    font-size: 1rem;
    color: #1e293b;
    line-height: 1.65;
    margin-bottom: 4px;
}
.def-vi {
    font-size: 0.95rem;
    color: #475569;
    line-height: 1.55;
}

/* ── Cloze sentence ── */
.section-label {
    font-size: 0.7rem;
    font-weight: 700;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    margin-bottom: 6px;
    margin-top: 18px;
}
.cloze-text {
    font-size: 1.15rem;
    font-weight: 500;
    color: #1e293b;
    line-height: 1.7;
    background: #f8fafc;
    border-radius: 10px;
    padding: 14px 18px;
    border-left: 3px solid #3b82f6;
}

/* ── Examples ── */
.examples { margin-top: 4px; }
.example-item {
    padding: 8px 0 8px 14px;
    border-left: 2px solid #e2e8f0;
    margin-bottom: 10px;
}
.example-text {
    font-size: 0.95rem;
    font-style: italic;
    color: #334155;
    line-height: 1.6;
}
.example-trans {
    font-size: 0.88rem;
    color: #64748b;
    margin-top: 3px;
}

/* ── Divider ── */
.divider {
    border: none;
    border-top: 1px solid #e2e8f0;
    margin: 20px 0;
}

.audio-row { margin-top: 16px; }
audio { width: 100%; max-width: 260px; }
.cloze { color: #2563eb; font-weight: 700; }
#typeans {
    width: 100%;
    box-sizing: border-box;
    margin-top: 12px;
    padding: 11px 16px;
    border: 1px solid #cbd5e1;
    border-radius: 10px;
    background: #f8fafc;
    color: #0f172a;
    font-size: 1rem;
}
.typeGood { color: #0f766e; }
.typeBad  { color: #dc2626; }
.typeMissed { color: #2563eb; }
#answer { margin: 0; border: 0; }
"#
}

fn study4_front() -> &'static str {
    r#"<div class="card-wrap">
  <div class="header">
    {{#Image}}<div class="thumb">{{Image}}</div>{{/Image}}
    <div class="word-block">
      <div class="section-label">Fill in the blank</div>
      <div class="cloze-text">{{cloze:Text}}</div>
      {{type:cloze:Text}}
      {{#IPA}}<div class="ipa">{{IPA}}</div>{{/IPA}}
      {{#DefinitionEN}}<div class="def-en">{{DefinitionEN}}</div>{{/DefinitionEN}}
    </div>
  </div>
  {{#Audio}}<div class="audio-row">{{Audio}}</div>{{/Audio}}
</div>"#
}

fn study4_back() -> &'static str {
    r#"<div class="card-wrap">
  <div class="header">
    {{#Image}}<div class="thumb">{{Image}}</div>{{/Image}}
    <div class="word-block">
      <div class="word-row">
        <span class="word">{{Word}}</span>
        {{#PartOfSpeech}}<span class="pos">{{PartOfSpeech}}</span>{{/PartOfSpeech}}
      </div>
      {{#IPA}}<div class="ipa">{{IPA}}</div>{{/IPA}}
      {{#DefinitionEN}}<div class="def-en">{{DefinitionEN}}</div>{{/DefinitionEN}}
      {{#DefinitionVI}}<div class="def-vi">{{DefinitionVI}}</div>{{/DefinitionVI}}
    </div>
  </div>

  <hr class="divider">

  <div class="section-label">Sentence</div>
  <div class="cloze-text">{{cloze:Text}}</div>

  {{#Examples}}
  <div class="section-label">Examples</div>
  <div class="examples">{{Examples}}</div>
  {{/Examples}}

  {{#Audio}}<div class="audio-row">{{Audio}}</div>{{/Audio}}

  {{#Source}}<div style="font-size:0.7rem;color:#94a3b8;margin-top:18px;">{{Source}}</div>{{/Source}}
</div>"#
}
