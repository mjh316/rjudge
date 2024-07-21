use axum::{extract::State, http::StatusCode, routing::get, Router};
use piston_rs::Client;
use tokio;

#[derive(Clone)]
struct AppState {
    client: piston_rs::Client,
}

async fn root() -> &'static str {
    "Hello, rjudge!"
}

async fn get_runtimes() -> (StatusCode, String) {
    let client = piston_rs::Client::with_url("http://localhost:2000/api/v2");
    assert_eq!(client.get_url(), "http://localhost:2000/api/v2");
    if let Ok(runtimes) = client.fetch_runtimes().await {
        return (
            StatusCode::OK,
            format!(
                "Available runtimes: [{}]",
                runtimes
                    .iter()
                    .map(|x| format!("{}: {}", &x.language, &x.version))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        );
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch runtimes".to_string(),
        );
    }
}

async fn get_packages(AppState { client }: AppState) -> String {}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/runtimes", get(get_runtimes))
        .with_state(AppState {
            client: piston_rs::Client::with_url("http://localhost:2000/api/v2"),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
