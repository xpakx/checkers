use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use serde::{Deserialize, Serialize};

use crate::{board::{generate_bit_board, move_to_bitboard}, rabbit::DESTINATION_EXCHANGE, rules::{get_rules, MoveVerification}};

pub fn set_move_delegate(consumer: lapin::Consumer, channel: Channel) {
    consumer.set_delegate({
        move |delivery: DeliveryResult| {
            println!("New move verification request");
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
                let message: MoveEvent = match serde_json::from_str(message) {
                    Ok(msg) => msg,
                    Err(err) => {
                        println!("Failed to deserialize game event: {:?}", err);
                        return; // TODO
                    }
                };
                println!("Received message: {:?}", &message);


                let response = process_move(message);
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
                    .expect("Failed to acknowledge message"); // TODO
            }
        }
    }
    );
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveEvent {
    game_id: usize,
    game_state: String,
    #[serde(rename = "move")]
    mov: String,
    ruleset: RuleSet,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EngineEvent {
    game_id: usize,
    legal: bool,
    new_state: String,
    user: String,
    #[serde(rename = "move")]
    mov: String,
    ai: bool,
    finished: bool,
    lost: bool,
    won: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RuleSet {
    British,
}

impl Default for EngineEvent {
    fn default() -> EngineEvent {
        EngineEvent { 
            game_id: 0,
            legal: true,
            new_state: "".into(),
            user: "".into(),
            mov: "".into(),
            ai: false,
            finished: false,
            lost: false,
            won: false,
        } 
    } 
}

// TODO
fn process_move(message: MoveEvent) -> EngineEvent {
    let mov = move_to_bitboard(message.mov);
    let board = generate_bit_board(message.game_state);
    let rules = get_rules(match message.ruleset {
        RuleSet::British => crate::rules::RuleSet::British,
    });

    let _legality = match (mov, board) {
        (Ok(mov), Ok(board)) => rules.verify_move(&board, mov, &crate::Color::Red),
        (_, _) => MoveVerification::Illegal, 
    };
    EngineEvent {game_id: message.game_id, ..Default::default()}
}
