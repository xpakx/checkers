use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use ::serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{game::repository::{self, get_game_details}, rabbit::STATE_EXCHANGE, AppState};

use super::update_consumer::GameStatus;


pub fn set_game_delegate(consumer: lapin::Consumer, channel: Channel, state: Arc<AppState>) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New game message");
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
                let message: GameEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to deserialize game event: {:?}", err);
                        return; // TODO
                    }
                };
                info!("Received message: {:?}", &message);

                let game = get_game_details(&state.db, &message.game_id).await;
                
                let response = match game {
                    Err(_) => StateEvent { 
                        game_id: message.game_id, 
                        error: true,
                        error_message: Some("".into()),
                        ..Default::default()
                    },
                    Ok(game) => StateEvent {
                        game_id: game.id,
                        user: game.user,
                        opponent: match game.opponent {
                            None => "AI".into(),
                            Some(opp) => opp,
                        },
                        user_turn: game.user_turn,
                        user_starts: game.user_starts,
                        current_state: game.current_state,
                        game_type: match game.game_type {
                            repository::GameType::AI => GameType::AI,
                            repository::GameType::User => GameType::User,
                        },
                        ruleset: match game.ruleset {
                            repository::RuleSet::British => RuleSet::British,
                        },
                        ai_type: match game.ai_type {
                            repository::AIType::None => AIType::None,
                            repository::AIType::Random => AIType::Random,
                        },
                        status: match game.status {
                            repository::GameStatus::NotFinished => GameStatus::NotFinished,
                            repository::GameStatus::Lost => GameStatus::Lost,
                            repository::GameStatus::Won => GameStatus::Won,
                            repository::GameStatus::Drawn => GameStatus::Drawn,
                        },
                        ..Default::default()
                    },
                };
                info!("Response: {:?}", &response);
                let response = serde_json::to_string(&response).unwrap();

                if let Err(err) = channel
                    .basic_publish(
                        STATE_EXCHANGE,
                        "state",
                        Default::default(),
                        response.into_bytes().as_slice(),
                        Default::default(),
                        )
                        .await {
                            error!("Failed to publish message to destination exchange: {:?}", err);
                        };

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to acknowledge message");
            }
        }
    }
    );
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameEvent {
    game_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StateEvent {
    game_id: i64,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum AIType {
    None, Random,
}

impl Default for StateEvent {
    fn default() -> StateEvent {
        StateEvent { 
            game_id: 0,
            error: false,
            error_message: None,
            user: "".into(),
            opponent: "AI".into(),
            user_turn: true,
            user_starts: true,
            current_state: "".into(),
            game_type: GameType::User,
            ruleset: RuleSet::British,
            ai_type: AIType::None,
            status: GameStatus::NotFinished,
        } 
    } 
}
