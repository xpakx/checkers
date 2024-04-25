use std::time::Duration;

use lapin::{options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, types::FieldTable, ExchangeKind};

mod board;
mod ai;
mod rules;
use crate::ai::{get_engine, EngineType};
use crate::rules::{get_rules, RuleSet, MoveVerification};
use regex::Regex;

#[tokio::main]
async fn main() {
    let board = "xxxxxxxxxxxx........oooooooooooo";
    let bitboard = board::generate_bit_board(board.into());
    println!("{:?}", bitboard);
    let bitboard = bitboard.unwrap();

    println!("{:032b}", bitboard.white_pawns);
    println!("{:032b}", bitboard.red_pawns);
    bitboard.print();
    let mut engine = get_engine(EngineType::Random);
    let rules = get_rules(RuleSet::British);
    println!("{}", engine.get_name());
    let mov = engine.get_move(&bitboard, &Color::White, &rules);
    println!("{:032b}", mov);
    let bitboard = bitboard.apply_move(mov, Color::White);
    bitboard.print();
    println!("{:?}", bitboard);
    let mov = engine.get_move(&bitboard, &Color::Red, &rules);
    let bitboard = bitboard.apply_move(mov, Color::Red);
    bitboard.print();
    println!("{:?}", bitboard);
    let mov = engine.get_move(&bitboard, &Color::White, &rules);
    let bitboard = bitboard.apply_move(mov, Color::White);
    bitboard.print();
    println!("{:?}", bitboard);
    let mov = engine.get_move(&bitboard, &Color::Red, &rules);
    let bitboard = bitboard.apply_move(mov, Color::Red);
    bitboard.print();
    println!("{:?}", bitboard);

    println!("rules: {:?}", rules.get_definition());
    println!("white: {:032b}", rules.get_possible_movers(&bitboard, &Color::White));
    println!("red: {:032b}", rules.get_possible_movers(&bitboard, &Color::Red));

    let moves = vec!["10x13", "10-1", "10-1-15-4", "2x5x4", "12x32x30", "12x34x56", "10xxx10", "x10x10"];
    for mov in moves {
        match move_to_bitboard(String::from(mov)) {
            Ok(bitboard) => println!("{}, Bitboard representation: {:032b}", mov, bitboard.mov),
            Err(err) => println!("{}, Error: {:?}", mov, err),
        }
    }
    let board = "xxxxxxxxxxxx........oooooooooooo";
    let bitboard = board::generate_bit_board(board.into());
    let bitboard = bitboard.unwrap();
    bitboard.print();
    let mov = move_to_bitboard("10-15".into());
    let mov = rules.verify_move(&bitboard, mov.unwrap(), &Color::White);
    if let MoveVerification::Ok(mov) = mov {
        let bitboard = bitboard.apply_move(mov, Color::White);
        bitboard.print();
    }


    let rabbit_url = "amqp://guest:guest@localhost:5672";
    let mut cfg = deadpool_lapin::Config::default();
    cfg.url = Some(rabbit_url.into());
    let lapin_pool = cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)).unwrap();
    lapin_listen(lapin_pool.clone()).await;
}

pub async fn lapin_listen(pool: deadpool_lapin::Pool) {
    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("Connecting rmq consumer...");
        match init_lapin_listen(pool.clone()).await {
            Ok(_) => println!("RabbitMq listen returned"),
            Err(e) => println!("RabbitMq listen had an error: {}", e),
        };
    }
}

const EXCHANGE_NAME: &str = "checkers.moves.topic";
const MOVES_QUEUE: &str = "checkers.moves.queue"; // move
const AI_QUEUE: &str = "checkers.moves.ai.queue"; // move_ai

pub const DESTINATION_EXCHANGE: &str = "checkers.engine.topic";

async fn init_lapin_listen(pool: deadpool_lapin::Pool) -> Result<(), Box<dyn std::error::Error>> {
    let rmq_con = pool.get().await
        .map_err(|e| {
        println!("Could not get RabbitMQ connnection: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    channel.queue_declare(
        MOVES_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await
        .expect("Cannot declare queue");

    channel
        .queue_bind(
            MOVES_QUEUE,
            EXCHANGE_NAME,
            "move",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await
        .expect("Cannot bind queue");

    channel.queue_declare(
        AI_QUEUE,
        QueueDeclareOptions::default(),
        Default::default(),
        )
        .await
        .expect("Cannot declare queue");

    channel
        .queue_bind(
            AI_QUEUE,
            EXCHANGE_NAME,
            "move_ai",
            QueueBindOptions::default(),
            FieldTable::default(),
            )
        .await
        .expect("Cannot bind queue");

    channel
        .exchange_declare(
            DESTINATION_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
            )
        .await
        .expect("Cannot declare exchange");

    let _move_consumer = channel.basic_consume(
        MOVES_QUEUE,
        "engine_move_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await
        .expect("Cannot create consumer");

    let _ai_consumer = channel.basic_consume(
        AI_QUEUE,
        "engine_ai_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default())
        .await
        .expect("Cannot create consumer");
    
    let mut test_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        test_interval.tick().await;
        match channel.status().connected() {
            false => break,
            true => {},
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum Color {
    White,
    Red,
}

#[derive(Debug)]
enum ParseError {
    InvalidFormat,
    NumberOverflow,
    InvalidDigit,
}

const BIT_MASK: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

fn move_to_bitboard(move_string: String) -> Result<MoveBit, ParseError> {
    let move_regex = Regex::new(r"^(\d+(x|-))*\d+$").unwrap();

    if !move_regex.is_match(move_string.as_str()) {
        return Err(ParseError::InvalidFormat);
    }

    let mut current_num = 0;
    let mut mov: u32 = 0;
    let mut start_end: u32 = 0;

    for c in move_string.chars() {
        if c.is_digit(10) {
            current_num *= 10;
            current_num += c.to_digit(10).ok_or(ParseError::InvalidDigit)?;
            if current_num > 32 {
                return Err(ParseError::NumberOverflow);
            }
        } 
        match c {
            'x' => {
                if start_end == 0 {
                    start_end = BIT_MASK >> (current_num - 1);
                    current_num = 0;
                    continue;
                }
                if current_num != 0 {
                    mov |= BIT_MASK >> (current_num - 1);
                    current_num = 0;
                }
            },
            '-' => {
                if start_end == 0 {
                    start_end = BIT_MASK >> (current_num - 1);
                }
                current_num = 0;
            },
            _ => {}
        }
    }

    start_end |=  BIT_MASK >> (current_num - 1);
    mov |= start_end;
    Ok(MoveBit { start_end, mov })
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MoveBit {
    mov: u32,
    start_end: u32,
}
