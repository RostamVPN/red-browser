use red_protocol::Response;

pub async fn search(client: &reqwest::Client, query: &str) -> Response {
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query));

    match client.get(&url)
        .header("Accept", "text/html")
        .send()
        .await
    {
        Ok(resp) => {
            match resp.text().await {
                Ok(body) => {
                    // Extract search results from DDG HTML
                    let clean = extract_ddg_results(&body, query);
                    use flate2::write::GzEncoder;
                    use flate2::Compression;
                    use std::io::Write;
                    let mut enc = GzEncoder::new(Vec::new(), Compression::fast());
                    let _ = enc.write_all(clean.as_bytes());
                    let compressed = enc.finish().unwrap_or_default();
                    Response::SearchResults {
                        query: query.to_string(),
                        html: compressed,
                    }
                }
                Err(e) => Response::Error { code: 502, message: format!("Search failed: {}", e) },
            }
        }
        Err(e) => Response::Error { code: 502, message: format!("Search failed: {}", e) },
    }
}

fn extract_ddg_results(html: &str, query: &str) -> String {
    let doc = scraper::Html::parse_document(html);
    let result_sel = scraper::Selector::parse(".result").unwrap_or_else(|_|
        scraper::Selector::parse("div").unwrap()
    );
    let link_sel = scraper::Selector::parse("a.result__a").unwrap_or_else(|_|
        scraper::Selector::parse("a").unwrap()
    );
    let snippet_sel = scraper::Selector::parse(".result__snippet").unwrap_or_else(|_|
        scraper::Selector::parse("span").unwrap()
    );

    let mut results_html = String::new();
    let mut count = 0;

    for result in doc.select(&result_sel) {
        if count >= 15 { break; }

        let link = result.select(&link_sel).next();
        let snippet = result.select(&snippet_sel).next();

        if let Some(a) = link {
            let href = a.value().attr("href").unwrap_or("#");
            let title: String = a.text().collect::<String>();
            let title = title.trim();
            if title.is_empty() { continue; }

            let snip: String = snippet.map(|s| s.text().collect::<String>())
                .unwrap_or_default();

            results_html.push_str(&format!(
                "<div style=\"margin-bottom:16px\"><a href=\"{}\" style=\"font-size:1.1em\">{}</a><br><span style=\"color:#666;font-size:.85em\">{}</span><br><span style=\"color:#888;font-size:.8em\">{}</span></div>",
                href, title, href, snip.trim()
            ));
            count += 1;
        }
    }

    if results_html.is_empty() {
        results_html = "<p>No results found.</p>".to_string();
    }

    // Wrap in template
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Search: {}</title><style>body{{max-width:680px;margin:0 auto;padding:16px;font:16px/1.5 system-ui,sans-serif;color:#222;background:#fff}}a{{color:#1a73e8;text-decoration:none}}@media(prefers-color-scheme:dark){{body{{background:#1a1a1a;color:#e0e0e0}}a{{color:#8ab4f8}}}}</style></head><body><h2>Search: {}</h2><hr style="margin:12px 0">{}</body></html>"#,
        query, query, results_html
    )
}

fn urlencoding(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
            String::from(b as char)
        }
        b' ' => "+".to_string(),
        _ => format!("%{:02X}", b),
    }).collect()
}
