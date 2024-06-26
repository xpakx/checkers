use lapin::{message::DeliveryResult, options::BasicAckOptions, Channel};

use serde::{Deserialize, Serialize};

use crate::{ai::get_engine, board::{generate_bit_board, have_captures, have_promotions}, rabbit::DESTINATION_EXCHANGE, rules::get_rules, Color};

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
    noncapture_moves: usize,
    nonpromoting_moves: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RuleSet {
    British,
}

#[derive(Serialize, Deserialize, Debug)]
enum AIType {
    Random,
    Counting,
}

// TODO
fn process_ai_event(message: AiEvent) -> EngineEvent {
    let old_board = generate_bit_board(message.game_state).unwrap(); // TODO
    let rules = get_rules(match message.ruleset {
        RuleSet::British => crate::rules::RuleSet::British,
    });
    let mut engine = get_engine(match message.ai_type {
        AIType::Random => crate::ai::EngineType::Random,
        AIType::Counting => crate::ai::EngineType::Counting,
    });
    let mov = engine.get_move(&old_board, &message.color, &rules);
    println!("move: {:032b}", mov);
    let move_string = rules.move_to_string(&old_board, mov, &message.color);
    println!("move string: {}", move_string);
    let board = old_board.apply_move(mov, &message.color);
    println!("old white pawns: {:032b}", old_board.white_pawns);
    println!("old red pawns:   {:032b}", old_board.red_pawns);
    println!("old white kings: {:032b}", old_board.white_kings);
    println!("old red kings:   {:032b}", old_board.red_kings);
    println!("white pawns:     {:032b}", board.white_pawns);
    println!("red pawns:       {:032b}", board.red_pawns);
    println!("white kings:     {:032b}", board.white_kings);
    println!("red kings:       {:032b}", board.red_kings);
    let won = rules.is_game_won(&board, &message.color);
    let noncaptures = match have_captures(&old_board, &board, &message.color) {
        true => 0,
        false => message.noncapture_moves,
    };
    let nonpromotions = match have_promotions(&old_board, &board, &message.color) {
        true => 0,
        false => message.nonpromoting_moves,
    };
    println!("last capture: {}, last promotion: {}", noncaptures, nonpromotions);
    let drawn = !won && rules.is_game_drawn(noncaptures, nonpromotions);
    let finished = won || drawn;

    EngineEvent {
        game_id: message.game_id,
        new_state: board.to_string(),
        ai: true,
        legal: true,
        won,
        finished,
        mov: move_string,
        ..Default::default()
    }
}
