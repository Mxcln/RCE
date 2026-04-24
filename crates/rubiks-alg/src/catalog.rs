use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use include_dir::{include_dir, Dir};
use rubiks_core::{parse_notation, resolve_notation, CubeState, CubeStateParts, Orientation};

use crate::data::{RawAlgorithm, RawCaseFile, RawPattern};
use crate::error::{LoadError, LookupError};
use crate::pattern::{is_f2l_solved, is_oll_solved, OllPattern, PllPattern};
use crate::types::{
    AlgEntry, AlgorithmSource, AlgorithmSourceKind, OllCase, PllCase,
};
use crate::Auf;

static EMBEDDED_DATA: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/data");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LookupKind {
    Oll,
    Pll,
}

impl core::fmt::Display for LookupKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Oll => f.write_str("OLL"),
            Self::Pll => f.write_str("PLL"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CaseMatch<'a, T> {
    pub case: &'a T,
    pub auf: Auf,
}

pub trait AlgCatalog {
    fn lookup_oll(&self, cube: &CubeState) -> Result<Option<CaseMatch<'_, OllCase>>, LookupError>;
    fn lookup_pll(&self, cube: &CubeState) -> Result<Option<CaseMatch<'_, PllCase>>, LookupError>;
}

#[derive(Clone, Debug, Default)]
pub struct Catalog {
    oll_cases: Vec<OllCase>,
    pll_cases: Vec<PllCase>,
    oll_index: HashMap<OllPattern, (usize, Auf)>,
    pll_index: HashMap<PllPattern, (usize, Auf)>,
}

impl Catalog {
    pub fn embedded() -> Result<Self, LoadError> {
        let oll_files = read_embedded_case_files("oll")?;
        let pll_files = read_embedded_case_files("pll")?;
        build_catalog_from_raw(oll_files, pll_files)
    }

    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(LoadError::EmbeddedUnavailable);
        }

        let oll_root = path.join("oll");
        let pll_root = path.join("pll");

        let oll_files = read_case_files(&oll_root)?;
        let pll_files = read_case_files(&pll_root)?;

        build_catalog_from_raw(oll_files, pll_files)
    }

    pub fn oll_cases(&self) -> &[OllCase] {
        &self.oll_cases
    }

    pub fn pll_cases(&self) -> &[PllCase] {
        &self.pll_cases
    }

    pub fn get_oll_case(&self, case_id: &str) -> Option<&OllCase> {
        self.oll_cases.iter().find(|case| case.case_id == case_id)
    }

    pub fn get_pll_case(&self, case_id: &str) -> Option<&PllCase> {
        self.pll_cases.iter().find(|case| case.case_id == case_id)
    }
}

fn build_catalog_from_raw(
    oll_files: Vec<RawCaseFile>,
    pll_files: Vec<RawCaseFile>,
) -> Result<Catalog, LoadError> {
    let mut algorithm_ids = HashSet::new();
    let mut case_ids = HashSet::new();

    let mut oll_cases = Vec::with_capacity(oll_files.len());
    let mut pll_cases = Vec::with_capacity(pll_files.len());

    for raw in oll_files {
        if !case_ids.insert(raw.case_id.clone()) {
            return Err(LoadError::DuplicateCaseId {
                case_id: raw.case_id,
            });
        }
        oll_cases.push(build_oll_case(raw, &mut algorithm_ids)?);
    }

    for raw in pll_files {
        if !case_ids.insert(raw.case_id.clone()) {
            return Err(LoadError::DuplicateCaseId {
                case_id: raw.case_id,
            });
        }
        pll_cases.push(build_pll_case(raw, &mut algorithm_ids)?);
    }

    let oll_index = build_oll_index(&oll_cases)?;
    let pll_index = build_pll_index(&pll_cases)?;

    Ok(Catalog {
        oll_cases,
        pll_cases,
        oll_index,
        pll_index,
    })
}

