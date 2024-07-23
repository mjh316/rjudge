use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicConsumeArguments, BasicPublishArguments, QueueBindArguments, QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
    BasicProperties,
};
use axum::{
    debug_handler,
    extract::{FromRef, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use piston_rs::{Client, Executor, PackagePayload};
use tokio::{self, time};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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

#[debug_handler]
async fn install_package(
    State(client): State<Client>,
    Json(payload): Json<PackagePayload>,
) -> (StatusCode, String) {
    if let Ok(_) = client
        .install_package(&payload.language, &payload.version)
        .await
    {
        return (StatusCode::OK, "Package installed".to_string());
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to install package".to_string(),
        );
    }
}

async fn post_submission(
    State(_client): State<Client>,
    Json(payload): Json<Executor>,
) -> (StatusCode, String) {
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let channel = connection.open_channel(None).await.unwrap();

    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::default())
        .await
        .unwrap()
        .unwrap();

    let routing_key = "amqprs.example";
    let exchange_name = "amq.topic";

    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            &exchange_name,
            routing_key,
        ))
        .await
        .unwrap();

    let content = serde_json::to_string(&payload).unwrap().into_bytes();
    let args = BasicPublishArguments::new(&exchange_name, routing_key);

    channel
        .basic_publish(BasicProperties::default(), content, args)
        .await
        .unwrap();

    time::sleep(time::Duration::from_secs(1)).await;

    channel.close().await.unwrap();
    connection.close().await.unwrap();

    (StatusCode::OK, "Submission received".to_string())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();

    let state = AppState {
        client: piston_rs::Client::with_url("http://localhost:2000/api/v2"),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/runtimes", get(get_runtimes))
        .route("/packages", get(get_packages))
        .route("/packages", post(install_package))
        .route("/submission", post(post_submission))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
