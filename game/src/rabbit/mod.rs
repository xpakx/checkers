use std::{sync::Arc, time::Duration};

use deadpool_lapin::lapin::types::FieldTable;
use lapin::{options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, ExchangeKind};
use tracing::{debug, info};

use crate::AppState;

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
