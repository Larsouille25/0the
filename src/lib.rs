use std::{
    borrow::Cow,
    cell::RefCell,
    error::Error,
    fmt::{self, Display},
    io::{self, Write},
    ops::Not,
    str::FromStr,
};

use player::Player;
use termcolor::{StandardStream, WriteColor};

pub mod player;
pub mod style;

pub const VERSION_AND_GIT_HASH: &str = env!("VERSION_AND_GIT_HASH");
pub const LICENSE: &str = include_str!("../LICENSE");
/// The [official rules][or] of the Othello game of the
/// [World Othello Federation][wof].
///
/// [or]: https://www.worldothello.org/about/about-othello/othello-rules/official-rules/english
/// [wof]: https://www.worldothello.org
pub const OTHELLO_RULES: &str = include_str!("../OTHELLO_RULES");

pub(crate) type Result<T, E = OthebotError> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum OthebotError {
    InvalidAlgebric(String),
    IllegalMove { row: u8, col: u8 },
    LegalMovesNotComputed,
    IoError(io::Error),
    InvalidLenghtOfNotation,
    InvalidCharInNotation { ch: char },
}

impl Error for OthebotError {}

impl Display for OthebotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OthebotError::InvalidAlgebric(notation) => write!(f, "invalid algebric notation {notation:?}, valid e.g: `a5`"),
            OthebotError::IllegalMove{ row, col} => write!(f, "illegal move (row: {row}, col: {col}), you can't put your disc here"),
            OthebotError::LegalMovesNotComputed => write!(f, "INTERNAL ERROR: legal moves were not computed before calling a function that depends on legal moves."),
            OthebotError::IoError(e) => write!(f, "IO ERROR: {e}"),
            OthebotError::InvalidLenghtOfNotation => write!(f, "the Othello Notation must be 64 characters long"),
            OthebotError::InvalidCharInNotation { ch } => write!(f, "invalid character {ch:?} in Othello Notation"),
        }
    }
}

impl From<io::Error> for OthebotError {
    fn from(value: io::Error) -> Self {
        OthebotError::IoError(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disc {
    White,
    Black,
    Empty,
}

impl Not for Disc {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Disc::White => Disc::Black,
            Disc::Black => Disc::White,
            // it shouldn't be called if `Disc` is `Empty` but if it did, don't
            // change because there is no opposite of `Empty`
            Disc::Empty => Disc::Empty,
        }
    }
}

impl Display for Disc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Disc::White => write!(f, "White"),
            Disc::Black => write!(f, "Black"),
            Disc::Empty => write!(f, "Empty"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    squares: [Disc; 64],
}

impl Board {
    /// Create a new board with the starting layout
    pub const fn new() -> Board {
        use Disc::Black as B;
        use Disc::Empty as E;
        use Disc::White as W;
        Board {
            squares: [
                E, E, E, E, E, E, E, E, // This
                E, E, E, E, E, E, E, E, // is
                E, E, E, E, E, E, E, E, // to
                E, E, E, W, B, E, E, E, // trick
                E, E, E, B, W, E, E, E, // the
                E, E, E, E, E, E, E, E, // rust
                E, E, E, E, E, E, E, E, // formater
                E, E, E, E, E, E, E, E, // ;)
            ],
        }
    }

    /// Get the disc located at those X and Y coordinates, check if coordinates
    /// are in bounds
    #[inline]
    #[must_use]
    pub fn get_disc(&self, (col, row): (u8, u8)) -> Disc {
        assert!(col < 8);
        assert!(row < 8);
        // UNSAFE: we checked that they are in bounds
        unsafe { self.get_disc_unchecked(col, row) }
    }

    /// Get the disc at those X and Y coordiantes, don't check if they are in
    /// bounds are not.
    ///
    /// # Safety
    ///
    /// If either `x` or `y` are greater that 8, it will get the wrong disc, or
    /// panic. It is the responsability of the caller to check the coordinates
    /// are valid.
    #[inline]
    #[must_use]
    pub unsafe fn get_disc_unchecked(&self, col: u8, row: u8) -> Disc {
        self.squares[(row * 8 + col) as usize]
    }

    /// Change the disc at those coordinates, don't check if this move is legal.
    #[track_caller]
    fn change_disc(&mut self, Move { col, row }: Move, disc: Disc) {
        assert!(col < 8);
        assert!(row < 8);
        // UNSAFE: we checked that they are in bounds
        let idx = (row * 8 + col) as usize;
        *self.squares.get_mut(idx).unwrap() = disc;
    }

