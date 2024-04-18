use std::{fmt::write, sync::Arc};

use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use ::serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{game::repository::get_game_details, rabbit::STATE_EXCHANGE, AppState};


pub fn set_game_delegate(consumer: lapin::Consumer, channel: Channel, state: Arc<AppState>) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New game message");
            let channel = channel.clone();
            let state = state.clone();
            async move {
                let _channel = channel.clone();
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
                let message: GameEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to deserialize move message: {:?}", err);
                        return;
                    }
                };
                info!("Received message: {:?}", &message);

                let game = get_game_details(&state.db, &message.game_id).await;
                
                // TODO
                let response = match game {
                    Err(_) => StateEvent { game_id: message.game_id, error: true, error_message: Some("".into()) },
                    Ok(game) => StateEvent { game_id: game.id, error: false, error_message: None },
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
                    .expect("Failed to acknowledge message"); // TODO
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
    // TODO
}
