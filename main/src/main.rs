use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::post, Form, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "checkers=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = "postgresql://root:password@localhost:5432/checkers";
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
 
    info!("Connection to database established.");
    
    info!("Applying migrations...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/authenticate", post(register))
        .with_state(Arc::new(state));

    info!("Initializing router…");
    let host = "0.0.0.0";
    let port = 8080;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn register(State(state): State<Arc<AppState>>, Form(user): Form<UserRequest>) -> impl IntoResponse {
    info!("Creating new user requested…");

    // TODO: validation

    debug!("Trying to add user {:?} to db...", user.username);
    let query_result =
        sqlx::query("INSERT INTO account (username, password) VALUES ($1, $2)")
        .bind(&user.username)
        .bind(&user.password) // TODO: hashing password
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        debug!("cannot add user to db!");
        if err.contains("duplicate key") && err.contains("username") {
            // TODO
        }
        if !err.contains("duplicate key") {
            // TODO
        }
        debug!(err);
    }

    info!("User {:?} succesfully created.", user.username);
    return "Hello world"
}


#[allow(dead_code)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    username: Option<String>,
    password: Option<String>,
    password_re: Option<String>,
}