    /// Returns the scores of the current board, in the tuple, white's score is
    /// first, and black's score is second, and empty squares third
    pub fn scores(&self) -> (u8, u8, u8) {
        let mut white = 0;
        let mut black = 0;
        let mut empty = 0;
        for disc in self.squares {
            match disc {
                Disc::White => white += 1,
                Disc::Black => black += 1,
                Disc::Empty => empty += 1,
            }
        }
        (white, black, empty)
    }

    /// Return the current legal moves for the `player` into a bitfield format.
    ///
    /// The first bit of the bitfield is the first disc at index 0 and the last
    /// bit is index 63.
    #[must_use]
    #[track_caller]
    pub fn legal_moves(&self, player: Disc) -> u64 {
        let mut bitfield = 0;

        if player == Disc::Empty {
            panic!("The player should not be an empty disc.")
        }

        let directions: [(i32, i32); 8] = [
            (-1, -1), // RIGHT UP
            (0, -1),  // UP
            (1, -1),  // LEFT-UP
            (-1, 0),  // RIGHT
            (1, 0),   // LEFT
            (-1, 1),  // LEFT-DOWN
            (0, 1),   // DOWN
            (1, 1),   // RIGHT-DOWN
        ];

        for y in 0..8 {
            for x in 0..8 {
                let idx = y * 8 + x;

                // The disc is already filed
                if self.squares[idx] != Disc::Empty {
                    continue;
                }

                for (dx, dy) in directions {
                    // coordinates of next disc in direction
                    let mut nx = x as i32 + dx;
                    let mut ny = y as i32 + dy;

                    // whetever a disc of the other color was present in the
                    // line of the direction
                    let mut captured = false;

                    while (0..8).contains(&nx) && (0..8).contains(&ny) {
                        let n_idx = (ny * 8 + nx) as usize;

                        if self.squares[n_idx] == Disc::Empty {
                            break;
                        }

                        if self.squares[n_idx] == player {
                            if captured {
                                // we already encountered an opposite disc, we
                                // know it is a good move
                                bitfield |= 1 << idx;
                            }
                            break;
                        }
                        // we encountered an opposite disc, so if later we
                        // encounter in the same direction a disc of player's
                        // color, it's a valid move
                        captured = true;
                        // update the coordinates to continue in this direction
                        nx += dx;
                        ny += dy;
                    }
                }
            }
        }

        bitfield
    }

    /// Compute the discs that will be outflanked from a move.
    ///
    /// # Note
    ///
    /// If you try to know which move is legal, you should use the
    /// [`legal_moves`] method.
    ///
    /// [`legal_moves`]: Board::legal_moves
    pub fn move_outflanks(&self, player: Disc, Move { col: x, row: y }: Move) -> u64 {
        let mut bitfield = 0;

        if player == Disc::Empty {
            panic!("The player should not be an empty disc.")
        }

        // TODO: make this kinda global because in `legal_moves` we have the
        // same slice.
        let directions: [(i32, i32); 8] = [
            (-1, -1), // RIGHT UP
            (0, -1),  // UP
            (1, -1),  // LEFT-UP
            (-1, 0),  // RIGHT
            (1, 0),   // LEFT
            (-1, 1),  // LEFT-DOWN
            (0, 1),   // DOWN
            (1, 1),   // RIGHT-DOWN
        ];

        for (dx, dy) in directions {
            let mut nx = x as i32 + dx;
            let mut ny = y as i32 + dy;
            // this is a bitfield that contains opponent's discs that could be
            // outflanked if it is correctly sandwiched
            let mut may_outflank = 0;

            while (0..8).contains(&nx) && (0..8).contains(&ny) {
                let n_idx = (ny * 8 + nx) as usize;

                if self.squares[n_idx] == Disc::Empty {
                    // Not a correct sandwich of opponent's disc, because there
                    // is a gap
                    break;
                }

                if self.squares[n_idx] == player && may_outflank != 0 {
                    // We are able to outflank at least one opponent's disc
                    bitfield |= may_outflank;
                    break;
                }
                may_outflank |= 1 << n_idx;
                nx += dx;
                ny += dy;
            }
        }

        bitfield
    }

