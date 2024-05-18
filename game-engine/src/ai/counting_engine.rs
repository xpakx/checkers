use rand::{rngs::ThreadRng, thread_rng};

use crate::{ai::Engine, board::BitBoard, rules::Rules, Color};

#[allow(dead_code)]
pub struct CountingEngine {
    rng: ThreadRng,
}

#[allow(dead_code)]
impl CountingEngine {
    pub fn new() -> CountingEngine {
        CountingEngine {
            rng: thread_rng(),
        }
    }
}

#[allow(dead_code)]
impl Engine for CountingEngine {
    fn get_name(&self) -> String {
        String::from("Counting Engine")
    }

    fn get_move(&mut self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>) -> u32 {
        return self.min_max_decision(board, color, rules, 10)
    }

}

#[allow(dead_code)]
impl CountingEngine {
    fn evaluate(&self, board: &BitBoard, color: &Color) -> u32 {
        // FIXME
        let (kings, opponent_kings) = match color {
            Color::Red => (board.red_kings, board.white_kings),
            Color::White => (board.white_kings, board.red_kings),
        };
        let (pawns, opponent_pawns) = match color {
            Color::Red => (board.red_pawns, board.white_pawns),
            Color::White => (board.white_pawns, board.red_pawns),
        };
        let strength = 10*kings.count_ones() + pawns.count_ones();
        let opponent_strength = 10*opponent_kings.count_zeros() + opponent_pawns.count_ones();
        strength - opponent_strength
    }

    fn generate_moves(&self, _board: &BitBoard, _rules: &Box<dyn Rules>) -> Vec<u32> {
        return vec![]; // TODO
    }

    fn next_color(&self, color: &Color) -> Color {
        match color {
            Color::Red => Color::White,
            Color::White => Color::Red
        }
    }

    fn min_max_decision(&self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>, depth: u32) -> u32 {
        let moves = self.generate_moves(board, rules);
        let next_player = self.next_color(color);
        let mut best_move = 0;
        let mut best_result = -1;
        for mov in moves {
            let new_board = board.apply_move(mov, color);
            let min = self.min_value(&new_board, &next_player, rules, depth);
            if min > best_result {
                best_result = min;
                best_move = mov;
            }
        }
        return best_move;
    }

    fn max_value(&self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>, depth: u32) -> i8 {
        if rules.is_game_won(board, color) {
            return -1;
        }
        if rules.is_game_drawn(board, color) {
            return 0;
        }
        // TODO: cut on depth==0
        let moves = self.generate_moves(board, rules);
        let next_player = self.next_color(color);
        let mut best_result = -1;
        for mov in moves {
            let new_board = board.apply_move(mov, color);
            let min = self.min_value(&new_board, &next_player, rules, depth-1);
            if min > best_result {
                best_result = min;
            }
        }
        return best_result;
    }

    fn min_value(&self, board: &BitBoard, color: &Color, rules: &Box<dyn Rules>, depth: u32) -> i8 {
        if rules.is_game_won(board, color) {
            return 1;
        }
        if rules.is_game_drawn(board, color) {
            return 0;
        }
        // TODO: cut on depth==0
        let moves = self.generate_moves(board, rules);
        let next_player = self.next_color(color);
        let mut best_result = 1;
        for mov in moves {
            let new_board = board.apply_move(mov, color);
            let max = self.max_value(&new_board, &next_player, rules, depth-1);
            if max < best_result {
                best_result = max;
            }
        }
        return best_result;
    }
}
