use std::collections::HashSet;
use url::Url;

const TRACKING_PARAMS: &[&str] = &[
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "fbclid", "gclid", "mc_cid", "mc_eid", "_ga", "_gl",
    "ref", "source", "ocid", "icid", "ncid",
];

const TRACKING_URL_PATTERNS: &[&str] = &[
    "/pixel", "/beacon", "1x1", "__utm", "tr.gif", "spacer.gif",
    "/track", "/analytics", "collect?", "log?",
];

fn clean_url(href: &str, base_url: &Option<Url>, trackers: &HashSet<String>) -> Option<String> {
    let resolved = resolve_url(href, base_url);
    if let Ok(mut url) = Url::parse(&resolved) {
        // Check if domain is a tracker
        if let Some(domain) = url.domain() {
            if is_tracker_domain(domain, trackers) {
                return None;
            }
        }
        // Strip tracking query params
        let pairs: Vec<(String, String)> = url.query_pairs()
            .filter(|(k, _)| !TRACKING_PARAMS.contains(&k.as_ref()))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        url.set_query(None);
        if !pairs.is_empty() {
            let qs: String = pairs.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.set_query(Some(&qs));
        }
        Some(url.to_string())
    } else {
        Some(resolved)
    }
}

fn resolve_url(href: &str, base_url: &Option<Url>) -> String {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("data:") {
        return href.to_string();
    }
    if let Some(base) = base_url {
        if let Ok(resolved) = base.join(href) {
            return resolved.to_string();
        }
    }
    href.to_string()
}

fn is_tracker_domain(domain: &str, trackers: &HashSet<String>) -> bool {
    let domain = domain.strip_prefix("www.").unwrap_or(domain);
    if trackers.contains(domain) { return true; }
    // Check parent domains
    let parts: Vec<&str> = domain.split('.').collect();
    for i in 1..parts.len().saturating_sub(1) {
        let parent = parts[i..].join(".");
        if trackers.contains(&parent) { return true; }
    }
    false
}

fn is_tracking_image(src: &str, trackers: &HashSet<String>) -> bool {
    // Check URL patterns
    let lower = src.to_lowercase();
    for pattern in TRACKING_URL_PATTERNS {
        if lower.contains(pattern) { return true; }
    }
    // Check domain
    if let Ok(url) = Url::parse(src) {
        if let Some(domain) = url.domain() {
            if is_tracker_domain(domain, trackers) { return true; }
        }
    }
    false
}

