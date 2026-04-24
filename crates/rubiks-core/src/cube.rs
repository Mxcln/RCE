use crate::color::Color;
use crate::state::{CubeState, CubeStateError};
use crate::moves::{ExtMoveSequence, MoveSequence};
use crate::notation::{parse_canonical_notation, parse_notation, ParseError, resolve_notation};
use crate::orientation::Orientation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cube {
    state: CubeState,
    orientation: Orientation,
}

impl Cube {
    pub fn solved() -> Self {
        Self {
            state: CubeState::solved(),
            orientation: Orientation::SOLVED,
        }
    }

    pub fn from_state(state: CubeState) -> Self {
        Self::from_state_with_orientation(state, Orientation::SOLVED)
    }

    pub fn from_state_with_orientation(state: CubeState, orientation: Orientation) -> Self {
        Self { state, orientation }
    }

    pub fn state(&self) -> &CubeState {
        &self.state
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn into_state(self) -> CubeState {
        self.state
    }

    pub fn reset(&mut self) {
        self.state.reset();
        self.orientation = Orientation::SOLVED;
    }

    pub fn reset_orientation(&mut self) {
        self.orientation = Orientation::SOLVED;
    }

    pub fn is_solved(&self) -> bool {
        self.state.is_solved()
    }

    pub fn validate(&self) -> Result<(), CubeStateError> {
        self.state.validate()
    }

    pub fn is_valid(&self) -> bool {
        self.state.is_valid()
    }

    pub fn apply_canonical_sequence(&mut self, seq: &MoveSequence) {
        self.state.apply_sequence(seq);
    }

    pub fn apply_ext_sequence(&mut self, seq: &ExtMoveSequence) {
        let resolved = resolve_notation(seq, self.orientation);
        self.state.apply_sequence(&resolved.flattened);
        self.orientation = resolved.final_orientation;
    }

    pub fn apply_canonical_notation(&mut self, input: &str) -> Result<(), ParseError> {
        let seq = parse_canonical_notation(input)?;
        self.apply_canonical_sequence(&seq);
        Ok(())
    }

    pub fn apply_notation(&mut self, input: &str) -> Result<(), ParseError> {
        let seq = parse_notation(input)?;
        self.apply_ext_sequence(&seq);
        Ok(())
    }

    pub fn to_facelets(&self) -> [[Color; 9]; 6] {
        self.orientation.remap_facelets(self.state.to_facelets())
    }
}

#[cfg(test)]
mod tests {
    use crate::moves::{Axis, Direction, Face, Move};

    use super::*;

    #[test]
    fn canonical_sequence_does_not_change_orientation() {
        let mut cube = Cube::solved();
        cube.apply_canonical_sequence(&MoveSequence(vec![Move::new(Face::R, Direction::CW)]));
        assert_eq!(cube.orientation(), Orientation::SOLVED);
    }

    #[test]
    fn whole_cube_rotation_changes_orientation_but_not_solvedness() {
        let mut cube = Cube::solved();
        cube.apply_notation("x").unwrap();
        assert_eq!(cube.orientation(), Orientation::SOLVED.after_rotation(Axis::X, Direction::CW));
        assert!(cube.is_solved());
    }

    #[test]
    fn wide_and_slice_moves_apply_without_parse_errors() {
        let mut cube = Cube::solved();
        cube.apply_notation("r M S E").unwrap();
        assert!(cube.is_valid());
    }
}
