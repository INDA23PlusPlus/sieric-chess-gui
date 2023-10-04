mod chess_engine;
mod local_engine;

use chess_engine::*;
use local_engine::LocalGame;

use std::{collections::HashMap, env, path};

use ggez::{self, event, GameResult, GameError, Context};
use ggez::winit::event::VirtualKeyCode;
use ggez::glam::*;
use ggez::audio::{self, SoundSource};
use ggez::graphics::{self, Rect, DrawParam};
use ggez::conf::{WindowMode, WindowSetup};

#[allow(dead_code)]
enum GameState {
    Init,
    Hosting,
    Joining,
    InGame,
}

struct MainState<'a> {
    state: GameState,
    game: Box<dyn ChessGame + 'a>,
    music: audio::Source,
    selected: Option<IVec2>,
    moves: HashMap<ChessLoc, ChessMove>,
    flip_mode: bool,
}

impl<'a> MainState<'a> {
    fn new(ctx: &mut Context) -> GameResult<MainState<'a>> {
        return Ok(MainState {
            state: GameState::InGame,
            game: Box::new(LocalGame::new()),
            music: audio::Source::new(ctx, "/copyright_infringement.flac")?,
            selected: None,
            moves: HashMap::new(),
            flip_mode: false,
        });
    }

    fn ingame_draw(&mut self, ctx: &mut Context) -> GameResult {
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
                let i = if !self.game.get_player()
                    && self.flip_mode {
                    fake_i
                } else {
                    7 - fake_i
                };

                let (piece_white, piece_text) = self.game.get_piece(&(j, i));

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
                                if mv.capture {
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

        let joever_text: Option<String> = match self.game.get_state() {
            ChessState::Ongoing => None,
            ChessState::JoeverBlack => Some(String::from("Black Checkmate")),
            ChessState::JoeverWhite => Some(String::from("White Checkmate")),
            ChessState::JoeverDraw => Some(String::from("Stalemate")),
            ChessState::JoeverIndeterminate => Some(String::from("tu madre")),
        };
        if let Some(text) = joever_text {
            canvas.draw(
                &graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Rect::new(0., 0., win_w, win_h),
                    graphics::Color::from([0., 0., 0., 0.5]),
                )?,
                Vec2::new(0., 0.),
            );

            canvas.draw(
                graphics::Text::new(text)
                    .set_scale(100.),
                DrawParam::default()
                    .dest(Vec2::new(0., 0.))
                    .color(graphics::Color::from([1., 1., 1., 1.])),
            );
        }

        canvas.finish(ctx)?;
        return Ok(());
    }

    fn ingame_mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let (win_w, win_h) = ctx.gfx.drawable_size();

        let pos = IVec2::new(
            (x*8. / win_w).floor() as i32,
            if !self.game.get_player() && self.flip_mode {
                (y*8. / win_h).floor() as i32
            } else {
                7 - (y*8. / win_h).floor() as i32
            },
        );
        match self.moves.get(&(pos.x, pos.y)) {
            Some(mv) => {
                self.game.apply_move(&mv);
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

        self.moves = self.game.get_moves(&(pos.x, pos.y));

        return Ok(());
    }

    fn ingame_key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> GameResult {
        if let Some(key) = input.keycode {
            match key {
                VirtualKeyCode::F => self.flip_mode = !self.flip_mode,
                VirtualKeyCode::Q => ctx.request_quit(),
                VirtualKeyCode::M => if self.music.paused() {
                    self.music.resume();
                } else {
                    self.music.pause();
                },
                _ => (),
            }
        }

        return Ok(());
    }
}

impl event::EventHandler<GameError> for MainState<'_> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if !self.music.playing() {
            let _ = self.music.play_later();
            self.music.pause();
        }

        return Ok(());
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        use GameState::*;

        return match self.state {
            InGame => self.ingame_draw(ctx),
            _ => Ok(()),
        };
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        use GameState::*;

        return match self.state {
            InGame => self.ingame_mouse_button_down_event(ctx, button, x, y),
            _ => Ok(()),
        };
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool,
    ) -> GameResult {
        use GameState::*;

        return match self.state {
            InGame => self.ingame_key_down_event(ctx, input, repeated),
            _ => Ok(()),
        };
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("Chess", "EmmaEricsson")
        .add_resource_path(resource_dir)
        .window_mode(WindowMode::default().dimensions(800., 800.))
        .window_setup(WindowSetup::default().title("Chessss"));
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state);
}
