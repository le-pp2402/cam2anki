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

#[cfg(test)]
mod test_anki {
    use crate::anki::util::norm_anki_base_url;

    fn test_norm_anki_base_url() {
        let expected = "http://localhost:8765".to_string();

        assert_eq!(norm_anki_base_url(""), expected);
        assert_eq!(norm_anki_base_url("localhost:8765"), expected);
        assert_eq!(norm_anki_base_url("http://localhost:8765"), expected);
    }
}
