use std::io::{self, BufRead, Write};

use rubiks_alg::{ScrambleGenerator, ScrambleMode, TrainingScrambleGenerator};
use rubiks_core::Cube;

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
            Err(ReplError::InvalidCommand(err)) | Err(ReplError::InvalidNotation(err)) => {
                writeln!(stdout, "error: {err}")?
            }
        }
    }

    Ok(())
}

fn help_text() -> &'static str {
    "commands: reset, history, show, validate, scramble [length], help, exit"
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
}
