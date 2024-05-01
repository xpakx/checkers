use std::sync::Arc;

use lapin::{message::{Delivery, DeliveryResult}, options::BasicAckOptions, Channel};
use redis::Commands;
use ::serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{AppState, Color, Game, GameStatus, GameType, Msg};

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
        if !event.ai {
            let msg = get_error_message(game.id, game.get_current_user(), event.mov);
            let _ = state.tx.send(msg);
        }
        return
    }

    let old_state = game.current_state.clone();
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
    let color = game.get_current_color();
    let user = game.get_current_user();

    let game_data = serde_json::to_string(&game).unwrap();
    let _: () = state.redis
        .lock()
        .unwrap()
        .set(format!("room_{}", game.id), game_data.clone()).unwrap();

    let msg = get_move_message(game.id, &old_state, &(game.current_state), user, event.mov.clone(), &color);
    let _ = state.tx.send(msg);

    let event = UpdateEvent {
        game_id: game.id,
        status: game.status,
        current_state: game.current_state.clone(),
        user_turn: game.first_user_turn,
        last_move: event.mov,
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
    ai: bool,
    finished: bool,
    lost: bool,
    won: bool,
    #[serde(rename = "move")]
    pub mov: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveWsMessage {
    player: String,
    mov: String,
    legal: bool,
    details: Option<MoveDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveDetails {
    start: usize,
    end: usize,
    captures: Vec<usize>,
}

fn get_move_message(id: usize, state_old: &String, state_new: &String, player: String, mov: String, color: &Color) -> Msg {
    let len = state_old.len();
    let old: Vec<char> = state_old.to_lowercase().chars().collect();
    let new: Vec<char> = state_new.to_lowercase().chars().collect();
    let mut captures = Vec::new();
    for i in 0..len {
        match color {
            Color::Red => if old[i] == 'x' && new[i] != 'x' {
                captures.push(i+1);
            }, 
            Color::White => if old[i] == 'o' && new[i] != 'o'  {
                captures.push(i+1);
            }, 
        }
    }

    let (start, end) = match get_start_end(&mov) {
        Some(x) => x,
        None => (0, 0),
    };

    let msg = MoveWsMessage { 
        player, 
        mov, 
        legal: true, 
        details: Some(MoveDetails {
            start,
            end,
            captures,
        }),
    };
    let msg = serde_json::to_string(&msg).unwrap();
    Msg { msg, room: id, user: None }
}

fn get_start_end(mov: &String) -> Option<(usize, usize)> {
    let parts: Vec<&str> = mov.split(&['x', '-'][..]).collect();
    if let Some(first) = parts.first().and_then(|s| s.parse::<usize>().ok()) {
        if let Some(last) = parts.last().and_then(|s| s.parse::<usize>().ok()) {
            return Some((first, last));
        }
    }
    None
}

fn get_error_message(id: usize, player: String, mov: String) -> Msg {
    let msg = MoveWsMessage { 
        player: player.clone(), 
        mov, 
        legal: false, 
        details: None,
    };
    let msg = serde_json::to_string(&msg).unwrap();
    Msg { msg, room: id, user: Some(player) }
}
