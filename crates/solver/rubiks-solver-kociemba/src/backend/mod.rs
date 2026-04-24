pub(crate) mod external_process;
pub(crate) mod ffi;
pub(crate) mod in_process;

use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BackendError {
    Unavailable {
        backend: &'static str,
        message: String,
    },
    ExhaustedBudget {
        backend: &'static str,
        message: String,
    },
    Failure {
        backend: &'static str,
        message: String,
    },
}

pub(crate) trait TwoPhaseBackend {
    fn id(&self) -> &'static str;

    fn solve_facelets(
        &self,
        cubestring: &str,
        max_depth: u8,
        timeout: Option<Duration>,
    ) -> Result<String, BackendError>;
}
