use rubiks_core::{CubeState, Move, MoveSequence};

use crate::SolveError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SolveOptions {
    pub timeout: Option<std::time::Duration>,
    pub max_nodes: Option<u64>,
    pub diagnostics: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolvePhase {
    pub name: &'static str,
    pub moves: MoveSequence,
}

impl SolvePhase {
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution {
    pub solver_name: &'static str,
    pub moves: MoveSequence,
    pub phases: Vec<SolvePhase>,
}

impl Solution {
    pub fn total_moves(&self) -> &MoveSequence {
        &self.moves
    }

    pub fn total_len(&self) -> usize {
        self.moves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn ensure_phases_match_moves(&self) -> Result<(), SolveError> {
        if self.phases.is_empty() {
            return Ok(());
        }

        let flattened = self
            .phases
            .iter()
            .flat_map(|phase| phase.moves.0.iter().copied())
            .collect::<Vec<Move>>();

        if flattened == self.moves.0 {
            Ok(())
        } else {
            Err(SolveError::BackendFailure {
                solver: self.solver_name,
                message: "solution phases do not match flattened moves".to_string(),
            })
        }
    }
}

pub trait Solver {
    fn id(&self) -> &'static str;

    fn solve(&self, cube: &CubeState, options: &SolveOptions) -> Result<Solution, SolveError>;
}
