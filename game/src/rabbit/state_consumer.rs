use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use redis::Commands;
use ::serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{rabbit::MOVES_EXCHANGE, AppState, Game, Msg};


pub fn set_state_delegate(consumer: lapin::Consumer, channel: Channel, state: Arc<AppState>) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New state message");
            let channel = channel.clone();
            let state = state.clone();
            async move {
                let channel = channel.clone();
                let state = state.clone();
                let delivery = match delivery {
                    Ok(Some(delivery)) => delivery,
                    Ok(None) => return,
                    Err(error) => {
                        error!("Failed to consume queue message {}", error);
                        return;
                    }
                };

                let message = std::str::from_utf8(&delivery.data).unwrap();
                let message: StateEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to deserialize state event: {:?}", err);
                        return; // TODO
                    }
                };
                info!("Received message: {:?}", &message);

                if message.error {
                    debug!("Error in state event for game {}.", message.game_id);
                    let msg = Msg { msg: message.error_message.unwrap(), room: message.game_id, user: None };
                    let _ = state.tx.send(msg);
                    return; // TODO
                }

                if message.status != GameStatus::NotFinished {
                    debug!("Finished state event for game {}.", message.game_id);
                    let msg = Msg { msg: "Game is already finished.".into(), room: message.game_id, user: None };
                    let _ = state.tx.send(msg);
                    return; // TODO
                }

                debug!("Adding state for game {} to Redis", message.game_id);
                // TODO
                let game_db = Game {
                    blocked: false,
                    finished: false,
                    first_user_turn: message.user_turn,
                    id: message.game_id,
                    opponent: message.opponent,
                    user: message.user,
                };

                let game_data = serde_json::to_string(&game_db).unwrap();
                let _: () = state.redis
                    .lock()
                    .unwrap()
                    .set(format!("room_{}", message.game_id), game_data.clone()).unwrap();

                // TODO
                let msg = Msg { msg: game_data, room: message.game_id, user: None };
                let _ = state.tx.send(msg);

                if !message.user_turn && message.ai_type != AIType::None {
                    let engine_event = String::from(""); // TODO
                    if let Err(err) = channel
                        .basic_publish(
                            MOVES_EXCHANGE,
                            "move_ai",
                            Default::default(),
                            engine_event.into_bytes().as_slice(),
                            Default::default(),
                            )
                            .await {
                                error!("Failed to publish message to destination exchange: {:?}", err);
                            };
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
struct StateEvent {
    game_id: usize, // TODO
    error: bool,
    error_message: Option<String>,

    user: String,
    opponent: String,
    user_turn: bool,
    user_starts: bool,
    current_state: String,
    game_type: GameType,
    ruleset: RuleSet,
    ai_type: AIType,
    status: GameStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameType {
    User, AI,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RuleSet {
    British,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AIType {
    None, Random,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameStatus {
    NotFinished, Won, Lost, Drawn,
}
