use std::path::PathBuf;

use rubiks_core::{parse_canonical_notation, CubeState};
use rubiks_solver_core::{Solution, SolveError, SolveOptions, Solver};

use crate::backend::external_process::ExternalProcessBackend;
use crate::backend::in_process::InProcessBackend;
use crate::backend::{BackendError, TwoPhaseBackend};
use crate::facelets::encode_kociemba_facelets;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KociembaConfig {
    pub max_depth: u8,
    pub backend: KociembaBackendConfig,
}

impl Default for KociembaConfig {
    fn default() -> Self {
        Self {
            max_depth: 24,
            backend: KociembaBackendConfig::InProcess,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KociembaBackendConfig {
    InProcess,
    ExternalProcess {
        program: PathBuf,
        args: Vec<String>,
    },
}

pub struct KociembaSolver {
    max_depth: u8,
    backend: Box<dyn TwoPhaseBackend>,
}

impl KociembaSolver {
    pub fn new(config: KociembaConfig) -> Self {
        let backend: Box<dyn TwoPhaseBackend> = match config.backend {
            KociembaBackendConfig::InProcess => {
                let cache_dir = default_cache_dir();
                Box::new(InProcessBackend::new(cache_dir))
            }
            KociembaBackendConfig::ExternalProcess { program, args } => {
                Box::new(ExternalProcessBackend::new(program, args))
            }
        };

        Self {
            max_depth: config.max_depth,
            backend,
        }
    }
}

fn default_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("RCE_KOCIEMBA_CACHE_DIR") {
        return PathBuf::from(dir);
    }

    std::env::temp_dir().join("rce-kociemba-cache")
}

impl Solver for KociembaSolver {
    fn id(&self) -> &'static str {
        "kociemba"
    }

    fn solve(&self, cube: &CubeState, options: &SolveOptions) -> Result<Solution, SolveError> {
        cube.validate().map_err(SolveError::InvalidState)?;

        if let Some(max_nodes) = options.max_nodes {
            if max_nodes == 0 {
                return Err(SolveError::ExhaustedBudget);
            }
        }

        let cubestring = encode_kociemba_facelets(cube);
        let raw = self
            .backend
            .solve_facelets(&cubestring, self.max_depth, options.timeout)
            .map_err(map_backend_error)?;

        let moves = parse_canonical_notation(&raw).map_err(|err| SolveError::BackendFailure {
            solver: self.id(),
            message: format!("backend returned invalid canonical notation: {err}"),
        })?;

        let mut replay = cube.clone();
        replay.apply_sequence(&moves);
        if !replay.is_solved() {
            return Err(SolveError::BackendFailure {
                solver: self.id(),
                message: "backend returned a sequence that does not solve the cube".to_string(),
            });
        }

        let solution = Solution {
            solver_name: self.id(),
            moves,
            phases: Vec::new(),
        };
        solution.ensure_phases_match_moves()?;
        Ok(solution)
    }
}

fn map_backend_error(err: BackendError) -> SolveError {
    match err {
        BackendError::Unavailable { backend, .. } => SolveError::BackendUnavailable {
            solver: "kociemba",
            backend,
        },
        BackendError::ExhaustedBudget { .. } => SolveError::ExhaustedBudget,
        BackendError::Failure { message, .. } => SolveError::BackendFailure {
            solver: "kociemba",
            message,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rubiks_core::{parse_canonical_notation, CubeState};
    use rubiks_solver_core::{SolveOptions, Solver};

    use crate::{KociembaBackendConfig, KociembaConfig, KociembaSolver};

    #[test]
    fn inprocess_solver_solves_a_simple_scramble() {
        let mut cube = CubeState::solved();
        let scramble = parse_canonical_notation("R U R' U'").unwrap();
        cube.apply_sequence(&scramble);

        let solver = KociembaSolver::new(KociembaConfig {
            max_depth: 24,
            backend: KociembaBackendConfig::InProcess,
        });
        let solution = solver.solve(&cube, &SolveOptions::default()).unwrap();

        let mut replay = cube.clone();
        replay.apply_sequence(solution.total_moves());
        assert!(replay.is_solved());
    }

    #[test]
    fn external_process_solver_can_use_a_shell_script_backend() {
        let script_dir = std::env::temp_dir().join("rce-kociemba-script-test");
        let _ = fs::create_dir_all(&script_dir);
        let script_path = script_dir.join("solver.sh");
        fs::write(&script_path, "#!/usr/bin/env sh\nprintf \"U R U' R'\\n\"\n").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let solver = KociembaSolver::new(KociembaConfig {
            max_depth: 24,
            backend: KociembaBackendConfig::ExternalProcess {
                program: script_path,
                args: Vec::new(),
            },
        });

        let mut cube = CubeState::solved();
        let scramble = parse_canonical_notation("R U R' U'").unwrap();
        cube.apply_sequence(&scramble);

        let solution = solver.solve(&cube, &SolveOptions::default()).unwrap();
        let mut replay = cube.clone();
        replay.apply_sequence(solution.total_moves());
        assert!(replay.is_solved());
    }
}
