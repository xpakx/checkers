use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use ::serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::AppState;

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

                let message = std::str::from_utf8(&delivery.data).unwrap();
                let message: EngineEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to deserialize game event: {:?}", err);
                        return; // TODO
                    }
                };
                info!("Received message: {:?}", &message);
                // TODO

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
struct EngineEvent {
    game_id: i64,
}
