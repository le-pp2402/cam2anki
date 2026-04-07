pub fn norm_anki_base_url(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return "http://127.0.0.1:8765".to_string();
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    }
}

pub fn convert_html_phoetic(phoetic: &str) -> String {
    format!(
        "<div style=\"color: #1d2a57; font-size: 1.2em; margin-bottom: 8px;\">
            <strong>{}</strong> <i>(n)</i>
        </div>",
        phoetic
    )
}

pub fn convert_html_definition(def: &str) -> String {
    format!(
        "<div style=\"color: #1d2a57; margin-bottom: 16px; border-bottom: 1px solid #1d2a57; padding-bottom: 6px;\">
            <strong>{}</strong>
        </div>", 
        def
    )
}

pub fn convert_html_example(exp: &str) -> String {
    format!(
        "<span style=\"font-style: italic;\">
        {}
        </span>",
        exp
    )
}

#[cfg(test)]
mod test_anki {
    use crate::anki::util::norm_anki_base_url;

    #[test]
    fn uses_default_url_when_input_is_empty() {
        let expected = "http://127.0.0.1:8765".to_string();

        assert_eq!(norm_anki_base_url(""), expected);
    }

    #[test]
    fn test_norm_anki_base_url() {
        let expected = "http://localhost:8765".to_string();

        assert_eq!(norm_anki_base_url("localhost:8765"), expected);
        assert_eq!(norm_anki_base_url("http://localhost:8765"), expected);
    }
}