    /// Put the discs (`player` arg) according to the provided bitfield.
    ///
    /// The first bit of the bitfield is the first disc at index 0 and the last
    /// bit is index 63. (just like legal_moves)
    pub fn put_discs(&mut self, bitfield: u64, player: Disc) {
        for i in 0..self.squares.len() {
            if (1_u64 << i & bitfield) != 0 {
                self.squares[i] = player;
            }
        }
    }
}

impl Default for Board {
    #[inline]
    fn default() -> Self {
        Board::new()
    }
}

impl FromStr for Board {
    type Err = OthebotError;

    /// The board can be constructed from a string, [the format is common to
    /// othello programs.][this-articles]
    ///
    /// `XO---XXX-OOO-OOO-OOOOOO---OOXO---OOXOOO-OOXOOOOOXXXXX---XXXXXX--`
    /// In this notation, dashes (`-`) represent empty squares, `X` represent
    /// black discs and `O` represent White's discs. The string is 64 character
    /// long.
    ///
    /// [this-article]: https://mirror.math.princeton.edu/pub/CTAN/macros/latex/contrib/othelloboard/othelloboard.pdf#page=8
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: write tests for this function
        if s.len() != 64 {
            return Err(OthebotError::InvalidLenghtOfNotation);
        }
        let mut board = [Disc::Empty; 64];
        for (i, c) in s.char_indices() {
            match c {
                '-' =>
                    /* we do nothing because it is already an empty square*/
                    {}
                'O' => board[i] = Disc::White,
                'X' => board[i] = Disc::Black,
                ch => {
                    return Err(OthebotError::InvalidCharInNotation { ch });
                }
            }
        }
        Ok(Board { squares: board })
    }
}

/// A position on the Othello Board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub col: u8,
    pub row: u8,
}

impl Move {
    pub fn from_algebric(pos: &str) -> Result<Move> {
        let (col, row) = algebric2xy(pos)?;
        Ok(Move { col, row })
    }

    pub fn into_idx(self) -> usize {
        self.row as usize * 8 + self.col as usize
    }

    pub fn from_idx(idx: u8) -> Move {
        Move {
            row: idx / 8,
            col: idx % 8,
        }
    }
}

/// Converts an algebric notation like `a1`, `g8`, `b7` etc to `(0, 0)`,
/// `(6, 7)`, `(1, 6)`.
fn algebric2xy(pos: &str) -> Result<(u8, u8)> {
    if pos.len() != 2 {
        return Err(OthebotError::InvalidAlgebric(pos.to_string()));
    }

    let mut it = pos.chars();
    let col = it.next().unwrap() as u8;
    let row = it.next().unwrap() as u8;

    if !(b'a'..=b'h').contains(&col) || !(b'1'..=b'8').contains(&row) {
        return Err(OthebotError::InvalidAlgebric(pos.to_string()));
    }

    Ok((col - b'a', row - b'1'))
}

pub fn bitfield_to_indexes(bitfield: u64) -> Vec<u8> {
    let mut positions = Vec::new();
    for i in 0..64 {
        if (bitfield & (1 << i)) != 0 {
            positions.push(i);
        }
    }
    positions
}

#[derive(Debug, Clone)]
pub enum State {
    /// The game is currently being played.
    Playing,
    /// One of the player winned the game.
    Winned {
        /// Who won the game?
        winner_color: Disc,
        /// Championship style score, the winner's score include empty squares
        winner_score: u8,
        /// Championship style score, the winner's score include empty squares
        loser_score: u8,
    },
    /// The game ended in an equality of scores.
    Draw,
    /// The current player cannot play, his turn is forfeited (Rule no. 2)
    TurnForfeited,
}

#[derive(Debug)]
pub struct Game {
    board: Board,
    /// Black player
    black_player: Box<dyn Player>,
    /// White player
    white_player: Box<dyn Player>,
    /// Who's next turn?
    ///
    /// Note:
    ///
    /// `turn` cannot be `Disc::Empty`.
    turn: Disc,
    /// The legal moves of the current player (`turn` field).
    current_legal_moves: Option<u64>,
    /// The stream, usualy stdout where we render the game.
    stream: RefCell<StandardStream>,
    /// The state of the game
    state: State,
}

impl Game {
    pub fn new(
        white_player: Box<dyn Player>,
        black_player: Box<dyn Player>,
        stream: StandardStream,
    ) -> Game {
        Game::with_board(Board::new(), white_player, black_player, stream)
    }

