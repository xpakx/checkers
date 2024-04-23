use std::sync::Arc;

use lapin::Channel;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{AppState, GameStatus};

use super::UPDATES_EXCHANGE;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEvent {
    pub game_id: usize,
    pub status: GameStatus,
    pub current_state: String,
    pub user_turn: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub last_move: String,
}

pub fn update_publisher(channel: Channel, state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = state.txupdates.subscribe();
        while let Ok(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if let Err(err) = channel
                .basic_publish(
                    UPDATES_EXCHANGE,
                    "update",
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
