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
    format!("/{}/", phoetic.trim_matches('/'))
}

pub fn convert_html_definition(def: &str) -> String {
    format!(
        "<div class=\"text-base leading-7 text-slate-700 text-pretty\">{}</div>",
        def
    )
}

pub fn convert_html_example(exp: &str) -> String {
    format!(
        "<div class=\"border-l-2 border-slate-200 pl-4 text-base italic leading-7 text-slate-600 text-pretty\">{}</div>",
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
