use red_protocol::Response;

pub async fn handle_login() -> Response {
    Response::Error { code: 501, message: "Instagram integration coming soon".into() }
}

pub async fn handle_feed() -> Response {
    Response::Error { code: 501, message: "Instagram integration coming soon".into() }
}

pub async fn handle_post(_shortcode: &str) -> Response {
    Response::Error { code: 501, message: "Instagram integration coming soon".into() }
}

pub async fn handle_search(_query: &str) -> Response {
    Response::Error { code: 501, message: "Instagram integration coming soon".into() }
}

pub async fn handle_stub() -> Response {
    Response::Error { code: 501, message: "Instagram integration coming soon".into() }
}
