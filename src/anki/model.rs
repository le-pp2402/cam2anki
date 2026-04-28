use serde::{Deserialize, Serialize};

use crate::{
    anki::util::{convert_html_definition, convert_html_phoetic},
    cam::model::Entry,
};

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
    #[serde(rename = "Source")]
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct Note {
    #[serde(rename = "deckName")]
    pub deck_name: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    #[serde(rename = "fields")]
    pub fields: ModelField,
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
    let mut definitions: Vec<String> = Vec::new();
    let mut examples: Vec<String> = Vec::new();

    if let Some(phonetic) = &word_entry.phonetic.us {
        definitions.push(convert_html_phoetic(phonetic));
    }

    for def in &word_entry.definitions {
        definitions.push(convert_html_definition(&def.meaning));
        if !def.examples.is_empty() {
            examples.push(convert_html_definition(&def.examples.join("<br>")));
        }
    }

    AnkiRequest {
        action: "addNote".to_string(),
        version: 6,
        params: Some(AddNoteParams {
            note: Note {
                deck_name: deck_name.to_string(),
                model_name: model_name.to_string(),
                fields: ModelField {
                    word: word_entry.word.clone(),
                    ipa: word_entry.phonetic.us.clone(),
                    part_of_speech: word_entry
                        .part_of_speech
                        .clone()
                        .or_else(|| Some("Unknown".to_string())),
                    definition: Some(definitions.join("<br>")),
                    examples: Some(examples.join("<br>")),
                    source: Some("Cambridge Dictionary".to_string()),
                },
            },
        }),
    }
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
                    model_name: "Cam2Anki Elegant v1".to_string(),
                    fields: ModelField {
                        word: "hello".to_string(),
                        ipa: Some("h@.loU".to_string()),
                        part_of_speech: Some("noun".to_string()),
                        definition: Some("a greeting".to_string()),
                        examples: Some("example usage".to_string()),
                        source: Some("Cambridge Dictionary".to_string()),
                    },
                },
            }),
        };

        print!("{}", serde_json::to_string_pretty(&req).unwrap());
    }
}