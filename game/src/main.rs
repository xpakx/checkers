use redis::{Connection, Commands};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, info};
use std::sync::{Arc, Mutex, RwLock};
use axum::{extract::{ws::{Message, WebSocket}, FromRef, FromRequestParts, State, WebSocketUpgrade}, http::{request::Parts, StatusCode}, response::{IntoResponse, Response}, routing::get, Router};
use axum::async_trait;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::rabbit::lapin_listen;
use crate::rabbit::move_publisher::MoveEvent;

mod rabbit;

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

    // TODO?: use connection pool
    info!("Creating redis connection…");
    let redis_db = "redis://default:redispw@localhost:6379";
    let redis = redis::Client::open(redis_db)
        .expect("Failed to connect to Redis");
    let redis = redis.get_connection()
        .expect("Failed to connect to Redis");
    info!("Connected to redis…");

    let (txmoves, _rxrabbit) = broadcast::channel(100);
    let state = AppState { jwt: String::from("secret"), tx, redis: Mutex::from(redis), txmoves };
    let state = Arc::new(state);

    let lapin_state = state.clone();
    let rabbit_url = "redis://default:redispw@localhost:6379";
    let mut cfg = deadpool_lapin::Config::default();
    cfg.url = Some(rabbit_url.into());
    let lapin_pool = cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)).unwrap();
    tokio::spawn(async move {lapin_listen(lapin_pool.clone(), lapin_state).await});

    let app = Router::new()
        .route("/ws", get(handle))
        .with_state(state.clone());

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
    redis: Mutex<Connection>,
    txmoves: broadcast::Sender<MoveEvent>,
}

async fn handle(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>, user: UserData) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state, user.username))
}

#[derive(Clone)]
pub struct Msg {
    msg: String,
    room: usize,
    user: Option<String>,
}

async fn websocket(stream: WebSocket, state: Arc<AppState>, username: String) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = state.tx.subscribe();

    let room = Arc::new(RwLock::new(0));

    let rm_send = room.clone();
    let u = username.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Some(user) = msg.user {
                if user != u {
                    continue;
                }
            }
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
    let state = state.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let state = state.clone();
            debug!(text);
            let path: WsPath = match serde_json::from_str(text.as_str())  {
                Err(_) => continue,
                Ok(path) => path
            };
            let path = path.path;
            match path.as_str() {
                "/subscribe" => {
                    let room_request: SubscribeRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    *rm.write().unwrap() = room_request.game_id;
                },
                "/move" => {
                    let move_request: MoveRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    let res = make_move(state, &name, *rm.read().unwrap(), move_request);
                    if let Err(res) = res {
                        let msg: Msg = Msg {room: *rm.read().unwrap(), msg: res, user: Some(name.clone()) };
                        let _ = tx.send(msg);
                    }
                },
                "/chat" => {
                    let chat_request: ChatRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    chat(state, &name, *rm.read().unwrap(), chat_request)},
                _ => continue,
            };
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}


fn make_move(state: Arc<AppState>, username: &String, room: usize, _request: MoveRequest) -> Result<(), String> {
    let game_db: Option<String> = state.redis
        .lock()
        .unwrap()
        .get(format!("room_{}", room)).unwrap();
    let Some(game_db) = game_db else {
        return Err("Game not loaded!".into())
    };
    let mut game: Game = serde_json::from_str(game_db.as_str()).unwrap();
    if username != &game.user && username != &game.opponent {
        return Err("Not in game!".into())
    }
    if game.finished {
        return Err("Game is finished!".into())
    }
    if game.blocked {
        return Err("Cannot move now!".into())
    }
    if !((username == &game.user && game.first_user_turn) || (username == &game.opponent && !game.first_user_turn)) {
        return Err("Cannot move now!".into())
    }
    game.blocked = true;
    let game = serde_json::to_string(&game).unwrap();
    let _: () = state.redis
        .lock()
        .unwrap()
        .set(format!("room_{}", room), game).unwrap();

    // TODO: publish move to rabbitmq
    Ok(())
}

fn chat(state: Arc<AppState>, username: &String, room: usize, request: ChatRequest) {
    let msg = ChatMessage {player: username.clone(), message: request.message };
    let msg = serde_json::to_string(&msg).unwrap();
    let msg: Msg = Msg {room, msg, user: None };
    let _ = state.tx.send(msg);
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

#[derive(Debug, Serialize, Deserialize)]
struct WsPath {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: usize,
    pub user: String,
    pub opponent: String,
    pub finished: bool,
    pub first_user_turn: bool,
    pub blocked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveRequest {
    x: usize,
    y: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscribeRequest {
    game_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    player: String,
    message: String,
}
