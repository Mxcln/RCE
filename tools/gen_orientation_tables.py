#!/usr/bin/env python3
"""Generate orientation remap tables for rubiks-core.

The generated tables assume the Rust enum order:

    Face::{U, R, F, D, L, B} = 0..5
    Axis rotations are ordered as x, x', y, y', z, z'

Orientation i is encoded by a pair (top, front), where:

    physical U face -> canonical `top`
    physical F face -> canonical `front`

Only perpendicular (top, front) pairs are valid, giving 6 * 4 = 24
orientations. Orientation 0 is always (U, F), the solved orientation.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


FACES = ("U", "R", "F", "D", "L", "B")
FACE_TO_INDEX = {face: index for index, face in enumerate(FACES)}

# Right-handed canonical cube coordinates.
# U/D = +/-Y, R/L = +/-X, F/B = +/-Z.
FACE_VEC = {
    "U": (0, 1, 0),
    "R": (1, 0, 0),
    "F": (0, 0, 1),
    "D": (0, -1, 0),
    "L": (-1, 0, 0),
    "B": (0, 0, -1),
}
VEC_FACE = {vec: face for face, vec in FACE_VEC.items()}
OPPOSITE = {
    "U": "D",
    "D": "U",
    "R": "L",
    "L": "R",
    "F": "B",
    "B": "F",
}

# Facelet row/column bases. For a facelet on face N, row increases along DOWN
# on that face and col increases along RIGHT on that face, as seen from outside.
FACE_BASIS = {
    "U": {"normal": (0, 1, 0), "right": (1, 0, 0), "down": (0, 0, 1)},
    "R": {"normal": (1, 0, 0), "right": (0, 0, -1), "down": (0, -1, 0)},
    "F": {"normal": (0, 0, 1), "right": (1, 0, 0), "down": (0, -1, 0)},
    "D": {"normal": (0, -1, 0), "right": (1, 0, 0), "down": (0, 0, -1)},
    "L": {"normal": (-1, 0, 0), "right": (0, 0, 1), "down": (0, -1, 0)},
    "B": {"normal": (0, 0, -1), "right": (-1, 0, 0), "down": (0, -1, 0)},
}


Vec3 = tuple[int, int, int]


@dataclass(frozen=True)
class Frame:
    """Maps physical basis vectors to canonical basis vectors."""

    right: Vec3
    up: Vec3
    front: Vec3


def dot(a: Vec3, b: Vec3) -> int:
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a: Vec3, b: Vec3) -> Vec3:
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def neg(v: Vec3) -> Vec3:
    return (-v[0], -v[1], -v[2])


def add(a: Vec3, b: Vec3) -> Vec3:
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def scale(v: Vec3, k: int) -> Vec3:
    return (v[0] * k, v[1] * k, v[2] * k)


def frame_from_top_front(top: str, front: str) -> Frame:
    up = FACE_VEC[top]
    front_vec = FACE_VEC[front]
    if dot(up, front_vec) != 0:
        raise ValueError(f"top and front must be perpendicular: {top}, {front}")
    return Frame(right=cross(up, front_vec), up=up, front=front_vec)


def apply_frame(frame: Frame, physical_vec: Vec3) -> Vec3:
    """Map a physical direction vector into canonical coordinates."""

    return add(
        add(scale(frame.right, physical_vec[0]), scale(frame.up, physical_vec[1])),
        scale(frame.front, physical_vec[2]),
    )


def compose(a: Frame, b: Frame) -> Frame:
    """Return a frame equivalent to applying b, then a."""

    return Frame(
        right=apply_frame(a, b.right),
        up=apply_frame(a, b.up),
        front=apply_frame(a, b.front),
    )


def orientation_frames() -> list[Frame]:
    """Return 24 frames with solved orientation at index 0."""

    pairs = [("U", "F")]
    for top in FACES:
        for front in FACES:
            if top == "U" and front == "F":
                continue
            if dot(FACE_VEC[top], FACE_VEC[front]) == 0:
                pairs.append((top, front))

    frames = [frame_from_top_front(top, front) for top, front in pairs]
    if len(frames) != 24:
        raise AssertionError(f"expected 24 orientations, got {len(frames)}")
    if len(set(frames)) != 24:
        raise AssertionError("orientation frames are not unique")
    return frames


def frame_name(frame: Frame) -> tuple[str, str]:
    return (VEC_FACE[frame.up], VEC_FACE[frame.front])


def rotation_frame(axis: str, quarter_turns: int) -> Frame:
    """Frame for physical cube rotation.

    `quarter_turns` is positive for Singmaster CW notation: x, y, z are
    clockwise when looking at the R, U, F face respectively. In the right-hand
    coordinate system this is a -90 degree active rotation around +X/+Y/+Z.
    """

    if quarter_turns not in (-1, 1, 2):
        raise ValueError(f"unsupported quarter_turns: {quarter_turns}")

    if axis == "x":
        cw = Frame(right=(1, 0, 0), up=(0, 0, -1), front=(0, 1, 0))
    elif axis == "y":
        cw = Frame(right=(0, 0, 1), up=(0, 1, 0), front=(-1, 0, 0))
    elif axis == "z":
        cw = Frame(right=(0, -1, 0), up=(1, 0, 0), front=(0, 0, 1))
    else:
        raise ValueError(f"unknown axis: {axis}")

    if quarter_turns == 1:
        return cw
    if quarter_turns == -1:
        return compose(compose(cw, cw), cw)
    return compose(cw, cw)


def remap_face(frame: Frame, physical_face: str) -> str:
    return VEC_FACE[apply_frame(frame, FACE_VEC[physical_face])]


def face_remap_table(frames: list[Frame]) -> list[list[int]]:
    return [
        [FACE_TO_INDEX[remap_face(frame, face)] for face in FACES]
        for frame in frames
    ]


def rotation_table(frames: list[Frame]) -> list[list[int]]:
    frame_to_index = {frame: index for index, frame in enumerate(frames)}
    # Orientation maps physical -> canonical. A whole-cube rotation actively
    # moves old physical directions to new physical directions, so the updated
    # orientation composes the current frame with the inverse active rotation.
    rotations = [
        rotation_frame("x", -1),
        rotation_frame("x", 1),
        rotation_frame("y", -1),
        rotation_frame("y", 1),
        rotation_frame("z", -1),
        rotation_frame("z", 1),
    ]

    table: list[list[int]] = []
    for frame in frames:
        row = []
        for rotation in rotations:
            row.append(frame_to_index[compose(frame, rotation)])
        table.append(row)
    return table


def facelet_position(face: str, slot: int) -> tuple[Vec3, Vec3]:
    basis = FACE_BASIS[face]
    row, col = divmod(slot, 3)
    # Coordinates use -1, 0, 1 on each axis. Facelets are identified by their
    # outward normal and center position.
    center = add(
        add(basis["normal"], scale(basis["right"], col - 1)),
        scale(basis["down"], row - 1),
    )
    return basis["normal"], center


def facelet_index_from_position(normal: Vec3, center: Vec3) -> int:
    face = VEC_FACE[normal]
    basis = FACE_BASIS[face]
    offset = add(center, neg(normal))
    col = dot(offset, basis["right"]) + 1
    row = dot(offset, basis["down"]) + 1
    if row not in (0, 1, 2) or col not in (0, 1, 2):
        raise AssertionError(f"invalid facelet coordinates: {face}, row={row}, col={col}")
    return FACE_TO_INDEX[face] * 9 + row * 3 + col


def facelet_remap_table(frames: list[Frame]) -> list[list[int]]:
    table: list[list[int]] = []
    for frame in frames:
        row = []
        for physical_face in FACES:
            for slot in range(9):
                physical_normal, physical_center = facelet_position(physical_face, slot)
                canonical_normal = apply_frame(frame, physical_normal)
                canonical_center = apply_frame(frame, physical_center)
                row.append(facelet_index_from_position(canonical_normal, canonical_center))
        table.append(row)
    return table


def fmt_rust_array(name: str, ty: str, rows: Iterable[Iterable[int]], row_width: int | None = None) -> str:
    rows = [list(row) for row in rows]
    lines = [f"pub const {name}: {ty} = ["]
    for row in rows:
        if row_width is None:
            values = ", ".join(str(value) for value in row)
            lines.append(f"    [{values}],")
        else:
            lines.append("    [")
            for start in range(0, len(row), row_width):
                values = ", ".join(f"{value:2d}" for value in row[start : start + row_width])
                lines.append(f"        {values},")
            lines.append("    ],")
    lines.append("];")
    return "\n".join(lines)


def fmt_rust_bool_array(name: str, ty: str, rows: Iterable[Iterable[bool]]) -> str:
    rows = [list(row) for row in rows]
    lines = [f"pub const {name}: {ty} = ["]
    for row in rows:
        values = ", ".join("true" if value else "false" for value in row)
        lines.append(f"    [{values}],")
    lines.append("];")
    return "\n".join(lines)


def validate_tables(
    frames: list[Frame],
    face_remap: list[list[int]],
    rotations: list[list[int]],
    facelets: list[list[int]],
) -> None:
    if frame_name(frames[0]) != ("U", "F"):
        raise AssertionError(f"orientation 0 is not solved: {frame_name(frames[0])}")

    if face_remap[0] != [0, 1, 2, 3, 4, 5]:
        raise AssertionError(f"solved face remap is wrong: {face_remap[0]}")

    if facelets[0] != list(range(54)):
        raise AssertionError("solved facelet remap is not identity")

    frame_to_index = {frame_name(frame): index for index, frame in enumerate(frames)}
    expected_solved_rotations = [
        frame_to_index[("F", "D")],  # x
        frame_to_index[("B", "U")],  # x'
        frame_to_index[("U", "R")],  # y
        frame_to_index[("U", "L")],  # y'
        frame_to_index[("L", "F")],  # z
        frame_to_index[("R", "F")],  # z'
    ]
    if rotations[0] != expected_solved_rotations:
        raise AssertionError(
            f"solved rotation row is wrong: expected {expected_solved_rotations}, got {rotations[0]}"
        )

    for index, row in enumerate(face_remap):
        if sorted(row) != list(range(6)):
            raise AssertionError(f"face remap row {index} is not a permutation: {row}")

    for index, row in enumerate(facelets):
        if sorted(row) != list(range(54)):
            raise AssertionError(f"facelet remap row {index} is not a permutation")

    for index, row in enumerate(rotations):
        if len(row) != 6:
            raise AssertionError(f"rotation row {index} has wrong length")

    for start in range(24):
        for col in range(6):
            current = start
            for _ in range(4):
                current = rotations[current][col]
            if current != start:
                raise AssertionError(f"rotation column {col} from {start} is not order 4")

    reachable = {0}
    frontier = [0]
    while frontier:
        current = frontier.pop()
        for nxt in rotations[current]:
            if nxt not in reachable:
                reachable.add(nxt)
                frontier.append(nxt)
    if len(reachable) != 24:
        raise AssertionError(f"only {len(reachable)} orientations reachable from solved")


def render_rust_tables() -> str:
    frames = orientation_frames()
    face_remap = face_remap_table(frames)
    rotations = rotation_table(frames)
    facelets = facelet_remap_table(frames)
    validate_tables(frames, face_remap, rotations, facelets)

    frame_rows = [
        [FACE_TO_INDEX[top], FACE_TO_INDEX[front]]
        for top, front in (frame_name(frame) for frame in frames)
    ]
    # All entries are false because Orientation is a proper 3D rotation, not a
    # mirror/reflection. Face turn handedness is preserved by conjugation.
    direction_flip = [[False] * 6 for _ in range(24)]

    chunks = [
        "// Generated by tools/gen_orientation_tables.py",
        "// Face order: U, R, F, D, L, B = 0..5",
        "// Rotation order: x, x', y, y', z, z'",
        "// Orientation i means physical U -> ORIENTATION_FRAMES[i][0],",
        "// and physical F -> ORIENTATION_FRAMES[i][1].",
        "// FACE_REMAP[o][physical_face] = canonical_face.",
        "// FACE_DIR_FLIP is always false because cube orientations are rotations, not mirrors.",
        "// FACELET_REMAP[o][physical_index] = canonical_index.",
        "",
        fmt_rust_array("ORIENTATION_FRAMES", "[[u8; 2]; 24]", frame_rows),
        "",
        fmt_rust_array("FACE_REMAP", "[[u8; 6]; 24]", face_remap),
        "",
        fmt_rust_bool_array("FACE_DIR_FLIP", "[[bool; 6]; 24]", direction_flip),
        "",
        fmt_rust_array("ROTATION_TABLE", "[[u8; 6]; 24]", rotations),
        "",
        fmt_rust_array("FACELET_REMAP", "[[u8; 54]; 24]", facelets, row_width=18),
        "",
    ]
    return "\n".join(chunks)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        help="write generated Rust const tables to this path instead of stdout",
    )
    args = parser.parse_args()

    rendered = render_rust_tables()
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")


if __name__ == "__main__":
    main()
