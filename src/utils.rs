pub fn input_to_url(input: &str) -> String {
    if input.starts_with("https://") || input.starts_with("http://") {
        input.to_string()
    } else {
        let encoded = urlencoding::encode(input.trim());
        format!(
            "https://dictionary.cambridge.org/dictionary/english/{}",
            encoded
        )
    }
}
