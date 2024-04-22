use std::sync::Arc;

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

pub fn move_publisher(channel: Channel, state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = state.txmoves.subscribe();
        while let Ok(event) = rx.recv().await {
            //TODO
            let msg = serde_json::to_string(&event).unwrap();
            let routing_key = match event.ai {
                true => "move_ai", // is it even needed?
                false => "move",
            };
            if let Err(err) = channel
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
    })
}
