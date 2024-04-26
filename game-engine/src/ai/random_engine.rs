use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::{ai::Engine, board::BitBoard, rules::Rules, Color, BIT_MASK};

pub struct RandomEngine {
    rng: ThreadRng,
}

impl RandomEngine {
    pub fn new() -> RandomEngine {
        RandomEngine {
            rng: thread_rng(),
        }
    }
}

impl Engine for RandomEngine {
    fn get_name(&self) -> String {
        String::from("Random Engine")
    }

    fn get_move(&mut self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>) -> u32 {
        let def = rules.get_definition();
        let jumpers = rules.get_possible_jumpers(board, color);
        let movers = match !def.capture_forced || jumpers == 0 {
            true => rules.get_possible_movers(board, color),
            false => 0,
        };
        
        let mut moves = Vec::new();
        for i in 1..=32 {
            let mover = jumpers & (BIT_MASK >> i-1);
            if mover > 0 {
                let mut new_jumps = rules.get_jumps(board, mover, color);
                moves.append(&mut new_jumps);
            }
            let mover = movers & (BIT_MASK >> i-1);
            if mover > 0 {
                let mut new_moves = rules.get_moves(board, mover, color);
                moves.append(&mut new_moves);
            }
        }

        if moves.len() == 0 {
            return 0;
        }

        let index = self.rng.gen_range(0..moves.len());
        moves[index]
    }
}
