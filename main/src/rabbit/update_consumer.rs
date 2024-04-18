use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use tracing::{info, error};

pub fn set_update_delegate(consumer: lapin::Consumer, channel: Channel) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            info!("New update message");
            let channel = channel.clone();
            async move {
                let _channel = channel.clone();
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
