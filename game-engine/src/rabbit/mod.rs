use std::time::Duration;

use lapin::{options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, types::FieldTable, ExchangeKind};

pub async fn lapin_listen(pool: deadpool_lapin::Pool) {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("Connecting rmq consumer...");
        match init_lapin_listen(pool.clone()).await {
            Ok(_) => println!("RabbitMq listen returned"),
            Err(e) => println!("RabbitMq listen had an error: {}", e),
        };
    }
}

const EXCHANGE_NAME: &str = "checkers.moves.topic";
const MOVES_QUEUE: &str = "checkers.moves.queue"; // move
const AI_QUEUE: &str = "checkers.moves.ai.queue"; // move_ai

pub const DESTINATION_EXCHANGE: &str = "checkers.engine.topic";

async fn init_lapin_listen(pool: deadpool_lapin::Pool) -> Result<(), Box<dyn std::error::Error>> {
    let rmq_con = pool.get().await
        .map_err(|e| {
        println!("Could not get RabbitMQ connnection: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    channel.queue_declare(
        MOVES_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await
        .expect("Cannot declare queue");

    channel
        .queue_bind(
            MOVES_QUEUE,
            EXCHANGE_NAME,
            "move",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await
        .expect("Cannot bind queue");

    channel.queue_declare(
        AI_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await
        .expect("Cannot declare queue");

    channel
        .queue_bind(
            AI_QUEUE,
            EXCHANGE_NAME,
            "move_ai",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await
        .expect("Cannot bind queue");

    channel
        .exchange_declare(
            DESTINATION_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await
        .expect("Cannot declare exchange");

    let _move_consumer = channel.basic_consume(
        MOVES_QUEUE,
        "engine_move_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await
        .expect("Cannot create consumer");

    let _ai_consumer = channel.basic_consume(
        AI_QUEUE,
        "engine_ai_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await
        .expect("Cannot create consumer");
    
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
