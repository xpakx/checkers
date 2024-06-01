use rand::{rngs::ThreadRng, thread_rng};

use crate::{ai::Engine, board::BitBoard, rules::Rules, Color, BIT_MASK};

#[allow(dead_code)]
pub struct CountingEngine {
    rng: ThreadRng,
}

impl CountingEngine {
    pub fn new() -> CountingEngine {
        CountingEngine {
            rng: thread_rng(),
        }
    }
}

impl Engine for CountingEngine {
    fn get_name(&self) -> String {
        String::from("Counting Engine")
    }

    fn get_move(&mut self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>) -> u32 {
        return self.min_max_decision(board, color, rules, 10)
    }

}

impl CountingEngine {
    fn evaluate(&self, board: &BitBoard, color: &Color) -> i16 {
        let (kings, opponent_kings) = match color {
            Color::Red => (board.red_kings, board.white_kings),
            Color::White => (board.white_kings, board.red_kings),
        };
        let (pawns, opponent_pawns) = match color {
            Color::Red => (board.red_pawns, board.white_pawns),
            Color::White => (board.white_pawns, board.red_pawns),
        };
        let strength = (10*kings.count_ones() + pawns.count_ones()) as i16; // max = 120, min = 0
        let opponent_strength = (10*opponent_kings.count_zeros() + opponent_pawns.count_ones()) as i16;
        strength - opponent_strength
    }

    fn generate_moves(&self, board: &BitBoard, rules: &Box<dyn Rules>, color: &Color) -> Vec<u32> {
        let mut moves = vec![];
        let jumpers = rules.get_possible_jumpers(board, color);
        let any_jumper = jumpers.count_ones() > 0;
        if any_jumper {
            for i in 1..=32 {
                let mover = jumpers & (BIT_MASK >> i-1);
                if mover > 0 {
                    let mut new_jumps = rules.get_jumps(board, mover, color);
                    moves.append(&mut new_jumps);
                }
            }
        }

        if !any_jumper || !rules.get_definition().capture_forced {
            let movers = rules.get_possible_movers(board, color);
            if movers.count_ones() > 0 {
                for i in 1..=32 {
                    let mover = movers & (BIT_MASK >> i-1);
                    if mover > 0 {
                        let mut new_moves = rules.get_moves(board, mover, color);
                        moves.append(&mut new_moves);
                    }
                }
            }
        }
        return moves
    }

    fn next_color(&self, color: &Color) -> Color {
        match color {
            Color::Red => Color::White,
            Color::White => Color::Red
        }
    }

    fn min_max_decision(&self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>, depth: u32) -> u32 {
        let moves = self.generate_moves(board, rules, color);
        let next_player = self.next_color(color);
        let mut best_move = 0;
        let mut best_result = -200;
        for mov in moves {
            let new_board = board.apply_move(mov, color);
            let min = self.min_value(&new_board, &next_player, &color, rules, depth-1);
            if min >= best_result {
                best_result = min;
                best_move = mov;
            }
        }
        return best_move;
    }

    fn max_value(&self, board: &BitBoard, color: &Color, start_color: &Color, rules: &Box<dyn Rules>, depth: u32) -> i16 {
        if rules.is_game_won(board, color) {
            return -200;
        }
        if rules.is_game_drawn(0, 0) { //TODO
            return 0;
        }
        if depth == 0 {
            return self.evaluate(board, start_color);
        }
        let moves = self.generate_moves(board, rules, color);
        let next_player = self.next_color(color);
        let mut best_result = -200;
        for mov in moves {
            let new_board = board.apply_move(mov, color);
            let min = self.min_value(&new_board, &next_player, &start_color, rules, depth-1);
            if min > best_result {
                best_result = min;
            }
        }
        return best_result;
    }

    fn min_value(&self, board: &BitBoard, color: &Color, start_color: &Color, rules: &Box<dyn Rules>, depth: u32) -> i16 {
        if rules.is_game_won(board, color) {
            return 200;
        }
        if rules.is_game_drawn(0, 0) { //TODO
            return 0;
        }
        if depth == 0 {
            return self.evaluate(board, start_color);
        }
        let moves = self.generate_moves(board, rules, color);
        let next_player = self.next_color(color);
        let mut best_result = 200;
        for mov in moves {
            let new_board = board.apply_move(mov, color);
            let max = self.max_value(&new_board, &next_player, &start_color, rules, depth-1);
            if max < best_result {
                best_result = max;
            }
        }
        return best_result;
    }
}