fn collapse_whitespace(html: &str) -> String {
    // Replace 3+ consecutive <br> with just 2
    let br_re = regex::Regex::new(r"(<br\s*/?>[\s]*){3,}").unwrap();
    br_re.replace_all(html, "<br><br>").to_string()
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('"', "&quot;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

// Regex-based sanitization on the HTML string
pub fn sanitize_simple(html: &str, base_url: &str, tracker_domains: &HashSet<String>) -> String {
    let base = Url::parse(base_url).ok();

    // Remove dangerous tags and their content (one regex per tag since Rust regex has no backrefs)
    let mut cleaned = html.to_string();
    for tag in &["script", "style", "noscript", "iframe", "object", "embed", "applet", "svg", "math", "canvas", "form"] {
        let pattern = format!(r"(?si)<{tag}[^>]*>.*?</{tag}>");
        let re = regex::Regex::new(&pattern).unwrap();
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Remove self-closing versions
    let remove_self = regex::Regex::new(
        r"(?i)<(?:script|style|iframe|object|embed|applet|canvas|form)[^>]*/>"
    ).unwrap();
    cleaned = remove_self.replace_all(&cleaned, "").to_string();

    // Remove all event handlers (double-quoted)
    let events = regex::Regex::new(r#"(?i)\s+on\w+\s*=\s*"[^"]*""#).unwrap();
    cleaned = events.replace_all(&cleaned, "").to_string();
    // Single-quoted
    let events2 = regex::Regex::new(r"(?i)\s+on\w+\s*=\s*'[^']*'").unwrap();
    cleaned = events2.replace_all(&cleaned, "").to_string();

    // Remove data-* attributes
    let data_attr = regex::Regex::new(r#"(?i)\s+data-[\w-]+\s*=\s*"[^"]*""#).unwrap();
    cleaned = data_attr.replace_all(&cleaned, "").to_string();

    // Remove style attributes
    let style_attr = regex::Regex::new(r#"(?i)\s+style\s*=\s*"[^"]*""#).unwrap();
    cleaned = style_attr.replace_all(&cleaned, "").to_string();

    // Remove class and id attributes
    let class_attr = regex::Regex::new(r#"(?i)\s+(class|id)\s*=\s*"[^"]*""#).unwrap();
    cleaned = class_attr.replace_all(&cleaned, "").to_string();

    // Remove HTML comments
    let comments = regex::Regex::new(r"(?s)<!--.*?-->").unwrap();
    cleaned = comments.replace_all(&cleaned, "").to_string();

    // Clean URLs in href attributes (strip tracking params)
    let href_re = regex::Regex::new(r#"href\s*=\s*"([^"]*)""#).unwrap();
    cleaned = href_re.replace_all(&cleaned, |caps: &regex::Captures| {
        let href = &caps[1];
        match clean_url(href, &base, tracker_domains) {
            Some(url) => format!("href=\"{}\"", html_escape_attr(&url)),
            None => String::new(),
        }
    }).to_string();

    // Remove tracking images
    let img_re = regex::Regex::new(r#"<img[^>]*src\s*=\s*"([^"]*)"[^>]*>"#).unwrap();
    cleaned = img_re.replace_all(&cleaned, |caps: &regex::Captures| {
        let src = &caps[1];
        if is_tracking_image(src, tracker_domains) {
            String::new()
        } else {
            caps[0].to_string()
        }
    }).to_string();

    // Resolve relative image URLs
    if let Some(ref base) = base {
        let img_src_re = regex::Regex::new(r#"src\s*=\s*"([^"]+)""#).unwrap();
        cleaned = img_src_re.replace_all(&cleaned, |caps: &regex::Captures| {
            let src = &caps[1];
            if src.starts_with("http") || src.starts_with("data:") {
                caps[0].to_string()
            } else if let Ok(resolved) = base.join(src) {
                format!("src=\"{}\"", html_escape_attr(resolved.as_str()))
            } else {
                caps[0].to_string()
            }
        }).to_string();
    }

    // Collapse whitespace
    collapse_whitespace(&cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_scripts() {
        let html = r#"<p>Hello</p><script>alert('xss')</script><p>World</p>"#;
        let trackers = HashSet::new();
        let result = sanitize_simple(html, "https://example.com", &trackers);
        assert!(!result.contains("script"));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_strip_tracking_params() {
        let html = r#"<a href="https://example.com/page?utm_source=twitter&id=5&fbclid=abc">Link</a>"#;
        let trackers = HashSet::new();
        let result = sanitize_simple(html, "https://example.com", &trackers);
        assert!(result.contains("id=5"));
        assert!(!result.contains("utm_source"));
        assert!(!result.contains("fbclid"));
    }

    #[test]
    fn test_strip_tracking_image() {
        let html = r#"<img src="https://pixel.facebook.com/tr.gif"><img src="https://example.com/photo.jpg" alt="photo">"#;
        let mut trackers = HashSet::new();
        trackers.insert("pixel.facebook.com".to_string());
        let result = sanitize_simple(html, "https://example.com", &trackers);
        assert!(!result.contains("facebook"));
        assert!(result.contains("photo.jpg"));
    }

    #[test]
    fn test_strip_event_handlers() {
        let html = r#"<div onclick="evil()" onload="track()"><p>content</p></div>"#;
        let trackers = HashSet::new();
        let result = sanitize_simple(html, "https://example.com", &trackers);
        assert!(!result.contains("onclick"));
        assert!(!result.contains("onload"));
        assert!(result.contains("content"));
    }
}
