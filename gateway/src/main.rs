use axum::{Router, routing::get, extract::{Request, State}, body::{Body, to_bytes}};
use reqwest::Client;
use tracing::{info, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "checkers=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client = reqwest::Client::new();
    info!("Created reqwest client.");

    let app = Router::new()
        .route("/", get(index))
        .route("/game/*path", get(game))
        .with_state(client);

    info!("Initializing router…");
    let host = "0.0.0.0";
    let port = 3000;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn index() -> String {
    debug!("Index requested.");
    return String::from("Hello, world!")
}

async fn game(State(client): State<Client>, req: Request<Body>) -> String {
    debug!("{} matched against 'game/**'.", req.uri());
    let target = "http://localhost";
    let target_port = 8081;
    let uri = format!("{}:{}{}", target, target_port, req.uri().path());
    debug!("Calling {}.", uri);
    let (parts, body) = req.into_parts();

    let response = client.request(parts.method, uri)
        .version(parts.version)
        .headers(parts.headers)
        .body(to_bytes(body, usize::MAX).await.unwrap())
        .send()
        .await;

    let result = response.unwrap().text().await.unwrap();

    return String::from(result)
}
