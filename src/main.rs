use std::collections::HashMap;

use ggez::{self, event, conf::{WindowMode, WindowSetup}, GameResult, GameError, Context, graphics::{self, Rect, DrawParam}, glam::{Vec2, IVec2}, winit::event::VirtualKeyCode};

fn piece_to_string(piece: &chess::Piece) -> String {
    return String::from(piece.kind.name);
}

struct MainState {
    game: chess::Game,
    selected: Option<IVec2>,
    moves: HashMap<(i32, i32), chess::Move>,
    flip_mode: bool,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let game = chess::Game::new();

        return Ok(MainState {
            game,
            selected: None,
            moves: HashMap::new(),
            flip_mode: false,
        });
    }
}

impl event::EventHandler<GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        return Ok(());
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::Color::from([0.1, 0.2, 0.3, 1.0])
        );

        let (win_w, win_h) = ctx.gfx.drawable_size();

        for fake_i in 0..8 {
            for j in 0..8 {
                let pos = Vec2::new(
                    (win_w/8.) * j as f32,
                    (win_h/8.) * fake_i as f32,
                );
                let i = if self.game.player() == chess::Player::Black
                    && self.flip_mode {
                    fake_i
                } else {
                    7 - fake_i
                };

                let board = self.game.board();
                let square = board.at(chess::Loc {x: j, y: i });
                let (piece_white, piece_text) = match square {
                    chess::Square::Occupied(piece) => (
                        piece.is_player(chess::Player::White),
                        piece_to_string(piece),
                    ),
                    _ => (true, String::from(" ")),
                };

                canvas.draw(
                    &graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        Rect::new(0., 0., win_w/8., win_h/8.),
                        if (i + j) % 2 == 0 {
                            graphics::Color::from([0.5, 0.5, 0.5, 1.])
                        } else {
                            graphics::Color::from([0.12, 0.4, 0., 1.])
                        }
                    )?,
                    pos,
                );

                match self.moves.get(&(j, i)) {
                    Some(mv) => {
                        canvas.draw(
                            &graphics::Mesh::new_rectangle(
                                ctx,
                                graphics::DrawMode::fill(),
                                Rect::new(0., 0., win_w/8., win_h/8.),
                                if mv.is_capture() {
                                    graphics::Color::from([1., 0., 0., 0.5])
                                } else {
                                    graphics::Color::from([0., 0., 1., 0.5])
                                },
                            )?,
                            pos,
                        );
                    },
                    _ => (),
                }

                canvas.draw(
                    graphics::Text::new(piece_text)
                        .set_scale(win_w/8.),
                    DrawParam::default()
                        .dest(pos)
                        .color(if piece_white {
                            graphics::Color::from([1., 1., 1., 1.])
                        } else {
                            graphics::Color::from([0., 0., 0., 1.])
                        }),
                );
            }
        }

        canvas.finish(ctx)?;
        return Ok(());
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let (win_w, win_h) = ctx.gfx.drawable_size();

        let pos = IVec2::new(
            (x*8. / win_w).floor() as i32,
            if self.game.player() == chess::Player::Black && self.flip_mode {
                (y*8. / win_h).floor() as i32
            } else {
                7 - (y*8. / win_h).floor() as i32
            },
        );
        match self.moves.get(&(pos.x, pos.y)) {
            Some(mv) => {
                self.game.play_move(&mv);
                self.selected = None;
                self.moves = HashMap::new();
                return Ok(());
            },
            _ => (),
        }

        if self.selected == Some(pos) {
            self.selected = None;
            self.moves = HashMap::new();
            return Ok(());
        }
        self.selected = Some(pos);

        let board = self.game.board();
        let loc = chess::Loc { x: pos.x, y: pos.y };
        let square = board.at(loc);

        let moves = match square {
            chess::Square::Occupied(_) => self.game.get_moves(Some(loc), None),
            _ => Vec::new(),
        };
        self.moves = HashMap::new();
        for mv in moves {
            if let Some(kind) = mv.is_promotion() {
                if kind.name != "Q" {
                    continue;
                }
            }
            self.moves.insert((mv.to.x, mv.to.y), mv);
        }

        return Ok(());
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let Some(key) = input.keycode {
            match key {
                VirtualKeyCode::F => self.flip_mode = !self.flip_mode,
                _ => (),
            }
        }

        return Ok(());
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("Chess", "EmmaEricsson")
        .window_mode(WindowMode::default().dimensions(800., 800.))
        .window_setup(WindowSetup::default().title("Chessss"));
    let (ctx, event_loop) = cb.build()?;
    event::run(ctx, event_loop, MainState::new()?);
}