impl AlgCatalog for Catalog {
    fn lookup_oll(&self, cube: &CubeState) -> Result<Option<CaseMatch<'_, OllCase>>, LookupError> {
        let parts = cube.parts();
        if !is_f2l_solved(&parts) {
            return Err(LookupError::PrerequisiteNotMet {
                lookup: LookupKind::Oll,
                requirement: "F2L solved",
            });
        }

        if is_oll_solved(&parts) {
            return Ok(None);
        }

        let pattern = OllPattern::from_parts(&parts);
        let (index, auf) = self.oll_index.get(&pattern).ok_or_else(|| LookupError::CatalogInvariant {
            message: format!("missing OLL pattern {pattern}"),
        })?;
        Ok(Some(CaseMatch {
            case: &self.oll_cases[*index],
            auf: *auf,
        }))
    }

    fn lookup_pll(&self, cube: &CubeState) -> Result<Option<CaseMatch<'_, PllCase>>, LookupError> {
        let parts = cube.parts();
        if !is_oll_solved(&parts) {
            return Err(LookupError::PrerequisiteNotMet {
                lookup: LookupKind::Pll,
                requirement: "OLL solved",
            });
        }

        if is_pll_skip(&parts) {
            return Ok(None);
        }

        let pattern = PllPattern::from_parts(&parts);
        let (index, auf) = self.pll_index.get(&pattern).ok_or_else(|| LookupError::CatalogInvariant {
            message: format!("missing PLL pattern {pattern}"),
        })?;
        Ok(Some(CaseMatch {
            case: &self.pll_cases[*index],
            auf: *auf,
        }))
    }
}

fn read_case_files(root: &Path) -> Result<Vec<RawCaseFile>, LoadError> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = fs::read_dir(root)
        .map_err(|err| LoadError::Io {
            path: root.display().to_string(),
            message: err.to_string(),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| LoadError::Io {
            path: root.display().to_string(),
            message: err.to_string(),
        })?;

    files.sort_by_key(|entry| entry.path());

    let mut result = Vec::with_capacity(files.len());
    for entry in files {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|err| LoadError::Io {
            path: path.display().to_string(),
            message: err.to_string(),
        })?;

        let parsed: RawCaseFile = toml::from_str(&content).map_err(|err| LoadError::Parse {
            path: path.display().to_string(),
            message: err.to_string(),
        })?;

        let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or_default();
        if stem != parsed.case_id {
            return Err(LoadError::Parse {
                path: path.display().to_string(),
                message: format!("file stem {stem} does not match case_id {}", parsed.case_id),
            });
        }

        result.push(parsed);
    }

    Ok(result)
}

fn read_embedded_case_files(subdir: &str) -> Result<Vec<RawCaseFile>, LoadError> {
    let Some(dir) = EMBEDDED_DATA.get_dir(subdir) else {
        return Ok(Vec::new());
    };

    let mut files = dir
        .files()
        .filter(|file| file.path().extension().and_then(|ext| ext.to_str()) == Some("toml"))
        .collect::<Vec<_>>();
    files.sort_by_key(|file| file.path().to_string_lossy().to_string());

    let mut result = Vec::with_capacity(files.len());
    for file in files {
        let path = file.path().display().to_string();
        let content = file.contents_utf8().ok_or_else(|| LoadError::Parse {
            path: path.clone(),
            message: "embedded file is not valid UTF-8".to_string(),
        })?;
        let parsed: RawCaseFile = toml::from_str(content).map_err(|err| LoadError::Parse {
            path: path.clone(),
            message: err.to_string(),
        })?;
        let stem = file
            .path()
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if stem != parsed.case_id {
            return Err(LoadError::Parse {
                path,
                message: format!("file stem {stem} does not match case_id {}", parsed.case_id),
            });
        }
        result.push(parsed);
    }

    Ok(result)
}

