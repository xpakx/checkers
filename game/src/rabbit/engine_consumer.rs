use std::sync::Arc;

use lapin::{message::{Delivery, DeliveryResult}, options::BasicAckOptions, Channel};
use redis::Commands;
use ::serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{AppState, Game, Msg};

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

async fn process_message(event: EngineEvent, state: Arc<AppState>, _channel: Channel) {
    let game_db: Option<String> = state.redis
        .lock()
        .unwrap()
        .get(format!("room_{}", event.game_id)).unwrap();
    let Some(game_db) = game_db else {
        return; // TODO
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
    game.first_user_turn = !game.first_user_turn;
    let game_data = serde_json::to_string(&game).unwrap();
    let _: () = state.redis
        .lock()
        .unwrap()
        .set(format!("room_{}", game.id), game_data.clone()).unwrap();
    // game finished?

    let msg = Msg { msg: "Move accepted".into(), room: game.id, user: Some(game.user) }; // TODO: more informative response
    let _ = state.tx.send(msg);
    // if ai move, sent new event
    // TODO: send update event
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
    row: usize,
    column: usize,
    ai: bool,
}
