use std::io::{self, BufRead, Write};

use rubiks_alg::{ScrambleGenerator, ScrambleMode, TrainingScrambleGenerator};
use rubiks_core::Cube;
use rubiks_solver::{solve_state, SolveOptions, SolverKind};

use crate::alg_output::{alg_list_output, alg_show_output};
use crate::render::ascii;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplState {
    pub cube: Cube,
    pub history: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplEvent {
    Render,
    Print(String),
    PrintAndRender(String),
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplError {
    InvalidCommand(String),
    InvalidNotation(String),
    SolveFailed(String),
}

impl ReplState {
    pub fn new() -> Self {
        Self {
            cube: Cube::solved(),
            history: Vec::new(),
        }
    }

    pub fn handle_input(&mut self, line: &str) -> Result<ReplEvent, ReplError> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(ReplEvent::Print(String::new()));
        }

        if let Some(rest) = trimmed.strip_prefix("scramble") {
            let length = parse_scramble_length(rest)?;
            let generator = TrainingScrambleGenerator;
            let sequence = generator
                .generate(ScrambleMode::TrainingFaceTurn { length })
                .map_err(|err| ReplError::InvalidCommand(err.to_string()))?;

            let scramble = sequence.to_notation();
            self.cube.reset();
            self.history.clear();
            self.cube.apply_canonical_sequence(&sequence);
            self.history.push(scramble.clone());

            return Ok(ReplEvent::PrintAndRender(format!("scramble: {scramble}")));
        }

        if trimmed == "solve" || trimmed.starts_with("solve ") {
            return handle_solve_command(self, trimmed);
        }

        if trimmed == "alg" || trimmed.starts_with("alg ") {
            return handle_alg_command(trimmed);
        }

        match trimmed {
            "exit" | "quit" => Ok(ReplEvent::Exit),
            "reset" => {
                self.cube.reset();
                self.history.clear();
                Ok(ReplEvent::Render)
            }
            "history" => {
                if self.history.is_empty() {
                    Ok(ReplEvent::Print("history is empty".to_string()))
                } else {
                    Ok(ReplEvent::Print(self.history.join("\n")))
                }
            }
            "show" => Ok(ReplEvent::Render),
            "validate" => match self.cube.validate() {
                Ok(()) => Ok(ReplEvent::Print("cube state is valid".to_string())),
                Err(err) => Ok(ReplEvent::Print(format!("invalid cube state: {err}"))),
            },
            "help" => Ok(ReplEvent::Print(help_text().to_string())),
            _ => {
                self.cube
                    .apply_notation(trimmed)
                    .map_err(|err| ReplError::InvalidNotation(err.to_string()))?;
                self.history.push(trimmed.to_string());
                Ok(ReplEvent::Render)
            }
        }
    }
}

impl Default for ReplState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run(stdin: impl BufRead, stdout: &mut impl Write) -> io::Result<()> {
    let mut state = ReplState::new();
    let mut lines = stdin.lines();

    writeln!(stdout, "{}", ascii(&state.cube))?;
    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        let Some(line) = lines.next() else {
            break;
        };
        let line = line?;

        match state.handle_input(&line) {
            Ok(ReplEvent::Render) => writeln!(stdout, "{}", ascii(&state.cube))?,
            Ok(ReplEvent::Print(text)) => {
                if !text.is_empty() {
                    writeln!(stdout, "{text}")?;
                }
            }
            Ok(ReplEvent::PrintAndRender(text)) => {
                if !text.is_empty() {
                    writeln!(stdout, "{text}")?;
                }
                writeln!(stdout, "{}", ascii(&state.cube))?;
            }
            Ok(ReplEvent::Exit) => {
                writeln!(stdout, "bye")?;
                break;
            }
            Err(ReplError::InvalidCommand(err))
            | Err(ReplError::InvalidNotation(err))
            | Err(ReplError::SolveFailed(err)) => writeln!(stdout, "error: {err}")?,
        }
    }

    Ok(())
}

fn help_text() -> &'static str {
    "commands: reset, history, show, validate, solve, scramble [length], alg list <oll|pll>, alg show <oll|pll> <case_id>, help, exit"
}

fn parse_scramble_length(rest: &str) -> Result<usize, ReplError> {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        return Ok(25);
    }

    let mut parts = trimmed.split_whitespace();
    let Some(length_text) = parts.next() else {
        return Ok(25);
    };

    if parts.next().is_some() {
        return Err(ReplError::InvalidCommand(
            "usage: scramble [length]".to_string(),
        ));
    }

    length_text.parse::<usize>().map_err(|_| {
        ReplError::InvalidCommand(format!("invalid scramble length: {length_text}"))
    })
}

fn handle_alg_command(line: &str) -> Result<ReplEvent, ReplError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.as_slice() {
        ["alg", "list", family] => alg_list_output(family)
            .map(ReplEvent::Print)
            .map_err(ReplError::InvalidCommand),
        ["alg", "show", family, case_id] => alg_show_output(family, case_id)
            .map(ReplEvent::Print)
            .map_err(ReplError::InvalidCommand),
        _ => Err(ReplError::InvalidCommand(
            "usage: alg list <oll|pll> | alg show <oll|pll> <case_id>".to_string(),
        )),
    }
}