fn build_oll_case(
    raw: RawCaseFile,
    algorithm_ids: &mut HashSet<String>,
) -> Result<OllCase, LoadError> {
    validate_default_algorithm(&raw)?;
    let canonical_pattern = parse_oll_pattern(&raw.case_id, &raw.pattern)?;
    let algorithms = raw
        .algorithms
        .into_iter()
        .map(|alg| build_alg_entry(&raw.case_id, alg, algorithm_ids))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(OllCase {
        case_id: raw.case_id,
        display_name: raw.display_name,
        canonical_pattern,
        algorithms,
    })
}

fn build_pll_case(
    raw: RawCaseFile,
    algorithm_ids: &mut HashSet<String>,
) -> Result<PllCase, LoadError> {
    validate_default_algorithm(&raw)?;
    let canonical_pattern = parse_pll_pattern(&raw.case_id, &raw.pattern)?;
    let algorithms = raw
        .algorithms
        .into_iter()
        .map(|alg| build_alg_entry(&raw.case_id, alg, algorithm_ids))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PllCase {
        case_id: raw.case_id,
        display_name: raw.display_name,
        canonical_pattern,
        algorithms,
    })
}

fn validate_default_algorithm(raw: &RawCaseFile) -> Result<(), LoadError> {
    let defaults = raw.algorithms.iter().filter(|alg| alg.is_default).count();
    match defaults {
        0 => Err(LoadError::MissingDefaultAlgorithm {
            case_id: raw.case_id.clone(),
        }),
        1 => Ok(()),
        _ => Err(LoadError::MultipleDefaultAlgorithms {
            case_id: raw.case_id.clone(),
        }),
    }
}

fn build_alg_entry(
    _case_id: &str,
    raw: RawAlgorithm,
    algorithm_ids: &mut HashSet<String>,
) -> Result<AlgEntry, LoadError> {
    if !algorithm_ids.insert(raw.id.clone()) {
        return Err(LoadError::DuplicateAlgorithmId {
            algorithm_id: raw.id,
        });
    }

    let source = AlgorithmSource {
        kind: parse_source_kind(&raw.source.kind).ok_or_else(|| LoadError::Parse {
            path: raw.id.clone(),
            message: format!("invalid source kind {}", raw.source.kind),
        })?,
        name: raw.source.name,
        url: raw.source.url,
        license: raw.source.license,
        retrieved_at: raw.source.retrieved_at,
        notes: raw.source.notes,
    };

    let parsed = parse_notation(&raw.notation).map_err(|err| LoadError::InvalidNotation {
        algorithm_id: raw.id.clone(),
        notation: raw.notation.clone(),
        detail: err.to_string(),
    })?;
    let _resolved = resolve_notation(&parsed, Orientation::SOLVED);

    let post_auf = match raw.post_auf {
        Some(text) => Some(parse_auf_text(&text).ok_or_else(|| LoadError::Parse {
            path: raw.id.clone(),
            message: format!("invalid post_auf {text}"),
        })?),
        None => None,
    };

    Ok(AlgEntry {
        id: raw.id,
        display_name: raw.display_name,
        notation: raw.notation,
        is_default: raw.is_default,
        tags: raw.tags,
        notes: raw.notes,
        post_auf,
        source,
    })
}

fn parse_source_kind(input: &str) -> Option<AlgorithmSourceKind> {
    match input {
        "imported" => Some(AlgorithmSourceKind::Imported),
        "transcribed" => Some(AlgorithmSourceKind::Transcribed),
        "original" => Some(AlgorithmSourceKind::Original),
        _ => None,
    }
}

fn parse_auf_text(input: &str) -> Option<Auf> {
    match input {
        "I" => Some(Auf::IDENTITY),
        "U" => Some(Auf::U),
        "U2" => Some(Auf::U2),
        "U'" => Some(Auf::U_PRIME),
        _ => None,
    }
}

