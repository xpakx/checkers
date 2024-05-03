use std::sync::Arc;

use lapin::Channel;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::AppState;

use super::GAMES_EXCHANGE;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub game_id: usize,
}

pub fn game_publisher(channel: Channel, state: Arc<AppState>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = state.txgames.subscribe();
        while let Ok(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if let Err(err) = channel
                .basic_publish(
                    GAMES_EXCHANGE,
                    "game",
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
