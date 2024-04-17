use std::{sync::Arc, time::Duration};

use axum::{routing::{post, get}, Router};
use deadpool_lapin::lapin::types::FieldTable;
use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{debug, info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};

use crate::user::{service::{register, login, refresh_token}, AuthResponse};
use crate::game::service::{games, archive, requests, new_game, accept_request, game, moves};
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

    let mut cfg = deadpool_lapin::Config::default();
    cfg.url = Some("amqp://guest:guest@localhost:5672".into());
    let lapin_pool = cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)).unwrap();
    tokio::spawn(async move {lapin_listen(lapin_pool.clone()).await});

    let state = AppState { db: pool, jwt: config.jwt_secret };

    let app = Router::new()
        .route("/register", post(register))
        .route("/authenticate", post(login))
        .route("/refresh", post(refresh_token))
        .route("/game", get(games))
        .route("/game/archive", get(archive))
        .route("/game/request", get(requests))
        .route("/game", post(new_game))
        .route("/game/:id/request", post(accept_request))
        .route("/game/:id", get(game))
        .route("/game/:id/history", get(moves))
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
    jwt: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct UserModel {
    id: i64,
    username: String,
    password: String,
}

async fn lapin_listen(pool: deadpool_lapin::Pool) {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("connecting rmq consumer...");
        match init_lapin_listen(pool.clone()).await {
            Ok(_) => debug!("RabbitMq listen returned"),
            Err(e) => debug!("RabbitMq listen had an error: {}", e),
        };
    }
}

async fn init_lapin_listen(pool: deadpool_lapin::Pool) -> Result<(), Box<dyn std::error::Error>> {
    let rmq_con = pool.get().await
        .map_err(|e| {
        debug!("Could not get RabbitMQ connnection: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    let queue = channel
        .queue_declare(
            "test_queue",
            deadpool_lapin::lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    debug!("Declared queue {:?}", queue);

    let consumer = channel
        .basic_consume(
            "test_queue",
            "my_consumer",
            deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    debug!("Consumer connected, waiting for messages");
    set_delegate(consumer, channel);
    Ok(())
}

pub fn set_delegate(consumer: lapin::Consumer, channel: Channel) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New AMQP message");
            let channel = channel.clone();
            async move {
                let _channel = channel.clone();
                let delivery = match delivery {
                    Ok(Some(delivery)) => delivery,
                    Ok(None) => return,
                    Err(error) => {
                        error!("Failed to consume queue message {}", error);
                        return;
                    }
                };

                let message = std::str::from_utf8(&delivery.data).unwrap();
                // TODO: deserialize
                info!("Received message: {:?}", &message);

                // TODO: Process and serialize
                // TODO: publish response

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to acknowledge message"); // TODO
            }
        }
    }
    );
}
