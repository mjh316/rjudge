use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    routing::get,
    Router,
};
use piston_rs::Client;
use tokio;

#[derive(Clone)]
struct AppState {
    client: piston_rs::Client,
}

impl FromRef<AppState> for Client {
    fn from_ref(state: &AppState) -> Client {
        state.client.clone()
    }
}

async fn root() -> &'static str {
    "Hello, rjudge!"
}

async fn get_runtimes(State(client): State<Client>) -> (StatusCode, String) {
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

async fn get_packages(State(client): State<Client>) -> String {
    if let Ok(packages) = client.fetch_packages().await {
        return format!(
            "Available packages: [{}]",
            packages
                .iter()
                .map(|x| format!("{:#?}", &x))
                .collect::<Vec<String>>()
                .join(", ")
        );
    } else {
        return "Failed to fetch packages".to_string();
    }
}

async fn install_package(State(client): State<Client>) -> (StatusCode, String) {
    if let Ok(_) = client.install_package("python", "3.9.5").await {
        return (StatusCode::OK, "Package installed".to_string());
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to install package".to_string(),
        );
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        client: piston_rs::Client::with_url("http://localhost:2000/api/v2"),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/runtimes", get(get_runtimes))
        .route("/packages", get(get_packages))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
