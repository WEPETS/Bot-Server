use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::Html,
    routing::{any_service, MethodRouter},
};
use tower_http::services::ServeDir;

use crate::{get_config, utils::read_file};

pub fn serve_dir() -> MethodRouter {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Resource not found.")
    }

    any_service(
        ServeDir::new(&get_config().SERVICE_WEB_FOLDER)
            .not_found_service(handle_404.into_service()),
    )
}

pub async fn welcome(username: &str, msg: Option<&str>, pub_key: Option<&str>) -> Html<String> {
    // Read the content of the HTML file dynamically
    let file_content: String = read_file("web-public/index.html").await;

    // Replace the placeholder with the actual string
    let mut html_content = file_content.replace("{username}", username);
    if let Some(msg) = msg {
        html_content = html_content.replace("{message}", msg);
    }
    if let Some(pub_key) = pub_key {
        html_content = html_content.replace("{public_key}", pub_key);
    }

    Html(html_content)
}
