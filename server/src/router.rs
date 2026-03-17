use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use red_protocol::{Request, Response, encode_response, decode_request};

use crate::liteweb;
use crate::instagram;
use crate::telegram;
use crate::whatsapp;
use crate::search;

#[derive(Clone)]
pub struct ServerConfig {
    pub image_quality: u8,
    pub image_max_width: u32,
    pub max_renders: usize,
}

#[derive(Clone)]
pub struct Server {
    pub config: ServerConfig,
    pub liteweb: Arc<liteweb::LiteWebEngine>,
    pub http_client: reqwest::Client,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .build()
            .expect("HTTP client build failed");

        let liteweb = Arc::new(liteweb::LiteWebEngine::new(
            http_client.clone(),
            config.image_quality,
            config.image_max_width,
        ));

        Server { config, liteweb, http_client }
    }

    pub async fn handle_request(&self, req: Request) -> Response {
        match req {
            // Web browsing
            Request::Browse { url } => self.liteweb.process_url(&url).await,
            Request::Search { query } => search::search(&self.http_client, &query).await,
            Request::ImageFull { hash, quality } => {
                self.liteweb.get_cached_image(&hash, quality).await
            }

            // Instagram (stubs for now)
            Request::IgLogin { .. } => instagram::handle_login().await,
            Request::IgFeed { .. } => instagram::handle_feed().await,
            Request::IgPost { shortcode } => instagram::handle_post(&shortcode).await,
            Request::IgSearch { query } => instagram::handle_search(&query).await,
            Request::IgChallenge { .. }
            | Request::IgExplore { .. }
            | Request::IgUserProfile { .. }
            | Request::IgStories { .. }
            | Request::IgLike { .. }
            | Request::IgUnlike { .. }
            | Request::IgComment { .. }
            | Request::IgDmList
            | Request::IgDmThread { .. }
            | Request::IgDmSend { .. } => instagram::handle_stub().await,

            // Telegram (stubs for now)
            Request::TgAuth { .. } => telegram::handle_auth_stub().await,
            Request::TgGetChats { .. } => telegram::handle_chats_stub().await,
            Request::TgSendMessage { .. } => telegram::handle_send_stub().await,
            Request::TgAuthCode { .. }
            | Request::TgAuthPassword { .. }
            | Request::TgGetMessages { .. }
            | Request::TgGetChat { .. }
            | Request::TgSearchChats { .. }
            | Request::TgMarkRead { .. } => telegram::handle_stub().await,

            // WhatsApp (stubs for now)
            Request::WaLinkRequest => whatsapp::handle_link_stub().await,
            Request::WaGetChats => whatsapp::handle_chats_stub().await,
            Request::WaGetMessages { .. }
            | Request::WaSendMessage { .. } => whatsapp::handle_stub().await,

            // System
            Request::Ping { ts } => Response::Pong {
                client_ts: ts,
                server_ts: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            },
            Request::RawConnect { .. } => Response::Error {
                code: 501,
                message: "Raw connect not available in standalone mode".into(),
            },
        }
    }
}

/// Handle a TCP connection: read requests, dispatch, write responses.
pub async fn handle_connection(mut stream: TcpStream, server: &Server) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 256 * 1024]; // 256KB read buffer

    loop {
        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e.into()),
        }
        let msg_len = u32::from_be_bytes(len_buf) as usize;
        if msg_len > buf.len() {
            let resp = Response::Error { code: 413, message: "Request too large".into() };
            write_response(&mut stream, &resp).await?;
            continue;
        }

        // Read CBOR payload
        stream.read_exact(&mut buf[..msg_len]).await?;

        let req = match decode_request(&buf[..msg_len]) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("decode error: {}", e);
                let resp = Response::Error { code: 400, message: format!("Decode error: {}", e) };
                write_response(&mut stream, &resp).await?;
                continue;
            }
        };

        log::debug!("request: {:?}", std::mem::discriminant(&req));
        let resp = server.handle_request(req).await;
        write_response(&mut stream, &resp).await?;
    }
}

async fn write_response(stream: &mut TcpStream, resp: &Response) -> anyhow::Result<()> {
    let cbor = encode_response(resp).map_err(|e| anyhow::anyhow!(e))?;
    let len = (cbor.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&cbor).await?;
    Ok(())
}
