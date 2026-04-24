use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long};

use crate::backend::BackendError;

#[repr(C)]
struct RceKociembaResult {
    status: c_int,
    solution_text: *mut c_char,
}

const RCE_KOCIEMBA_OK: c_int = 0;
const RCE_KOCIEMBA_INVALID_INPUT: c_int = 1;
const RCE_KOCIEMBA_MAX_DEPTH_EXCEEDED: c_int = 2;
const RCE_KOCIEMBA_TIMEOUT: c_int = 3;
const RCE_KOCIEMBA_INTERNAL_ERROR: c_int = 4;

unsafe extern "C" {
    fn rce_kociemba_solve(
        facelets: *const c_char,
        max_depth: c_int,
        timeout_seconds: c_long,
        cache_dir: *const c_char,
    ) -> RceKociembaResult;

    fn rce_kociemba_free_string(ptr: *mut c_char);
}

pub(crate) fn solve(
    cubestring: &str,
    max_depth: u8,
    timeout_seconds: c_long,
    cache_dir: &str,
) -> Result<String, BackendError> {
    let facelets = CString::new(cubestring).map_err(|err| BackendError::Failure {
        backend: "kociemba-inprocess-c",
        message: format!("invalid cubestring for C FFI: {err}"),
    })?;
    let cache_dir = CString::new(cache_dir).map_err(|err| BackendError::Failure {
        backend: "kociemba-inprocess-c",
        message: format!("invalid cache dir for C FFI: {err}"),
    })?;

    let result = unsafe {
        rce_kociemba_solve(
            facelets.as_ptr(),
            c_int::from(max_depth),
            timeout_seconds,
            cache_dir.as_ptr(),
        )
    };

    match result.status {
        RCE_KOCIEMBA_OK => {
            if result.solution_text.is_null() {
                return Err(BackendError::Failure {
                    backend: "kociemba-inprocess-c",
                    message: "C backend returned success without a solution string".to_string(),
                });
            }

            let text = unsafe { CStr::from_ptr(result.solution_text) }
                .to_string_lossy()
                .trim()
                .to_string();

            unsafe { rce_kociemba_free_string(result.solution_text) };
            Ok(text)
        }
        RCE_KOCIEMBA_INVALID_INPUT => Err(BackendError::Failure {
            backend: "kociemba-inprocess-c",
            message: "C backend rejected the cubestring as invalid".to_string(),
        }),
        RCE_KOCIEMBA_MAX_DEPTH_EXCEEDED => Err(BackendError::ExhaustedBudget {
            backend: "kociemba-inprocess-c",
            message: format!("no solution found within max depth {max_depth}"),
        }),
        RCE_KOCIEMBA_TIMEOUT => Err(BackendError::ExhaustedBudget {
            backend: "kociemba-inprocess-c",
            message: "C backend timed out".to_string(),
        }),
        RCE_KOCIEMBA_INTERNAL_ERROR => Err(BackendError::Failure {
            backend: "kociemba-inprocess-c",
            message: "C backend reported an internal error".to_string(),
        }),
        status => Err(BackendError::Failure {
            backend: "kociemba-inprocess-c",
            message: format!("C backend returned unknown status code {status}"),
        }),
    }
}
