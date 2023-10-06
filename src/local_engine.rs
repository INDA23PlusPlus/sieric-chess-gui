use std::collections::HashMap;

use chess::*;
use crate::chess_engine::*;

fn to_chess_move(mv: &Move) -> ChessMove {
    let from = (mv.from.x, mv.from.y);
    let to = (mv.to.x, mv.to.y);
    return ChessMove {
        from,
        to,
        capture: mv.is_capture(),
        promotion: mv.is_promotion().is_some(),
    };
}

pub struct LocalGame {
    game: Game,
}

impl LocalGame {
    pub fn new() -> Self {
        return LocalGame {
            game: Game::new(),
        };
    }

    pub fn get_all_moves(&self) -> Vec<ChessMove> {
        let moves = self.game.get_moves(None, None);
        return moves.iter().map(to_chess_move).collect();
    }

    pub fn get_board(&self) -> [Square; 8*8] {
        return self.game.board().squares;
    }
}

impl ChessGame for LocalGame {
    fn get_moves(&mut self, loc: &ChessLoc) -> HashMap<ChessLoc, ChessMove> {
        let loc2 = Loc { x: loc.0, y: loc.1 };
        let moves = self.game.get_moves(Some(loc2), None);

        let mut map = HashMap::new();
        for mv in moves {
            if let Some(kind) = mv.is_promotion() {
                if kind.name != "Q" {
                    continue;
                }
            }

            map.insert((mv.to.x, mv.to.y), to_chess_move(&mv));
        }

        return map;
    }

    fn apply_move(&mut self, mv2: &ChessMove) -> bool {
        let from = Loc { x: mv2.from.0, y: mv2.from.1 };
        let to = Loc { x: mv2.to.0, y: mv2.to.1 };

        /* why */
        let moves = self.game.get_moves(Some(from), Some(to));
        let mv = moves.iter().filter(|m| match m.is_promotion() {
            Some(kind) => kind.name == "Q",
            _ => true,
        }).next();

        if let Some(mv) = mv {
            self.game.play_move(&mv);
            return true;
        }
        return false;
    }

    fn get_piece(&mut self, loc: &ChessLoc) -> (bool, String) {
        return match self.game.board().at(Loc { x: loc.0, y: loc.1 }) {
            Square::Occupied(piece) => (
                piece.is_player(Player::White),
                String::from(piece.kind.name),
            ),
            _ => (true, String::from(" ")),
        };
    }

    fn get_player(&self) -> bool {
        return self.game.player() == Player::White;
    }

    fn get_state(&mut self) -> ChessState {
        return match self.game.state() {
            State::Playing => ChessState::Ongoing,
            State::Checkmate => if self.get_player() {
                ChessState::JoeverBlack
            } else {
                ChessState::JoeverWhite
            },
            State::Stalemate => ChessState::JoeverDraw,
        }
    }
}
