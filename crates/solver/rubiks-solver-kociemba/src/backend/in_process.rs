use std::path::PathBuf;
use std::time::Duration;

use super::ffi::kociemba_c;
use super::{BackendError, TwoPhaseBackend};

#[derive(Clone, Debug)]
pub(crate) struct InProcessBackend {
    cache_dir: PathBuf,
}

impl InProcessBackend {
    pub(crate) fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }
}

impl TwoPhaseBackend for InProcessBackend {
    fn id(&self) -> &'static str {
        "kociemba-inprocess-c"
    }

    fn solve_facelets(
        &self,
        cubestring: &str,
        max_depth: u8,
        timeout: Option<Duration>,
    ) -> Result<String, BackendError> {
        let timeout_seconds = timeout
            .map(|duration| duration.as_secs().max(1))
            .unwrap_or(1_000);
        let cache_dir = self.cache_dir.to_string_lossy().to_string();

        kociemba_c::solve(cubestring, max_depth, timeout_seconds as _, &cache_dir)
    }
}
