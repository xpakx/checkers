use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{game::repository::{self, get_game, update_game, GameModel}, AppState};

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
                        return; // TODO
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

                let game = get_game(&state.db, &message.game_id).await;
                if let Ok(game) = game {
                    let game = GameModel {
                        id: game.id, 
                        status: match message.status {
                            GameStatus::NotFinished => repository::GameStatus::NotFinished,
                            GameStatus::Won => repository::GameStatus::Won,
                            GameStatus::Lost => repository::GameStatus::Lost,
                            GameStatus::Drawn => repository::GameStatus::Drawn,
                        },
                        current_state: message.current_state,
                        user_turn: message.user_turn,
                        ..Default::default()
                    };
                    _ = update_game(&state.db, game).await;
                    // TODO: save move to db
                }

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
