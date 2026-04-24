use core::fmt;

use rubiks_core::CubeStateError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolveError {
    InvalidState(CubeStateError),
    Unsolvable,
    ExhaustedBudget,
    BackendUnavailable {
        solver: &'static str,
        backend: &'static str,
    },
    BackendFailure {
        solver: &'static str,
        message: String,
    },
    Unsupported {
        solver: &'static str,
        feature: &'static str,
    },
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState(err) => write!(f, "invalid cube state: {err}"),
            Self::Unsolvable => f.write_str("cube is unsolvable"),
            Self::ExhaustedBudget => f.write_str("solver exhausted available budget"),
            Self::BackendUnavailable { solver, backend } => {
                write!(f, "{solver} backend unavailable: {backend}")
            }
            Self::BackendFailure { solver, message } => {
                write!(f, "{solver} backend failure: {message}")
            }
            Self::Unsupported { solver, feature } => {
                write!(f, "{solver} does not support {feature}")
            }
        }
    }
}
