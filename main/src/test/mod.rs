use std::{env, sync::{Arc, Mutex}};

use axum::{extract::Request, body::{Body, to_bytes}, http::StatusCode, Router};
use once_cell::sync::Lazy;
use tower::ServiceExt;

use sqlx::{postgres::PgPoolOptions, PgPool};
use testcontainers::{runners::SyncRunner, Container};
use testcontainers_modules::postgres;

use crate::{config::get_config, AppState};

static DB: Lazy<Mutex<Container<postgres::Postgres>>> = Lazy::new(|| { config() });

fn config() -> Mutex<Container<postgres::Postgres>> {
    let container = postgres::Postgres::default()
        .with_user("user")
        .with_password("password")
        .with_db_name("checkers")
        .start();

    println!("{:?}", container.ports());

    println!("Container started.");
    let postgres_ip = container.get_host();
    println!("Container host {}", postgres_ip);
    let postgres_port = container.get_host_port_ipv4(5432);
    println!("Container port {}", postgres_port);
    let postgres_url = format!("postgresql://user:password@{}:{}/checkers", postgres_ip, postgres_port);
    println!("Container address {}", postgres_url);

    env::set_var("DEBUG_LEVEL", "info");
    env::set_var("SERVER_PORT", "8080");
    env::set_var("JWT_SECRET", "secret");
    env::set_var("DB_URL", postgres_url);
    env::set_var("RABBIT_URL", "");
    Mutex::new(container)
}

async fn get_app() -> (PgPool, Router) {
    let config = get_config();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.db)
        .await
        .unwrap();
 
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();
    let state = Arc::from(AppState {jwt: config.jwt_secret, db: pool.clone() });
    (pool, crate::get_router(state))
}

#[test]
fn test_container() {
    let config = DB.lock().unwrap();
    let host = config.get_host();
    assert!(host.to_string().len() > 0);
}

#[tokio::test]
async fn username_should_be_unique() {
    let _config = DB.lock().unwrap();
    let (db, app) = get_app().await;

    let _ = sqlx::query("INSERT INTO account (username, password) VALUES ($1, $2)")
        .bind("Test")
        .bind("password")
        .execute(&db)
        .await;

    let response = app
        .oneshot(
            Request::builder()
            .method("POST")
            .header("Content-Type", "application/json")
            .uri("/register")
            .body(Body::from("username=Test&psw=password&psw_repeat=password"))
            .unwrap()
            )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 1000).await;
    assert!(body.is_ok());
    let bytes = body.unwrap();
    let content = std::str::from_utf8(&*bytes).unwrap();
    assert!(content.contains("username exists"));
}
