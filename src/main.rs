// TODO: Seperate the Othello interface (the binary) from the library (Othello Engine)

// TODO: rerename this projet `0the` why? because it's simple like this project
// and when the engine will be separated from the client, name it `othengine`
use std::{
    error::Error,
    fs::{self, File},
    io::{self, Read, Write},
    str::FromStr,
};

use othe::{
    player::{HumanPlayer, Player, RandomPlayer},
    style, Board, Disc, Game, GameSave, GameSettings, OthelloError, State, LICENSE, OTHELLO_RULES,
    VERSION_AND_GIT_HASH,
};
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn player_init(s: &mut StandardStream, color: Disc) -> Result<Box<dyn Player>, OthelloError> {
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
        _ => Err(OthelloError::InvalidPlayerType),
    }
}

pub fn start_game(notation: Option<&str>, settings: GameSettings) -> Result<(), OthelloError> {
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
    game.post_play()?;

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
) -> Result<(), OthelloError> {
    // TODO: save the settings as a TOML config.

    write!(
        s,
        "\
Settings:
 1. Show legal moves: {:3}         Show the dots on the board indicating the
                                  legals moves of the player.
 2. Saves game directory: {}
                                  Directory where the games are saved, must be
                                  set if you enable game recordings.
 3. Game recordings: {:3}          Record the games and store them to the
                                  saves directory

Choose a settings to change or type `q`: \
",
        yes_no(settings.show_legal_moves),
        settings
            .clone()
            .saves_game_dir
            .map(|p| p.display().to_string())
            .unwrap_or(String::from("None")),
        yes_no(settings.game_record)
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
        "2" => todo!("implement this setting"),
        "3" => {
            buf.clear();
            write!(s, "`Yes` or `No`? ")?;
            s.flush()?;
            io::stdin().read_line(&mut buf)?;
            // pop the newline character
            buf.pop();

            settings.game_record = match buf.to_lowercase().trim() {
                "yes" => true,
                "no" => false,
                _ => return Ok(()),
            };
        }
        _ => return Ok(()),
    }

    Ok(())
}

pub fn replay_game(s: &mut StandardStream, settings: &GameSettings) -> Result<(), OthelloError> {
    if let Some(saves_path) = &settings.saves_game_dir {
        let save_paths: Vec<_> = fs::read_dir(&saves_path)?
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file())
            .enumerate()
            .collect();
        writeln!(
            s,
            "Replay a Game, saves are located in {}",
            saves_path.display()
        )?;
        for (i, path) in &save_paths {
            let mut save_file = File::open(&path)?;
            let mut save_json = String::new();

            save_file.read_to_string(&mut save_json)?;
            let save = GameSave::from_json(&save_json)?;

            // keep only the file name and extension, here we unwrap it should
            // never panic because we know the save path contains the path to
            // the save dir.
            let pretty_path = path.strip_prefix(saves_path.clone()).unwrap();
            writeln!(s, "{}. {}, {:?}", i + 1, save.title, pretty_path.display())?;

            match save.end_state {
                State::Winned {
                    winner_color,
                    winner_name,
                    winner_score,
                    loser_score,
                } => {
                    writeln!(
                        s,
                        "   {} ({}) winned, with {}-{}",
                        winner_name, winner_color, winner_score, loser_score
                    )?;
                    writeln!(s)?;
                }
                State::Draw => {
                    writeln!(s, "  The game ended in a draw, congrats for both of you.")?;
                    writeln!(s)?;
                }
                _ => unreachable!("Not an end game state."),
            }
        }

        let mut buf = String::new();
        writeln!(s)?;
        write!(s, "What save do you want to replay? (or type `q` to quit) ")?;
        s.flush()?;
        io::stdin().read_line(&mut buf)?;
        // pop the new line character
        buf.pop();

        match buf.as_str() {
            "q" => return Ok(()),
            int => {
                // TODO: replace this unwrap
                let i: usize = int.parse().unwrap();

                // TODO: replace this slice sintax with `.get()` and this minus
                // one with `checked_sub`.
                let mut save_file = File::open(&save_paths[i - 1].1)?;
                let mut save_json = String::new();

                save_file.read_to_string(&mut save_json)?;
                let mut save = GameSave::from_json(&save_json)?;
                let stream = StandardStream::stdout(ColorChoice::Auto);
                save.replay(stream)?;
            }
        }
    } else {
        s.set_color(&style::ERROR)?;
        writeln!(s, "The game save directory isn't set.")?;
        s.reset()?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut s = StandardStream::stdout(ColorChoice::Auto);

    writeln!(s, "Welcome, in 0the CLI!\n")?;
    let help = format!(
        "\
{} {}
{}

COMMANDS:
    play, p             Start a new game
    import <notation>   Import a game using the Othello Notation
    replay, r           Replay a previously saved game
    set                 Alter 0the settings
    rules               Print the rules of Othello
    license             Print the license of the program
    help, h             Print this message
    quit, q             Quit the program\
    ",
        env!("CARGO_BIN_NAME"),
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
            // TODO: don't clone the settings but use some kind of (smart) pointer
            ["play" | "p"] => start_game(None, settings.clone()),
            ["import", notation] => start_game(Some(notation), settings.clone()),
            ["replay" | "r"] => replay_game(&mut s, &settings),
            ["set"] => settings_menu(&mut s, &mut settings),
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
            _ => {
                writeln!(s, r#"Unknown command {cmd:?}, type "help" for help."#)?;
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

        writeln!(s)?;
    }

    Ok(())
}
