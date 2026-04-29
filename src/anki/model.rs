use serde::{Deserialize, Serialize};
use std::{env, path::Path};

use crate::{
    anki::util::{convert_html_definition, convert_html_example, convert_html_phoetic},
    cam::{
        downloader::{build_audio_filename, build_audio_output_path},
        model::{Definition, Entry},
    },
};

const MAX_BACK_EXAMPLES: usize = 2;

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

#[derive(Serialize)]
pub struct ModelField {
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "Word")]
    pub word: String,
    #[serde(rename = "IPA")]
    pub ipa: Option<String>,
    #[serde(rename = "PartOfSpeech")]
    pub part_of_speech: Option<String>,
    #[serde(rename = "Definition")]
    pub definition: Option<String>,
    #[serde(rename = "Examples")]
    pub examples: Option<String>,
    #[serde(rename = "Audio")]
    pub audio: Option<String>,
    #[serde(rename = "Source")]
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct NoteAudio {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub fields: Vec<String>,
}

#[derive(Serialize)]
pub struct Note {
    #[serde(rename = "deckName")]
    pub deck_name: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(rename = "fields")]
    pub fields: ModelField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Vec<NoteAudio>>,
}

#[derive(Serialize)]
pub struct AddNoteParams {
    #[serde(rename = "note")]
    pub note: Note,
}

pub fn create_add_word_request(
    word_entry: &Entry,
    deck_name: &str,
    model_name: &str,
) -> AnkiRequest<AddNoteParams> {
    let primary_definition = pick_primary_definition(word_entry);
    let cloze_source =
        primary_definition.and_then(|definition| find_cloze_example(definition, &word_entry.word));
    let definition = primary_definition.map(|def| convert_html_definition(&def.meaning));
    let examples =
        primary_definition.and_then(|definition| build_examples_html(definition, cloze_source));
    let text = build_cloze_text(word_entry, cloze_source);
    let audio = build_note_audio(word_entry);

    AnkiRequest {
        action: "addNote".to_string(),
        version: 6,
        params: Some(AddNoteParams {
            note: Note {
                deck_name: deck_name.to_string(),
                model_name: model_name.to_string(),
                fields: ModelField {
                    text,
                    word: word_entry.word.clone(),
                    ipa: word_entry
                        .phonetic
                        .us
                        .as_ref()
                        .map(|phonetic| convert_html_phoetic(phonetic)),
                    part_of_speech: word_entry.part_of_speech.clone(),
                    definition,
                    examples,
                    audio: Some(String::new()),
                    source: Some("Cambridge Dictionary".to_string()),
                },
                audio,
            },
        }),
    }
}

fn pick_primary_definition(word_entry: &Entry) -> Option<&Definition> {
    word_entry
        .definitions
        .iter()
        .find(|definition| find_cloze_example(definition, &word_entry.word).is_some())
        .or_else(|| {
            word_entry
                .definitions
                .iter()
                .find(|definition| !definition.examples.is_empty())
        })
        .or_else(|| word_entry.definitions.first())
}

fn find_cloze_example<'a>(definition: &'a Definition, word: &str) -> Option<&'a str> {
    definition
        .examples
        .iter()
        .find(|example| contains_word_boundary(example, word))
        .map(String::as_str)
}

fn build_examples_html(definition: &Definition, cloze_source: Option<&str>) -> Option<String> {
    let html = definition
        .examples
        .iter()
        .filter(|example| Some(example.as_str()) != cloze_source)
        .take(MAX_BACK_EXAMPLES)
        .map(|example| convert_html_example(example))
        .collect::<Vec<_>>()
        .join("");

    if html.is_empty() { None } else { Some(html) }
}

fn build_cloze_text(word_entry: &Entry, cloze_source: Option<&str>) -> String {
    cloze_source
        .and_then(|example| wrap_first_match_in_cloze(example, &word_entry.word))
        .unwrap_or_else(|| format!("{{{{c1::{}}}}}", word_entry.word))
}

fn wrap_first_match_in_cloze(text: &str, word: &str) -> Option<String> {
    if word.trim().is_empty() {
        return None;
    }

    let lower_text = text.to_lowercase();
    let lower_word = word.to_lowercase();
    let mut search_start = 0usize;

    while let Some(relative_pos) = lower_text[search_start..].find(&lower_word) {
        let start = search_start + relative_pos;
        let end = start + lower_word.len();
        let before = lower_text[..start].chars().next_back();
        let after = lower_text[end..].chars().next();

        if is_word_boundary(before) && is_word_boundary(after) {
            let matched = &text[start..end];
            return Some(format!(
                "{}{{{{c1::{}}}}}{}",
                &text[..start],
                matched,
                &text[end..]
            ));
        }

        search_start = end;
    }

    None
}

fn contains_word_boundary(text: &str, word: &str) -> bool {
    if word.trim().is_empty() {
        return false;
    }

    let lower_text = text.to_lowercase();
    let lower_word = word.to_lowercase();
    let mut search_start = 0usize;

    while let Some(relative_pos) = lower_text[search_start..].find(&lower_word) {
        let start = search_start + relative_pos;
        let end = start + lower_word.len();
        let before = lower_text[..start].chars().next_back();
        let after = lower_text[end..].chars().next();

        if is_word_boundary(before) && is_word_boundary(after) {
            return true;
        }

        search_start = end;
    }

    false
}

fn is_word_boundary(ch: Option<char>) -> bool {
    ch.is_none_or(|c| !c.is_alphanumeric())
}

fn build_note_audio(word_entry: &Entry) -> Option<Vec<NoteAudio>> {
    let (region, remote_url) = if let Some(url) = word_entry.audio.us.as_ref() {
        ("us", Some(url.clone()))
    } else if let Some(url) = word_entry.audio.uk.as_ref() {
        ("uk", Some(url.clone()))
    } else {
        let fallback_region = if word_entry.audio.us.is_some() {
            "us"
        } else if word_entry.audio.uk.is_some() {
            "uk"
        } else {
            return None;
        };
        (fallback_region, None)
    };

    let local_path = build_audio_output_path(&word_entry.word, region);
    let absolute_path = env::current_dir()
        .ok()
        .map(|cwd| cwd.join(&local_path))
        .and_then(|path| path.to_str().map(ToString::to_string));
    let local_file_exists = Path::new(&local_path).exists();

    Some(vec![NoteAudio {
        filename: build_audio_filename(&word_entry.word, region),
        path: if remote_url.is_none() && local_file_exists {
            absolute_path
        } else {
            None
        },
        url: remote_url,
        fields: vec!["Audio".to_string()],
    }])
}

#[cfg(test)]
mod test {
    use crate::anki::model::{AddNoteParams, AnkiRequest, ModelField, Note};

    #[test]
    fn test_request_format() {
        let req: AnkiRequest<AddNoteParams> = AnkiRequest {
            action: "addNote".to_string(),
            version: 6,
            params: Some(AddNoteParams {
                note: Note {
                    deck_name: "English".to_string(),
                    model_name: "Cam2Anki Cloze v2".to_string(),
                    fields: ModelField {
                        text: "Say {{c1::hello}} to me.".to_string(),
                        word: "hello".to_string(),
                        ipa: Some("/h@.loU/".to_string()),
                        part_of_speech: Some("noun".to_string()),
                        definition: Some("a greeting".to_string()),
                        examples: Some("example usage".to_string()),
                        audio: Some(String::new()),
                        source: Some("Cambridge Dictionary".to_string()),
                    },
                    audio: None,
                },
            }),
        };

        print!("{}", serde_json::to_string_pretty(&req).unwrap());
    }
}