fn handle_solve_command(state: &ReplState, line: &str) -> Result<ReplEvent, ReplError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let solver_kind = match parts.as_slice() {
        ["solve"] => SolverKind::default(),
        ["solve", solver_name] => {
            SolverKind::parse(solver_name).map_err(ReplError::InvalidCommand)?
        }
        _ => return Err(ReplError::InvalidCommand("usage: solve [solver]".to_string())),
    };

    let summary = solve_state(state.cube.state(), solver_kind, &SolveOptions::default())
        .map_err(|err| ReplError::SolveFailed(err.to_string()))?;
    Ok(ReplEvent::Print(format_repl_solve_output(&summary)))
}

fn format_repl_solve_output(summary: &rubiks_solver::SolveSummary) -> String {
    format!(
        "solver: {}\n{}\nlength: {}",
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
    fn notation_is_applied_and_recorded() {
        let mut state = ReplState::new();
        assert_eq!(state.handle_input("R U").unwrap(), ReplEvent::Render);
        assert_eq!(state.history, vec!["R U".to_string()]);
    }

    #[test]
    fn reset_clears_history() {
        let mut state = ReplState::new();
        state.handle_input("R").unwrap();
        assert_eq!(state.handle_input("reset").unwrap(), ReplEvent::Render);
        assert!(state.history.is_empty());
        assert!(state.cube.is_solved());
    }

    #[test]
    fn history_reports_empty_state() {
        let mut state = ReplState::new();
        assert_eq!(
            state.handle_input("history").unwrap(),
            ReplEvent::Print("history is empty".to_string())
        );
    }

    #[test]
    fn invalid_notation_returns_error() {
        let mut state = ReplState::new();
        let err = state.handle_input("?").unwrap_err();
        assert!(matches!(err, ReplError::InvalidNotation(_)));
    }

    #[test]
    fn scramble_resets_cube_and_records_generated_sequence() {
        let mut state = ReplState::new();
        state.handle_input("R U").unwrap();

        let event = state.handle_input("scramble 1").unwrap();
        assert!(matches!(event, ReplEvent::PrintAndRender(_)));
        assert_eq!(state.history.len(), 1);
        assert!(!state.cube.is_solved());
    }

    #[test]
    fn invalid_scramble_length_returns_command_error() {
        let mut state = ReplState::new();
        let err = state.handle_input("scramble nope").unwrap_err();
        assert_eq!(
            err,
            ReplError::InvalidCommand("invalid scramble length: nope".to_string())
        );
    }

    #[test]
    fn alg_list_command_returns_catalog_output() {
        let mut state = ReplState::new();
        let event = state.handle_input("alg list pll").unwrap();
        match event {
            ReplEvent::Print(text) => {
                assert!(text.starts_with("CASE"));
                assert!(text.contains("Aa"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn alg_show_command_returns_case_details() {
        let mut state = ReplState::new();
        let event = state.handle_input("alg show oll 3").unwrap();
        match event {
            ReplEvent::Print(text) => assert!(text.contains("OLL Case OLL03")),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn invalid_alg_command_returns_command_error() {
        let mut state = ReplState::new();
        let err = state.handle_input("alg show oll").unwrap_err();
        assert_eq!(
            err,
            ReplError::InvalidCommand(
                "usage: alg list <oll|pll> | alg show <oll|pll> <case_id>".to_string()
            )
        );
    }

    #[test]
    fn solve_returns_solution_without_mutating_state() {
        let mut state = ReplState::new();
        state.handle_input("R U R' U'").unwrap();

        let cube_before = state.cube.clone();
        let history_before = state.history.clone();

        let event = state.handle_input("solve").unwrap();
        match event {
            ReplEvent::Print(text) => {
                assert!(text.contains("solver: kociemba"));
                assert!(text.contains("solution: "));
                assert!(text.contains("length: 4"));
            }
            other => panic!("unexpected event: {other:?}"),
        }

        assert_eq!(state.cube, cube_before);
        assert_eq!(state.history, history_before);
        assert!(!state.cube.is_solved());
    }

    #[test]
    fn solve_on_solved_state_returns_empty_solution() {
        let mut state = ReplState::new();
        let event = state.handle_input("solve").unwrap();

        match event {
            ReplEvent::Print(text) => {
                assert!(text.contains("solver: kociemba"));
                assert!(text.lines().any(|line| line == "solution:"));
                assert!(text.contains("length: 0"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn solve_rejects_extra_arguments() {
        let mut state = ReplState::new();
        let err = state.handle_input("solve now").unwrap_err();
        assert_eq!(
            err,
            ReplError::InvalidCommand("unknown solver: now (available: kociemba)".to_string())
        );
    }

    #[test]
    fn solve_accepts_explicit_solver_name() {
        let mut state = ReplState::new();
        state.handle_input("R U R' U'").unwrap();

        let event = state.handle_input("solve kociemba").unwrap();
        match event {
            ReplEvent::Print(text) => assert!(text.contains("solver: kociemba")),
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn solve_reports_unimplemented_solver_name() {
        let mut state = ReplState::new();
        let err = state.handle_input("solve cfop").unwrap_err();
        assert_eq!(
            err,
            ReplError::InvalidCommand("solver not yet implemented: cfop".to_string())
        );
    }
}
