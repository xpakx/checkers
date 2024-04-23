use crate::{board::BitBoard, Color};

mod british;

pub trait Rules {
    fn get_possible_movers(&self, board: &BitBoard, color: Color) -> u32;
}

pub enum RuleSet {
    British,
}

pub fn get_rules(ruleset: RuleSet) -> Box<dyn Rules> {
    match ruleset {
        RuleSet::British => Box::new(british::BritishRules::new()),
    }
}
