fn build_jobs_from_stdin() -> Result<Vec<CrawlJob>> {
    println!("Enter words or URLs, one per line. Submit empty line to finish.");
    print!("> ");
    io::stdout().flush()?;

    let mut jobs = Vec::new();

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        jobs.push(CrawlJob {
            original_input: line.to_string(),
            url: input_to_url(line),
        });

        print!("> ");
        io::stdout().flush()?;
    }

    Ok(jobs)
}

fn collect_jobs() -> Result<Vec<CrawlJob>> {
    let jobs = build_jobs_from_args();

    if !jobs.is_empty() {
        return Ok(jobs);
    }

    build_jobs_from_stdin()
}

fn input_to_url(input: &str) -> String {
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
