use scraper::{Html, Selector};

pub struct ReadabilityResult {
    pub title: String,
    pub byline: Option<String>,
    pub content_html: String,
    pub text_length: usize,
}

pub fn extract(html: &str, _url: &str) -> ReadabilityResult {
    let document = Html::parse_document(html);

    let title = extract_title(&document);
    let byline = extract_byline(&document);

    // Score candidate elements
    let candidates = score_candidates(&document);

    // Pick the best candidate
    let content_html = if let Some((_, html)) = candidates.into_iter()
        .max_by_key(|(score, _)| *score)
    {
        html
    } else {
        // Fallback: get all text from body
        extract_body_text(&document)
    };

    let text_length = content_html.chars().filter(|c| !c.is_whitespace()).count();

    ReadabilityResult { title, byline, content_html, text_length }
}

fn extract_title(doc: &Html) -> String {
    // Try og:title first
    if let Some(og) = doc.select(&sel("meta[property='og:title']")).next() {
        if let Some(content) = og.value().attr("content") {
            if !content.is_empty() { return content.to_string(); }
        }
    }
    // Try <title>
    if let Some(t) = doc.select(&sel("title")).next() {
        let text = t.text().collect::<String>();
        // Clean common patterns: "Title - Site Name" -> "Title"
        if let Some(pos) = text.rfind(" - ").or_else(|| text.rfind(" | ")) {
            return text[..pos].trim().to_string();
        }
        return text.trim().to_string();
    }
    // Try <h1>
    if let Some(h1) = doc.select(&sel("h1")).next() {
        return h1.text().collect::<String>().trim().to_string();
    }
    "Untitled".to_string()
}

fn extract_byline(doc: &Html) -> Option<String> {
    // Try meta author
    for meta_sel in &["meta[name='author']", "meta[property='article:author']"] {
        if let Some(el) = doc.select(&sel(meta_sel)).next() {
            if let Some(content) = el.value().attr("content") {
                if !content.is_empty() { return Some(content.to_string()); }
            }
        }
    }
    // Try .author, [rel=author]
    for sel_str in &[".author", "[rel='author']", ".byline", ".post-author"] {
        if let Some(el) = doc.select(&sel(sel_str)).next() {
            let text = el.text().collect::<String>().trim().to_string();
            if !text.is_empty() && text.len() < 100 { return Some(text); }
        }
    }
    None
}

fn score_candidates(doc: &Html) -> Vec<(i32, String)> {
    let positive_re = regex::Regex::new(
        r"(?i)article|body|content|entry|main|page|post|text|blog|story"
    ).unwrap();
    let negative_re = regex::Regex::new(
        r"(?i)hidden|banner|combx|comment|community|disqus|extra|foot|header|legends|menu|related|remark|rss|sharedaddy|sidebar|skyscraper|social|sponsor|ad-break|pagination|pager|popup|nav"
    ).unwrap();

    let container_sel = sel("article, div, section, main, td");
    let link_sel = sel("a");
    let mut results: Vec<(i32, String)> = Vec::new();

    for element in doc.select(&container_sel) {
        let mut score: i32 = 0;

        // Score by class/id
        let class = element.value().attr("class").unwrap_or("");
        let id = element.value().attr("id").unwrap_or("");
        let class_id = format!("{} {}", class, id);

        if positive_re.is_match(&class_id) { score += 25; }
        if negative_re.is_match(&class_id) { score -= 25; }

        // Prefer <article> and <main>
        match element.value().name() {
            "article" => score += 20,
            "main" => score += 15,
            "section" => score += 5,
            _ => {}
        }

        // Score by text content
        let text: String = element.text().collect();
        let text_len = text.len();
        score += (text_len / 100) as i32;

        // Commas indicate prose
        let comma_count = text.matches(',').count();
        score += comma_count as i32;

        // Must have substantial text
        if text_len < 100 { continue; }

        // Penalize if mostly links
        let link_text_len: usize = element.select(&link_sel)
            .map(|a| a.text().collect::<String>().len())
            .sum();
        if link_text_len > text_len / 2 { score -= 30; }

        // Extract inner HTML
        let inner = element.inner_html();
        results.push((score, inner));
    }

    results
}

fn extract_body_text(doc: &Html) -> String {
    if let Some(body) = doc.select(&sel("body")).next() {
        body.inner_html()
    } else {
        doc.root_element().inner_html()
    }
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).unwrap()
}
