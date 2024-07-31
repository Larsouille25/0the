use std::{
    error::Error,
    io::{self, Write},
    str::FromStr,
};

use othebot::{
    player::{HumanPlayer, Player, RandomPlayer},
    style, Board, Disc, Game, OthebotError, LICENSE, OTHELLO_RULES, VERSION_AND_GIT_HASH,
};
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn player_init(s: &mut StandardStream, color: Disc) -> Result<Box<dyn Player>, OthebotError> {
    let mut buf = String::new();
    write!(s, "{color} player's type (1): ")?;
    s.flush()?;
    io::stdin().read_line(&mut buf)?;
    buf.pop();
    match buf.as_str() {
        "" | "1" => {
            // human player
            buf.clear();
            write!(s, "                   name: ")?;
            s.flush()?;
            io::stdin().read_line(&mut buf)?;
            buf.pop();
            Ok(Box::new(HumanPlayer::new(buf)))
        }
        "2" => {
            // random bot player
            Ok(Box::new(RandomPlayer::default()))
        }
        _ => {
            s.set_color(&style::ERROR)?;
            writeln!(s, "Choose one of the available player types.")?;
            s.reset()?;
            s.flush()?;
            todo!("Make this an error of OthebotError")
        }
    }
}
pub fn start_game(notation: Option<&str>) -> Result<(), OthebotError> {
    let mut s = StandardStream::stdout(ColorChoice::Auto);

    writeln!(
        s,
        "\
Available player types:
 1. Human
 2. Random Bot
"
    )?;

    let black_player = player_init(&mut s, Disc::Black)?;
    let white_player = player_init(&mut s, Disc::White)?;

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
