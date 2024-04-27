use std::sync::Arc;

use lapin::Channel;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{AppState, RuleSet, Color};

use super::MOVES_EXCHANGE;

#[derive(Clone, Serialize, Deserialize)]
pub struct MoveEvent {
    pub game_id: usize,
    pub game_state: String,
    #[serde(rename = "move")]
    pub mov: String,
    pub ruleset: RuleSet,
    pub color: Color,
}

pub fn move_publisher(channel: Channel, state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = state.txmoves.subscribe();
        while let Ok(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if let Err(err) = channel
                .basic_publish(
                    MOVES_EXCHANGE,
                    "move",
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
