use std::io::{self, BufRead, Write};

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
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplError {
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
            Ok(ReplEvent::Exit) => {
                writeln!(stdout, "bye")?;
                break;
            }
            Err(ReplError::InvalidNotation(err)) => writeln!(stdout, "error: {err}")?,
        }
    }

    Ok(())
}

fn help_text() -> &'static str {
    "commands: reset, history, show, validate, help, exit"
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
}
