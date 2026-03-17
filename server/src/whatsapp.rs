use red_protocol::Response;

pub async fn handle_link_stub() -> Response {
    Response::Error { code: 501, message: "WhatsApp integration coming soon".into() }
}

pub async fn handle_chats_stub() -> Response {
    Response::Error { code: 501, message: "WhatsApp integration coming soon".into() }
}

pub async fn handle_stub() -> Response {
    Response::Error { code: 501, message: "WhatsApp integration coming soon".into() }
}
