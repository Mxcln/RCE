use core::fmt;

use rubiks_core::{Direction, Face, Move, MoveSequence};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Auf(u8);

impl Auf {
    pub const IDENTITY: Self = Self(0);
    pub const U: Self = Self(1);
    pub const U2: Self = Self(2);
    pub const U_PRIME: Self = Self(3);

    pub const ALL: [Self; 4] = [Self::IDENTITY, Self::U, Self::U2, Self::U_PRIME];

    pub fn try_from_index(index: u8) -> Option<Self> {
        (index < 4).then_some(Self(index))
    }

    pub const fn index(self) -> u8 {
        self.0
    }

    pub fn inverse(self) -> Self {
        match self.0 {
            0 => Self::IDENTITY,
            1 => Self::U_PRIME,
            2 => Self::U2,
            3 => Self::U,
            _ => unreachable!("Auf is always 0..=3"),
        }
    }

    pub fn compose(self, rhs: Self) -> Self {
        Self((self.0 + rhs.0) % 4)
    }

    pub fn to_move_sequence(self) -> MoveSequence {
        match self.0 {
            0 => MoveSequence(Vec::new()),
            1 => MoveSequence(vec![Move::new(Face::U, Direction::CW)]),
            2 => MoveSequence(vec![Move::new(Face::U, Direction::Double)]),
            3 => MoveSequence(vec![Move::new(Face::U, Direction::CCW)]),
            _ => unreachable!("Auf is always 0..=3"),
        }
    }

    pub fn to_notation(self) -> &'static str {
        match self.0 {
            0 => "I",
            1 => "U",
            2 => "U2",
            3 => "U'",
            _ => unreachable!("Auf is always 0..=3"),
        }
    }
}

impl fmt::Display for Auf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_notation())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_round_trips() {
        for auf in Auf::ALL {
            assert_eq!(auf.compose(auf.inverse()), Auf::IDENTITY);
            assert_eq!(auf.inverse().compose(auf), Auf::IDENTITY);
        }
    }

    #[test]
    fn notation_is_stable() {
        assert_eq!(Auf::IDENTITY.to_notation(), "I");
        assert_eq!(Auf::U.to_notation(), "U");
        assert_eq!(Auf::U2.to_notation(), "U2");
        assert_eq!(Auf::U_PRIME.to_notation(), "U'");
    }
}
