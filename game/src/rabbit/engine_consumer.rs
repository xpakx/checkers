use std::sync::Arc;

use lapin::{message::{Delivery, DeliveryResult}, options::BasicAckOptions, Channel};
use redis::Commands;
use ::serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{AppState, Game, GameStatus, GameType, Msg};

use super::{state_consumer::AIMoveEvent, update_publisher::UpdateEvent, MOVES_EXCHANGE};

pub fn set_engine_delegate(consumer: lapin::Consumer, channel: Channel, state: Arc<AppState>) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New engine message");
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

                if let Ok(game) = get_event_from_message(&delivery) {
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

async fn process_message(event: EngineEvent, state: Arc<AppState>, channel: Channel) {
    let game_db: Option<String> = state.redis
        .lock()
        .unwrap()
        .get(format!("room_{}", event.game_id)).unwrap();
    let Some(game_db) = game_db else {
        return;
    };
    let mut game: Game = serde_json::from_str(game_db.as_str()).unwrap();

    if !event.legal {
        game.blocked = false;
        let game_data = serde_json::to_string(&game).unwrap();
        let _: () = state.redis
            .lock()
            .unwrap()
            .set(format!("room_{}", game.id), game_data.clone()).unwrap();
        if event.ai {
            let msg = Msg { msg: "Not legal move".into(), room: game.id, user: Some(game.user) }; // TODO: more informative response
            let _ = state.tx.send(msg);
        }
        return
    }

    game.current_state = event.new_state;
    game.blocked = false;
    if event.finished {
        game.finished = true;
        game.status = match (event.lost, event.won, game.first_user_turn) {
            (true, false, true) => GameStatus::Lost,
            (false, true, true) => GameStatus::Won,
            (true, false, false) => GameStatus::Won,
            (false, true, false) => GameStatus::Lost,
            _ => GameStatus::Drawn,
        };
    }
    game.first_user_turn = !game.first_user_turn;
    let color = game.get_current_color(); // TODO

    let game_data = serde_json::to_string(&game).unwrap();
    let _: () = state.redis
        .lock()
        .unwrap()
        .set(format!("room_{}", game.id), game_data.clone()).unwrap();

    let msg = Msg { msg: "Move accepted".into(), room: game.id, user: Some(game.user) }; // TODO: more informative response
    let _ = state.tx.send(msg);

    let event = UpdateEvent {
        game_id: game.id,
        status: game.status,
        current_state: game.current_state.clone(),
        user_turn: game.first_user_turn,
        last_move: event.mov, // TODO
        timestamp: chrono::Utc::now(),
    };
    let _ = state.txupdates.send(event);

    if !game.first_user_turn && game.game_type == GameType::AI && !game.finished {
        let engine_event = AIMoveEvent {
            game_id: game.id,
            game_state: game.current_state,
            ruleset: game.ruleset,
            ai_type: game.ai_type,
            color,
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

fn get_event_from_message(delivery: &Delivery) -> Result<EngineEvent, ()> {
    let message = std::str::from_utf8(&delivery.data).unwrap();
    let message: EngineEvent = match serde_json::from_str(message) {
        Ok(msg) => msg,
        Err(err) => {
            error!("Failed to deserialize game event: {:?}", err);
            return Err(());
        }
    };
    info!("Received message: {:?}", &message);
    return Ok(message);
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EngineEvent {
    game_id: i64,
    legal: bool,
    new_state: String,
    user: String,
    ai: bool,
    finished: bool,
    lost: bool,
    won: bool,
    #[serde(rename = "move")]
    pub mov: String,
}
