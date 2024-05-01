use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};
use serde::{Deserialize, Serialize};

use crate::{board::{generate_bit_board, move_to_bitboard}, rabbit::DESTINATION_EXCHANGE, rules::{get_rules, MoveVerification}, Color};

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
                    .expect("Failed to acknowledge message");
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
    color: Color,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineEvent {
    pub game_id: usize,
    pub legal: bool,
    pub new_state: String,
    #[serde(rename = "move")]
    pub mov: String,
    pub ai: bool,
    pub finished: bool,
    pub lost: bool,
    pub won: bool,
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
    let mov = move_to_bitboard(message.mov.clone());
    let board = generate_bit_board(message.game_state.clone());
    let rules = get_rules(match message.ruleset {
        RuleSet::British => crate::rules::RuleSet::British,
    });

    let legality = match (mov, &board) {
        (Ok(mov), Ok(board)) => rules.verify_move(board, mov, &message.color),
        (_, _) => MoveVerification::Illegal, 
    };
    let legal = match legality {
        MoveVerification::Ok(_) => true,
        _ => false,
    };
    let state = match (legality, &board) {
        (MoveVerification::Ok(mov), Ok(board)) => board.apply_move(mov, message.color).to_string(),
        _ => message.game_state,
    };
    EngineEvent {
        game_id: message.game_id,
        new_state: state,
        mov: message.mov,
        legal,
        ..Default::default()
    }
}
