use std::{collections::VecDeque, vec};

use crate::{board::{BitBoard, MoveBit}, rules::Rules, Color, BIT_MASK};

use super::{MoveVerification, RuleDefiniton};

// no flying kings, 8x8, pawns cannot move backwards, 
// any capture sequence can be chosen, but captures are forced, 
// red moves first 
pub struct BritishRules {
}

impl BritishRules {
    pub fn new() -> BritishRules {
        BritishRules { }
    }
}

const MASK_3_DOWN: u32 = 0b0000_0111_0000_0111_0000_0111_0000_0000;
const MASK_3_UP: u32 = 0b0000_0000_1110_0000_1110_0000_1110_0000;
const MASK_5_DOWN: u32 = 0b1110_0000_1110_0000_1110_0000_1110_0000;
const MASK_5_UP: u32 = 0b0000_0111_0000_0111_0000_0111_0000_0111;

impl Rules for BritishRules {
    fn get_definition(&self) -> RuleDefiniton {
        RuleDefiniton { 
            backward_pawns: false,
            board_size: 8,
            capture_forced: true,
            flying_kings: false,
            maximal_capture: false,
        }
    }

    fn get_possible_movers(&self, board: &BitBoard, color: &Color) -> u32 {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
        match color {
            Color::White => self.get_white_movers(board, not_occupied),
            Color::Red => self.get_red_movers(board, not_occupied),
        }
    }

    fn get_possible_jumpers(&self, board: &BitBoard, color: &Color) -> u32 {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
        match color {
            Color::White => self.get_white_jumpers(board, not_occupied),
            Color::Red => self.get_red_jumpers(board, not_occupied),
        }
    }

    // mover should have only one bit set
    fn get_moves(&self, board: &BitBoard, mover: u32, color: &Color) -> Vec<u32> {
        match color {
            Color::White => self.get_white_moves(board, mover),
            Color::Red => self.get_red_moves(board, mover),
        }
    }

    // mover should have only one bit set
    fn get_jumps(&self, board: &BitBoard, mover: u32, color: &Color) -> Vec<u32> {
        match color {
            Color::White => self.get_white_jumps(board, mover),
            Color::Red => self.get_red_jumps(board, mover),
        }
    }

    fn verify_move(&self, board: &BitBoard, mov: MoveBit, color: &Color) -> MoveVerification {
        let start = match color {
            Color::White => mov.start_end & (board.white_pawns | board.white_kings),
            Color::Red => mov.start_end & (board.red_pawns | board.red_kings),
        };
        if start.count_ones() != 1 {
            return MoveVerification::Illegal
        }
        let jumps = self.get_jumps_with_positions(board, start, color);
        let matched_jumps: Vec<&u32> = jumps.iter()
            .filter(|&j| {
                j.start_end == mov.start_end
                    &&
                j.intermediate_positions & mov.mov == mov.mov
            })
            .map(|j| &j.mov)
            .collect();
        if matched_jumps.len() == 1 {
            return MoveVerification::Ok(*matched_jumps[0])
        } else if matched_jumps.len() > 1 {
            return MoveVerification::Ambiguous
        }
        let any_jumpers = self.get_possible_jumpers(board, color) != 0;
        if any_jumpers {
            return MoveVerification::Illegal
        }
        let moves = self.get_moves(board, start, color);
        let matched_moves: Vec<&u32> = moves.iter().filter(|&j| j & mov.start_end == mov.start_end).collect();
        match matched_moves.len() {
            1 => MoveVerification::Ok(*matched_moves[0]),
            0 => MoveVerification::Illegal,
            _ => MoveVerification::Ambiguous,
        }
    }

    fn is_game_won(&self, board: &BitBoard, color: &Color) -> bool {
        let opponent = match color {
            Color::Red => board.white_pawns | board.white_kings,
            Color::White => board.red_pawns | board.red_kings,
        };
        if opponent.count_ones() == 0 {
            return true;
        };
        let color = match color {
            Color::Red => Color::White,
            Color::White => Color::Red,
        };
        let jumpers = self.get_possible_jumpers(board, &color);
        let movers = self.get_possible_movers(board, &color);
        let movers = movers | jumpers;
        if movers.count_ones() == 0 {
            return true;
        }
        false
    }

