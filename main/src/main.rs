use std::sync::Arc;

use axum::{routing::{post, get}, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};

use crate::user::{service::{register, login, refresh_token}, AuthResponse};
use crate::game::service::{games, archive, requests, new_game};
use crate::config::get_config;

mod security;
mod user;
mod validation;
mod game;
mod config;

#[tokio::main]
async fn main() {
    info!("Getting config…");
    let config = get_config();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("checkers={}", config.debug_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = config.db;
    info!("Connecting to database…");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
 
    info!("Connection to database established.");
    
    info!("Applying migrations…");
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/register", post(register))
        .route("/authenticate", post(login))
        .route("/refresh", post(refresh_token))
        .route("/game", get(games))
        .route("/game/archive", get(archive))
        .route("/game/request", get(requests))
        .route("/game", post(new_game))
        .with_state(Arc::new(state));

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
    db: PgPool,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct UserModel {
    id: i32,
    username: String,
    password: String,
}
