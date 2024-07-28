use std::{
    error::Error,
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

use othebot::{algebric2xy, Game, OthebotError, LICENSE, VERSION_AND_GIT_HASH};
use termcolor::{ColorChoice, StandardStream};

pub fn start_game() -> Result<(), OthebotError> {
    // TODO: change the err type of the result to OthebotError
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

    let mut s = StandardStream::stdout(ColorChoice::Auto);

    let mut game = Game::new(white, black);

    loop {
        if false {
            break;
        }
        game.legal_moves();
        game.render(&mut s)?;

        let mut pos;
        loop {
            let mut mov = String::new();
            print!("{} ({})'s turn: ", game.turn(), game.player_name());
            io::stdout().flush()?;
            io::stdin().read_line(&mut mov)?;
            // we pop the newline
            mov.pop();
            let res = algebric2xy(&mov);

            match res {
                Ok(p) => {
                    pos = p;
                }
                Err(e @ OthebotError::InvalidAlgebric(_)) => {
                    println!("{e}");
                    // here we sleep to show the player we made an error
                    // because after this error message the board will be
                    // re-rendered and could confuse the player
                    sleep(Duration::from_secs_f32(1.5));
                    break;
                }
                Err(e) => return Err(e),
            }

            match game.make_turn(pos) {
                Ok(()) => break,
                Err(e @ OthebotError::IllegalMove { .. }) => println!("{e}"),
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome, in Othebot!\n");
    let help = format!(
        "\
{} {}
{}

COMMANDS:
    game, g             Start a new game
    license             Prints the license of the program
    help, h             Prints this message
    quit, q             Quit of the program\
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

        match cmd.as_str() {
            "game" | "g" => start_game()?,
            "license" => println!("{}", LICENSE),
            "help" | "h" => println!("{help}"),
            "quit" | "q" => break,
            unknown => println!(r#"Unknown command {unknown:?}, type "help" for help."#),
        }
        println!();
    }

    Ok(())
}
