use red_protocol::Response;

pub async fn handle_auth_stub() -> Response {
    Response::Error { code: 501, message: "Telegram integration coming soon".into() }
}

pub async fn handle_chats_stub() -> Response {
    Response::Error { code: 501, message: "Telegram integration coming soon".into() }
}

pub async fn handle_send_stub() -> Response {
    Response::Error { code: 501, message: "Telegram integration coming soon".into() }
}

pub async fn handle_stub() -> Response {
    Response::Error { code: 501, message: "Telegram integration coming soon".into() }
}
