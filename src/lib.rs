use std::{
    error::Error,
    fmt::{self, Display},
    ops::Not,
};

pub const VERSION_AND_GIT_HASH: &str = env!("VERSION_AND_GIT_HASH");

pub const LICENSE: &str = include_str!("../LICENSE");

#[derive(Debug, Clone)]
pub enum OthebotError {
    InvalidAlgebric(String),
    IllegalMove { row: u8, col: u8 },
    LegalMovesNotComputed,
}

impl Error for OthebotError {}

impl Display for OthebotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OthebotError::InvalidAlgebric(notation) => write!(f, "invalid algebric notation {notation:?}, valid e.g: `a5`"),
            OthebotError::IllegalMove{ row, col} => write!(f, "illegal move (row: {row}, col: {col}), you can't put your disc here"),
            OthebotError::LegalMovesNotComputed => write!(f, "INTERNAL ERROR: legal moves were not computed before calling a function that depends on legal moves.")
        }
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
    fn change_disc(&mut self, (col, row): (u8, u8), disc: Disc) {
        assert!(col < 8);
        assert!(row < 8);
        // UNSAFE: we checked that they are in bounds
        let idx = (row * 8 + col) as usize;
        *self.squares.get_mut(idx).unwrap() = disc;
    }

    /// Returns the scores of the current board, in the tuple, white's score is
    /// first, and black's score is second
    pub fn scores(&self) -> (u8, u8) {
        let mut white = 0;
        let mut black = 0;
        for disc in self.squares {
            match disc {
                Disc::White => white += 1,
                Disc::Black => black += 1,
                Disc::Empty => {}
            }
        }
        (white, black)
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
    pub fn move_outflanks(&self, player: Disc, (x, y): (u8, u8)) -> u64 {
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

/// Converts an algebric notation like `a1`, `g8`, `b7` etc to `(0, 0)`,
/// `(6, 7)`, `(1, 6)`.
pub fn algebric2xy(pos: &str) -> Result<(u8, u8), OthebotError> {
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

pub fn bitfield_to_indexes(bitfield: u64) -> Vec<usize> {
    let mut positions = Vec::new();
    for i in 0..64 {
        if (bitfield & (1 << i)) != 0 {
            positions.push(i);
        }
    }
    positions
}

pub struct Game {
    board: Board,

    // TODO: if the given usernames are empty, don't use them, use instead their color.
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
    /// The legal moves of the current player (`turn` field).
    current_legal_moves: Option<u64>,
}

impl Game {
    pub fn new(white_player: impl Into<String>, black_player: impl Into<String>) -> Game {
        Game {
            board: Board::new(),
            white_player: white_player.into(),
            black_player: black_player.into(),
            turn: Disc::Black,
            current_legal_moves: None,
        }
    }

    pub fn turn(&self) -> Disc {
        self.turn
    }

    fn is_legal(bitfield: u64, index: usize) -> bool {
        (bitfield & (1 << index)) != 0
    }

    pub fn is_legal_move(&self, index: usize) -> Result<bool, OthebotError> {
        let Some(moves) = self.current_legal_moves else {
            return Err(OthebotError::LegalMovesNotComputed);
        };
        Ok(Self::is_legal(moves, index))
    }

    pub fn make_turn(&mut self, mov @ (col, row): (u8, u8)) -> Result<(), OthebotError> {
        // ensure the move is inside the legal moves.
        let idx = (row * 8 + col) as u64;
        if !self.is_legal_move(idx as usize)? {
            return Err(OthebotError::IllegalMove { row, col });
        }
        self.board.change_disc(mov, self.turn);
        let outflanks = self.board.move_outflanks(self.turn, mov);
        self.board.put_discs(outflanks, self.turn);

        self.turn = !self.turn;

        self.current_legal_moves = None;
        Ok(())
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
    pub fn render(&self) -> Result<(), OthebotError> {
        // TODO: Add colors.
        let Some(legal_moves) = self.current_legal_moves else {
            return Err(OthebotError::LegalMovesNotComputed);
        };

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
                let idx = row * 8 + col;
                let is_legal_move = (1 << idx) & legal_moves != 0;
                let disc = self.board.squares[idx];
                print!("| ");
                match disc {
                    Disc::White => print!("W"),
                    Disc::Black => print!("B"),
                    Disc::Empty if is_legal_move => print!("•"),
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

        Ok(())
    }

    /// Compute and store the legal moves of the current player.
    pub fn legal_moves(&mut self) {
        self.current_legal_moves = Some(self.board.legal_moves(self.turn()));
    }

    pub fn moves(&self) -> u64 {
        self.current_legal_moves.unwrap()
    }
}
