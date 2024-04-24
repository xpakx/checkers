use crate::{board::BitBoard, Color};

mod british;

pub trait Rules {
    fn get_possible_movers(&self, board: &BitBoard, color: Color) -> u32;
    fn get_possible_jumpers(&self, board: &BitBoard, color: Color) -> u32;
    fn get_moves(&self, board: &BitBoard, mover: u32, color: Color) -> Vec<u32>;
    fn get_jumps(&self, board: &BitBoard, mover: u32, color: Color) -> Vec<u32>;
    fn get_definition(&self) -> RuleDefiniton;
}

pub enum RuleSet {
    British,
}

#[derive(Debug)]
pub struct RuleDefiniton {
    flying_kings: bool,
    board_size: usize,
    backward_pawns: bool,
    maximal_capture: bool,
    capture_forced: bool,
}

pub fn get_rules(ruleset: RuleSet) -> Box<dyn Rules> {
    match ruleset {
        RuleSet::British => Box::new(british::BritishRules::new()),
    }
}
