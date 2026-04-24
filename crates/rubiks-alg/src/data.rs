use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct RawCaseFile {
    pub case_id: String,
    pub display_name: String,
    pub pattern: RawPattern,
    pub algorithms: Vec<RawAlgorithm>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RawPattern {
    pub corners: Vec<toml::Value>,
    pub edges: Vec<toml::Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RawAlgorithm {
    pub id: String,
    pub display_name: String,
    pub notation: String,
    pub is_default: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub post_auf: Option<String>,
    pub source: RawAlgorithmSource,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RawAlgorithmSource {
    pub kind: String,
    pub name: String,
    pub url: Option<String>,
    pub license: Option<String>,
    pub retrieved_at: Option<String>,
    pub notes: Option<String>,
}
