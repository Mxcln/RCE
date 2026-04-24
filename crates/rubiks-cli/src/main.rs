mod render;
mod repl;

use std::env;
use std::io::{self, BufReader};

use rubiks_core::Cube;

use crate::render::ascii;
use crate::repl::run as run_repl;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [cmd] if cmd == "new" => {
            println!("{}", ascii(&Cube::solved()));
            Ok(())
        }
        [cmd, notation] if cmd == "apply" => {
            let mut cube = Cube::solved();
            cube.apply_notation(notation).map_err(|err| err.to_string())?;
            println!("{}", ascii(&cube));
            Ok(())
        }
        [cmd] if cmd == "repl" => {
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            run_repl(BufReader::new(stdin.lock()), &mut stdout).map_err(|err| err.to_string())
        }
        _ => Err(usage().to_string()),
    }
}

fn usage() -> &'static str {
    "usage:\n  rubiks-cli new\n  rubiks-cli apply \"<notation>\"\n  rubiks-cli repl"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_is_returned_for_invalid_args() {
        assert_eq!(try_main_with(vec![]).unwrap_err(), usage());
        assert_eq!(try_main_with(vec!["apply".into()]).unwrap_err(), usage());
    }

    fn try_main_with(args: Vec<String>) -> Result<(), &'static str> {
        match args.as_slice() {
            [cmd] if cmd == "new" => Ok(()),
            [cmd, _notation] if cmd == "apply" => Ok(()),
            [cmd] if cmd == "repl" => Ok(()),
            _ => Err(usage()),
        }
    }
}
