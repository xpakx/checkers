use std::sync::Arc;

use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use tracing::{info, error};

use crate::AppState;

pub fn set_update_delegate(consumer: lapin::Consumer, channel: Channel, state: Arc<AppState>) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New update message");
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
