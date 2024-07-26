use std::{
    fmt::{self, Display},
    ops::Not,
};

pub const VERSION_AND_GIT_HASH: &str = env!("VERSION_AND_GIT_HASH");

pub const LICENSE: &str = include_str!("../LICENSE");

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

pub struct Board {
    discs: [Disc; 64],
}

impl Board {
    /// Create a new board with the starting layout
    pub const fn new() -> Board {
        use Disc::Black as B;
        use Disc::Empty as E;
        use Disc::White as W;
        Board {
            discs: [
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
        self.discs[(row * 8 + col) as usize]
    }

    #[track_caller]
    pub fn change_disc(&mut self, (col, row): (u8, u8), disc: Disc) {
        assert!(col < 8);
        assert!(row < 8);
        // UNSAFE: we checked that they are in bounds
        let idx = (row * 8 + col) as usize;
        *self.discs.get_mut(idx).unwrap() = disc;
    }

    /// Returns the scores of the current board, in the tuple, white's score is
    /// first, and black's score is second
    pub fn scores(&self) -> (u8, u8) {
        let mut white = 0;
        let mut black = 0;
        for disc in self.discs {
            match disc {
                Disc::White => white += 1,
                Disc::Black => black += 1,
                Disc::Empty => {}
            }
        }
        (white, black)
    }
}

impl Default for Board {
    #[inline]
    fn default() -> Self {
        Board::new()
    }
}

/// Converts an algebric notation like `a1`, `g8`, `b7` etc to `(0, 0)`,
/// `(6, 7)`, `(1, 6)`.
pub fn algebric2xy(pos: &str) -> Option<(u8, u8)> {
    if pos.len() != 2 {
        return None;
    }

    let mut it = pos.chars();
    let col = it.next().unwrap() as u8;
    let row = it.next().unwrap() as u8;

    if !(b'a'..=b'h').contains(&col) {
        return None;
    }
    if !(b'1'..=b'8').contains(&row) {
        return None;
    }

    Some((col - b'a', row - b'1'))
}

pub struct Game {
    board: Board,

    /// White player name
    white_player: String,

    /// Black player name
    black_player: String,

    /// Who's next turn?
    ///
    /// Note:
    ///
    /// `turn` cannot be `Disc::Empty`.
    turn: Disc,
}

impl Game {
    pub fn new(white_player: impl Into<String>, black_player: impl Into<String>) -> Game {
        Game {
            board: Board::new(),
            white_player: white_player.into(),
            black_player: black_player.into(),
            turn: Disc::Black,
        }
    }

    pub fn turn(&self) -> Disc {
        self.turn
    }

    pub fn make_turn(&mut self, mov: (u8, u8)) {
        self.board.change_disc(mov, self.turn);
        self.turn = !self.turn;
    }

    #[inline]
    #[must_use]
    pub fn white_name(&self) -> &str {
        &self.white_player
    }

    #[inline]
    #[must_use]
    pub fn black_name(&self) -> &str {
        &self.black_player
    }

    #[inline]
    #[must_use]
    pub fn player_name(&self) -> &str {
        match self.turn {
            Disc::White => self.white_name(),
            Disc::Black => self.black_name(),
            Disc::Empty => unreachable!(),
        }
    }

    /// Renders the board game to stdout
    pub fn render(&self) {
        for row in 0..8 {
            print!("+---+---+---+---+---+---+---+---+");

            // print the scores
            if row == 7 {
                let (white_score, black_score) = self.board.scores();
                print!(
                    "    {}: {}  {}: {}",
                    self.black_name(),
                    black_score,
                    self.white_name(),
                    white_score,
                );
            }

            println!();

            for col in 0..8 {
                let disc = unsafe { self.board.get_disc_unchecked(col, row) };
                print!("| ");
                match disc {
                    Disc::White => print!("W"),
                    Disc::Black => print!("B"),
                    Disc::Empty => print!(" "),
                }
                print!(" ");
            }

            print!("| {}", row + 1);

            // print the score
            if row == 6 {
                print!("  SCORES:");
            }

            println!();
        }
        println!("+---+---+---+---+---+---+---+---+");
        println!("  a   b   c   d   e   f   g   h");
    }
}
