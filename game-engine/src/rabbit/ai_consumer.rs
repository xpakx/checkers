use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};

use serde::{Deserialize, Serialize};

use crate::{ai::get_engine, board::generate_bit_board, rabbit::DESTINATION_EXCHANGE, rules::get_rules, Color};

use super::move_consumer::EngineEvent;

pub fn set_ai_delegate(consumer: lapin::Consumer, channel: Channel) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            println!("New ai move request");
            let channel = channel.clone();
            async move {
                let channel = channel.clone();
                let delivery = match delivery {
                    Ok(Some(delivery)) => delivery,
                    Ok(None) => return,
                    Err(error) => {
                        println!("Failed to consume queue message {}", error);
                        return;
                    }
                };

                let message = std::str::from_utf8(&delivery.data).unwrap();
                let message: AiEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        println!("Failed to deserialize ai event: {:?}", err);
                        return; // TODO
                    }
                };
                println!("Received message: {:?}", &message);


                let response = process_ai_event(message);
                println!("Response: {:?}", &response);
                let response = serde_json::to_string(&response).unwrap();

                if let Err(err) = channel
                    .basic_publish(
                        DESTINATION_EXCHANGE,
                        "engine",
                        Default::default(),
                        response.into_bytes().as_slice(),
                        Default::default(),
                        )
                        .await {
                            println!("Failed to publish message to destination exchange: {:?}", err);
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
struct AiEvent {
    game_id: usize,
    game_state: String,
    ruleset: RuleSet,
    ai_type: AIType,
    color: Color,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RuleSet {
    British,
}

#[derive(Serialize, Deserialize, Debug)]
enum AIType {
    Random,
}

// TODO
fn process_ai_event(message: AiEvent) -> EngineEvent {
    let board = generate_bit_board(message.game_state).unwrap(); // TODO
    let rules = get_rules(match message.ruleset {
        RuleSet::British => crate::rules::RuleSet::British,
    });
    let mut engine = get_engine(match message.ai_type {
        AIType::Random => crate::ai::EngineType::Random,
    });
    let mov = engine.get_move(&board, &message.color, &rules);
    let board = board.apply_move(mov, message.color);

    EngineEvent {game_id: message.game_id, new_state: board.to_string(), ..Default::default()}
}