    pub fn with_board(
        board: Board,
        mut white_player: Box<dyn Player>,
        mut black_player: Box<dyn Player>,
        stream: StandardStream,
    ) -> Game {
        white_player.init_color(Disc::White);
        black_player.init_color(Disc::Black);

        Game {
            board,
            white_player,
            black_player,
            turn: Disc::Black,
            current_legal_moves: None,
            stream: RefCell::new(stream),
            state: State::Playing,
        }
    }

    pub fn turn(&self) -> Disc {
        debug_assert_ne!(self.turn, Disc::Empty);
        self.turn
    }

    fn is_legal(bitfield: u64, index: usize) -> bool {
        (bitfield & (1 << index)) != 0
    }

    pub fn is_legal_move(&self, index: usize) -> Result<bool> {
        let Some(moves) = self.current_legal_moves else {
            return Err(OthebotError::LegalMovesNotComputed);
        };
        Ok(Self::is_legal(moves, index))
    }

    pub fn make_turn(&mut self, mov @ Move { col, row }: Move) -> Result<()> {
        // ensure the move is inside the legal moves.
        let idx = (row * 8 + col) as u64;
        if !self.is_legal_move(idx as usize)? {
            return Err(OthebotError::IllegalMove { row, col });
        }
        self.board.change_disc(mov, self.turn);
        let outflanks = self.board.move_outflanks(self.turn, mov);
        self.board.put_discs(outflanks, self.turn);

        self.next_turn();

        Ok(())
    }

    fn next_turn(&mut self) {
        // Change the turn to the opponent
        self.turn = !self.turn;
        // Reset the current legal moves to `None`, just a simple safety used
        // not to confuse between Black's and White's legal moves
        self.current_legal_moves = None;
        // Set the state to playing, the player before could of had a State of
        // `TurnForfeited` and if the other player can play it must not forfeit
        // is turn
        self.state = State::Playing;
    }

    /// Start the game of Othello between the two players
    pub fn play(&mut self) -> Result<()> {
        loop {
            self.legal_moves();
            if self.current_player().is_human() {
                self.render(None)?;
            }

            match self.state {
                State::Playing => {}
                State::Winned {
                    winner_color,
                    winner_score,
                    loser_score,
                } => {
                    let s = &mut *self.stream.borrow_mut();
                    let winner = match winner_color {
                        Disc::White => &self.white_player,
                        Disc::Black => &self.black_player,
                        Disc::Empty => unreachable!(),
                    };

                    writeln!(s)?;
                    writeln!(
                        s,
                        "  Congratulation, {} ({})! you win with {}-{}",
                        winner.force_name(),
                        winner_color,
                        winner_score,
                        loser_score
                    )?;
                    break;
                }
                State::Draw => {
                    let s = &mut *self.stream.borrow_mut();
                    writeln!(s)?;
                    writeln!(s, "  The game ended in a draw, congrats for both of you.")?;
                    break;
                }
                State::TurnForfeited => {
                    // the current player can't play so we pass the turn to the
                    // opponent that can play.
                    {
                        let s = &mut *self.stream.borrow_mut();
                        writeln!(
                            s,
                            "The turn of {} has been forfeited, he cannot play.",
                            self.turn()
                        )?;
                    }
                    self.next_turn();
                    continue;
                }
            }

            let mut previous_err = None;
            let mov = loop {
                let res = self.player_think(previous_err);

                if let Ok(mov) = res {
                    break mov;
                }
                // TODO: we may only recall `think` if the error is not an io error.
                let Err(e) = res else { unreachable!() };
                previous_err = Some(e);
            };

            match self.make_turn(mov) {
                Ok(()) => {}
                Err(e @ OthebotError::IllegalMove { .. }) => {
                    let s = &mut *self.stream.borrow_mut();
                    s.set_color(&style::ERROR)?;
                    writeln!(s, "{e}")?;
                    s.reset()?;
                }
                Err(e) => return Err(e),
            };
        }
        Ok(())
    }

    /// Call the method `think` on the current player.
    fn player_think(&self, previous_err: Option<OthebotError>) -> Result<Move> {
        match self.turn() {
            Disc::Black => self.black_player.think(self, previous_err),
            Disc::White => self.white_player.think(self, previous_err),
            Disc::Empty => unreachable!(),
        }
    }

    pub fn current_player(&self) -> &Box<dyn Player> {
        match self.turn() {
            Disc::White => &self.white_player,
            Disc::Black => &self.black_player,
            Disc::Empty => unreachable!(),
        }
    }

