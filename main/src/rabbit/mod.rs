use std::{sync::Arc, time::Duration};

use deadpool_lapin::lapin::types::FieldTable;
use lapin::{options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, ExchangeKind};
use tracing::{debug, info};

use crate::{rabbit::{game_consumer::set_game_delegate, update_consumer::set_update_delegate}, AppState};

mod game_consumer;
mod update_consumer;

pub const STATE_EXCHANGE: &str = "checkers.state.topic";

const UPDATES_EXCHANGE: &str = "checkers.updates.topic";
const UPDATES_QUEUE: &str = "checkers.updates.queue";

const GAMES_EXCHANGE: &str = "checkers.games.topic";
const GAMES_QUEUE: &str = "checkers.games.queue";

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

    channel.queue_declare(
        UPDATES_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await?;
    debug!("Declared queue {:?}", UPDATES_QUEUE);

    channel
        .queue_bind(
            UPDATES_QUEUE,
            UPDATES_EXCHANGE,
            "update",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await?;
    debug!("Declared bind {:?} -> {:?}", UPDATES_EXCHANGE, UPDATES_QUEUE);

    channel.queue_declare(
        GAMES_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await?;
    debug!("Declared queue {:?}", GAMES_QUEUE);

    channel
        .queue_bind(
            GAMES_QUEUE,
            GAMES_EXCHANGE,
            "game",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await?;
    debug!("Declared bind {:?} -> {:?}", GAMES_EXCHANGE, GAMES_QUEUE);

    let game_consumer = channel.basic_consume(
        GAMES_QUEUE,
        "games_main_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await?;

    let update_consumer = channel.basic_consume(
        UPDATES_QUEUE,
        "updates_main_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await?;

    debug!("Consumer connected, waiting for messages");
    set_game_delegate(game_consumer, channel.clone(), state.clone());
    set_update_delegate(update_consumer, channel.clone(), state.clone());
    let mut test_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        test_interval.tick().await;
        match channel.status().connected() {
            false => break,
            true => {},
        }
    }
    Ok(())
}
