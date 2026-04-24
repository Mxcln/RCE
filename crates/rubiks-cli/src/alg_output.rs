use rubiks_alg::{AlgEntry, Catalog, OllCase, PllCase};

pub fn alg_list_output(family: &str) -> Result<String, String> {
    let catalog = load_catalog()?;
    match normalize_family(family)? {
        Family::Oll => Ok(format_oll_list(catalog.oll_cases())),
        Family::Pll => Ok(format_pll_list(catalog.pll_cases())),
    }
}

pub fn alg_show_output(family: &str, case_id: &str) -> Result<String, String> {
    let catalog = load_catalog()?;
    match normalize_family(family)? {
        Family::Oll => {
            let normalized = normalize_oll_case_id(case_id)?;
            let case = catalog
                .get_oll_case(&normalized)
                .ok_or_else(|| format!("unknown OLL case: {normalized}"))?;
            Ok(format_oll_case(case))
        }
        Family::Pll => {
            let normalized = normalize_pll_case_id(case_id);
            let case = catalog
                .get_pll_case(&normalized)
                .ok_or_else(|| format!("unknown PLL case: {normalized}"))?;
            Ok(format_pll_case(case))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    Oll,
    Pll,
}

fn load_catalog() -> Result<Catalog, String> {
    Catalog::embedded().map_err(|err| err.to_string())
}

fn normalize_family(input: &str) -> Result<Family, String> {
    match input.to_ascii_lowercase().as_str() {
        "oll" => Ok(Family::Oll),
        "pll" => Ok(Family::Pll),
        other => Err(format!("unknown algorithm family: {other}")),
    }
}

fn normalize_oll_case_id(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    let digits = trimmed
        .strip_prefix("OLL")
        .or_else(|| trimmed.strip_prefix("oll"))
        .unwrap_or(trimmed);
    let number = digits
        .parse::<usize>()
        .map_err(|_| format!("invalid OLL case id: {input}"))?;
    if !(1..=57).contains(&number) {
        return Err(format!("invalid OLL case id: {input}"));
    }
    Ok(format!("OLL{number:02}"))
}

fn normalize_pll_case_id(input: &str) -> String {
    let trimmed = input.trim();
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "aa" => "Aa".to_string(),
        "ab" => "Ab".to_string(),
        "ga" => "Ga".to_string(),
        "gb" => "Gb".to_string(),
        "gc" => "Gc".to_string(),
        "gd" => "Gd".to_string(),
        "ja" => "Ja".to_string(),
        "jb" => "Jb".to_string(),
        "na" => "Na".to_string(),
        "nb" => "Nb".to_string(),
        "ra" => "Ra".to_string(),
        "rb" => "Rb".to_string(),
        "ua" => "Ua".to_string(),
        "ub" => "Ub".to_string(),
        _ if trimmed.len() == 1 => trimmed.to_ascii_uppercase(),
        _ => trimmed.to_string(),
    }
}

fn format_oll_list(cases: &[OllCase]) -> String {
    let case_width = cases
        .iter()
        .map(|case| case.case_id.len())
        .max()
        .unwrap_or(0)
        .max("CASE".len());

    let mut lines = vec![format!("{:<case_width$}  ALGORITHM", "CASE")];
    lines.extend(cases.iter().map(|case| {
        let default = default_alg(&case.algorithms);
        format!("{:<case_width$}  {}", case.case_id, default.notation)
    }));
    lines.join("\n")
}

fn format_pll_list(cases: &[PllCase]) -> String {
    let case_width = cases
        .iter()
        .map(|case| case.case_id.len())
        .max()
        .unwrap_or(0)
        .max("CASE".len());
    let alg_width = cases
        .iter()
        .map(|case| default_alg(&case.algorithms).notation.len())
        .max()
        .unwrap_or(0)
        .max("ALGORITHM".len());
    let post_width = cases
        .iter()
        .map(|case| {
            default_alg(&case.algorithms)
                .post_auf
                .map(|auf| auf.to_notation().len())
                .unwrap_or(1)
        })
        .max()
        .unwrap_or(0)
        .max("POST".len());

    let mut lines = vec![format!(
        "{:<case_width$}  {:<alg_width$}  {:<post_width$}",
        "CASE", "ALGORITHM", "POST"
    )];
    lines.extend(cases.iter().map(|case| {
        let default = default_alg(&case.algorithms);
        let post_auf = default
            .post_auf
            .map(|auf| auf.to_notation())
            .unwrap_or("-");
        format!(
            "{:<case_width$}  {:<alg_width$}  {:<post_width$}",
            case.case_id, default.notation, post_auf
        )
    }));
    lines.join("\n")
}

fn format_oll_case(case: &OllCase) -> String {
    let mut lines = vec![format!("OLL Case {}", case.case_id)];
    if case.display_name != case.case_id {
        lines.push(format!("Name: {}", case.display_name));
    }
    lines.push(String::new());
    lines.push(String::from("Algorithms"));
    lines.extend(format_alg_lines(&case.algorithms));
    lines.join("\n")
}

fn format_pll_case(case: &PllCase) -> String {
    let mut lines = vec![format!("PLL Case {}", case.case_id)];
    if case.display_name != case.case_id {
        lines.push(format!("Name: {}", case.display_name));
    }
    lines.push(String::new());
    lines.push(String::from("Algorithms"));
    lines.extend(format_alg_lines(&case.algorithms));
    lines.join("\n")
}

fn format_alg_lines(algs: &[AlgEntry]) -> Vec<String> {
    algs.iter()
        .enumerate()
        .map(|(index, alg)| format!("{}. {}", index + 1, format_alg_line(alg)))
        .collect()
}

fn format_alg_line(alg: &AlgEntry) -> String {
    let mut annotations = Vec::new();
    if alg.is_default {
        annotations.push(String::from("default"));
    }
    if let Some(post_auf) = alg.post_auf {
        annotations.push(format!("post_auf={}", post_auf.to_notation()));
    }

    if annotations.is_empty() {
        alg.notation.clone()
    } else {
        format!("{} [{}]", alg.notation, annotations.join(", "))
    }
}

fn default_alg(algs: &[AlgEntry]) -> &AlgEntry {
    algs.iter()
        .find(|alg| alg.is_default)
        .expect("catalog guarantees exactly one default algorithm")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alg_list_output_contains_known_cases() {
        let output = alg_list_output("pll").unwrap();
        assert!(output.starts_with("CASE"));
        assert!(output.contains("Aa"));
        assert!(output.contains("Z"));
    }

    #[test]
    fn alg_show_output_contains_default_algorithm() {
        let output = alg_show_output("oll", "3").unwrap();
        assert!(output.contains("OLL Case OLL03"));
        assert!(output.contains("Algorithms"));
        assert!(output.contains("[default]"));
    }
}
