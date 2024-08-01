use std::{
    error::Error,
    io::{self, Write},
    str::FromStr,
};

use othebot::{
    player::{HumanPlayer, Player, RandomPlayer},
    style, Board, Disc, Game, GameSettings, OthebotError, LICENSE, OTHELLO_RULES,
    VERSION_AND_GIT_HASH,
};
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn player_init(s: &mut StandardStream, color: Disc) -> Result<Box<dyn Player>, OthebotError> {
    let mut buf = String::new();
    write!(s, "{color} player's type (1): ")?;
    s.flush()?;
    io::stdin().read_line(&mut buf)?;
    // pop the newline character
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
        _ => Err(OthebotError::InvalidPlayerType),
    }
}

pub fn start_game(notation: Option<&str>, settings: GameSettings) -> Result<(), OthebotError> {
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
        Game::with_board(
            Board::from_str(notation)?,
            white_player,
            black_player,
            s,
            settings,
        )
    } else {
        Game::new(white_player, black_player, s, settings)
    };
    game.play()?;

    Ok(())
}

pub fn yes_no(yes: bool) -> &'static str {
    if yes {
        "Yes"
    } else {
        "No"
    }
}

pub fn settings_menu(
    s: &mut StandardStream,
    settings: &mut GameSettings,
) -> Result<(), OthebotError> {
    write!(
        s,
        "\
Settings:
 1. Show legal moves: {:3}         Show the dots on the board indicating the
                                  legals moves of the player.

Choose a settings to change or type `q`: \
",
        yes_no(settings.show_legal_moves),
    )?;

    let mut buf = String::new();
    s.flush()?;
    io::stdin().read_line(&mut buf)?;
    // pop the newline character
    buf.pop();
    match buf.as_str() {
        "1" => {
            buf.clear();
            write!(s, "`Yes` or `No`? ")?;
            s.flush()?;
            io::stdin().read_line(&mut buf)?;
            // pop the newline character
            buf.pop();

            settings.show_legal_moves = match buf.to_lowercase().trim() {
                "yes" => true,
                "no" => false,
                _ => return Ok(()),
            };
        }
        _ => return Ok(()),
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut s = StandardStream::stdout(ColorChoice::Auto);

    writeln!(s, "Welcome, in Othebot!\n")?;
    let help = format!(
        "\
{} {}
{}

COMMANDS:
    play, p             Start a new game
    import <notation>   Import a game using the Othello Notation.
    settings            Change settings of Games
    rules               Print the rules of Othello
    license             Print the license of the program
    help, h             Print this message
    quit, q             Quit the program\
    ",
        env!("CARGO_PKG_NAME"),
        VERSION_AND_GIT_HASH,
        env!("CARGO_PKG_AUTHORS"),
    );

    let mut settings = GameSettings::default();

    let mut cmd = String::new();
    loop {
        write!(s, "Command (h for help): ")?;
        io::stdout().flush()?;
        cmd.clear();
        io::stdin().read_line(&mut cmd)?;
        // remove the newline
        cmd.pop();

        let vec = cmd.split_whitespace().collect::<Vec<_>>();
        let args = vec.as_slice();

        let res = match args {
            ["play" | "p"] => start_game(None, settings.clone()),
            ["import", notation] => start_game(Some(notation), settings.clone()),
            ["settings"] => settings_menu(&mut s, &mut settings),
            ["rules"] => {
                writeln!(s, "{}", OTHELLO_RULES)?;
                Ok(())
            }
            ["license"] => {
                writeln!(s, "{}", LICENSE)?;
                Ok(())
            }
            ["help" | "h"] => {
                writeln!(s, "{help}")?;
                Ok(())
            }
            ["quit" | "q"] => break,
            unknown => {
                writeln!(s, r#"Unknown command {unknown:?}, type "help" for help."#)?;
                Ok(())
            }
        };
        match res {
            Ok(()) => {}
            Err(e) => {
                s.set_color(&style::ERROR)?;
                writeln!(s, "{e}")?;
                s.reset()?;
            }
        }

        println!();
    }

    Ok(())
}
