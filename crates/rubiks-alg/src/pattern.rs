use core::fmt;

use rubiks_core::CubeStateParts;

use crate::auf::Auf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OllPattern(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PllPattern(u16);

impl OllPattern {
    pub fn from_key(key: u16) -> Option<Self> {
        (key < 1296).then_some(Self(key))
    }

    pub const fn key(self) -> u16 {
        self.0
    }

    pub fn rotate_u(self, auf: Auf) -> Self {
        let (mut corners, mut edges) = self.decode();
        rotate_left_4(&mut corners, auf.index() as usize);
        rotate_left_4(&mut edges, auf.index() as usize);
        Self::from_components(corners, edges)
    }

    pub fn from_parts(parts: &CubeStateParts) -> Self {
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
        Self::from_components(corners, edges)
    }

    pub fn from_components(corners: [u8; 4], edges: [u8; 4]) -> Self {
        let corner_code = corners[0] as u16
            + 3 * corners[1] as u16
            + 9 * corners[2] as u16
            + 27 * corners[3] as u16;
        let edge_code =
            edges[0] as u16 + 2 * edges[1] as u16 + 4 * edges[2] as u16 + 8 * edges[3] as u16;
        Self(corner_code * 16 + edge_code)
    }

    fn decode(self) -> ([u8; 4], [u8; 4]) {
        let corner_code = self.0 / 16;
        let edge_code = self.0 % 16;
        (
            [
                (corner_code % 3) as u8,
                ((corner_code / 3) % 3) as u8,
                ((corner_code / 9) % 3) as u8,
                ((corner_code / 27) % 3) as u8,
            ],
            [
                (edge_code % 2) as u8,
                ((edge_code / 2) % 2) as u8,
                ((edge_code / 4) % 2) as u8,
                ((edge_code / 8) % 2) as u8,
            ],
        )
    }
}

impl fmt::Display for OllPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (corners, edges) = self.decode();
        write!(f, "OLL(corners={corners:?}, edges={edges:?})")
    }
}

impl PllPattern {
    pub fn from_key(key: u16) -> Option<Self> {
        (key < 576).then_some(Self(key))
    }

    pub const fn key(self) -> u16 {
        self.0
    }

    pub fn rotate_u(self, auf: Auf) -> Self {
        let (mut corners, mut edges) = self.decode();
        rotate_left_4(&mut corners, auf.index() as usize);
        rotate_left_4(&mut edges, auf.index() as usize);
        Self::from_components(corners, edges)
    }

    pub fn from_parts(parts: &CubeStateParts) -> Self {
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
        Self::from_components(corners, edges)
    }

    pub fn from_components(corners: [u8; 4], edges: [u8; 4]) -> Self {
        let corner_rank = lehmer_rank(corners) as u16;
        let edge_rank = lehmer_rank(edges) as u16;
        Self(corner_rank * 24 + edge_rank)
    }

    fn decode(self) -> ([u8; 4], [u8; 4]) {
        let corner_rank = (self.0 / 24) as usize;
        let edge_rank = (self.0 % 24) as usize;
        (lehmer_unrank(corner_rank), lehmer_unrank(edge_rank))
    }
}

impl fmt::Display for PllPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (corners, edges) = self.decode();
        write!(f, "PLL(corners={corners:?}, edges={edges:?})")
    }
}

fn rotate_left_4<T: Copy>(array: &mut [T; 4], amount: usize) {
    let amount = amount % 4;
    if amount == 0 {
        return;
    }
    let original = *array;
    for index in 0..4 {
        array[index] = original[(index + amount) % 4];
    }
}

fn lehmer_rank(values: [u8; 4]) -> usize {
    let mut rank = 0usize;
    for i in 0..4 {
        let mut smaller = 0usize;
        for j in (i + 1)..4 {
            if values[j] < values[i] {
                smaller += 1;
            }
        }
        rank += smaller * factorial(3 - i);
    }
    rank
}

