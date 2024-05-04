use rabbit::{game_publisher::GameEvent, update_publisher::UpdateEvent};
use redis::{Connection, Commands};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, info};
use std::sync::{Arc, Mutex, RwLock};
use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::rabbit::lapin_listen;
use crate::rabbit::move_publisher::MoveEvent;
use crate::config::get_config;
use crate::rabbit::state_consumer::GameResponse;

mod rabbit;
mod config;

#[derive(Clone, Serialize, Deserialize)]
pub enum Color {
    Red, White,
}

#[tokio::main]
async fn main() {
    let config = get_config();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("checkers={}", config.debug_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let (tx, _rx) = broadcast::channel(100);

    // TODO?: use connection pool
    info!("Creating redis connection…");
    let redis = redis::Client::open(config.redis)
        .expect("Failed to connect to Redis");
    let redis = redis.get_connection()
        .expect("Failed to connect to Redis");
    info!("Connected to redis…");

    let (txmoves, _rxrabbit) = broadcast::channel(100);
    let (txgames, _rxrabbit) = broadcast::channel(100);
    let (txupdates, _rxrabbit) = broadcast::channel(100);
    let state = AppState { jwt: config.jwt_secret, tx, redis: Mutex::from(redis), txmoves, txgames, txupdates };
    let state = Arc::new(state);

    let lapin_state = state.clone();
    let mut cfg = deadpool_lapin::Config::default();
    cfg.url = Some(config.rabbit.into());
    let lapin_pool = cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)).unwrap();
    tokio::spawn(async move {lapin_listen(lapin_pool.clone(), lapin_state).await});

    let app = Router::new()
        .route("/ws", get(handle))
        .with_state(state.clone());

    info!("Initializing router…");
    let host = "0.0.0.0";
    let port = config.port;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

pub struct AppState {
    jwt: String,
    tx: broadcast::Sender<Msg>,
    redis: Mutex<Connection>,
    txmoves: broadcast::Sender<MoveEvent>,
    txgames: broadcast::Sender<GameEvent>,
    txupdates: broadcast::Sender<UpdateEvent>,
}

async fn handle(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

#[derive(Clone)]
pub struct Msg {
    msg: String,
    room: usize,
    user: Option<String>,
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();

    let mut rx = state.tx.subscribe();

    let room = Arc::new(RwLock::new(0));

    let timestamped_name = format!("guest_{}", chrono::Utc::now().timestamp());
    let username = Arc::new(RwLock::new(timestamped_name));

    let rm_send = room.clone();
    let u = username.clone();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Some(user) = msg.user {
                if *u.read().unwrap() != user {
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
            let username = name
                .try_read()
                .map(|a| a.clone());
            let Ok(username) = username else {
                continue;
            };
            match path.as_str() {
                "/subscribe" => {
                    let room_request: SubscribeRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    *rm.write().unwrap() = room_request.game_id;
                    let room = *rm.read().unwrap();

                    let game_loaded: Option<String> = state.redis
                        .lock()
                        .unwrap()
                        .get(format!("room_{}", room)).unwrap();

                    match game_loaded {
                        None => {
                            let event = GameEvent { game_id: room };
                            let _ = state.txgames.send(event);
                        },
                        Some(game) => {
                            let game: Game = serde_json::from_str(game.as_str()).unwrap();
                            let msg = GameResponse::from(&game);
                            let msg = serde_json::to_string(&msg).unwrap();
                            let msg = Msg { msg, room, user: Some(username) };
                            let _ = state.tx.send(msg);
                        }
                    }
                },
                "/move" => {
                    let move_request: MoveRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    let res = make_move(state, &username, *rm.read().unwrap(), move_request);
                    if let Err(res) = res {
                        let msg: Msg = Msg {room: *rm.read().unwrap(), msg: res, user: Some(username) };
                        let _ = tx.send(msg);
                    }
                },
                "/chat" => {
                    let chat_request: ChatRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    chat(state, &username, *rm.read().unwrap(), chat_request)
                },
                "/auth" => {
                    let auth_request: AuthRequest = match serde_json::from_str(text.as_str())  {
                        Err(_) => continue,
                        Ok(request) => request,
                    };
                    match token_to_username(auth_request.jwt, state) {
                        Ok(username) => {
                            *name.write().unwrap() = username.clone();
                            let msg = AuthMessage {authenticated: true, username: username.clone(), error: None };
                            let msg = serde_json::to_string(&msg).unwrap();
                            let msg: Msg = Msg {room: *rm.read().unwrap(), msg, user: Some(username) };
                            let _ = tx.send(msg);
                        },
                        Err(err) => {
                            let msg = AuthMessage {authenticated: false, username: username.clone(), error: Some(err.to_string()) };
                            let msg = serde_json::to_string(&msg).unwrap();
                            let msg: Msg = Msg {room: *rm.read().unwrap(), msg, user: Some(username) };
                            let _ = tx.send(msg);
                        },
                    };
                },
                _ => continue,
            };
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}


fn make_move(state: Arc<AppState>, username: &String, room: usize, request: MoveRequest) -> Result<(), String> {
    let game_db: Option<String> = state.redis
        .lock()
        .unwrap()
        .get(format!("room_{}", room)).unwrap();
    let Some(game_db) = game_db else {
        let event = GameEvent { game_id: room };
        let _ = state.txgames.send(event);
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
    let game_ser = serde_json::to_string(&game).unwrap();
    let _: () = state.redis
        .lock()
        .unwrap()
        .set(format!("room_{}", room), game_ser).unwrap();
    let color = game.get_current_color(); // TODO switch if rules definition says starting color is inverted

    let event = MoveEvent {
        game_id: game.id,
        game_state: game.current_state,
        mov: request.mov,
        ruleset: game.ruleset,
        color, 
    };
    let _ = state.txmoves.send(event);
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

impl TokenError {
    fn to_string(self) -> String {
        match self {
            TokenError::Expired => "Token expired",
            TokenError::Malformed => "Malformed token",
            TokenError::RefreshToken => "Cannot use refresh token for auth",
            TokenError::MissingToken => "No token",
        }.to_string()
    }
}


fn token_to_username(token: String, state: Arc<AppState>) -> Result<String, TokenError> {
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

    return Ok(claims.claims.sub);
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
    pub first_user_starts: bool,
    pub blocked: bool,
    pub current_state: String,
    pub ai_type: AIType,
    pub game_type: GameType,
    pub ruleset: RuleSet,
    pub status: GameStatus,
}

impl Game {
    pub fn get_current_color(&self) -> Color {
        match (self.first_user_starts, self.first_user_turn) {
            (true, true) => Color::White,
            (true, false) => Color::Red,
            (false, true) => Color::Red,
            (false, false) => Color::White,
        }
    }
    pub fn get_current_user(&self) -> String {
        match (self.first_user_starts, self.first_user_turn) {
            (true, true) => &self.user,
            (true, false) => &self.opponent,
            (false, true) => &self.opponent,
            (false, false) => &self.user,
        }.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum AIType {
    None, Random,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameType {
    User, AI,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RuleSet {
    British,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum GameStatus {
    NotFinished, Won, Lost, Drawn,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveRequest {
    #[serde(rename="move")]
    mov: String,
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

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest {
    jwt: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthMessage {
    username: String,
    authenticated: bool,
    error: Option<String>,
}
