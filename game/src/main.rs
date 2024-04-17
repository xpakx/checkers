use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, info};
use std::sync::{Arc, RwLock};
use axum::{extract::{ws::{Message, WebSocket}, FromRef, FromRequestParts, State, WebSocketUpgrade}, http::{request::Parts, StatusCode}, response::{IntoResponse, Response}, routing::get, Router};
use axum::async_trait;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};

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

async fn handle(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>, user: UserData) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state, user.username))
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Msg {
    msg: String,
    room: usize,
    author: Option<String>,
}

async fn websocket(stream: WebSocket, state: Arc<AppState>, username: String) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = state.tx.subscribe();

    let room = Arc::new(RwLock::new(0));

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

pub struct UserData {
    pub username: String,
}

#[derive(Debug)]
pub enum TokenError {
    Expired,
    Malformed,
    RefreshToken,
    MissingToken,
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            TokenError::Expired => (StatusCode::FORBIDDEN, "Token expired"),
            TokenError::Malformed => (StatusCode::FORBIDDEN, "Malformed token"),
            TokenError::RefreshToken => (StatusCode::FORBIDDEN, "Cannot use refresh token for auth"),
            TokenError::MissingToken => (StatusCode::FORBIDDEN, "No token"),
        }
        .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserData
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = TokenError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("Authorization").ok_or(TokenError::MissingToken)?;
        let token = match auth_header.to_str() {
            Ok(header_value) => {
                if header_value.starts_with("Bearer ") {
                    header_value.trim_start_matches("Bearer ").to_string()
                } else {
                    return Err(TokenError::Malformed);
                }
            }
            Err(_) => return Err(TokenError::Malformed),
        };

        let state: Arc<AppState> = Arc::from_ref(state);
        let claims = decode::<TokenClaims>(
            &token,
            &DecodingKey::from_secret(state.jwt.as_ref()),
            &Validation::default(),
            );

        let claims = match claims {
            Ok(c) => c,
            Err(err) => match &err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => return Err(TokenError::Expired),
                _ => return Err(TokenError::Malformed),
            },
        };

        if claims.claims.refresh {
            return Err(TokenError::RefreshToken);
        }

        return Ok(UserData { username: claims.claims.sub });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    pub refresh: bool,
}
