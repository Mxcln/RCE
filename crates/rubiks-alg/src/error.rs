use core::fmt;

use crate::catalog::LookupKind;
use crate::ScrambleMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LookupError {
    PrerequisiteNotMet {
        lookup: LookupKind,
        requirement: &'static str,
    },
    AmbiguousCase {
        lookup: LookupKind,
        pattern_debug: String,
    },
    CatalogInvariant {
        message: String,
    },
}

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrerequisiteNotMet { lookup, requirement } => {
                write!(f, "{lookup} prerequisite not met: {requirement}")
            }
            Self::AmbiguousCase {
                lookup,
                pattern_debug,
            } => write!(f, "{lookup} ambiguous case for pattern {pattern_debug}"),
            Self::CatalogInvariant { message } => write!(f, "catalog invariant violation: {message}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    Parse {
        path: String,
        message: String,
    },
    DuplicateCaseId {
        case_id: String,
    },
    DuplicateAlgorithmId {
        algorithm_id: String,
    },
    MissingDefaultAlgorithm {
        case_id: String,
    },
    MultipleDefaultAlgorithms {
        case_id: String,
    },
    InvalidPattern {
        case_id: String,
        detail: String,
    },
    InvalidNotation {
        algorithm_id: String,
        notation: String,
        detail: String,
    },
    PatternCollision {
        family: LookupKind,
        pattern_debug: String,
        existing_case_id: String,
        duplicate_case_id: String,
    },
    Io {
        path: String,
        message: String,
    },
    EmbeddedUnavailable,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { path, message } => write!(f, "failed to parse {path}: {message}"),
            Self::DuplicateCaseId { case_id } => write!(f, "duplicate case id: {case_id}"),
            Self::DuplicateAlgorithmId { algorithm_id } => {
                write!(f, "duplicate algorithm id: {algorithm_id}")
            }
            Self::MissingDefaultAlgorithm { case_id } => {
                write!(f, "missing default algorithm in case {case_id}")
            }
            Self::MultipleDefaultAlgorithms { case_id } => {
                write!(f, "multiple default algorithms in case {case_id}")
            }
            Self::InvalidPattern { case_id, detail } => {
                write!(f, "invalid pattern in case {case_id}: {detail}")
            }
            Self::InvalidNotation {
                algorithm_id,
                notation,
                detail,
            } => write!(
                f,
                "invalid notation in algorithm {algorithm_id} ({notation}): {detail}"
            ),
            Self::PatternCollision {
                family,
                pattern_debug,
                existing_case_id,
                duplicate_case_id,
            } => write!(
                f,
                "{family} pattern collision for {pattern_debug}: {existing_case_id} vs {duplicate_case_id}"
            ),
            Self::Io { path, message } => write!(f, "I/O error reading {path}: {message}"),
            Self::EmbeddedUnavailable => f.write_str("embedded catalog data is unavailable"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrambleError {
    UnsupportedMode {
        mode: ScrambleMode,
    },
    InvalidLength {
        length: usize,
    },
}

impl fmt::Display for ScrambleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode { mode } => write!(f, "unsupported scramble mode: {mode:?}"),
            Self::InvalidLength { length } => write!(f, "invalid scramble length: {length}"),
        }
    }
}
