use std::{
    error::Error,
    io::{self, Write},
};

use othebot::{algebric2xy, Game, LICENSE, VERSION_AND_GIT_HASH};

pub fn start_game() -> Result<(), Box<dyn Error>> {
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

    let mut game = Game::new(white, black);

    let i = 0;
    loop {
        if i >= 10 {
            break;
        }
        game.render();

        let pos;
        loop {
            let mut mov = String::new();
            print!("{} ({})'s turn: ", game.turn(), game.player_name());
            io::stdout().flush()?;
            io::stdin().read_line(&mut mov)?;
            // we pop the newline
            mov.pop();
            let res = algebric2xy(&mov);
            match res {
                Some(p) => {
                    pos = p;
                    break;
                }
                None => {
                    println!(r#"incorrect movement {mov:?}, e.g of movement "a4", "g8"."#);
                    panic!();
                }
            }
        }

        game.make_turn(pos);
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