    #[inline]
    #[must_use]
    pub fn white_name(&self) -> Cow<'_, str> {
        self.white_player.force_name()
    }

    #[inline]
    #[must_use]
    pub fn black_name(&self) -> Cow<'_, str> {
        self.black_player.force_name()
    }

    #[inline]
    #[must_use]
    pub fn player_name(&self) -> Cow<'_, str> {
        match self.turn {
            Disc::White => self.white_name(),
            Disc::Black => self.black_name(),
            Disc::Empty => unreachable!(),
        }
    }

    #[inline]
    #[must_use]
    pub fn maybe_name(&self) -> Option<Cow<'static, str>> {
        match self.turn {
            Disc::White => self.white_player.name(),
            Disc::Black => self.black_player.name(),
            Disc::Empty => unreachable!(),
        }
    }

    /// Renders the board game to stdout
    pub fn render(&self, s: Option<&mut StandardStream>) -> Result<()> {
        let mut _s = self.stream.borrow_mut();
        let s: &mut StandardStream = s.unwrap_or(&mut *_s);
        let Some(legal_moves) = self.current_legal_moves else {
            return Err(OthebotError::LegalMovesNotComputed);
        };

        for row in 0..8 {
            s.set_color(&style::BOARD_EDGES)?;
            write!(s, "+---+---+---+---+---+---+---+---+")?;
            s.reset()?;

            // print the scores
            if row == 7 {
                let (white_score, black_score, _) = self.board.scores();
                write!(s, "    ")?;

                s.set_color(&style::BLACK_PLAYER)?;
                write!(s, "{}", self.black_name())?;
                s.reset()?;
                write!(s, ": {black_score}  ")?;

                s.set_color(&style::WHITE_PLAYER)?;
                write!(s, "{}", self.white_name())?;
                s.reset()?;
                write!(s, ": {white_score}")?;
            }

            writeln!(s)?;

            for col in 0..8 {
                let idx = row * 8 + col;
                let is_legal_move = (1 << idx) & legal_moves != 0;
                let disc = self.board.squares[idx];

                s.set_color(&style::BOARD_EDGES)?;
                write!(s, "|")?;
                s.reset()?;

                match disc {
                    Disc::White => {
                        s.set_color(&style::WHITE_PLAYER)?;
                        write!(s, " W ")?;
                    }
                    Disc::Black => {
                        s.set_color(&style::BLACK_PLAYER)?;
                        write!(s, " B ")?;
                    }
                    Disc::Empty if is_legal_move => {
                        s.set_color(&style::LEGAL_MOVE)?;
                        write!(s, " • ")?;
                    }
                    Disc::Empty => write!(s, "   ")?,
                }
                s.reset()?;
            }

            s.set_color(&style::BOARD_EDGES)?;
            write!(s, "|")?;
            s.reset()?;

            s.set_color(&style::WHITE_BOLD)?;
            write!(s, " {}", row + 1)?;

            // print the score
            if row == 6 {
                write!(s, "  SCORES:")?;
            }
            s.reset()?;

            writeln!(s)?;
        }
        s.set_color(&style::BOARD_EDGES)?;
        writeln!(s, "+---+---+---+---+---+---+---+---+")?;
        s.reset()?;

        s.set_color(&style::WHITE_BOLD)?;
        writeln!(s, "  a   b   c   d   e   f   g   h")?;
        s.reset()?;

        Ok(())
    }

    /// Compute and store the legal moves of the current player.
    fn legal_moves(&mut self) {
        self.current_legal_moves = Some(self.board.legal_moves(self.turn()));

        if let Some(0) = self.current_legal_moves {
            if self.board.legal_moves(!self.turn()) != 0 {
                // the opponent can play, so we forfeit this turn
                self.state = State::TurnForfeited;
                return;
            }
            // No one can move this is either a draw or a win.
            let (white, black, empty) = self.board.scores();
            if white == black {
                // this is a draw.
                self.state = State::Draw;
                return;
            }
            // TODO: here a simple opti is storing `white > black`
            let winner_score = white.max(black) + empty;
            let loser_score = white.min(black);

            let winner_color = if white > black {
                Disc::White
            } else {
                Disc::Black
            };

            self.state = State::Winned {
                winner_color,
                winner_score,
                loser_score,
            };
        }
    }

    #[inline]
    pub fn moves(&self) -> u64 {
        self.current_legal_moves.unwrap()
    }
}
