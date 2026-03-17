//! RedBrowser protocol — compact binary messages over DNS tunnel.
//!
//! All messages are CBOR-encoded for minimal wire size.
//! Flows over smux streams inside the nooshdaroo DNS tunnel.

use serde::{Deserialize, Serialize};

/// Client -> Server request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    // -- Web browsing --
    Browse { url: String },
    Search { query: String },
    ImageFull { hash: Vec<u8>, quality: u8 },

    // -- Instagram --
    IgLogin { username: String, enc_password: Vec<u8> },
    IgChallenge { code: String },
    IgFeed { cursor: Option<String> },
    IgExplore { cursor: Option<String> },
    IgUserProfile { username: String },
    IgPost { shortcode: String },
    IgStories { user_id: u64 },
    IgLike { media_id: String },
    IgUnlike { media_id: String },
    IgComment { media_id: String, text: String },
    IgDmList,
    IgDmThread { thread_id: String, cursor: Option<String> },
    IgDmSend { thread_id: String, text: String },
    IgSearch { query: String },

    // -- Telegram --
    TgAuth { phone: String },
    TgAuthCode { code: String },
    TgAuthPassword { password: String },
    TgGetChats { offset: i32, limit: u8 },
    TgGetMessages { chat_id: i64, from_message_id: i64, limit: u8 },
    TgSendMessage { chat_id: i64, text: String },
    TgGetChat { chat_id: i64 },
    TgSearchChats { query: String },
    TgMarkRead { chat_id: i64, message_id: i64 },

    // -- WhatsApp --
    WaLinkRequest,
    WaGetChats,
    WaGetMessages { chat_id: String, count: u8 },
    WaSendMessage { chat_id: String, text: String },

    // -- System --
    RawConnect { host: String, port: u16 },
    Ping { ts: u64 },
}

/// Server -> Client response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    // -- Web --
    Page {
        url: String,
        title: String,
        html: Vec<u8>,  // gzip compressed
        original_size: u32,
    },
    SearchResults {
        query: String,
        html: Vec<u8>,
    },
    ImageData {
        hash: Vec<u8>,
        data: Vec<u8>,  // WebP bytes
    },

    // -- Instagram --
    IgLoginOk { username: String },
    IgChallengeRequired { challenge_type: String },
    IgFeedResult { posts: Vec<IgPost>, next_cursor: Option<String> },
    IgExploreResult { posts: Vec<IgPost>, next_cursor: Option<String> },
    IgUserResult { user: IgUser },
    IgPostResult { post: IgPostFull },
    IgStoriesResult { stories: Vec<IgStory> },
    IgDmListResult { threads: Vec<IgDmThread> },
    IgDmThreadResult { messages: Vec<IgDmMessage>, cursor: Option<String> },
    IgSearchResult { users: Vec<IgUserBrief>, tags: Vec<String> },

    // -- Telegram --
    TgAuthOk { user_name: String },
    TgAuthCodeNeeded,
    TgAuthPasswordNeeded { hint: String },
    TgChatsResult { chats: Vec<TgChat> },
    TgMessagesResult { messages: Vec<TgMessage> },
    TgChatResult { chat: TgChatFull },
    TgNewMessage { message: TgMessage },  // server push

    // -- WhatsApp --
    WaQrCode { png_data: Vec<u8> },
    WaLinked { phone: String },
    WaChatsResult { chats: Vec<WaChat> },
    WaMessagesResult { messages: Vec<WaMessage> },
    WaNewMessage { message: WaMessage },  // server push

    // -- System --
    RawConnected,
    RawData { data: Vec<u8> },
    Pong { client_ts: u64, server_ts: u64 },
    Error { code: u16, message: String },
    Ok,
}