    fn is_game_drawn(&self, _board: &BitBoard, _color: &Color) -> bool {
        // TODO
        false
    }
}

impl BritishRules {
    fn get_white_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let pieces = board.white_pawns | board.white_kings;
        let movers = (not_occupied << 4) & pieces;
        let movers_3 = (not_occupied & MASK_3_UP) << 3 & pieces;
        let movers_5 = (not_occupied & MASK_5_UP) << 5 & pieces;
        let mut movers = movers | movers_3 | movers_5;
        if board.white_kings != 0 {
            let kmovers = (not_occupied >> 4) & board.white_kings;
            let kmovers_3 = (not_occupied & MASK_3_DOWN) >> 3 & board.white_kings;
            let kmovers_5 = (not_occupied & MASK_5_DOWN) >> 5 & board.white_kings;
            movers = movers | kmovers | kmovers_3 | kmovers_5;
        }
        movers
    }

    fn get_red_movers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let pieces = board.red_pawns | board.red_kings;
        let movers = (not_occupied >> 4) & pieces;
        let movers_3 = (not_occupied & MASK_3_DOWN) >> 3 & pieces;
        let movers_5 = (not_occupied & MASK_5_DOWN) >> 5 & pieces;
        let mut movers = movers | movers_3 | movers_5;
        if board.red_kings != 0 {
            let kmovers = (not_occupied << 4) & board.red_kings;
            let kmovers_3 = (not_occupied & MASK_3_UP) << 3 & board.red_kings;
            let kmovers_5 = (not_occupied & MASK_5_UP) << 5 & board.red_kings;
            movers = movers | kmovers | kmovers_3 | kmovers_5;
        }
        movers
    }

    fn get_white_jumpers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let mut jumpers = 0;
        let pieces = board.white_pawns | board.white_kings;
        let opponent = board.red_pawns | board.red_kings;

        let targets = (not_occupied << 4) & opponent;
        if targets != 0 {
            jumpers |= (((targets & MASK_3_UP) << 3) | ((targets & MASK_5_UP) << 5)) & pieces;
        }

        let targets = (((not_occupied & MASK_3_UP) << 3) | ((not_occupied & MASK_5_UP) << 5)) & opponent;
        if targets != 0 {
            jumpers |= (targets << 4) & pieces;
        }

        if board.white_kings != 0 {
            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 {
                jumpers |= (((targets & MASK_3_DOWN) >> 3) | ((targets & MASK_5_DOWN) >> 5)) & board.white_kings;
            }

            let targets = (((not_occupied & MASK_3_DOWN) >> 3) | ((not_occupied & MASK_5_DOWN) >> 5)) & opponent;
            if targets != 0 {
                jumpers |= (targets << 4) & board.white_kings;
            }
        }
        jumpers
    }

    fn get_red_jumpers(&self, board: &BitBoard, not_occupied: u32) -> u32 {
        let mut jumpers = 0;
        let pieces = board.red_pawns | board.red_kings;
        let opponent = board.white_pawns | board.white_kings;

        let targets = (not_occupied >> 4) & opponent;
        if targets != 0 {
            jumpers |= (((targets & MASK_3_DOWN) >> 3) | ((targets & MASK_5_DOWN) >> 5)) & pieces;
        }

        let targets = (((not_occupied & MASK_3_DOWN) >> 3) | ((not_occupied & MASK_5_DOWN) >> 5)) & opponent;
        if targets != 0 {
            jumpers |= (targets >> 4) & pieces;
        }

        if board.red_kings != 0 {
            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 {
                jumpers |= (((targets & MASK_3_UP) << 3) | ((targets & MASK_5_UP) << 5)) & board.red_kings;
            }

            let targets = (((not_occupied & MASK_3_UP) << 3) | ((not_occupied & MASK_5_UP) << 5)) & opponent;
            if targets != 0 {
                jumpers |= (targets >> 4) & board.red_kings;
            }
        }
        jumpers
    }

    fn get_white_moves(&self, board: &BitBoard, mover: u32) -> Vec<u32> {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
    
        let mut moves = Vec::new();
        if (not_occupied << 4) & mover != 0 {
            moves.push(mover | (mover >> 4));
        }
        if (not_occupied & MASK_3_UP) << 3 & mover != 0 {
            moves.push(mover | (mover >> 3));
        }
        if (not_occupied & MASK_5_UP) << 5 & mover != 0 {
            moves.push(mover | (mover >> 5));
        }

        if (board.white_kings & mover) != 0 {
            if (not_occupied >> 4) & mover != 0 {
                moves.push(mover | (mover << 4));
            }
            if (not_occupied & MASK_3_DOWN) >> 3 & mover != 0 {
                moves.push(mover | (mover << 3));
            }
            if (not_occupied & MASK_5_DOWN) >> 5 & mover != 0 {
                moves.push(mover | (mover << 5));
            }
        }

        return moves
    }

    fn get_red_moves(&self, board: &BitBoard, mover: u32) -> Vec<u32> {
        let not_occupied: u32 = !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
        let mut moves = Vec::new();
        if (not_occupied >> 4) & mover != 0 {
            moves.push(mover | (mover << 4));
        }
        if (not_occupied & MASK_3_DOWN) >> 3 & mover != 0 {
            moves.push(mover | (mover << 3));
        }
        if (not_occupied & MASK_5_DOWN) >> 5 & mover != 0 {
            moves.push(mover | (mover << 5));
        }

        if (board.red_kings & mover) != 0 {
            if (not_occupied << 4) & mover != 0 {
                moves.push(mover | (mover >> 4));
            }
            if (not_occupied & MASK_3_DOWN) << 3 & mover != 0 {
                moves.push(mover | (mover >> 3));
            }
            if (not_occupied & MASK_5_DOWN) << 5 & mover != 0 {
                moves.push(mover | (mover >> 5));
            }
        }

        return moves
    }

    fn get_white_jumps(&self, board: &BitBoard, start: u32) -> Vec<u32> {
        let mut result = Vec::new();
        let mut queue: VecDeque<Captures> = VecDeque::new();
        queue.push_back(Captures{mover: start, captures: 0});

        while !queue.is_empty() {
            let Some(curr) = queue.pop_front() else {
                break;
            };
            let captures = curr.captures;
            let mover = curr.mover;
            let opponent = (board.red_pawns | board.red_kings) ^ captures;
            let not_occupied: u32 = start | captures | !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
            let mut jump_found = false;

            let targets = (not_occupied << 4) & opponent;
            if targets != 0 && ((targets & MASK_3_UP) << 3) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover >> 3), 
                        mover: mover >> 7
                    });
            }
            if targets != 0 && ((targets & MASK_5_UP) << 5) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover >> 5), 
                        mover: mover >> 9
                    });
            }

            let targets = ((not_occupied & MASK_3_UP) << 3) & opponent;
            if targets != 0 && (targets << 4) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 7
                    });
            }

            let targets = ((not_occupied & MASK_5_UP) << 5) & opponent;
            if targets != 0 && (targets << 4) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 9
                    });
            }

            if (board.white_kings & start) != 0 {
                let targets = (not_occupied >> 4) & opponent;
                if targets != 0 && ((targets & MASK_3_DOWN) >> 3) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover << 3), 
                        mover: mover << 7
                    });
                }
                if targets != 0 && ((targets & MASK_5_DOWN) >> 5) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover << 5), 
                        mover: mover << 9
                    });
                }
                let targets = ((not_occupied & MASK_3_DOWN) >> 3) & opponent;
                if targets != 0 && (targets >> 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover << 4), 
                        mover: mover << 7
                    });
                }

                let targets = ((not_occupied & MASK_5_DOWN) >> 5) & opponent;
                if targets != 0 && (targets >> 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover << 4), 
                        mover: mover << 9
                    });
                }
            }

            if !jump_found && captures != 0 {
                result.push(captures | start | mover);
            }
        }

        result
    }

    fn get_red_jumps(&self, board: &BitBoard, start: u32) -> Vec<u32> {
        let mut result = Vec::new();
        let mut queue: VecDeque<Captures> = VecDeque::new();
        queue.push_back(Captures{mover: start, captures: 0});

        while !queue.is_empty() {
            let Some(curr) = queue.pop_front() else {
                break;
            };
            let captures = curr.captures;
            let mover = curr.mover;
            let opponent = (board.white_pawns | board.white_kings) ^ captures;
            let not_occupied: u32 = start | captures | !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
            let mut jump_found = false;

            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 && ((targets & MASK_3_DOWN) >> 3) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover << 3), 
                        mover: mover << 7
                    });
            }
            if targets != 0 && ((targets & MASK_5_DOWN) >> 5) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover << 5), 
                        mover: mover << 9
                    });
            }

            let targets = ((not_occupied & MASK_3_DOWN) >> 3) & opponent;
            if targets != 0 && (targets >> 4) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover << 4), 
                        mover: mover << 7
                    });
            }

            let targets = ((not_occupied & MASK_5_DOWN) >> 5) & opponent;
            if targets != 0 && (targets >> 4) & mover != 0 {
                jump_found = true;
                queue.push_back(Captures {
                        captures: captures | (mover << 4), 
                        mover: mover << 9
                    });
            }

            if (board.red_kings & start) != 0 {
                let targets = (not_occupied << 4) & opponent;
                if targets != 0 && ((targets & MASK_3_UP) << 3) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover >> 3), 
                        mover: mover >> 7
                    });
                }
                if targets != 0 && ((targets & MASK_5_UP) << 5) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover >> 5), 
                        mover: mover >> 9
                    });
                }
                let targets = ((not_occupied & MASK_3_UP) << 3) & opponent;
                if targets != 0 && (targets << 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 7
                    });
                }

                let targets = ((not_occupied & MASK_5_UP) << 5) & opponent;
                if targets != 0 && (targets << 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(Captures {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 9
                    });
                }
            }

            if !jump_found && captures != 0 {
                result.push(captures | start | mover);
            }
        }

        result
    }

    fn get_white_jumps_with_positions(&self, board: &BitBoard, start: u32) -> Vec<MoveCandidate> {
        let mut result = Vec::new();
        let mut queue: VecDeque<CapturesWithPositions> = VecDeque::new();
        queue.push_back(CapturesWithPositions{mover: start, captures: 0, positions: start});

        while !queue.is_empty() {
            let Some(curr) = queue.pop_front() else {
                break;
            };
            let captures = curr.captures;
            let positions = curr.positions;
            let mover = curr.mover;
            let opponent = (board.red_pawns | board.red_kings) ^ captures;
            let not_occupied: u32 = start | captures | !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
            let mut jump_found = false;

            let targets = (not_occupied << 4) & opponent;
            if targets != 0 && ((targets & MASK_3_UP) << 3) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 3), 
                        mover: mover >> 7,
                        positions: positions | mover,
                    });
            }
            if targets != 0 && ((targets & MASK_5_UP) << 5) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 5), 
                        mover: mover >> 9,
                        positions: positions | mover,
                    });
            }

            let targets = ((not_occupied & MASK_3_UP) << 3) & opponent;
            if targets != 0 && (targets << 4) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 7,
                        positions: positions | mover,
                    });
            }

            let targets = ((not_occupied & MASK_5_UP) << 5) & opponent;
            if targets != 0 && (targets << 4) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 9,
                        positions: positions | mover,
                    });
            }

            if (board.white_kings & start) != 0 {
                let targets = (not_occupied >> 4) & opponent;
                if targets != 0 && ((targets & MASK_3_DOWN) >> 3) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 3), 
                        mover: mover << 7,
                        positions: positions | mover,
                    });
                }
                if targets != 0 && ((targets & MASK_5_DOWN) >> 5) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 5), 
                        mover: mover << 9,
                        positions: positions | mover,
                    });
                }
                let targets = ((not_occupied & MASK_3_DOWN) >> 3) & opponent;
                if targets != 0 && (targets >> 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 4), 
                        mover: mover << 7,
                        positions: positions | mover,
                    });
                }

                let targets = ((not_occupied & MASK_5_DOWN) >> 5) & opponent;
                if targets != 0 && (targets >> 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 4), 
                        mover: mover << 9,
                        positions: positions | mover,
                    });
                }
            }

            if !jump_found && captures != 0 {
                result.push(MoveCandidate {
                    mov: captures | start | mover,
                    start_end: start | mover,
                    intermediate_positions: positions | mover,
                });
            }
        }

        result
    }

    fn get_red_jumps_with_positions(&self, board: &BitBoard, start: u32) -> Vec<MoveCandidate> {
        let mut result = Vec::new();
        let mut queue: VecDeque<CapturesWithPositions> = VecDeque::new();
        queue.push_back(CapturesWithPositions{mover: start, captures: 0, positions: start});

        while !queue.is_empty() {
            let Some(curr) = queue.pop_front() else {
                break;
            };
            let captures = curr.captures;
            let mover = curr.mover;
            let positions = curr.positions;
            let opponent = (board.white_pawns | board.white_kings) ^ captures;
            let not_occupied: u32 = start | captures | !(board.white_pawns | board.red_pawns | board.red_kings | board.white_kings);
            let mut jump_found = false;

            let targets = (not_occupied >> 4) & opponent;
            if targets != 0 && ((targets & MASK_3_DOWN) >> 3) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 3), 
                        mover: mover << 7,
                        positions: positions | mover,
                    });
            }
            if targets != 0 && ((targets & MASK_5_DOWN) >> 5) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 5), 
                        mover: mover << 9,
                        positions: positions | mover,
                    });
            }

            let targets = ((not_occupied & MASK_3_DOWN) >> 3) & opponent;
            if targets != 0 && (targets >> 4) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 4), 
                        mover: mover << 7,
                        positions: positions | mover,
                    });
            }

            let targets = ((not_occupied & MASK_5_DOWN) >> 5) & opponent;
            if targets != 0 && (targets >> 4) & mover != 0 {
                jump_found = true;
                queue.push_back(CapturesWithPositions {
                        captures: captures | (mover << 4), 
                        mover: mover << 9,
                        positions: positions | mover,
                    });
            }

            if (board.red_kings & start) != 0 {
                let targets = (not_occupied << 4) & opponent;
                if targets != 0 && ((targets & MASK_3_UP) << 3) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 3), 
                        mover: mover >> 7,
                        positions: positions | mover,
                    });
                }
                if targets != 0 && ((targets & MASK_5_UP) << 5) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 5), 
                        mover: mover >> 9,
                        positions: positions | mover,
                    });
                }
                let targets = ((not_occupied & MASK_3_UP) << 3) & opponent;
                if targets != 0 && (targets << 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 7,
                        positions: positions | mover,
                    });
                }

                let targets = ((not_occupied & MASK_5_UP) << 5) & opponent;
                if targets != 0 && (targets << 4) & mover != 0 {
                    jump_found = true;
                    queue.push_back(CapturesWithPositions {
                        captures: captures | (mover >> 4), 
                        mover: mover >> 9,
                        positions: positions | mover,
                    });
                }
            }

            if !jump_found && captures != 0 {
                result.push(MoveCandidate {
                    mov: captures | start | mover,
                    start_end: start | mover,
                    intermediate_positions: positions | mover,
                });
            }
        }

        result
    }

    // mover should have only one bit set
    fn get_jumps_with_positions(&self, board: &BitBoard, mover: u32, color: &Color) -> Vec<MoveCandidate> {
        match color {
            Color::White => self.get_white_jumps_with_positions(board, mover),
            Color::Red => self.get_red_jumps_with_positions(board, mover),
        }
    }

    fn get_start_end(&self, board: &BitBoard, target: &BitBoard, color: &Color) -> (u32, u32, Vec<MoveCandidate>) {
        let jumpers = self.get_possible_jumpers(board, color);

        for i in 1..=32 {
            let mover = jumpers & (BIT_MASK >> i-1);
            if mover > 0 {
                let jumps = self.get_jumps_with_positions(board, mover, color);
                let jumps: Vec<MoveCandidate> = jumps
                    .into_iter()
                    .filter(|a| {
                        let brd = board.apply_move(a.mov, color);
                        brd.white_pawns == target.white_pawns && brd.white_kings == target.white_kings && brd.red_pawns == target.red_pawns && brd.red_kings == target.red_kings 
                    })
                .collect();
                if jumps.len() > 0 {
                    let start = mover;
                    let end = match jumps[0].start_end.count_ones() {
                        1 => start,
                        _ => jumps[0].start_end ^ start,
                    };
                    return (start, end, jumps);
                }
            }
        }
        (0, 0, vec![])
    }

    #[allow(dead_code, unused)]
    fn move_to_string(&self, board: &BitBoard, target: &BitBoard, color: &Color) -> String {
        let my_pre_move = match color {
            Color::White => board.white_pawns | board.white_kings,
            Color::Red => board.red_pawns | board.red_kings,
        };
        let my_post_move = match color {
            Color::White => target.white_pawns | target.white_kings,
            Color::Red => target.red_pawns | target.red_kings,
        };
        let captures = match color {
            Color::White => (target.red_pawns | target.red_kings) != (board.red_pawns | board.red_kings),
            Color::Red => (target.white_pawns | target.white_kings) != (board.white_pawns | board.white_kings),
        };
        let start = my_pre_move & !my_post_move;
        let end = !my_pre_move & my_post_move;
        
        let start_num = 32-start.trailing_zeros()+1;
        let end_num = 32-end.trailing_zeros()+1;
        if !captures {
            return format!("{}-{}", start_num, end_num)
        };

        let (start, end, jumps) = match start {
            0 => self.get_start_end(board, target, color),
            _ => (start, end, self.get_jumps_with_positions(board, start, color)),
        };

        let boards: Vec<MoveCandidate> = jumps
            .into_iter()
            .filter(|a| a.start_end == start | end)
            .collect();

        if boards.len() == 1 {
            return format!("{}x{}", start, end)
        };

        // fast intermediate position
        let mut diff = 0;
        let mut m = 0;

        for i in 0..boards.len() {
            let brd = board.apply_move(boards[i].mov, color);
            if brd.white_pawns == target.white_pawns && brd.white_kings == target.white_kings && brd.red_pawns == target.red_pawns && brd.red_kings == target.red_kings {
                m = boards[i].intermediate_positions;
                continue;
            }
            let diff = diff | boards[i].intermediate_positions;
        }

        let diff = (diff ^ m) & m;
        if diff != 0 {
            let intermediate = 32-diff.trailing_zeros()+1;
            return format!("{}x{}x{}", start, intermediate, end)
        }
        // TODO there can still be ambigous candidates but these need info about order of moves
        "".into()
    }
}

