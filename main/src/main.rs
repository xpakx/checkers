use std::{fs::File, sync::Arc, io::Read};

use axum::{routing::post, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};

use crate::user::{service::{register, login, refresh_token}, AuthResponse};

mod security;
mod user;
mod validation;

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

#[derive(Deserialize, Serialize)]
struct Config {
    debug_level: Option<String>,
    port: Option<usize>,
    jwt_secret: Option<String>,
    db: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { 
            debug_level: None,
            port: None,
            jwt_secret: None,
            db: None,
        } 
    } 
}

#[allow(dead_code)]
struct ConfigFin {
    debug_level: String,
    port: usize,
    jwt_secret: String,
    db: String,
}

fn load_yaml_config(path: &str) -> Config {
    debug!("Reading services from yaml file…");
    let file = File::open(path);
    let Ok(mut file) = file else {
        debug!("No yaml configuration found.");
        return Config::default()
    };
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    debug!("Deserializing…");
    let config: Config = serde_yaml::from_str(&content).unwrap();
    config
}

fn get_config() -> ConfigFin {
    let config = load_yaml_config("config.yaml");
    ConfigFin {
        debug_level: match config.debug_level {
            None => String::from("debug"),
            Some(value) => value,
        },
        port: match config.port {
            None => 8080,
            Some(value) => value,
        },
        jwt_secret: match config.jwt_secret {
            None => String::from("secret"),
            Some(value) => value,
        },
        db: match config.db {
            None => String::from("postgresql://root:password@localhost:5432/checkers"),
            Some(value) => value,
        },
    }
}