fn parse_oll_pattern(case_id: &str, pattern: &RawPattern) -> Result<OllPattern, LoadError> {
    if pattern.corners.len() != 4 || pattern.edges.len() != 4 {
        return Err(LoadError::InvalidPattern {
            case_id: case_id.to_string(),
            detail: "OLL pattern must contain 4 corners and 4 edges".to_string(),
        });
    }

    let mut corners = [0u8; 4];
    let mut edges = [0u8; 4];

    for (index, value) in pattern.corners.iter().enumerate() {
        corners[index] = value
            .as_integer()
            .and_then(|v| u8::try_from(v).ok())
            .filter(|&v| v < 3)
            .ok_or_else(|| LoadError::InvalidPattern {
                case_id: case_id.to_string(),
                detail: "OLL corner orientations must be integers in 0..=2".to_string(),
            })?;
    }

    for (index, value) in pattern.edges.iter().enumerate() {
        edges[index] = value
            .as_integer()
            .and_then(|v| u8::try_from(v).ok())
            .filter(|&v| v < 2)
            .ok_or_else(|| LoadError::InvalidPattern {
                case_id: case_id.to_string(),
                detail: "OLL edge orientations must be integers in 0..=1".to_string(),
            })?;
    }

    Ok(OllPattern::from_components(corners, edges))
}

fn parse_pll_pattern(case_id: &str, pattern: &RawPattern) -> Result<PllPattern, LoadError> {
    if pattern.corners.len() != 4 || pattern.edges.len() != 4 {
        return Err(LoadError::InvalidPattern {
            case_id: case_id.to_string(),
            detail: "PLL pattern must contain 4 corners and 4 edges".to_string(),
        });
    }

    let mut corners = [0u8; 4];
    let mut edges = [0u8; 4];

    for (index, value) in pattern.corners.iter().enumerate() {
        corners[index] = parse_corner_name(value.as_str()).ok_or_else(|| LoadError::InvalidPattern {
            case_id: case_id.to_string(),
            detail: format!("invalid PLL corner entry at index {index}"),
        })?;
    }

    for (index, value) in pattern.edges.iter().enumerate() {
        edges[index] = parse_edge_name(value.as_str()).ok_or_else(|| LoadError::InvalidPattern {
            case_id: case_id.to_string(),
            detail: format!("invalid PLL edge entry at index {index}"),
        })?;
    }

    if !is_u_layer_permutation(&corners) || !is_u_layer_permutation(&edges) {
        return Err(LoadError::InvalidPattern {
            case_id: case_id.to_string(),
            detail: "PLL pattern must be a permutation of the U-layer cubies".to_string(),
        });
    }

    Ok(PllPattern::from_components(corners, edges))
}

fn build_oll_index(cases: &[OllCase]) -> Result<HashMap<OllPattern, (usize, Auf)>, LoadError> {
    let mut index: HashMap<OllPattern, (usize, Auf)> = HashMap::new();

    for (case_index, case) in cases.iter().enumerate() {
        for auf in Auf::ALL {
            let observed = case.canonical_pattern.rotate_u(auf);
            let returned = auf.inverse();
            if let Some((existing_index, _)) = index.get(&observed) {
                if *existing_index == case_index {
                    continue;
                }
                return Err(LoadError::PatternCollision {
                    family: LookupKind::Oll,
                    pattern_debug: observed.to_string(),
                    existing_case_id: cases[*existing_index].case_id.clone(),
                    duplicate_case_id: case.case_id.clone(),
                });
            }
            index.insert(observed, (case_index, returned));
        }
    }

    Ok(index)
}

fn build_pll_index(cases: &[PllCase]) -> Result<HashMap<PllPattern, (usize, Auf)>, LoadError> {
    let mut index: HashMap<PllPattern, (usize, Auf)> = HashMap::new();

    for (case_index, case) in cases.iter().enumerate() {
        for auf in Auf::ALL {
            let observed = case.canonical_pattern.rotate_u(auf);
            let returned = auf.inverse();
            if let Some((existing_index, _)) = index.get(&observed) {
                if *existing_index == case_index {
                    continue;
                }
                return Err(LoadError::PatternCollision {
                    family: LookupKind::Pll,
                    pattern_debug: observed.to_string(),
                    existing_case_id: cases[*existing_index].case_id.clone(),
                    duplicate_case_id: case.case_id.clone(),
                });
            }
            index.insert(observed, (case_index, returned));
        }
    }

    Ok(index)
}

