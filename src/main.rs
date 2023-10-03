use ggez::{self, event, conf::{WindowMode, WindowSetup}, GameResult, GameError, Context, graphics::{self, Rect, DrawParam}, glam::Vec2};

fn piece_to_string(piece: &chess::Piece) -> String {
    return String::from(piece.kind.name);
}

struct MainState {
    game: chess::Game,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let game = chess::Game::new();

        return Ok(MainState { game });
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

        for i in 0..8 {
            for j in 0..8 {
                let pos = Vec2::new(
                    (win_w/8.) * j as f32,
                    (win_h/8.) * i as f32,
                );

                let board = self.game.board();
                let square = board.at(chess::Loc {x: j, y: 7-i });
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
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("Chess", "EmmaEricsson")
        .window_mode(WindowMode::default().dimensions(800., 800.))
        .window_setup(WindowSetup::default().title("Chessss"));
    let (ctx, event_loop) = cb.build()?;
    event::run(ctx, event_loop, MainState::new()?);
}
