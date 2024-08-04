use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::{borrow::Cow, io};

use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use termcolor::WriteColor;

use crate::{bitfield_to_indexes, style, Disc, Game, Move, OthelloError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerType {
    /// Human player
    Human,
    /// Robot player
    Bot,
}

/// A player of the Othello Game, it may be Human, a bot like MinMax, AlphaBeta
/// pruning, Monte Carlo Tree Search, a fancy powerful AI..
pub trait Player: Debug {
    /// Return the player's color (black / white), cannot be `Disc::Empty`.
    fn color(&self) -> Disc;

    /// This function is called when it is the turn of this player, or when the
    /// previous call to this function resulted in a error (`err` arg) from the
    /// player, like illegal move etc..
    fn think(&self, game: &Game, err: Option<OthelloError>) -> Result<Move>;

    /// Return the name of the player.
    fn name(&self) -> Option<Cow<'static, str>>;

    /// Init the player color if the player stores its disc color.
    fn init_color(&mut self, color: Disc);

    /// Return the name of the player and if he have no name, its color.
    fn force_name(&self) -> Cow<'_, str> {
        match self.name() {
            Some(name) => name,
            None => match self.color() {
                Disc::White => "White",
                Disc::Black => "Black",
                Disc::Empty => unreachable!(),
            }
            .into(),
        }
    }

    /// What is the player type? Human or Bot?
    fn player_type(&self) -> PlayerType;

    /// Is the player, human, used to know when using the CLI whetever to print
    /// the board or not
    #[inline]
    fn render_board(&self) -> bool {
        self.player_type() == PlayerType::Human
    }
}

#[derive(Debug, Clone)]
pub struct HumanPlayer {
    color: Disc,
    name: Option<String>,
}

impl HumanPlayer {
    pub fn new(name: impl Into<Option<String>>) -> HumanPlayer {
        let name = name.into().filter(|n| !n.is_empty());

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

    fn think(&self, game: &Game, err: Option<OthelloError>) -> Result<Move> {
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

    fn name(&self) -> Option<Cow<'static, str>> {
        self.name.clone().map(Cow::Owned)
    }

    fn init_color(&mut self, color: Disc) {
        assert_eq!(self.color, Disc::Empty);
        assert_ne!(color, Disc::Empty);
        self.color = color;
    }

    #[inline]
    fn player_type(&self) -> PlayerType {
        PlayerType::Human
    }
}

#[derive(Debug, Clone)]
pub struct RandomPlayer {
    color: Disc,
}

impl Default for RandomPlayer {
    fn default() -> Self {
        RandomPlayer { color: Disc::Empty }
    }
}

impl Player for RandomPlayer {
    fn color(&self) -> Disc {
        self.color
    }

    fn think(&self, game: &Game, err: Option<OthelloError>) -> Result<Move> {
        // ensure there is no error(s).
        assert!(err.is_none());

        let Some(legal_moves) = game.current_legal_moves else {
            return Err(OthelloError::LegalMovesNotComputed);
        };

        let legal_moves = bitfield_to_indexes(legal_moves);

        let mut rand = rand::thread_rng();

        // it's safe to unwrap, it only return `None` if the vector is empty
        // and we know for a fact he is not because we can play
        Ok(Move::from_idx(
            *legal_moves.iter().choose(&mut rand).unwrap(),
        ))
    }

    fn name(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed("Random Bot"))
    }

    fn init_color(&mut self, color: Disc) {
        assert_eq!(self.color, Disc::Empty);
        assert_ne!(color, Disc::Empty);
        self.color = color;
    }

    #[inline]
    fn player_type(&self) -> PlayerType {
        PlayerType::Bot
    }
}

#[derive(Debug, Clone)]
pub struct ReplayPlayer {
    pub(crate) moves: Arc<Mutex<Vec<Move>>>,
    pub(crate) move_idx: Arc<Mutex<usize>>,
    pub(crate) color: Disc,
    pub(crate) player_type: PlayerType,
    pub(crate) name: Option<Cow<'static, str>>,
}

impl Player for ReplayPlayer {
    fn color(&self) -> Disc {
        self.color
    }

    fn think(&self, game: &Game, err: Option<OthelloError>) -> Result<Move> {
        // ensure there is no error(s).
        assert!(err.is_none());

        // it shouldn't panic because the players move one after the other
        let mut idx = self.move_idx.lock().unwrap();
        let mov = self.moves.lock().unwrap()[*idx];
        *idx += 1;

        // Prompt the user
        let mut s = game.stream.borrow_mut();
        write!(s, "Press any key to continue...")?;
        s.flush().unwrap();

        // Wait for input
        let _ = io::stdin().read(&mut [0u8])?;

        Ok(mov)
    }

    fn name(&self) -> Option<Cow<'static, str>> {
        self.name.clone()
    }

    fn init_color(&mut self, _: Disc) {
        // nothing we already init the color in the replay
    }

    fn player_type(&self) -> PlayerType {
        self.player_type
    }

    fn render_board(&self) -> bool {
        // this is used to render the board.
        true
    }
}
