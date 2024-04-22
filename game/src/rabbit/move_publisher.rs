use std::{sync::Arc, time::Duration};

use lapin::Channel;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::AppState;

use super::MOVES_EXCHANGE;


#[derive(Clone, Serialize, Deserialize)]
pub struct MoveEvent {
    pub game_id: usize,
    pub game_state: String,
    pub column: usize,
    pub row: usize,
    // pub ruleset: GameRuleset,
    #[serde(skip_serializing)]
    pub ai: bool,
}

pub async fn move_publisher(channel: Channel, state: Arc<AppState>) {
    let mut rx = state.txmoves.subscribe();

    let chan = channel.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            //TODO
            let msg = serde_json::to_string(&event).unwrap();
            let routing_key = match event.ai {
                true => "move_ai",
                false => "move",
            };
            if let Err(err) = chan
                .basic_publish(
                    MOVES_EXCHANGE,
                    routing_key,
                    Default::default(),
                    msg.into_bytes().as_slice(),
                    Default::default(),
                    )
                    .await {
                        error!("Failed to publish message to destination exchange: {:?}", err);
                    };
        }
    });

    let mut test_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        test_interval.tick().await;
        match channel.status().connected() {
            false => break,
            true => {},
        }
    }
    send_task.abort();
}
