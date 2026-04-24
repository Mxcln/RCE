#!/usr/bin/env python3

from __future__ import annotations

import dataclasses
import html
import os
import re
import shutil
import subprocess
import tempfile
import textwrap
from pathlib import Path
from urllib.request import urlopen


ROOT = Path(__file__).resolve().parents[1]
OLL_DIR = ROOT / "crates" / "rubiks-alg" / "data" / "oll"
PLL_DIR = ROOT / "crates" / "rubiks-alg" / "data" / "pll"
RETRIEVED_AT = "2026-04-25"
SOURCE_NAME = "SpeedCubeDB"
SOURCE_URL = "https://speedcubedb.com"

PLL_CASES = [
    "Aa",
    "Ab",
    "E",
    "F",
    "Ga",
    "Gb",
    "Gc",
    "Gd",
    "H",
    "Ja",
    "Jb",
    "Na",
    "Nb",
    "Ra",
    "Rb",
    "T",
    "Ua",
    "Ub",
    "V",
    "Y",
    "Z",
]


@dataclasses.dataclass
class CaseData:
    family: str
    case_id: str
    display_name: str
    setup: str
    algorithm: str


def fetch_text(url: str) -> str:
    with urlopen(url) as response:
        return response.read().decode("utf-8")


def normalize_notation(text: str) -> str:
    text = html.unescape(text).replace("\xa0", " ")
    text = re.sub(r"\s+", " ", text.strip())
    text = text.replace("’", "'").replace("`", "'")
    text = re.sub(r"([URFDLBMESxyzurfdlb])2'", r"\g<1>2", text)
    return text


def extract_case_blocks(listing_html: str, family: str) -> list[CaseData]:
    blocks = []
    starts = list(
        re.finditer(
            r'<div class="row singlealgorithm g-0" data-subgroup="(?P<subgroup>[^"]+)" data-alg="(?P<display>[^"]+)"[^>]*?>',
            listing_html,
        )
    )
    for index, match in enumerate(starts):
        display_name = html.unescape(match.group("display")).strip()
        start = match.start()
        end = starts[index + 1].start() if index + 1 < len(starts) else len(listing_html)
        body = listing_html[start:end]
        setup_match = re.search(
            r'<div class="setup-case align-items-center"><div>setup:</div>(?P<setup>.*?)</div>',
            body,
            re.DOTALL,
        )
        alg_match = re.search(
            r'<div class="formatted-alg">(?P<alg>.*?)</div>',
            body,
            re.DOTALL,
        )
        href_match = re.search(
            rf'href="a/3x3/{family}/(?P<href>[^"]+)"',
            body,
        )
        if not setup_match or not alg_match or not href_match:
            continue

        href = html.unescape(href_match.group("href")).strip()
        case_id = href.replace(" ", "_")
        if family == "OLL":
            case_id = case_id.replace("OLL_", "OLL")
            case_id = f"OLL{int(case_id[3:]):02d}"
        else:
            case_id = html.unescape(display_name).strip()

        blocks.append(
            CaseData(
                family=family,
                case_id=case_id,
                display_name=display_name,
                setup=normalize_notation(setup_match.group("setup")),
                algorithm=normalize_notation(alg_match.group("alg")),
            )
        )
    return blocks


def load_cases() -> tuple[list[CaseData], list[CaseData]]:
    oll_html = fetch_text("https://speedcubedb.com/a/3x3/OLL")
    pll_html = fetch_text("https://speedcubedb.com/a/3x3/PLL")

    oll_cases = extract_case_blocks(oll_html, "OLL")
    pll_cases = extract_case_blocks(pll_html, "PLL")

    if len(oll_cases) < 57:
        raise RuntimeError(f"expected at least 57 OLL cases, found {len(oll_cases)}")
    if len(pll_cases) < 21:
        raise RuntimeError(f"expected at least 21 PLL cases, found {len(pll_cases)}")

    oll_cases = sorted(
        {case.case_id: case for case in oll_cases}.values(),
        key=lambda case: int(case.case_id[3:]),
    )
    pll_map = {case.case_id: case for case in pll_cases}
    missing = [case for case in PLL_CASES if case not in pll_map]
    if missing:
        raise RuntimeError(f"missing PLL cases: {missing}")
    pll_cases = [pll_map[case_id] for case_id in PLL_CASES]

    if len(oll_cases) != 57:
        raise RuntimeError(f"expected 57 unique OLL cases, found {len(oll_cases)}")
    if len(pll_cases) != 21:
        raise RuntimeError(f"expected 21 unique PLL cases, found {len(pll_cases)}")

    return oll_cases, pll_cases