fn lehmer_unrank(mut rank: usize) -> [u8; 4] {
    let mut items = vec![0u8, 1, 2, 3];
    let mut output = [0u8; 4];
    for (index, slot) in output.iter_mut().enumerate() {
        let base = factorial(3 - index);
        let selected = rank / base;
        rank %= base;
        *slot = items.remove(selected);
    }
    output
}

const fn factorial(n: usize) -> usize {
    match n {
        0 | 1 => 1,
        2 => 2,
        3 => 6,
        _ => 1,
    }
}

pub fn is_f2l_solved(parts: &CubeStateParts) -> bool {
    for index in 4..8 {
        if parts.corner_perm[index] != index as u8 || parts.corner_orient[index] != 0 {
            return false;
        }
    }

    for index in 4..12 {
        if parts.edge_perm[index] != index as u8 || parts.edge_orient[index] != 0 {
            return false;
        }
    }

    permutation_is_subset(parts.corner_perm[..4].iter().copied(), 4)
        && permutation_is_subset(parts.edge_perm[..4].iter().copied(), 4)
}

pub fn is_oll_solved(parts: &CubeStateParts) -> bool {
    if !is_f2l_solved(parts) {
        return false;
    }
    parts.corner_orient[..4].iter().all(|&value| value == 0)
        && parts.edge_orient[..4].iter().all(|&value| value == 0)
}

fn permutation_is_subset(iter: impl Iterator<Item = u8>, upper: usize) -> bool {
    let mut seen = [false; 4];
    for value in iter {
        let index = value as usize;
        if index >= upper || seen[index] {
            return false;
        }
        seen[index] = true;
    }
    true
}

#[cfg(test)]
mod tests {
    use rubiks_core::{CubeState, Direction, Face, Move};

    use super::*;

    #[test]
    fn oll_pattern_round_trips() {
        for key in [0, 15, 398, 1295] {
            let pattern = OllPattern::from_key(key).unwrap();
            assert_eq!(pattern.key(), key);
        }
    }

    #[test]
    fn pll_pattern_round_trips() {
        for key in [0, 23, 57, 575] {
            let pattern = PllPattern::from_key(key).unwrap();
            assert_eq!(pattern.key(), key);
        }
    }

    #[test]
    fn f2l_and_oll_checks_work_for_solved() {
        let parts = CubeState::solved().parts();
        assert!(is_f2l_solved(&parts));
        assert!(is_oll_solved(&parts));
    }

    #[test]
    fn f2l_check_rejects_non_f2l_move() {
        let mut cube = CubeState::solved();
        cube.apply_move(Move::new(Face::R, Direction::CW));
        assert!(!is_f2l_solved(&cube.parts()));
    }

    #[test]
    fn oll_check_rejects_last_layer_orientation_changes() {
        let mut parts = CubeState::solved().parts();
        parts.corner_orient[0] = 1;
        parts.corner_orient[1] = 2;
        let cube = CubeState::try_from_parts(parts).unwrap();
        assert!(is_f2l_solved(&cube.parts()));
        assert!(!is_oll_solved(&cube.parts()));
    }

    #[test]
    fn rotate_u_for_oll_rotates_slots() {
        let pattern = OllPattern::from_components([0, 1, 2, 0], [0, 1, 0, 1]);
        let rotated = pattern.rotate_u(Auf::U);
        assert_eq!(
            rotated.to_string(),
            "OLL(corners=[1, 2, 0, 0], edges=[1, 0, 1, 0])"
        );
    }

    #[test]
    fn rotate_u_for_pll_rotates_slots() {
        let pattern = PllPattern::from_components([0, 1, 2, 3], [1, 2, 3, 0]);
        let rotated = pattern.rotate_u(Auf::U2);
        assert_eq!(
            rotated.to_string(),
            "PLL(corners=[2, 3, 0, 1], edges=[3, 0, 1, 2])"
        );
    }

    #[test]
    fn lehmer_rank_unrank_is_stable() {
        for rank in 0..24 {
            let perm = lehmer_unrank(rank);
            assert_eq!(lehmer_rank(perm), rank);
        }
    }
}
