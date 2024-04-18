use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::AppState;

pub fn set_update_delegate(consumer: lapin::Consumer, state: Arc<AppState>) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New update message");
            let state = state.clone();
            async move {
                let _state = state.clone();
                let delivery = match delivery {
                    Ok(Some(delivery)) => delivery,
                    Ok(None) => return,
                    Err(error) => {
                        error!("Failed to consume queue message {}", error);
                        return;
                    }
                };

                let message = std::str::from_utf8(&delivery.data).unwrap();
                let message: UpdateEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to deserialize update event: {:?}", err);
                        return;
                    }
                };
                info!("Received message: {:?}", &message);

                // TODO: update game in db
                // TODO: save move to db

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to acknowledge message"); // TODO
            }
        }
    }
    );
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateEvent {
    game_id: i64,
    status: GameStatus,
    current_state: String,
    user_turn: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
    last_move: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameStatus {
    NotFinished, Won, Lost, Drawn,
}
