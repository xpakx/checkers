use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{game::repository::{self, get_game, save_move, update_game, GameModel, MoveModel}, AppState};

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
                        return; // TODO
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
                        current_state: message.current_state.clone(),
                        user_turn: message.user_turn,
                        ..Default::default()
                    };
                    if let Ok(_) = update_game(&state.db, game).await {
                        let mv = MoveModel {
                            game_id: message.game_id,
                            current_state: message.current_state,
                            created_at: Some(message.timestamp),
                            last_move: message.last_move,
                            ..Default::default()
                        };
                        _ = save_move(&state.db, mv)
                    }
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
