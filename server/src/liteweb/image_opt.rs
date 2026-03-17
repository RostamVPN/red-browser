use image::GenericImageView;
use regex::Regex;

pub async fn optimize_images(
    client: &reqwest::Client,
    html: &str,
    _base_url: &str,
    max_width: u32,
    _quality: u8,
    max_images: usize,
) -> (String, usize) {
    let img_re = Regex::new(r#"<img[^>]*src\s*=\s*"([^"]+)"[^>]*>"#).unwrap();

    // Collect image URLs
    let mut image_urls: Vec<(String, String)> = Vec::new(); // (full_match, src_url)
    for caps in img_re.captures_iter(html) {
        let full_match = caps[0].to_string();
        let src = caps[1].to_string();
        if src.starts_with("data:") { continue; } // Already inlined
        image_urls.push((full_match, src));
        if image_urls.len() >= max_images { break; }
    }

    if image_urls.is_empty() {
        return (html.to_string(), 0);
    }

    // Download and optimize concurrently
    let mut handles = Vec::new();
    for (full_match, src) in &image_urls {
        let client = client.clone();
        let src = src.clone();
        let full = full_match.clone();
        let mw = max_width;
        handles.push(tokio::spawn(async move {
            match download_and_optimize(&client, &src, mw).await {
                Ok(data_uri) => (full, Some(data_uri)),
                Err(e) => {
                    log::debug!("[image_opt] failed {}: {}", src, e);
                    (full, None)
                }
            }
        }));
    }

    let mut result_html = html.to_string();
    let mut optimized_count = 0;

    let alt_re = Regex::new(r#"alt\s*=\s*"([^"]*)""#).unwrap();

    for handle in handles {
        if let Ok((original_tag, replacement)) = handle.await {
            match replacement {
                Some(data_uri) => {
                    let alt = alt_re.captures(&original_tag)
                        .map(|c| c[1].to_string())
                        .unwrap_or_default();
                    let new_tag = format!(
                        "<img src=\"{}\" alt=\"{}\" loading=\"lazy\">",
                        data_uri, alt
                    );
                    result_html = result_html.replace(&original_tag, &new_tag);
                    optimized_count += 1;
                }
                None => {
                    // Replace failed image with alt text
                    let alt = alt_re.captures(&original_tag)
                        .map(|c| format!("[Image: {}]", &c[1]))
                        .unwrap_or_else(|| "[Image]".to_string());
                    result_html = result_html.replace(&original_tag, &format!("<em>{}</em>", alt));
                }
            }
        }
    }

    (result_html, optimized_count)
}

async fn download_and_optimize(
    client: &reqwest::Client,
    url: &str,
    max_width: u32,
) -> anyhow::Result<String> {
    // Download with timeout
    let resp = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get(url).send(),
    ).await??;

    let bytes = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        resp.bytes(),
    ).await??;

    if bytes.len() > 10_000_000 {
        anyhow::bail!("Image too large: {} bytes", bytes.len());
    }

    // Decode
    let img = image::load_from_memory(&bytes)?;
    let (w, h) = img.dimensions();

    // Resize if needed
    let img = if w > max_width {
        let new_h = (h as f64 * max_width as f64 / w as f64) as u32;
        img.resize(max_width, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Encode to WebP
    let mut webp_buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut webp_buf, image::ImageFormat::WebP)?;
    let webp_bytes = webp_buf.into_inner();

    // Encode as data URI
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&webp_bytes);
    Ok(format!("data:image/webp;base64,{}", b64))
}
