use std::sync::Arc;

use lapin::{message::{Delivery, DeliveryResult}, options::BasicAckOptions, Channel};
use redis::Commands;
use ::serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{rabbit::MOVES_EXCHANGE, AIType, AppState, Color, Game, GameStatus, GameType, Msg, RuleSet};


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

                if let Ok(game) = get_game_from_message(&delivery, state.clone()) {
                    process_message(game, state, channel).await;
                }

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to acknowledge message");
            }
        }
    }
    );
}

async fn process_message(game: Game, state: Arc<AppState>, channel: Channel) {
    debug!("Adding state for game {} to Redis", game.id);
    let game_data = serde_json::to_string(&game).unwrap();
    let _: () = state.redis
        .lock()
        .unwrap()
        .set(format!("room_{}", game.id), game_data.clone()).unwrap();
    let msg = Msg { msg: game_data, room: game.id, user: None };
    let _ = state.tx.send(msg);

    if !game.first_user_turn && game.game_type == GameType::AI {
        let engine_event = AIMoveEvent {
            game_id: game.id,
            game_state: game.current_state,
            ruleset: game.ruleset,
            ai_type: game.ai_type,
            color: Color::White, // TODO
        };
        let engine_event = serde_json::to_string(&engine_event).unwrap();
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
}

fn get_game_from_message(delivery: &Delivery, state: Arc<AppState>) -> Result<Game, ()> {
    let message = std::str::from_utf8(&delivery.data).unwrap();
    let message: StateEvent = match serde_json::from_str(message) {
        Ok(msg) => msg,
        Err(err) => {
            error!("Failed to deserialize state event: {:?}", err);
            return Err(());
        }
    };
    info!("Received message: {:?}", &message);

    if message.error {
        debug!("Error in state event for game {}.", message.game_id);
        let msg = Msg { msg: message.error_message.unwrap(), room: message.game_id, user: None };
        let _ = state.tx.send(msg);
        return Err(());
    }

    if message.status != GameStatus::NotFinished {
        debug!("Finished state event for game {}.", message.game_id);
        let msg = Msg { msg: "Game is already finished.".into(), room: message.game_id, user: None };
        let _ = state.tx.send(msg);
        return Err(());
    }

    Ok(Game {
        blocked: false,
        finished: false,
        first_user_turn: message.user_turn,
        id: message.game_id,
        opponent: message.opponent,
        user: message.user,
        current_state: message.current_state,
        ai_type: message.ai_type,
        game_type: message.game_type,
        ruleset: message.ruleset,
        status: message.status,
    })
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


#[derive(Clone, Serialize, Deserialize)]
pub struct AIMoveEvent {
    pub game_id: usize,
    pub game_state: String,
    pub ruleset: RuleSet,
    pub ai_type: AIType,
    pub color: Color,
}