fn parse_corner_name(name: Option<&str>) -> Option<u8> {
    match name? {
        "URF" => Some(0),
        "UFL" => Some(1),
        "ULB" => Some(2),
        "UBR" => Some(3),
        _ => None,
    }
}

fn parse_edge_name(name: Option<&str>) -> Option<u8> {
    match name? {
        "UR" => Some(0),
        "UF" => Some(1),
        "UL" => Some(2),
        "UB" => Some(3),
        _ => None,
    }
}

fn is_u_layer_permutation(values: &[u8; 4]) -> bool {
    let mut seen = [false; 4];
    for &value in values {
        let index = value as usize;
        if index >= 4 || seen[index] {
            return false;
        }
        seen[index] = true;
    }
    true
}

fn is_pll_skip(parts: &CubeStateParts) -> bool {
    parts.corner_perm[..4]
        .iter()
        .enumerate()
        .all(|(index, &value)| value == index as u8)
        && parts.edge_perm[..4]
            .iter()
            .enumerate()
            .all(|(index, &value)| value == index as u8)
}

#[allow(dead_code)]
fn _parts_debug(parts: &CubeStateParts) -> String {
    format!(
        "cp={:?} co={:?} ep={:?} eo={:?}",
        parts.corner_perm, parts.corner_orient, parts.edge_perm, parts.edge_orient
    )
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use rubiks_core::CubeState;

    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rubiks_alg_test_{nanos}"))
    }

    #[test]
    fn empty_catalog_loads() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("oll")).unwrap();
        fs::create_dir_all(dir.join("pll")).unwrap();
        let catalog = Catalog::from_dir(&dir).unwrap();
        assert!(catalog.oll_cases.is_empty());
        assert!(catalog.pll_cases.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lookup_reports_missing_patterns_as_catalog_invariant() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("oll")).unwrap();
        fs::create_dir_all(dir.join("pll")).unwrap();
        let catalog = Catalog::from_dir(&dir).unwrap();
        let mut parts = CubeState::solved().parts();
        parts.corner_orient = [0, 1, 1, 1, 0, 0, 0, 0];
        parts.edge_orient = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let cube = CubeState::try_from_parts(parts).unwrap();
        let result = catalog.lookup_oll(&cube);
        assert!(matches!(result, Err(LookupError::CatalogInvariant { .. })));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn prerequisite_checks_are_enforced() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join("oll")).unwrap();
        fs::create_dir_all(dir.join("pll")).unwrap();
        let catalog = Catalog::from_dir(&dir).unwrap();
        let mut parts = CubeState::solved().parts();
        parts.corner_perm.swap(0, 4);
        parts.edge_perm.swap(0, 8);
        let cube = CubeState::try_from_parts(parts).unwrap();
        assert!(matches!(
            catalog.lookup_oll(&cube),
            Err(LookupError::PrerequisiteNotMet {
                lookup: LookupKind::Oll,
                ..
            })
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn embedded_catalog_loads_minimal_data() {
        let catalog = Catalog::embedded().unwrap();
        assert_eq!(catalog.oll_cases.len(), 57);
        assert_eq!(catalog.pll_cases.len(), 21);
        assert!(catalog.lookup_oll(&CubeState::solved()).unwrap().is_none());
        assert!(catalog.lookup_pll(&CubeState::solved()).unwrap().is_none());
        assert!(catalog.get_oll_case("OLL03").is_some());
        assert!(catalog.get_pll_case("Aa").is_some());
    }
}
