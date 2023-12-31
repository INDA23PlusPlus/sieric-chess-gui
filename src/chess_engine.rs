use std::collections::HashMap;

/**
 * Location on the chess board. The first value is the file (`0` is `a` and `7`
 * is `h`). The second value is the rank, but zero-indexed.
 */
pub type ChessLoc = (i32, i32);

/**
 * Representation of a state in the game.
 */
pub enum ChessState {
    /// The game is still ongoing
    Ongoing,
    /// The game is over with an undetermined winner
    JoeverIndeterminate,
    /// The game is over in a draw (all types of draws)
    JoeverDraw,
    /// The game is over and white won
    JoeverWhite,
    /// The game is over and black won
    JoeverBlack,
}

pub struct ChessMove {
    pub from: ChessLoc,
    pub to: ChessLoc,

    pub capture: bool,
    pub promotion: bool,
}

pub trait ChessGame {
    /**
     * Return all possible moves from the given starting location. The key in
     * the returned [HashMap] is the target location of the move.
     */
    fn get_moves(&mut self, loc: &ChessLoc) -> HashMap<ChessLoc, ChessMove>;

    /**
     * Applies the move type to the current game. Should return [true] if the
     * move succeeds.
     */
    fn apply_move(&mut self, mv: &ChessMove) -> bool;

    /**
     * Wait for opponents move. Only relevant in online games, does not need to
     * be implemented in local games. Should return [true] if the "opponent"
     * successfully makes a move.
     */
    fn wait_move(&mut self) -> bool {
        return true;
    }

    /**
     * Return the piece at `loc`'s color ([true] for white, [false] for black)
     * and the piece's name in chess notation.
     */
    fn get_piece(&mut self, loc: &ChessLoc) -> (bool, String);

    /**
     * Return the current player ([true] for white, [false] for black).
     */
    fn get_player(&self) -> bool;

    /**
     * Return the current state of the game.
     */
    fn get_state(&mut self) -> ChessState;
}
