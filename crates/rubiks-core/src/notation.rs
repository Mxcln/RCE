use core::fmt;

use crate::moves::{
    Axis, Direction, ExtMove, ExtMoveSequence, Face, Move, MoveSequence, Slice,
};
use crate::orientation::Orientation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidToken { position: usize, token: String },
    ExpectedCanonicalMove { position: usize, token: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidToken { position, token } => {
                write!(f, "invalid token at byte {position}: {token}")
            }
            Self::ExpectedCanonicalMove { position, token } => {
                write!(f, "expected canonical face move at byte {position}: {token}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedStep {
    pub input: ExtMove,
    pub canonical: Vec<Move>,
    pub orientation_before: Orientation,
    pub orientation_after: Orientation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSequence {
    pub source: ExtMoveSequence,
    pub steps: Vec<ResolvedStep>,
    pub flattened: MoveSequence,
    pub final_orientation: Orientation,
}

#[derive(Clone, Copy)]
enum Action {
    Face(Face, Direction),
    Rotate(Axis, Direction),
}

pub fn parse_canonical_notation(input: &str) -> Result<MoveSequence, ParseError> {
    let seq = parse_notation(input)?;
    let mut canonical = Vec::with_capacity(seq.len());
    for (position, mv) in scan_ext_moves(input)? {
        match mv {
            ExtMove::Face(face, dir) => canonical.push(Move::new(face, dir)),
            _ => {
                let token = mv.to_notation();
                return Err(ParseError::ExpectedCanonicalMove { position, token });
            }
        }
    }
    Ok(MoveSequence(canonical))
}

pub fn parse_notation(input: &str) -> Result<ExtMoveSequence, ParseError> {
    Ok(ExtMoveSequence(
        scan_ext_moves(input)?.into_iter().map(|(_, mv)| mv).collect(),
    ))
}

pub fn resolve_ext_move(input: ExtMove, orientation: Orientation) -> ResolvedStep {
    let mut current = orientation;
    let mut canonical = Vec::new();

    for action in actions_for(input) {
        match action {
            Action::Face(face, dir) => canonical.push(current.remap_move(face, dir)),
            Action::Rotate(axis, dir) => current = current.after_rotation(axis, dir),
        }
    }

    ResolvedStep {
        input,
        canonical: simplify_moves(canonical),
        orientation_before: orientation,
        orientation_after: current,
    }
}

pub fn resolve_notation(seq: &ExtMoveSequence, orientation: Orientation) -> ResolvedSequence {
    let mut current = orientation;
    let mut steps = Vec::with_capacity(seq.len());
    let mut flattened = Vec::new();

    for mv in seq.0.iter().copied() {
        let step = resolve_ext_move(mv, current);
        flattened.extend(step.canonical.iter().copied());
        current = step.orientation_after;
        steps.push(step);
    }

    ResolvedSequence {
        source: seq.clone(),
        steps,
        flattened: MoveSequence(flattened),
        final_orientation: current,
    }
}

fn scan_ext_moves(input: &str) -> Result<Vec<(usize, ExtMove)>, ParseError> {
    let mut result = Vec::new();
    let mut index = 0;

    while index < input.len() {
        index = skip_whitespace(input, index);
        if index >= input.len() {
            break;
        }

        let (mv, next_index) = parse_token(input, index)?;
        result.push((index, mv));
        index = next_index;
    }

    Ok(result)
}

fn parse_token(input: &str, start: usize) -> Result<(ExtMove, usize), ParseError> {
    let (first, mut index) = next_char(input, start).ok_or_else(|| ParseError::InvalidToken {
        position: start,
        token: String::new(),
    })?;

    let mut mv = if let Some(face) = Face::from_upper(first) {
        if let Some((next, next_index)) = next_char(input, index) {
            if matches!(next, 'w' | 'W') {
                index = next_index;
                ExtMove::Wide(face, Direction::CW)
            } else {
                ExtMove::Face(face, Direction::CW)
            }
        } else {
            ExtMove::Face(face, Direction::CW)
        }
    } else if let Some(face) = Face::from_lower(first) {
        ExtMove::Wide(face, Direction::CW)
    } else if let Some(slice) = Slice::from_char(first) {
        ExtMove::Slice(slice, Direction::CW)
    } else if let Some(axis) = Axis::from_char(first) {
        ExtMove::Rotation(axis, Direction::CW)
    } else {
        return Err(ParseError::InvalidToken {
            position: start,
            token: first.to_string(),
        });
    };

    if let Some((suffix, next_index)) = next_char(input, index) {
        match suffix {
            '\'' => {
                index = next_index;
                mv = set_direction(mv, Direction::CCW);
            }
            '2' => {
                index = next_index;
                mv = set_direction(mv, Direction::Double);
            }
            _ => {}
        }
    }

    Ok((mv, index))
}

fn set_direction(mv: ExtMove, dir: Direction) -> ExtMove {
    match mv {
        ExtMove::Face(face, _) => ExtMove::Face(face, dir),
        ExtMove::Slice(slice, _) => ExtMove::Slice(slice, dir),
        ExtMove::Rotation(axis, _) => ExtMove::Rotation(axis, dir),
        ExtMove::Wide(face, _) => ExtMove::Wide(face, dir),
    }
}

fn actions_for(mv: ExtMove) -> Vec<Action> {
    let (base, dir) = match mv {
        ExtMove::Face(face, dir) => (vec![Action::Face(face, Direction::CW)], dir),
        ExtMove::Rotation(axis, dir) => (vec![Action::Rotate(axis, Direction::CW)], dir),
        ExtMove::Wide(face, dir) => (base_wide_actions(face), dir),
        ExtMove::Slice(slice, dir) => (base_slice_actions(slice), dir),
    };

    expand_actions(&base, dir)
}

fn base_wide_actions(face: Face) -> Vec<Action> {
    match face {
        Face::R => vec![Action::Rotate(Axis::X, Direction::CW), Action::Face(Face::L, Direction::CW)],
        Face::L => vec![Action::Rotate(Axis::X, Direction::CCW), Action::Face(Face::R, Direction::CW)],
        Face::U => vec![Action::Rotate(Axis::Y, Direction::CW), Action::Face(Face::D, Direction::CW)],
        Face::D => vec![Action::Rotate(Axis::Y, Direction::CCW), Action::Face(Face::U, Direction::CW)],
        Face::F => vec![Action::Rotate(Axis::Z, Direction::CW), Action::Face(Face::B, Direction::CW)],
        Face::B => vec![Action::Rotate(Axis::Z, Direction::CCW), Action::Face(Face::F, Direction::CW)],
    }
}

fn base_slice_actions(slice: Slice) -> Vec<Action> {
    match slice {
        Slice::M => vec![
            Action::Face(Face::L, Direction::CCW),
            Action::Rotate(Axis::X, Direction::CCW),
            Action::Face(Face::R, Direction::CW),
        ],
        Slice::E => vec![
            Action::Face(Face::D, Direction::CCW),
            Action::Rotate(Axis::Y, Direction::CCW),
            Action::Face(Face::U, Direction::CW),
        ],
        Slice::S => vec![
            Action::Face(Face::F, Direction::CCW),
            Action::Rotate(Axis::Z, Direction::CW),
            Action::Face(Face::B, Direction::CW),
        ],
    }
}

fn expand_actions(base: &[Action], dir: Direction) -> Vec<Action> {
    match dir {
        Direction::CW => base.to_vec(),
        Direction::CCW => base.iter().rev().copied().map(invert_action).collect(),
        Direction::Double => {
            let mut actions = Vec::with_capacity(base.len() * 2);
            actions.extend_from_slice(base);
            actions.extend_from_slice(base);
            actions
        }
    }
}

fn invert_action(action: Action) -> Action {
    match action {
        Action::Face(face, dir) => Action::Face(face, dir.inverse()),
        Action::Rotate(axis, dir) => Action::Rotate(axis, dir.inverse()),
    }
}

fn simplify_moves(moves: Vec<Move>) -> Vec<Move> {
    let mut simplified: Vec<Move> = Vec::with_capacity(moves.len());

    for mv in moves {
        if let Some(last) = simplified.last_mut() {
            if last.face == mv.face {
                let turns = (direction_to_turns(last.dir) + direction_to_turns(mv.dir)) % 4;
                if turns == 0 {
                    simplified.pop();
                } else {
                    last.dir = turns_to_direction(turns);
                }
                continue;
            }
        }
        simplified.push(mv);
    }

    simplified
}

fn direction_to_turns(dir: Direction) -> u8 {
    match dir {
        Direction::CW => 1,
        Direction::Double => 2,
        Direction::CCW => 3,
    }
}

fn turns_to_direction(turns: u8) -> Direction {
    match turns {
        1 => Direction::CW,
        2 => Direction::Double,
        3 => Direction::CCW,
        _ => unreachable!("turn count is always reduced mod 4"),
    }
}

fn skip_whitespace(input: &str, mut index: usize) -> usize {
    while let Some((ch, next_index)) = next_char(input, index) {
        if !ch.is_whitespace() {
            break;
        }
        index = next_index;
    }
    index
}

fn next_char(input: &str, index: usize) -> Option<(char, usize)> {
    input[index..]
        .chars()
        .next()
        .map(|ch| (ch, index + ch.len_utf8()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_parser_reads_basic_sequence() {
        let parsed = parse_canonical_notation("R U R' U'").unwrap();
        assert_eq!(
            parsed,
            MoveSequence(vec![
                Move::new(Face::R, Direction::CW),
                Move::new(Face::U, Direction::CW),
                Move::new(Face::R, Direction::CCW),
                Move::new(Face::U, Direction::CCW),
            ])
        );
    }

    #[test]
    fn extended_parser_preserves_wide_moves() {
        let parsed = parse_notation("r M x").unwrap();
        assert_eq!(
            parsed,
            ExtMoveSequence(vec![
                ExtMove::Wide(Face::R, Direction::CW),
                ExtMove::Slice(Slice::M, Direction::CW),
                ExtMove::Rotation(Axis::X, Direction::CW),
            ])
        );
        assert_eq!(parsed.to_notation(), "r M x");
    }

    #[test]
    fn canonical_parser_rejects_extended_moves() {
        assert_eq!(
            parse_canonical_notation("r"),
            Err(ParseError::ExpectedCanonicalMove {
                position: 0,
                token: "r".to_string(),
            })
        );
    }

    #[test]
    fn parser_supports_adjacent_tokens() {
        let parsed = parse_canonical_notation("RUR'U'").unwrap();
        assert_eq!(parsed.to_notation(), "R U R' U'");
    }

    #[test]
    fn resolve_rotation_updates_orientation_without_moves() {
        let step = resolve_ext_move(ExtMove::Rotation(Axis::X, Direction::CW), Orientation::SOLVED);
        assert!(step.canonical.is_empty());
        assert_eq!(
            step.orientation_after,
            Orientation::SOLVED.after_rotation(Axis::X, Direction::CW)
        );
    }

    #[test]
    fn resolve_sequence_flattens_canonical_moves() {
        let seq = parse_notation("R x U").unwrap();
        let resolved = resolve_notation(&seq, Orientation::SOLVED);
        assert_eq!(resolved.steps.len(), 3);
        assert_eq!(resolved.flattened.to_notation(), "R F");
    }
}
