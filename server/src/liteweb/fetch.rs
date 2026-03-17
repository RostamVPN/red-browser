pub struct FetchResult {
    pub final_url: String,
    pub body: String,
    pub content_type: String,
}

pub async fn fetch_page(client: &reqwest::Client, url: &str) -> anyhow::Result<FetchResult> {
    let resp = client.get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await?;

    let final_url = resp.url().to_string();
    let content_type = resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.contains("text/html") && !content_type.contains("application/xhtml") {
        anyhow::bail!("Not HTML: {}", content_type);
    }

    let body = resp.text().await?;
    if body.len() > 5_000_000 {
        anyhow::bail!("Page too large: {} bytes", body.len());
    }

    Ok(FetchResult { final_url, body, content_type })
}
