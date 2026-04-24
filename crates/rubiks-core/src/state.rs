use core::fmt;

use crate::color::Color;
use crate::moves::{Face, Move, MoveSequence};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Corner {
    Urf = 0,
    Ufl = 1,
    Ulb = 2,
    Ubr = 3,
    Dfr = 4,
    Dlf = 5,
    Dbl = 6,
    Drb = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Edge {
    Ur = 0,
    Uf = 1,
    Ul = 2,
    Ub = 3,
    Dr = 4,
    Df = 5,
    Dl = 6,
    Db = 7,
    Fr = 8,
    Fl = 9,
    Bl = 10,
    Br = 11,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CubeStateParts {
    pub corner_perm: [u8; 8],
    pub corner_orient: [u8; 8],
    pub edge_perm: [u8; 12],
    pub edge_orient: [u8; 12],
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CubeState {
    corner_perm: [u8; 8],
    corner_orient: [u8; 8],
    edge_perm: [u8; 12],
    edge_orient: [u8; 12],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CubeStateError {
    InvalidCornerOrientation { index: usize, value: u8 },
    InvalidEdgeOrientation { index: usize, value: u8 },
    InvalidCornerPermutation { index: usize, value: u8 },
    InvalidEdgePermutation { index: usize, value: u8 },
    CornerOrientationSum { sum_mod_3: u8 },
    EdgeOrientationSum { sum_mod_2: u8 },
    ParityMismatch { corner_parity: u8, edge_parity: u8 },
}

impl fmt::Display for CubeStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCornerOrientation { index, value } => {
                write!(f, "invalid corner orientation at index {index}: {value}")
            }
            Self::InvalidEdgeOrientation { index, value } => {
                write!(f, "invalid edge orientation at index {index}: {value}")
            }
            Self::InvalidCornerPermutation { index, value } => {
                write!(f, "invalid corner permutation entry at index {index}: {value}")
            }
            Self::InvalidEdgePermutation { index, value } => {
                write!(f, "invalid edge permutation entry at index {index}: {value}")
            }
            Self::CornerOrientationSum { sum_mod_3 } => {
                write!(f, "corner orientation sum is not 0 mod 3: {sum_mod_3}")
            }
            Self::EdgeOrientationSum { sum_mod_2 } => {
                write!(f, "edge orientation sum is not 0 mod 2: {sum_mod_2}")
            }
            Self::ParityMismatch {
                corner_parity,
                edge_parity,
            } => write!(
                f,
                "corner/edge permutation parity mismatch: corners={corner_parity}, edges={edge_parity}"
            ),
        }
    }
}

#[derive(Clone, Copy)]
struct CubieMove {
    corner_perm: [u8; 8],
    corner_orient: [u8; 8],
    edge_perm: [u8; 12],
    edge_orient: [u8; 12],
}

const SOLVED_CORNER_PERM: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
const SOLVED_CORNER_ORIENT: [u8; 8] = [0; 8];
const SOLVED_EDGE_PERM: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const SOLVED_EDGE_ORIENT: [u8; 12] = [0; 12];

const CORNER_FACELETS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];

const EDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

const CORNER_COLORS: [[Color; 3]; 8] = [
    [Color::U, Color::R, Color::F],
    [Color::U, Color::F, Color::L],
    [Color::U, Color::L, Color::B],
    [Color::U, Color::B, Color::R],
    [Color::D, Color::F, Color::R],
    [Color::D, Color::L, Color::F],
    [Color::D, Color::B, Color::L],
    [Color::D, Color::R, Color::B],
];

const EDGE_COLORS: [[Color; 2]; 12] = [
    [Color::U, Color::R],
    [Color::U, Color::F],
    [Color::U, Color::L],
    [Color::U, Color::B],
    [Color::D, Color::R],
    [Color::D, Color::F],
    [Color::D, Color::L],
    [Color::D, Color::B],
    [Color::F, Color::R],
    [Color::F, Color::L],
    [Color::B, Color::L],
    [Color::B, Color::R],
];

