use std::fmt::Debug;
use std::io::Write;
use std::{borrow::Cow, io};

use termcolor::WriteColor;

use crate::{style, Disc, Game, Move, OthebotError, Result};

/// A player of the Othello Game, it may be Human, a bot like MinMax, AlphaBeta
/// pruning, Monte Carlo Tree Search, a fancy powerful AI..
pub trait Player: Debug {
    /// Return the player's color (black / white), cannot be `Disc::Empty`.
    fn color(&self) -> Disc;

    /// This function is called when it is the turn of this player, or when the
    /// previous call to this function resulted in a error (`err` arg) from the
    /// player, like illegal move etc..
    fn think(&self, game: &Game, err: Option<OthebotError>) -> Result<Move>;

    /// Return the name of the player.
    fn name(&self) -> Option<&String>;

    /// Init the player color if the player stores its disc color.
    fn init_color(&mut self, color: Disc);

    /// Return the name of the player and if he have no name, its color.
    fn force_name(&self) -> Cow<'_, str> {
        match self.name() {
            Some(name) => name.into(),
            None => match self.color() {
                Disc::White => "White",
                Disc::Black => "Black",
                Disc::Empty => unreachable!(),
            }
            .into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HumanPlayer {
    color: Disc,
    name: Option<String>,
}

impl HumanPlayer {
    pub fn new(name: impl Into<Option<String>>) -> HumanPlayer {
        let name = if let Some(n) = name.into() {
            if n.is_empty() {
                None
            } else {
                Some(n)
            }
        } else {
            None
        };

        HumanPlayer {
            color: Disc::Empty,
            name,
        }
    }
}

impl Player for HumanPlayer {
    fn color(&self) -> Disc {
        self.color
    }

    fn think(&self, game: &Game, err: Option<OthebotError>) -> Result<Move> {
        let s = &mut *game.stream.borrow_mut();

        if let Some(err) = err {
            s.set_color(&style::ERROR)?;
            writeln!(s, "{err}")?;
            s.reset()?;
        }

        let mut mov_str = String::with_capacity(3);

        write!(s, "{}", game.turn())?;
        if let Some(name) = self.name() {
            write!(s, " ({})", name)?;
        }
        write!(s, "'s turn: ")?;

        s.flush()?;
        io::stdin().read_line(&mut mov_str)?;
        // pop the newline char at the end
        mov_str.pop();

        Move::from_algebric(&mov_str)
    }

    fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    fn init_color(&mut self, color: Disc) {
        assert_eq!(self.color, Disc::Empty);
        assert_ne!(color, Disc::Empty);
        self.color = color;
    }
}
