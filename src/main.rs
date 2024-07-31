use std::{
    error::Error,
    io::{self, Write},
    str::FromStr,
};

use othebot::{
    player::HumanPlayer, Board, Disc, Game, OthebotError, LICENSE, OTHELLO_RULES,
    VERSION_AND_GIT_HASH,
};
use termcolor::{ColorChoice, StandardStream};

pub fn start_game(notation: Option<&str>) -> Result<(), OthebotError> {
    let mut black = String::new();
    print!("Black player's name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut black)?;
    black.pop();

    let mut white = String::new();
    print!("White player's name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut white)?;
    white.pop();

    let s = StandardStream::stdout(ColorChoice::Auto);

    let white_player = Box::new(HumanPlayer::new(white));
    let black_player = Box::new(HumanPlayer::new(black));
    let mut game = if let Some(notation) = notation {
        Game::with_board(Board::from_str(notation)?, white_player, black_player, s)
    } else {
        Game::new(white_player, black_player, s)
    };
    game.play()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome, in Othebot!\n");
    let help = format!(
        "\
{} {}
{}

COMMANDS:
    play, p             Start a new game
    import <notation>   Import a game using the Othello Notation.
    rules               Print the rules of Othello
    license             Print the license of the program
    help, h             Print this message
    quit, q             Quit the program\
    ",
        env!("CARGO_PKG_NAME"),
        VERSION_AND_GIT_HASH,
        env!("CARGO_PKG_AUTHORS"),
    );

    let mut cmd = String::new();
    loop {
        print!("Command (h for help): ");
        io::stdout().flush()?;
        cmd.clear();
        io::stdin().read_line(&mut cmd)?;
        // remove the newline
        cmd.pop();

        let vec = cmd.split_whitespace().collect::<Vec<_>>();
        let args = vec.as_slice();

        match args {
            // TODO: don't return here when calling `start_game`
            ["play" | "p"] => start_game(None)?,
            ["import", notation] => start_game(Some(notation))?,
            ["rules"] => println!("{}", OTHELLO_RULES),
            ["license"] => println!("{}", LICENSE),
            ["help" | "h"] => println!("{help}"),
            ["quit" | "q"] => break,
            unknown => println!(r#"Unknown command {unknown:?}, type "help" for help."#),
        }
        println!();
    }

    Ok(())
}
