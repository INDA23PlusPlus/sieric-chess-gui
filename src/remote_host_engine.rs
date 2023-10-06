use std::{collections::HashMap, net::{TcpStream, TcpListener}};
use serde::de::Deserialize;

use crate::chess_engine::*;
use crate::local_engine::LocalGame;
use chess_network_protocol::*;
use chess::Square;

fn local_piece_to_proto(piece: chess::Piece) -> Piece {
    use chess::Player::*;

    return match (piece.player, piece.kind.name) {
        (White, "Q") => Piece::WhiteQueen,
        (White, "K") => Piece::WhiteKing,
        (White, "R") => Piece::WhiteRook,
        (White, "N") => Piece::WhiteKnight,
        (White, "B") => Piece::WhiteBishop,
        (White, "P") => Piece::WhitePawn,

        (Black, "Q") => Piece::BlackQueen,
        (Black, "K") => Piece::BlackKing,
        (Black, "R") => Piece::BlackRook,
        (Black, "N") => Piece::BlackKnight,
        (Black, "B") => Piece::BlackBishop,
        (Black, "P") => Piece::BlackPawn,
        _ => Piece::None,
    };
}

pub struct RemoteHostGame {
    stream: TcpStream,
    engine: LocalGame,
    server_color: Color,
    last_client_move: Move,
}

impl RemoteHostGame {
    pub fn new(port: &String) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;

        let (stream, addr) = listener.accept()?;
        println!("Connected to {}", addr);

        let mut de = serde_json::Deserializer::from_reader(&stream);
        let engine = LocalGame::new();

        let c2sh = ClientToServerHandshake::deserialize(&mut de)?;
        println!("Received: {:?}", c2sh);

        let mut game = RemoteHostGame {
            engine, stream,
            server_color: c2sh.server_color,
            last_client_move: Move {
                start_x: 0,
                start_y: 0,
                end_x: 0,
                end_y: 0,
                promotion: Piece::None,
            },
        };

        let s2ch = ServerToClientHandshake {
            features: vec![
                Features::EnPassant,
                Features::Castling,
                Features::Promotion,
            ],
            board: game.get_proto_board(),
            moves: game.get_proto_moves(),
            joever: Joever::Ongoing,
        };
        println!("Send S2CH");
        serde_json::to_writer(&game.stream, &s2ch)?;

        /* the client is white and makes a move */
        if game.server_color == Color::Black {
            let _ = game.handle_client_move();
        }

        return Ok(game);
    }

    fn handle_client_move(&mut self) -> std::io::Result<()> {
        let mut de = serde_json::Deserializer::from_reader(&self.stream);

        let mut done = false;
        while !done {
            let c2s = ClientToServer::deserialize(&mut de)?;
            println!("Received: {:?}", c2s);

            match c2s {
                ClientToServer::Move(mv) => {
                    let moves = self.get_proto_moves();
                    if moves.contains(&mv) {
                        /* move got accepted */
                        self.last_client_move = mv;
                        done = true;
                    } else {
                        let s2c = ServerToClient::Error {
                            board: self.get_proto_board(),
                            moves,
                            joever: Joever::Ongoing,
                            message: String::from("tu madre"),
                        };
                        println!("Send illegal move");
                        serde_json::to_writer(&self.stream, &s2c)?;
                    }
                },
                ClientToServer::Resign => todo!(),
                ClientToServer::Draw => todo!(),
            }
        }

        let mv = self.last_client_move;
        self.engine.apply_move(&self.move_to_chess_move(&mv));
        println!("Send legal move");
        self.update_client(&mv)?;

        return Ok(());
    }

    fn update_client(&mut self, mv: &Move) -> std::io::Result<()> {
        use ChessState::*;

        let s2c = ServerToClient::State {
            board: self.get_proto_board(),
            moves: self.get_proto_moves(),
            joever: match self.get_state() {
                Ongoing => Joever::Ongoing,
                JoeverIndeterminate => Joever::Indeterminate,
                JoeverDraw => Joever::Draw,
                JoeverWhite => Joever::White,
                JoeverBlack => Joever::Black,
            },
            move_made: *mv,
        };
        serde_json::to_writer(&self.stream, &s2c)?;

        return Ok(());
    }

    fn chess_move_to_move(&self, mv: &ChessMove) -> Move {
        return Move {
            start_x: mv.from.0 as usize,
            start_y: mv.from.1 as usize,
            end_x: mv.to.0 as usize,
            end_y: mv.to.1 as usize,
            promotion: if mv.promotion {
                if self.get_player() {
                    Piece::WhiteQueen
                } else {
                    Piece::BlackQueen
                }
            } else {
                Piece::None
            },
        };
    }

    fn move_to_chess_move(&self, mv: &Move) -> ChessMove {
        let from = (mv.start_x as i32, mv.start_y as i32);
        let to = (mv.end_x as i32, mv.end_y as i32);
        return ChessMove {
            from,
            to,
            /* NOTE: this is only used for client visualization, where this
             * function will never be called */
            capture: false,
            promotion: mv.promotion != Piece::None,
        };
    }

    fn get_proto_board(&self) -> [[chess_network_protocol::Piece; 8]; 8] {
        let board = self.engine.get_board();
        let mut out = [[Piece::None; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                if let Square::Occupied(piece) = board[8*y + x] {
                    out[y][x] = local_piece_to_proto(piece);
                }
            }
        }

        return out;
    }

    fn get_proto_moves(&self) -> Vec<Move> {
        let moves = self.engine.get_all_moves();
        let mut out: Vec<Move> = Vec::new();
        for mv in moves.iter() {
            out.push(self.chess_move_to_move(&mv));
        }
        return out;
    }
}

impl ChessGame for RemoteHostGame {
    fn get_moves(&mut self, loc: &ChessLoc) -> HashMap<ChessLoc, ChessMove> {
        return self.engine.get_moves(loc);
    }

    fn apply_move(&mut self, mv: &ChessMove) -> bool {
        let ret = self.engine.apply_move(mv);

        println!("Send server move");
        let _ = self.update_client(&self.chess_move_to_move(mv));

        return ret;
    }

    fn wait_move(&mut self) -> bool {
        let _ = self.handle_client_move();
        return true;
    }

    fn get_piece(&mut self, loc: &ChessLoc) -> (bool, String) {
        return self.engine.get_piece(loc);
    }

    fn get_player(&self) -> bool {
        return self.engine.get_player();
    }

    fn get_state(&mut self) -> ChessState {
        return self.engine.get_state();
    }
}