struct Captures {
    captures: u32,
    mover: u32,
}

struct CapturesWithPositions {
    captures: u32,
    positions: u32,
    mover: u32,
}

pub struct MoveCandidate {
    start_end: u32,
    mov: u32,
    intermediate_positions: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn test_full_white_board() {
        let board = BitBoard {
            white_pawns: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b1111_1111_1111_1111_1111_1111_1111_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, &Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_full_red_board() {
        let board = BitBoard {
            white_pawns: 0b1111_1111_1111_1111_1111_1111_1111_1111,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, &Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_start_position_red() {
        let board = BitBoard {
            white_pawns: 0b1111_1111_1111_0000_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_1111_1111_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, &Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_1111_0000_0000);
    }

    #[test]
    fn test_start_position_white() {
        let board = BitBoard {
            white_pawns: 0b1111_1111_1111_0000_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_1111_1111_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, &Color::White);
        assert_eq!(movers, 0b0000_0000_1111_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_single_white() {
        let board = BitBoard {
            white_pawns: 0b0000_0000_0000_0010_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_1111,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_movers(&board, &Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0010_0000_0000_0000_0000);
    }

    #[test]
    fn test_white_capture_enforced() {
        let board = BitBoard {
            white_pawns: 0b0000_1111_0000_0010_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0100_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_jumpers(&board, &Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0010_0000_0000_0000_0000);
    }

    #[test]
    fn test_white_without_capture() {
        let board = BitBoard {
            white_pawns: 0b0000_1111_0000_0010_0000_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_1000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_jumpers(&board, &Color::White);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_red_capture_enforced() {
        let board = BitBoard {
            white_pawns:   0b0000_0000_0000_0010_0000_0000_0000_0000,
            white_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:     0b0000_0000_0000_0000_0100_0000_1111_0000,
            red_kings:     0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_jumpers(&board, &Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0100_0000_0000_0000);
    }

    #[test]
    fn test_red_without_capture() {
        let board = BitBoard {
            white_pawns:   0b0000_0000_0000_0001_0000_0000_0000_0000,
            white_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:     0b0000_0000_0000_0000_0100_0000_1111_0000,
            red_kings:     0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let movers = rules.get_possible_jumpers(&board, &Color::Red);
        assert_eq!(movers, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_get_white_moves_regular_pawn() {
        let board = BitBoard {
            white_pawns:   0b0000_0000_0000_0001_0000_0000_0000_0000,
            white_kings:   0,
            red_pawns:     0,
            red_kings:     0,
        };

        let expected_moves = vec![
                           0b0000_0000_0000_0001_0001_0000_0000_0000,
                           0b0000_0000_0000_0001_0010_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, &Color::White);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_get_white_moves_king() {
        let board = BitBoard {
            white_pawns:   0,
            white_kings:   0b0000_0000_0000_0001_0000_0000_0000_0000,
            red_pawns:     0,
            red_kings:     0,
        };

        let expected_moves = vec![
                           0b0000_0000_0000_0001_0001_0000_0000_0000,
                           0b0000_0000_0000_0001_0010_0000_0000_0000,
                           0b0000_0000_0001_0001_0000_0000_0000_0000,
                           0b0000_0000_0010_0001_0000_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, &Color::White);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_get_red_moves_regular_pawn() {
        let board = BitBoard {
            white_pawns:   0,
            white_kings:   0,
            red_pawns:     0b0000_0000_0000_0001_0000_0000_0000_0000,
            red_kings:     0,
        };

        let expected_moves = vec![
                           0b0000_0000_0001_0001_0000_0000_0000_0000,
                           0b0000_0000_0010_0001_0000_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, &Color::Red);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_get_red_moves_king() {
        let board = BitBoard {
            white_pawns:   0,
            white_kings:   0,
            red_pawns:     0,
            red_kings:     0b0000_0000_0000_0001_0000_0000_0000_0000,
        };

        let expected_moves = vec![
                           0b0000_0000_0000_0001_0001_0000_0000_0000,
                           0b0000_0000_0001_0001_0000_0000_0000_0000,
                           0b0000_0000_0010_0001_0000_0000_0000_0000,
        ];

        let mover =        0b0000_0000_0000_0001_0000_0000_0000_0000;
        let rules = BritishRules::new();
        let mut moves = rules.get_moves(&board, mover, &Color::Red);
        moves.sort();

        assert_eq!(moves, expected_moves);
    }

    #[test]
    fn test_white_jump_single_jump() {
        let board = BitBoard {
            white_pawns: 0b0000_0000_0000_0000_0100_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0010_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let jumps = rules.get_jumps(&board, board.white_pawns, &Color::White);
        assert_eq!(jumps.len(), 1);
        let expected_jumps = vec![
                        0b0000_0000_0000_0000_0100_0010_0010_0000,
        ];
        assert_eq!(jumps, expected_jumps);
    }

    #[test]
    fn test_white_jump_multi_jump() {
        let board = BitBoard {
            white_pawns: 0b0000_0000_0000_0000_0100_0000_0000_0000,
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0110_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,
        };

        let rules = BritishRules::new();
        let mut jumps = rules.get_jumps(&board, board.white_pawns, &Color::White);
        jumps.sort();
        assert_eq!(jumps.len(), 2);
        let expected_jumps = vec![
                        0b0000_0000_0000_0000_0100_0010_0010_0000,
                        0b0000_0000_0000_0000_0100_0100_1000_0000,
        ];
        assert_eq!(jumps, expected_jumps);
    }

    #[test]
    fn test_legal_move_no_ambiguity() {
        let board = BitBoard {
            white_pawns: 0b1000_0000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
        };

        let mov = MoveBit {
            start_end:   0b1000_1000_0000_0000_0000_0000_0000_0000,
            mov:         0b1000_1000_0000_0000_0000_0000_0000_0000,
        };
        
        let rules = BritishRules::new();
        let result = rules.verify_move(&board, mov, &Color::White);
        
        assert_eq!(result, MoveVerification::Ok(0b1000_1000_0000_0000_0000_0000_0000_0000));
    }

    #[test]
    fn test_illegal_backward_move() {
        let board = BitBoard {
            white_pawns: 0b0000_1000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
        };

        let mov = MoveBit {
            start_end:   0b1000_1000_0000_0000_0000_0000_0000_0000,
            mov:         0b1000_1000_0000_0000_0000_0000_0000_0000,
        };
        
        let rules = BritishRules::new();
        let result = rules.verify_move(&board, mov, &Color::White);
        
        assert_eq!(result, MoveVerification::Illegal);
    }

    #[test]
    fn test_illegal_far_move() {
        let board = BitBoard {
            white_pawns: 0b0000_1000_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
        };

        let mov = MoveBit {
            start_end:   0b0000_1000_0000_0000_1000_0000_0000_0000,
            mov:         0b0000_1000_0000_0000_1000_0000_0000_0000,
        };
        
        let rules = BritishRules::new();
        let result = rules.verify_move(&board, mov, &Color::White);
        
        assert_eq!(result, MoveVerification::Illegal);
    }

    #[test]
    fn test_ambigous_move() {
        let board = BitBoard {
            white_pawns: 0b0000_0100_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_1100_0000_1100_0000_0000_0000,  
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
        };

        let mov = MoveBit {
            start_end:   0b0000_0100_0000_0000_0000_0100_0000_0000,
            mov:         0b0000_0100_0000_0000_0000_0100_0000_0000,
        };
        
        let rules = BritishRules::new();
        let result = rules.verify_move(&board, mov, &Color::White);
        
        assert_eq!(result, MoveVerification::Ambiguous);
    }

    #[test]
    fn test_ambigous_move_precised() {
        let board = BitBoard {
            white_pawns: 0b0000_0100_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_1100_0000_1100_0000_0000_0000,  
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
        };

        let mov = MoveBit {
            start_end:   0b0000_0100_0000_0000_0000_0100_0000_0000,
            mov:         0b0000_0100_0000_1000_0000_0100_0000_0000,
        };
        
        let rules = BritishRules::new();
        let result = rules.verify_move(&board, mov, &Color::White);
        
        assert_eq!(result, MoveVerification::Ok(0b0000_0100_1000_0000_1000_0100_0000_0000));
    }

    #[test]
    fn test_shorthand_legal_move() {
        let board = BitBoard {
            white_pawns: 0b0000_0100_0000_0000_0000_0000_0000_0000,
            red_pawns:   0b0000_0000_1000_0000_1000_0000_0000_0000,  
            white_kings: 0b0000_0000_0000_0000_0000_0000_0000_0000,
            red_kings:   0b0000_0000_0000_0000_0000_0000_0000_0000,  
        };

        let mov = MoveBit {
            start_end:   0b0000_0100_0000_0000_0000_0100_0000_0000,
            mov:         0b0000_0100_0000_0000_0000_0100_0000_0000,
        };
        
        let rules = BritishRules::new();
        let result = rules.verify_move(&board, mov, &Color::White);
        
        assert_eq!(result, MoveVerification::Ok(0b0000_0100_1000_0000_1000_0100_0000_0000));
    }
}
