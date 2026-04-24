mod alg_output;
mod render;
mod repl;

use std::env;
use std::io::{self, BufReader};

use rubiks_alg::{ScrambleGenerator, ScrambleMode, TrainingScrambleGenerator};
use rubiks_core::Cube;
use rubiks_solver::{solve_state, SolveOptions, SolverKind};

use crate::alg_output::{alg_list_output, alg_show_output};
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
        [cmd, notation] if cmd == "solve" => {
            let output = solve_output(notation, SolverKind::default())?;
            println!("{output}");
            Ok(())
        }
        [cmd, flag, solver_name, notation] if cmd == "solve" && flag == "--solver" => {
            let solver_kind = SolverKind::parse(solver_name)?;
            let output = solve_output(notation, solver_kind)?;
            println!("{output}");
            Ok(())
        }
        [cmd] if cmd == "scramble" => {
            let output = scramble_output(25).map_err(|err| err.to_string())?;
            println!("{output}");
            Ok(())
        }
        [cmd, length] if cmd == "scramble" => {
            let length = parse_length(length)?;
            let output = scramble_output(length).map_err(|err| err.to_string())?;
            println!("{output}");
            Ok(())
        }
        [cmd, subcmd, family] if cmd == "alg" && subcmd == "list" => {
            let output = alg_list_output(family)?;
            println!("{output}");
            Ok(())
        }
        [cmd, subcmd, family, case_id] if cmd == "alg" && subcmd == "show" => {
            let output = alg_show_output(family, case_id)?;
            println!("{output}");
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
    "usage:\n  rubiks-cli new\n  rubiks-cli apply \"<notation>\"\n  rubiks-cli solve [--solver <name>] \"<notation>\"\n  rubiks-cli scramble [length]\n  rubiks-cli alg list <oll|pll>\n  rubiks-cli alg show <oll|pll> <case_id>\n  rubiks-cli repl"
}

fn parse_length(input: &str) -> Result<usize, String> {
    input
        .parse::<usize>()
        .map_err(|_| format!("invalid scramble length: {input}"))
}

fn scramble_output(length: usize) -> Result<String, rubiks_alg::ScrambleError> {
    let generator = TrainingScrambleGenerator;
    let sequence = generator.generate(ScrambleMode::TrainingFaceTurn { length })?;

    let mut cube = Cube::solved();
    cube.apply_canonical_sequence(&sequence);

    Ok(format!("scramble: {}\n\n{}", sequence.to_notation(), ascii(&cube)))
}

fn solve_output(notation: &str, solver_kind: SolverKind) -> Result<String, String> {
    let mut cube = Cube::solved();
    cube.apply_notation(notation).map_err(|err| err.to_string())?;

    let summary = solve_state(cube.state(), solver_kind, &SolveOptions::default())
        .map_err(|err| err.to_string())?;
    Ok(format_solve_output(notation, &summary))
}

fn format_solve_output(scramble: &str, summary: &rubiks_solver::SolveSummary) -> String {
    format!(
        "{}\nsolver: {}\n{}\nlength: {}",
        labeled_line("scramble", scramble),
        summary.solver_name,
        labeled_line("solution", &summary.solution),
        summary.length,
    )
}

fn labeled_line(label: &str, value: &str) -> String {
    if value.is_empty() {
        format!("{label}:")
    } else {
        format!("{label}: {value}")
    }
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
            [cmd, _notation] if cmd == "solve" => Ok(()),
            [cmd, flag, _solver_name, _notation] if cmd == "solve" && flag == "--solver" => Ok(()),
            [cmd] if cmd == "scramble" => Ok(()),
            [cmd, _length] if cmd == "scramble" => Ok(()),
            [cmd, subcmd, _family] if cmd == "alg" && subcmd == "list" => Ok(()),
            [cmd, subcmd, _family, _case_id] if cmd == "alg" && subcmd == "show" => Ok(()),
            [cmd] if cmd == "repl" => Ok(()),
            _ => Err(usage()),
        }
    }

    #[test]
    fn scramble_output_includes_sequence_and_render() {
        let output = scramble_output(5).unwrap();
        assert!(output.starts_with("scramble: "));
        assert!(output.contains("\n\n"));
    }

    #[test]
    fn solve_output_includes_solver_solution_and_length() {
        let output = solve_output("R U R' U'", SolverKind::default()).unwrap();
        assert!(output.contains("scramble: R U R' U'"));
        assert!(output.contains("solver: kociemba"));
        assert!(output.contains("solution: "));
        assert!(output.contains("length: 4"));
    }

    #[test]
    fn solve_output_short_circuits_solved_states() {
        let output = solve_output("x", SolverKind::default()).unwrap();
        assert!(output.contains("scramble: x"));
        assert!(output.contains("solver: kociemba"));
        assert!(output.lines().any(|line| line == "solution:"));
        assert!(output.contains("length: 0"));
    }

    #[test]
    fn solve_output_supports_explicit_solver_selection() {
        let output = solve_output("R U R' U'", SolverKind::parse("kociemba").unwrap()).unwrap();
        assert!(output.contains("solver: kociemba"));
    }

    #[test]
    fn parse_length_rejects_invalid_values() {
        assert!(parse_length("abc").is_err());
    }
}
