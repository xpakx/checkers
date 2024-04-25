use crate::{board::BitBoard, rules::Rules, Color};

mod random_engine;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Move {
    pub start: Pos,
    pub end: Pos,
    pub result: BitBoard,
}

pub trait Engine {
    fn get_name(&self) -> String;
    fn get_move(&mut self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>) -> u32;
}

pub enum EngineType {
    Random,
}

pub fn get_engine(engine: EngineType) -> Box<dyn Engine> {
    match engine {
        EngineType::Random => Box::new(random_engine::RandomEngine::new()),
    }
}
