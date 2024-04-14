use tokio::sync::broadcast;
use tracing::{debug, info};
use std::sync::{Arc, RwLock};
use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use futures::{sink::SinkExt, stream::StreamExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("checkers={}", "debug").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let (tx, _rx) = broadcast::channel(100);
    let state = AppState { jwt: String::from("secret"), tx };

    let app = Router::new()
        .route("/ws", get(handle))
        .with_state(Arc::new(state));

    info!("Initializing router…");
    let host = "0.0.0.0";
    let port = 8082;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

#[allow(dead_code)]
pub struct AppState {
    jwt: String,
    tx: broadcast::Sender<Msg>,
}

async fn handle(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Msg {
    msg: String,
    room: usize,
    author: Option<String>,
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = state.tx.subscribe();

    let room = Arc::new(RwLock::new(0));
    let username = String::from("test");

    let rm_send = room.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if msg.room == *rm_send.read().unwrap() {
                if sender.send(Message::Text(msg.msg)).await.is_err() {
                    break;
                }
            }
        }
    });

    let tx = state.tx.clone();
    let name = username.clone();
    let rm = room.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            debug!(text);
            let msg = Msg { msg: text, author: Some(name.clone()), room: *rm.read().unwrap() };
            let _ = tx.send(msg);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