def run_probe(cases: list[CaseData]) -> str:
    grouped = {"OLL": [], "PLL": []}
    for case in cases:
        grouped[case.family].append(case)

    rust_case_rows = []
    for family in ["OLL", "PLL"]:
        for case in grouped[family]:
            rust_case_rows.append(
                f'CaseInput {{ family: "{family}", case_id: "{case.case_id}", setup: {case.setup!r}, algorithm: {case.algorithm!r} }}'
            )

    probe_source = textwrap.dedent(
        f"""
        use rubiks_alg::{{Auf, OllPattern, PllPattern}};
        use rubiks_core::{{CubeState, parse_notation, resolve_notation, Orientation}};

        struct CaseInput {{
            family: &'static str,
            case_id: &'static str,
            setup: &'static str,
            algorithm: &'static str,
        }}

        fn apply_alg(cube: &mut CubeState, notation: &str, orientation: Orientation) -> Orientation {{
            let seq = parse_notation(notation).unwrap();
            let resolved = resolve_notation(&seq, orientation);
            cube.apply_sequence(&resolved.flattened);
            resolved.final_orientation
        }}

        fn main() {{
            let cases = vec![
                {", ".join(rust_case_rows)}
            ];

            for case in cases {{
                let mut cube = CubeState::solved();
                let setup_orientation = apply_alg(&mut cube, case.setup, Orientation::SOLVED);
                let parts = cube.parts();

                match case.family {{
                    "OLL" => {{
                        let pattern = OllPattern::from_parts(&parts);
                        let mut solved = cube.clone();
                        let _ = apply_alg(&mut solved, case.algorithm, setup_orientation);
                        let solved_parts = solved.parts();
                        let is_oll_solved = solved_parts.corner_orient[..4].iter().all(|&v| v == 0)
                            && solved_parts.edge_orient[..4].iter().all(|&v| v == 0);
                        if !is_oll_solved {{
                            panic!("OLL algorithm does not orient last layer: {{}}", case.case_id);
                        }}

                        let corners = [
                            parts.corner_orient[0],
                            parts.corner_orient[1],
                            parts.corner_orient[2],
                            parts.corner_orient[3],
                        ];
                        let edges = [
                            parts.edge_orient[0],
                            parts.edge_orient[1],
                            parts.edge_orient[2],
                            parts.edge_orient[3],
                        ];
                        println!(
                            "{{}}|OLL|{{:?}}|{{:?}}",
                            case.case_id,
                            corners,
                            edges
                        );
                        let _ = pattern;
                    }}
                    "PLL" => {{
                        let pattern = PllPattern::from_parts(&parts);
                        let corners = [
                            parts.corner_perm[0],
                            parts.corner_perm[1],
                            parts.corner_perm[2],
                            parts.corner_perm[3],
                        ];
                        let edges = [
                            parts.edge_perm[0],
                            parts.edge_perm[1],
                            parts.edge_perm[2],
                            parts.edge_perm[3],
                        ];

                        let mut solved_auf = None;
                        for auf in Auf::ALL {{
                            let mut solved = cube.clone();
                            let _ = apply_alg(&mut solved, case.algorithm, setup_orientation);
                            solved.apply_sequence(&auf.to_move_sequence());
                            if solved.is_solved() {{
                                solved_auf = Some(auf);
                                break;
                            }}
                        }}

                        let post_auf = solved_auf.expect("PLL algorithm does not solve after AUF");
                        println!(
                            "{{}}|PLL|{{:?}}|{{:?}}|{{}}",
                            case.case_id,
                            corners,
                            edges,
                            post_auf.to_notation()
                        );
                        let _ = pattern;
                    }}
                    _ => unreachable!(),
                }}
            }}
        }}
        """
    )

    core_rlib = next((ROOT / "target" / "debug" / "deps").glob("librubiks_core-*.rlib"))
    alg_rlib = sorted((ROOT / "target" / "debug" / "deps").glob("librubiks_alg-*.rlib"))[-1]

    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        src = td_path / "probe.rs"
        src.write_text(probe_source)
        bin_path = td_path / "probe"
        subprocess.run(
            [
                "rustc",
                "--edition=2021",
                str(src),
                "-L",
                str(ROOT / "target" / "debug" / "deps"),
                "--extern",
                f"rubiks_core={core_rlib}",
                "--extern",
                f"rubiks_alg={alg_rlib}",
                "-o",
                str(bin_path),
            ],
            check=True,
            cwd=ROOT,
        )
        return subprocess.check_output([str(bin_path)], text=True)


