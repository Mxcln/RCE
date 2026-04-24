use crate::{Auf, OllPattern, PllPattern};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlgorithmSourceKind {
    Imported,
    Transcribed,
    Original,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgorithmSource {
    pub kind: AlgorithmSourceKind,
    pub name: String,
    pub url: Option<String>,
    pub license: Option<String>,
    pub retrieved_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgEntry {
    pub id: String,
    pub display_name: String,
    pub notation: String,
    pub is_default: bool,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub post_auf: Option<Auf>,
    pub source: AlgorithmSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllCase {
    pub case_id: String,
    pub display_name: String,
    pub canonical_pattern: OllPattern,
    pub algorithms: Vec<AlgEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PllCase {
    pub case_id: String,
    pub display_name: String,
    pub canonical_pattern: PllPattern,
    pub algorithms: Vec<AlgEntry>,
}
