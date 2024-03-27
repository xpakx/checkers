use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let app = Router::new()
        .route("/", get(index))
        .with_state(client);

    let host = "0.0.0.0";
    let port = 3000;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn index() -> String {
    return String::from("Hello, world!")
}
