use crate::{board::{BitBoard, MoveBit}, Color};

mod british;

pub trait Rules {
    fn get_possible_movers(&self, board: &BitBoard, color: &Color) -> u32;
    fn get_possible_jumpers(&self, board: &BitBoard, color: &Color) -> u32;
    fn get_moves(&self, board: &BitBoard, mover: u32, color: &Color) -> Vec<u32>;
    fn get_jumps(&self, board: &BitBoard, mover: u32, color: &Color) -> Vec<u32>;
    fn get_definition(&self) -> RuleDefiniton;
    fn verify_move(&self, board: &BitBoard, mov: MoveBit, color: &Color) -> MoveVerification;
    fn is_game_won(&self, board: &BitBoard, color: &Color) -> bool;
    fn is_game_drawn(&self, board: &BitBoard, color: &Color) -> bool;
}

pub enum RuleSet {
    British,
}

#[derive(Debug, PartialEq)]
pub enum MoveVerification {
    Ok(u32),
    Illegal,
    Ambiguous,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct RuleDefiniton {
    pub flying_kings: bool,
    pub board_size: usize,
    pub backward_pawns: bool,
    pub maximal_capture: bool,
    pub capture_forced: bool,
}

pub fn get_rules(ruleset: RuleSet) -> Box<dyn Rules> {
    match ruleset {
        RuleSet::British => Box::new(british::BritishRules::new()),
    }
}
