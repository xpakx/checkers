use crate::{ai::Move, board::BitBoard};

mod british;

pub trait Rules {
    fn get_possible_moves(&self, board: &BitBoard) -> Vec<Move>;
}

pub enum RuleSet {
    British,
}

pub fn get_rules(ruleset: RuleSet) -> Box<dyn Rules> {
    match ruleset {
        RuleSet::British => Box::new(british::BritishRules::new()),
    }
}
