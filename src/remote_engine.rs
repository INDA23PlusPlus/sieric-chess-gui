use std::{collections::HashMap, net::TcpStream};
use serde::de::Deserialize;

use crate::chess_engine::*;
use chess_network_protocol::*;

fn parse_piece(piece: &Piece) -> (bool, String) {
    use Piece::*;

    return match piece {
        BlackPawn =>   (false, String::from("P")),
        BlackKnight => (false, String::from("N")),
        BlackBishop => (false, String::from("B")),
        BlackRook =>   (false, String::from("R")),
        BlackQueen =>  (false, String::from("Q")),
        BlackKing =>   (false, String::from("K")),
        WhitePawn =>   (true,  String::from("P")),
        WhiteKnight => (true,  String::from("N")),
        WhiteBishop => (true,  String::from("B")),
        WhiteRook =>   (true,  String::from("R")),
        WhiteQueen =>  (true,  String::from("Q")),
        WhiteKing =>   (true,  String::from("K")),
        None =>        (true,  String::from(" ")),
    }
}

pub struct RemoteGame {
    stream: TcpStream,
    moves: Vec<Move>,
    board: [[Piece; 8]; 8],
    joever: Joever,
    color: Color,
    waiting: bool,
}

impl RemoteGame {
    pub fn new() -> std::io::Result<Self> {
        let stream = TcpStream::connect("127.0.0.1:5000")?;
        let mut de = serde_json::Deserializer::from_reader(&stream);
        let server_color = Color::Black;

        let handshake = ClientToServerHandshake {
            server_color,
        };
        serde_json::to_writer(&stream, &handshake)?;

        let s2ch = ServerToClientHandshake::deserialize(&mut de)?;

        return Ok(RemoteGame {
            stream,
            moves: s2ch.moves,
            board: s2ch.board,
            joever: s2ch.joever,
            color: if server_color == Color::Black {
                Color::White
            } else {
                Color::Black
            },
            waiting: false,
        });
    }
}

impl ChessGame for RemoteGame {
    fn get_moves(&mut self, loc: &ChessLoc) -> HashMap<ChessLoc, ChessMove> {
        let mut map: HashMap<ChessLoc, ChessMove> = HashMap::new();
        for mv in self.moves.iter() {
            if *loc != (mv.start_x as i32, mv.start_y as i32) {
                continue;
            }

            let to = (mv.end_x as i32, mv.end_y as i32);
            let from = (mv.start_x as i32, mv.start_y as i32);
            map.insert(to, ChessMove {
                from, to,
                capture: self.board[mv.end_y][mv.end_x] != Piece::None,
                promotion: mv.promotion != Piece::None,
            });
        }

        return map;
    }

    fn apply_move(&mut self, mv: &ChessMove) -> bool {
        let promotion = if match self.board[mv.from.1 as usize][mv.from.0 as usize] {
            Piece::BlackPawn => true,
            Piece::WhitePawn => true,
            _ => false,
        } && (mv.to.1 == 7 || mv.to.1 == 0) {
            if self.color == Color::White {
                Piece::WhiteQueen
            } else {
                Piece::BlackQueen
            }
        } else {
            Piece::None
        };

        let mv2 = ClientToServer::Move(Move {
            start_x: mv.from.0 as usize,
            start_y: mv.from.1 as usize,
            end_x: mv.to.0 as usize,
            end_y: mv.to.1 as usize,
            promotion,
        });

        match serde_json::to_writer(&self.stream, &mv2) {
            Err(_) => return false,
            _ => (),
        };

        let mut de = serde_json::Deserializer::from_reader(&self.stream);

        let is_legal = match ServerToClient::deserialize(&mut de) {
            Ok(a) => a,
            _ => return false,
        };

        return match is_legal {
            ServerToClient::State { board, moves, joever, move_made: _ } => {
                self.waiting = true;
                self.board = board;
                self.moves = moves;
                self.joever = joever;
                true
            },
            ServerToClient::Error { board, moves, joever, message } => {
                self.board = board;
                self.moves = moves;
                self.joever = joever;
                println!("Illegal! {}", message);
                false
            },
            _ => false,
        };
    }

    fn wait_move(&mut self) -> bool {
        if !self.waiting {
            return false;
        }
        self.waiting = false;

        let mut de = serde_json::Deserializer::from_reader(&self.stream);

        let server_move = match ServerToClient::deserialize(&mut de) {
            Ok(a) => a,
            _ => return false,
        };

        return match server_move {
            ServerToClient::State { board, moves, joever, move_made: _ } => {
                self.board = board;
                self.moves = moves;
                self.joever = joever;
                true
            },
            _ => false,
        }
    }

    fn get_piece(&mut self, loc: &ChessLoc) -> (bool, String) {
        return parse_piece(&self.board[loc.1 as usize][loc.0 as usize]);
    }

    fn get_player(&mut self) -> bool {
        return self.color == Color::White;
    }

    fn get_state(&mut self) -> ChessState {
        use Joever::*;

        return match self.joever {
            White => ChessState::JoeverWhite,
            Black => ChessState::JoeverBlack,
            Draw => ChessState::JoeverDraw,
            Indeterminate => ChessState::JoeverIndeterminate,
            Ongoing => ChessState::Ongoing,
        };
    }
}
