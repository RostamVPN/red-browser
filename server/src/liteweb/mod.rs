pub mod fetch;
pub mod readability;
pub mod sanitize;
pub mod image_opt;
pub mod template;
pub mod tracker_list;

use std::sync::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;
use red_protocol::Response;

pub struct LiteWebEngine {
    client: reqwest::Client,
    image_quality: u8,
    image_max_width: u32,
    page_cache: Mutex<LruCache<String, CachedPage>>,
}

struct CachedPage {
    response: Response,
    cached_at: std::time::Instant,
}

const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(300);
const CACHE_SIZE: usize = 200;

impl LiteWebEngine {
    pub fn new(client: reqwest::Client, image_quality: u8, image_max_width: u32) -> Self {
        Self {
            client,
            image_quality,
            image_max_width,
            page_cache: Mutex::new(LruCache::new(NonZeroUsize::new(CACHE_SIZE).unwrap())),
        }
    }

    pub async fn process_url(&self, url: &str) -> Response {
        // Check cache
        if let Some(cached) = self.get_from_cache(url) {
            log::debug!("[liteweb] cache hit: {}", url);
            return cached;
        }

        match self.render_page(url).await {
            Ok(resp) => {
                self.put_in_cache(url, &resp);
                resp
            }
            Err(e) => {
                log::warn!("[liteweb] render failed for {}: {}", url, e);
                Response::Error {
                    code: 502,
                    message: format!("Failed to render page: {}", e),
                }
            }
        }
    }

    async fn render_page(&self, url: &str) -> anyhow::Result<Response> {
        // 1. Fetch
        let fetched = fetch::fetch_page(&self.client, url).await?;
        let original_size = fetched.body.len();
        log::info!("[liteweb] fetched {} ({} bytes)", url, original_size);

        // 2. Extract readable content
        let extracted = readability::extract(&fetched.body, &fetched.final_url);
        log::debug!("[liteweb] extracted: title={}, text_len={}", extracted.title, extracted.text_length);

        if extracted.text_length < 50 {
            anyhow::bail!("No readable content found (text_length={})", extracted.text_length);
        }

        // 3. Sanitize
        let tracker_domains = &tracker_list::TRACKER_DOMAINS;
        let sanitized = sanitize::sanitize_simple(&extracted.content_html, &fetched.final_url, tracker_domains);

        // 4. Optimize images
        let (html_with_images, _) = image_opt::optimize_images(
            &self.client,
            &sanitized,
            &fetched.final_url,
            self.image_max_width,
            self.image_quality,
            10,
        ).await;

        // 5. Wrap in template
        let final_html = template::wrap(
            &extracted.title,
            extracted.byline.as_deref(),
            &html_with_images,
        );

        // 6. Gzip compress
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(final_html.as_bytes())?;
        let compressed = encoder.finish()?;

        log::info!("[liteweb] {} -> {} bytes (compressed {} bytes, {:.0}x reduction)",
            url, final_html.len(), compressed.len(),
            original_size as f64 / compressed.len().max(1) as f64);

        Ok(Response::Page {
            url: fetched.final_url,
            title: extracted.title,
            html: compressed,
            original_size: original_size as u32,
        })
    }

    pub async fn get_cached_image(&self, _hash: &[u8], _quality: u8) -> Response {
        Response::Error { code: 404, message: "Image not in cache".into() }
    }

    fn get_from_cache(&self, url: &str) -> Option<Response> {
        let mut cache = self.page_cache.lock().ok()?;
        if let Some(entry) = cache.get(url) {
            if entry.cached_at.elapsed() < CACHE_TTL {
                return Some(entry.response.clone());
            }
            // Expired — remove it
        }
        cache.pop(url);
        None
    }

    fn put_in_cache(&self, url: &str, resp: &Response) {
        if let Ok(mut cache) = self.page_cache.lock() {
            cache.put(url.to_string(), CachedPage {
                response: resp.clone(),
                cached_at: std::time::Instant::now(),
            });
        }
    }
}
