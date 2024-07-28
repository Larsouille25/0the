use lazy_static::lazy_static;
use termcolor::{Color, ColorSpec};

lazy_static! {
    pub static ref BOARD_EDGES: ColorSpec = ColorSpec::new()
        .set_fg(Some(Color::Ansi256(28)))
        .set_bold(true)
        .clone();
    pub static ref BLACK_PLAYER: ColorSpec = ColorSpec::new()
        .set_fg(Some(Color::Ansi256(255)))
        .set_bg(Some(Color::Ansi256(235)))
        .set_bold(true)
        .clone();
    pub static ref WHITE_PLAYER: ColorSpec = ColorSpec::new()
        .set_fg(Some(Color::Ansi256(235)))
        .set_bg(Some(Color::Ansi256(255)))
        .set_bold(true)
        .clone();
    pub static ref WHITE_BOLD: ColorSpec = ColorSpec::new()
        .set_fg(Some(Color::White))
        .set_bold(true)
        .clone();
    pub static ref LEGAL_MOVE: ColorSpec = ColorSpec::new()
        .set_fg(Some(Color::Ansi256(15)))
        .set_bold(true)
        .clone();
}