def parse_probe_output(output: str) -> tuple[dict[str, tuple[list[int], list[int]]], dict[str, tuple[list[str], list[str], str]]]:
    oll_patterns: dict[str, tuple[list[int], list[int]]] = {}
    pll_patterns: dict[str, tuple[list[str], list[str], str]] = {}

    for line in output.strip().splitlines():
        parts = line.split("|")
        if len(parts) < 4:
            continue
        case_id = parts[0]
        family = parts[1]
        if family == "OLL":
            corners = [int(value.strip()) for value in parts[2].strip("[]").split(",")]
            edges = [int(value.strip()) for value in parts[3].strip("[]").split(",")]
            oll_patterns[case_id] = (corners, edges)
        else:
            corners = [corner_name(int(value.strip())) for value in parts[2].strip("[]").split(",")]
            edges = [edge_name(int(value.strip())) for value in parts[3].strip("[]").split(",")]
            pll_patterns[case_id] = (corners, edges, parts[4])

    return oll_patterns, pll_patterns


def corner_name(index: int) -> str:
    names = ["URF", "UFL", "ULB", "UBR"]
    return names[index]


def edge_name(index: int) -> str:
    names = ["UR", "UF", "UL", "UB"]
    return names[index]


def write_case_files(
    oll_cases: list[CaseData],
    pll_cases: list[CaseData],
    oll_patterns: dict[str, tuple[list[int], list[int]]],
    pll_patterns: dict[str, tuple[list[str], list[str], str]],
) -> None:
    for directory in [OLL_DIR, PLL_DIR]:
        if directory.exists():
            shutil.rmtree(directory)
        directory.mkdir(parents=True, exist_ok=True)

    for case in oll_cases:
        corners, edges = oll_patterns[case.case_id]
        path = OLL_DIR / f"{case.case_id}.toml"
        path.write_text(
            textwrap.dedent(
                f"""
                case_id = "{case.case_id}"
                display_name = "{case.display_name}"

                [pattern]
                corners = {corners}
                edges = {edges}

                [[algorithms]]
                id = "{case.case_id.lower()}-default"
                display_name = "Default"
                notation = "{case.algorithm}"
                is_default = true
                tags = ["default"]
                notes = "Default algorithm imported from SpeedCubeDB on {RETRIEVED_AT}"

                [algorithms.source]
                kind = "transcribed"
                name = "{SOURCE_NAME}"
                url = "{SOURCE_URL}/a/3x3/OLL/{case.display_name.replace(" ", "_")}"
                retrieved_at = "{RETRIEVED_AT}"
                notes = "Transcribed from SpeedCubeDB listing page"
                """
            ).strip()
            + "\n"
        )

    for case in pll_cases:
        corners, edges, post_auf = pll_patterns[case.case_id]
        path = PLL_DIR / f"{case.case_id}.toml"
        path.write_text(
            textwrap.dedent(
                f"""
                case_id = "{case.case_id}"
                display_name = "{case.display_name}"

                [pattern]
                corners = {corners}
                edges = {edges}

                [[algorithms]]
                id = "{case.case_id.lower()}-default"
                display_name = "Default"
                notation = "{case.algorithm}"
                is_default = true
                tags = ["default"]
                post_auf = "{post_auf}"
                notes = "Default algorithm imported from SpeedCubeDB on {RETRIEVED_AT}"

                [algorithms.source]
                kind = "transcribed"
                name = "{SOURCE_NAME}"
                url = "{SOURCE_URL}/a/3x3/PLL/{case.case_id}"
                retrieved_at = "{RETRIEVED_AT}"
                notes = "Transcribed from SpeedCubeDB listing page"
                """
            ).strip()
            + "\n"
        )


def main() -> None:
    oll_cases, pll_cases = load_cases()
    probe_output = run_probe(oll_cases + pll_cases)
    oll_patterns, pll_patterns = parse_probe_output(probe_output)
    write_case_files(oll_cases, pll_cases, oll_patterns, pll_patterns)
    print(f"generated {len(oll_cases)} OLL files and {len(pll_cases)} PLL files")


if __name__ == "__main__":
    main()
