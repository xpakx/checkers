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
                // TODO: deserialize
                info!("Received message: {:?}", &message);

                // TODO: Process and serialize
                // TODO: publish response

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to acknowledge message"); // TODO
            }
        }
    }
    );
}