// -- Instagram models --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgPost {
    pub shortcode: String,
    pub username: String,
    pub caption: String,  // truncated to 200 chars for feed
    pub thumbnail: Vec<u8>,  // WebP ~3KB
    pub like_count: u32,
    pub comment_count: u32,
    pub is_video: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgPostFull {
    pub shortcode: String,
    pub username: String,
    pub caption: String,
    pub image: Vec<u8>,  // WebP ~30KB (800px)
    pub like_count: u32,
    pub comment_count: u32,
    pub comments: Vec<IgComment>,
    pub is_video: bool,
    pub video_thumbnail: Option<Vec<u8>>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgComment {
    pub username: String,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgUser {
    pub username: String,
    pub full_name: String,
    pub bio: String,
    pub avatar: Vec<u8>,  // WebP ~5KB
    pub follower_count: u32,
    pub following_count: u32,
    pub post_count: u32,
    pub posts: Vec<IgPost>,  // latest 12
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgUserBrief {
    pub username: String,
    pub full_name: String,
    pub avatar: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgStory {
    pub user_id: u64,
    pub username: String,
    pub image: Vec<u8>,  // WebP
    pub timestamp: u64,
    pub is_video: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgDmThread {
    pub thread_id: String,
    pub participants: Vec<String>,
    pub last_message: String,
    pub timestamp: u64,
    pub unread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgDmMessage {
    pub sender: String,
    pub text: Option<String>,
    pub image: Option<Vec<u8>>,
    pub timestamp: u64,
}

// -- Telegram models --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TgChat {
    pub id: i64,
    pub title: String,
    pub chat_type: TgChatType,
    pub last_message: Option<TgMessagePreview>,
    pub unread_count: u32,
    pub avatar: Option<Vec<u8>>,  // tiny thumbnail
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TgChatType {
    Private,
    Group,
    Supergroup,
    Channel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TgMessagePreview {
    pub sender: String,
    pub text: String,  // truncated
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TgMessage {
    pub id: i64,
    pub chat_id: i64,
    pub sender: String,
    pub text: Option<String>,
    pub photo: Option<Vec<u8>>,  // thumbnail WebP
    pub timestamp: u64,
    pub is_outgoing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TgChatFull {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub member_count: Option<u32>,
    pub avatar: Option<Vec<u8>>,
}

// -- WhatsApp models --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaChat {
    pub id: String,
    pub name: String,
    pub last_message: String,
    pub timestamp: u64,
    pub unread_count: u32,
    pub is_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaMessage {
    pub id: String,
    pub chat_id: String,
    pub sender: String,
    pub text: Option<String>,
    pub image: Option<Vec<u8>>,
    pub timestamp: u64,
    pub is_outgoing: bool,
}

// -- Encoding helpers --

/// Encode a Request to CBOR bytes
pub fn encode_request(req: &Request) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    ciborium::into_writer(req, &mut buf)
        .map_err(|e| format!("cbor encode: {}", e))?;
    Ok(buf)
}

/// Decode a Request from CBOR bytes
pub fn decode_request(data: &[u8]) -> Result<Request, String> {
    ciborium::from_reader(data)
        .map_err(|e| format!("cbor decode request: {}", e))
}

/// Encode a Response to CBOR bytes
pub fn encode_response(resp: &Response) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    ciborium::into_writer(resp, &mut buf)
        .map_err(|e| format!("cbor encode: {}", e))?;
    Ok(buf)
}

/// Decode a Response from CBOR bytes
pub fn decode_response(data: &[u8]) -> Result<Response, String> {
    ciborium::from_reader(data)
        .map_err(|e| format!("cbor decode response: {}", e))
}

// -- Wire framing --
// Messages are length-prefixed: [u32 BE length][CBOR payload]

pub fn frame_message(cbor: &[u8]) -> Vec<u8> {
    let mut framed = Vec::with_capacity(4 + cbor.len());
    framed.extend_from_slice(&(cbor.len() as u32).to_be_bytes());
    framed.extend_from_slice(cbor);
    framed
}

pub fn read_frame(data: &[u8]) -> Option<(usize, &[u8])> {
    if data.len() < 4 { return None; }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + len { return None; }
    Some((4 + len, &data[4..4 + len]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_roundtrip() {
        let req = Request::Browse { url: "https://example.com".into() };
        let encoded = encode_request(&req).unwrap();
        let decoded = decode_request(&encoded).unwrap();
        match decoded {
            Request::Browse { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = Response::Page {
            url: "https://example.com".into(),
            title: "Example".into(),
            html: b"<h1>Hello</h1>".to_vec(),
            original_size: 50000,
        };
        let encoded = encode_response(&resp).unwrap();
        let decoded = decode_response(&encoded).unwrap();
        match decoded {
            Response::Page { title, .. } => assert_eq!(title, "Example"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_frame_roundtrip() {
        let data = b"hello world";
        let framed = frame_message(data);
        let (consumed, payload) = read_frame(&framed).unwrap();
        assert_eq!(consumed, framed.len());
        assert_eq!(payload, data);
    }

    #[test]
    fn test_ig_post_encoding_size() {
        let post = IgPost {
            shortcode: "CxYz1234".into(),
            username: "testuser".into(),
            caption: "A test post with some caption text".into(),
            thumbnail: vec![0u8; 3000],  // ~3KB thumbnail
            like_count: 12400,
            comment_count: 892,
            is_video: false,
            timestamp: 1710600000,
        };
        let encoded = encode_response(&Response::IgFeedResult {
            posts: vec![post; 20],
            next_cursor: Some("abc123".into()),
        }).unwrap();
        // 20 posts with 3KB thumbnails should be ~65-80KB total
        assert!(encoded.len() < 100_000, "feed too large: {} bytes", encoded.len());
    }
}
