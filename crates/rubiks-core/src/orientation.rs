use crate::color::Color;
use crate::moves::{Axis, Direction, Face, Move};
use crate::orientation_tables::{FACE_DIR_FLIP, FACELET_REMAP, FACE_REMAP, ORIENTATION_FRAMES, ROTATION_TABLE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Orientation(pub(crate) u8);

impl Orientation {
    pub const SOLVED: Self = Self(0);

    pub fn try_from_index(index: u8) -> Option<Self> {
        (index < 24).then_some(Self(index))
    }

    pub const fn index(self) -> u8 {
        self.0
    }

    pub fn frame(self) -> (Face, Face) {
        let row = ORIENTATION_FRAMES[self.0 as usize];
        (Face::from_index(row[0]), Face::from_index(row[1]))
    }

    pub fn after_rotation(self, axis: Axis, dir: Direction) -> Self {
        match dir {
            Direction::CW => self.apply_quarter_rotation(axis, true),
            Direction::CCW => self.apply_quarter_rotation(axis, false),
            Direction::Double => self
                .apply_quarter_rotation(axis, true)
                .apply_quarter_rotation(axis, true),
        }
    }

    pub fn remap_face(self, face: Face) -> Face {
        Face::from_index(FACE_REMAP[self.0 as usize][face.index()])
    }

    pub fn remap_move(self, face: Face, dir: Direction) -> Move {
        let canonical_face = self.remap_face(face);
        let flip = FACE_DIR_FLIP[self.0 as usize][face.index()];
        let canonical_dir = if flip { dir.inverse() } else { dir };
        Move::new(canonical_face, canonical_dir)
    }

    pub fn remap_facelets(self, facelets: [[Color; 9]; 6]) -> [[Color; 9]; 6] {
        let mut remapped = [[Color::U; 9]; 6];
        for physical_index in 0..54 {
            let canonical_index = FACELET_REMAP[self.0 as usize][physical_index] as usize;
            remapped[physical_index / 9][physical_index % 9] =
                facelets[canonical_index / 9][canonical_index % 9];
        }
        remapped
    }

    fn apply_quarter_rotation(self, axis: Axis, clockwise: bool) -> Self {
        let column = match (axis, clockwise) {
            (Axis::X, true) => 0,
            (Axis::X, false) => 1,
            (Axis::Y, true) => 2,
            (Axis::Y, false) => 3,
            (Axis::Z, true) => 4,
            (Axis::Z, false) => 5,
        };
        Self(ROTATION_TABLE[self.0 as usize][column])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn solved_orientation_is_identity() {
        assert_eq!(Orientation::SOLVED.frame(), (Face::U, Face::F));
        for face in Face::ALL {
            assert_eq!(Orientation::SOLVED.remap_face(face), face);
        }
    }

    #[test]
    fn quarter_rotations_have_order_four() {
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            let mut orientation = Orientation::SOLVED;
            for _ in 0..4 {
                orientation = orientation.after_rotation(axis, Direction::CW);
            }
            assert_eq!(orientation, Orientation::SOLVED);
        }
    }

    #[test]
    fn x_y_z_generate_all_24_orientations() {
        let mut seen = [false; 24];
        let mut queue = VecDeque::from([Orientation::SOLVED]);
        seen[Orientation::SOLVED.index() as usize] = true;

        while let Some(current) = queue.pop_front() {
            for axis in [Axis::X, Axis::Y, Axis::Z] {
                let next = current.after_rotation(axis, Direction::CW);
                if !seen[next.index() as usize] {
                    seen[next.index() as usize] = true;
                    queue.push_back(next);
                }
            }
        }

        assert!(seen.into_iter().all(|value| value));
    }
}