const MOVE_6: [CubieMove; 6] = [
    CubieMove {
        corner_perm: [3, 0, 1, 2, 4, 5, 6, 7],
        corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
        edge_perm: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11],
        edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    CubieMove {
        corner_perm: [4, 1, 2, 0, 7, 5, 6, 3],
        corner_orient: [2, 0, 0, 1, 1, 0, 0, 2],
        edge_perm: [8, 1, 2, 3, 11, 5, 6, 7, 4, 9, 10, 0],
        edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    CubieMove {
        corner_perm: [1, 5, 2, 3, 0, 4, 6, 7],
        corner_orient: [1, 2, 0, 0, 2, 1, 0, 0],
        edge_perm: [0, 9, 2, 3, 4, 8, 6, 7, 1, 5, 10, 11],
        edge_orient: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    },
    CubieMove {
        corner_perm: [0, 1, 2, 3, 5, 6, 7, 4],
        corner_orient: [0, 0, 0, 0, 0, 0, 0, 0],
        edge_perm: [0, 1, 2, 3, 5, 6, 7, 4, 8, 9, 10, 11],
        edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    CubieMove {
        corner_perm: [0, 2, 6, 3, 4, 1, 5, 7],
        corner_orient: [0, 1, 2, 0, 0, 2, 1, 0],
        edge_perm: [0, 1, 10, 3, 4, 5, 9, 7, 8, 2, 6, 11],
        edge_orient: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    },
    CubieMove {
        corner_perm: [0, 1, 3, 7, 4, 5, 2, 6],
        corner_orient: [0, 0, 1, 2, 0, 0, 2, 1],
        edge_perm: [0, 1, 2, 11, 4, 5, 6, 10, 8, 9, 3, 7],
        edge_orient: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
    },
];

impl CubeState {
    pub fn solved() -> Self {
        Self {
            corner_perm: SOLVED_CORNER_PERM,
            corner_orient: SOLVED_CORNER_ORIENT,
            edge_perm: SOLVED_EDGE_PERM,
            edge_orient: SOLVED_EDGE_ORIENT,
        }
    }

    pub fn try_from_parts(parts: CubeStateParts) -> Result<Self, CubeStateError> {
        let cube = Self::from_parts_unchecked(parts);
        cube.validate()?;
        Ok(cube)
    }

    pub fn parts(&self) -> CubeStateParts {
        CubeStateParts {
            corner_perm: self.corner_perm,
            corner_orient: self.corner_orient,
            edge_perm: self.edge_perm,
            edge_orient: self.edge_orient,
        }
    }

    pub fn is_solved(&self) -> bool {
        self.corner_perm == SOLVED_CORNER_PERM
            && self.corner_orient == SOLVED_CORNER_ORIENT
            && self.edge_perm == SOLVED_EDGE_PERM
            && self.edge_orient == SOLVED_EDGE_ORIENT
    }

    pub fn reset(&mut self) {
        *self = Self::solved();
    }

    pub fn validate(&self) -> Result<(), CubeStateError> {
        for (index, &value) in self.corner_orient.iter().enumerate() {
            if value >= 3 {
                return Err(CubeStateError::InvalidCornerOrientation { index, value });
            }
        }

        for (index, &value) in self.edge_orient.iter().enumerate() {
            if value >= 2 {
                return Err(CubeStateError::InvalidEdgeOrientation { index, value });
            }
        }

        validate_permutation(&self.corner_perm)
            .map_err(|(index, value)| CubeStateError::InvalidCornerPermutation { index, value })?;
        validate_permutation(&self.edge_perm)
            .map_err(|(index, value)| CubeStateError::InvalidEdgePermutation { index, value })?;

        let corner_sum = self.corner_orient.iter().copied().sum::<u8>() % 3;
        if corner_sum != 0 {
            return Err(CubeStateError::CornerOrientationSum {
                sum_mod_3: corner_sum,
            });
        }

        let edge_sum = self.edge_orient.iter().copied().sum::<u8>() % 2;
        if edge_sum != 0 {
            return Err(CubeStateError::EdgeOrientationSum {
                sum_mod_2: edge_sum,
            });
        }

        let corner_parity = permutation_parity(&self.corner_perm);
        let edge_parity = permutation_parity(&self.edge_perm);
        if corner_parity != edge_parity {
            return Err(CubeStateError::ParityMismatch {
                corner_parity,
                edge_parity,
            });
        }

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    pub fn apply_move(&mut self, mv: Move) {
        let base = &MOVE_6[mv.face.index()];
        for _ in 0..mv.dir.quarter_turns() {
            self.apply_cubie_move(base);
        }
    }

    pub fn apply_sequence(&mut self, seq: &MoveSequence) {
        for mv in &seq.0 {
            self.apply_move(*mv);
        }
    }

    pub fn to_facelets(&self) -> [[Color; 9]; 6] {
        let mut facelets = [[Color::U; 9]; 6];
        for face in Face::ALL {
            facelets[face.index()] = [face.solved_color(); 9];
        }

        for (position, _) in CORNER_FACELETS.iter().enumerate() {
            let cubie = self.corner_perm[position] as usize;
            let orientation = self.corner_orient[position] as usize;
            for offset in 0..3 {
                let facelet = CORNER_FACELETS[position][(offset + orientation) % 3];
                let color = CORNER_COLORS[cubie][offset];
                facelets[facelet / 9][facelet % 9] = color;
            }
        }

        for (position, _) in EDGE_FACELETS.iter().enumerate() {
            let cubie = self.edge_perm[position] as usize;
            let orientation = self.edge_orient[position] as usize;
            for offset in 0..2 {
                let facelet = EDGE_FACELETS[position][(offset + orientation) % 2];
                let color = EDGE_COLORS[cubie][offset];
                facelets[facelet / 9][facelet % 9] = color;
            }
        }

        facelets
    }

    pub(crate) fn from_parts_unchecked(parts: CubeStateParts) -> Self {
        Self {
            corner_perm: parts.corner_perm,
            corner_orient: parts.corner_orient,
            edge_perm: parts.edge_perm,
            edge_orient: parts.edge_orient,
        }
    }

    fn apply_cubie_move(&mut self, mv: &CubieMove) {
        let old_corner_perm = self.corner_perm;
        let old_corner_orient = self.corner_orient;
        let old_edge_perm = self.edge_perm;
        let old_edge_orient = self.edge_orient;

        for position in 0..8 {
            let source = mv.corner_perm[position] as usize;
            self.corner_perm[position] = old_corner_perm[source];
            self.corner_orient[position] = (old_corner_orient[source] + mv.corner_orient[position]) % 3;
        }

        for position in 0..12 {
            let source = mv.edge_perm[position] as usize;
            self.edge_perm[position] = old_edge_perm[source];
            self.edge_orient[position] = (old_edge_orient[source] + mv.edge_orient[position]) % 2;
        }
    }
}

fn validate_permutation<const N: usize>(perm: &[u8; N]) -> Result<(), (usize, u8)> {
    let mut seen = [false; N];
    for (index, &value) in perm.iter().enumerate() {
        let value_index = value as usize;
        if value_index >= N || seen[value_index] {
            return Err((index, value));
        }
        seen[value_index] = true;
    }
    Ok(())
}

fn permutation_parity<const N: usize>(perm: &[u8; N]) -> u8 {
    let mut parity = 0u8;
    for i in (1..N).rev() {
        for j in (0..i).rev() {
            if perm[j] > perm[i] {
                parity ^= 1;
            }
        }
    }
    parity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::Direction;

    fn mv(face: Face, dir: Direction) -> Move {
        Move { face, dir }
    }

    #[test]
    fn solved_cube_is_valid_and_solved() {
        let cube = CubeState::solved();
        assert!(cube.is_valid());
        assert!(cube.is_solved());
    }

    #[test]
    fn rejects_invalid_corner_orientation_values() {
        let mut parts = CubeState::solved().parts();
        parts.corner_orient[0] = 3;
        assert_eq!(
            CubeState::try_from_parts(parts),
            Err(CubeStateError::InvalidCornerOrientation { index: 0, value: 3 })
        );
    }

    #[test]
    fn rejects_parity_mismatch() {
        let mut parts = CubeState::solved().parts();
        parts.corner_perm.swap(0, 1);
        assert_eq!(
            CubeState::try_from_parts(parts),
            Err(CubeStateError::ParityMismatch {
                corner_parity: 1,
                edge_parity: 0,
            })
        );
    }

    #[test]
    fn each_basic_face_turn_has_order_four() {
        for face in Face::ALL {
            let mut cube = CubeState::solved();
            for _ in 0..4 {
                cube.apply_move(mv(face, Direction::CW));
            }
            assert_eq!(cube, CubeState::solved(), "face {:?}", face);
        }
    }

    #[test]
    fn move_followed_by_inverse_returns_to_start() {
        for face in Face::ALL {
            let mut cube = CubeState::solved();
            cube.apply_move(mv(face, Direction::CW));
            cube.apply_move(mv(face, Direction::CCW));
            assert_eq!(cube, CubeState::solved(), "face {:?}", face);
        }
    }

    #[test]
    fn solved_facelets_are_uniform() {
        let facelets = CubeState::solved().to_facelets();
        for face in Face::ALL {
            assert_eq!(facelets[face.index()], [face.solved_color(); 9]);
        }
    }
}
