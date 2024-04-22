use std::{sync::Arc, time::Duration};

use deadpool_lapin::lapin::types::FieldTable;
use lapin::{options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, ExchangeKind};
use tracing::{debug, info};

use crate::{rabbit::{engine_consumer::set_engine_delegate, move_publisher::move_publisher, state_consumer::set_state_delegate}, AppState};

mod engine_consumer;
mod state_consumer;
pub mod move_publisher;

const UPDATES_EXCHANGE: &str = "checkers.updates.topic";
const GAMES_EXCHANGE: &str = "checkers.games.topic";

pub const STATE_EXCHANGE: &str = "checkers.state.topic";
const STATE_QUEUE: &str = "checkers.states.queue";

pub const MOVES_EXCHANGE: &str = "checkers.moves.topic";

pub const ENGINE_EXCHANGE: &str = "checkers.engine.topic";
const ENGINE_QUEUE: &str = "checkers.engine.queue";

pub async fn lapin_listen(pool: deadpool_lapin::Pool, state: Arc<AppState>) {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        info!("Connecting rmq consumer...");
        match init_lapin_listen(pool.clone(), state.clone()).await {
            Ok(_) => debug!("RabbitMq listen returned"),
            Err(e) => debug!("RabbitMq listen had an error: {}", e),
        };
    }
}

async fn init_lapin_listen(pool: deadpool_lapin::Pool, state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let rmq_con = pool.get().await
        .map_err(|e| {
        debug!("Could not get RabbitMQ connnection: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    channel
        .exchange_declare(
            UPDATES_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await?;
    debug!("Declared exchange {:?}", UPDATES_EXCHANGE);

    channel
        .exchange_declare(
            GAMES_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await?;
    debug!("Declared exchange {:?}", GAMES_EXCHANGE);

    channel
        .exchange_declare(
            STATE_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await?;
    debug!("Declared exchange {:?}", STATE_EXCHANGE);
    
    channel.queue_declare(
        STATE_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await?;
    debug!("Declared queue {:?}", STATE_QUEUE);

    channel
        .queue_bind(
            STATE_QUEUE,
            STATE_EXCHANGE,
            "state",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await?;
    debug!("Declared bind {:?} -> {:?}", STATE_EXCHANGE, STATE_QUEUE);

    channel
        .exchange_declare(
            MOVES_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await?;
    debug!("Declared exchange {:?}", MOVES_EXCHANGE);

    channel
        .exchange_declare(
            ENGINE_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await?;
    debug!("Declared exchange {:?}", ENGINE_EXCHANGE);
    
    channel.queue_declare(
        ENGINE_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await?;
    debug!("Declared queue {:?}", ENGINE_QUEUE);

    channel
        .queue_bind(
            ENGINE_QUEUE,
            ENGINE_EXCHANGE,
            "engine_move",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await?;
    debug!("Declared bind {:?} -> {:?}", ENGINE_EXCHANGE, ENGINE_QUEUE);

    let engine_consumer = channel.basic_consume(
        ENGINE_QUEUE,
        "engine_game_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await?;

    let state_consumer = channel.basic_consume(
        STATE_QUEUE,
        "state_game_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await?;

    debug!("Consumer connected, waiting for messages");
    set_engine_delegate(engine_consumer, channel.clone(), state.clone());
    set_state_delegate(state_consumer, channel.clone(), state.clone());


    let mut handles = vec![];
    handles.push(move_publisher(channel.clone(), state.clone()));

    let mut test_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        test_interval.tick().await;
        match channel.status().connected() {
            false => break,
            true => {},
        }
    }

    for task in handles {
        task.abort();
    };
    Ok(())
}
