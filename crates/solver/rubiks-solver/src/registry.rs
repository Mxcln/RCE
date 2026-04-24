use rubiks_core::CubeState;
use rubiks_solver_core::{SolveError, SolveOptions, Solver};
use rubiks_solver_kociemba::{KociembaConfig, KociembaSolver};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverKind {
    Kociemba,
}

impl SolverKind {
    pub fn parse(input: &str) -> Result<Self, String> {
        match input.to_ascii_lowercase().as_str() {
            "kociemba" => Ok(Self::Kociemba),
            "cfop" => Err("solver not yet implemented: cfop".to_string()),
            _ => Err(format!(
                "unknown solver: {input} (available: {})",
                Self::available()
            )),
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Kociemba => "kociemba",
        }
    }

    pub fn available() -> &'static str {
        "kociemba"
    }

    fn solve(self, state: &CubeState, options: &SolveOptions) -> Result<SolveSummary, SolveError> {
        match self {
            Self::Kociemba => {
                let solution = KociembaSolver::new(KociembaConfig::default()).solve(state, options)?;
                Ok(SolveSummary {
                    solver_name: solution.solver_name,
                    solution: solution.moves.to_notation(),
                    length: solution.total_len(),
                })
            }
        }
    }
}

impl Default for SolverKind {
    fn default() -> Self {
        Self::Kociemba
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolveSummary {
    pub solver_name: &'static str,
    pub solution: String,
    pub length: usize,
}

pub fn solve_state_with_default(state: &CubeState) -> Result<SolveSummary, SolveError> {
    solve_state(state, SolverKind::default(), &SolveOptions::default())
}

pub fn solve_state(
    state: &CubeState,
    solver_kind: SolverKind,
    options: &SolveOptions,
) -> Result<SolveSummary, SolveError> {
    if state.is_solved() {
        return Ok(SolveSummary {
            solver_name: solver_kind.id(),
            solution: String::new(),
            length: 0,
        });
    }

    solver_kind.solve(state, options)
}

#[cfg(test)]
mod tests {
    use rubiks_core::{parse_canonical_notation, CubeState};

    use super::*;

    #[test]
    fn parse_accepts_kociemba_case_insensitively() {
        assert_eq!(SolverKind::parse("kociemba").unwrap(), SolverKind::Kociemba);
        assert_eq!(SolverKind::parse("KOCIEMBA").unwrap(), SolverKind::Kociemba);
    }

    #[test]
    fn parse_reports_unimplemented_cfop() {
        assert_eq!(
            SolverKind::parse("cfop").unwrap_err(),
            "solver not yet implemented: cfop"
        );
    }

    #[test]
    fn solve_state_short_circuits_solved_state() {
        let summary = solve_state_with_default(&CubeState::solved()).unwrap();
        assert_eq!(summary.solver_name, "kociemba");
        assert!(summary.solution.is_empty());
        assert_eq!(summary.length, 0);
    }

    #[test]
    fn solve_state_uses_selected_solver() {
        let mut cube = CubeState::solved();
        let scramble = parse_canonical_notation("R U R' U'").unwrap();
        cube.apply_sequence(&scramble);

        let summary = solve_state(&cube, SolverKind::Kociemba, &SolveOptions::default()).unwrap();
        assert_eq!(summary.solver_name, "kociemba");
        assert_eq!(summary.length, 4);
    }
}
