use core::fmt;

use crate::color::Color;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    U = 0,
    R = 1,
    F = 2,
    D = 3,
    L = 4,
    B = 5,
}

impl Face {
    pub const ALL: [Self; 6] = [Self::U, Self::R, Self::F, Self::D, Self::L, Self::B];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn as_char(self) -> char {
        match self {
            Self::U => 'U',
            Self::R => 'R',
            Self::F => 'F',
            Self::D => 'D',
            Self::L => 'L',
            Self::B => 'B',
        }
    }

    pub(crate) const fn as_lower_char(self) -> char {
        match self {
            Self::U => 'u',
            Self::R => 'r',
            Self::F => 'f',
            Self::D => 'd',
            Self::L => 'l',
            Self::B => 'b',
        }
    }

    pub(crate) const fn from_index(index: u8) -> Self {
        match index {
            0 => Self::U,
            1 => Self::R,
            2 => Self::F,
            3 => Self::D,
            4 => Self::L,
            5 => Self::B,
            _ => panic!("invalid face index"),
        }
    }

    pub(crate) fn from_upper(ch: char) -> Option<Self> {
        match ch {
            'U' => Some(Self::U),
            'R' => Some(Self::R),
            'F' => Some(Self::F),
            'D' => Some(Self::D),
            'L' => Some(Self::L),
            'B' => Some(Self::B),
            _ => None,
        }
    }

    pub(crate) fn from_lower(ch: char) -> Option<Self> {
        match ch {
            'u' => Some(Self::U),
            'r' => Some(Self::R),
            'f' => Some(Self::F),
            'd' => Some(Self::D),
            'l' => Some(Self::L),
            'b' => Some(Self::B),
            _ => None,
        }
    }

    pub(crate) const fn solved_color(self) -> Color {
        match self {
            Self::U => Color::U,
            Self::R => Color::R,
            Self::F => Color::F,
            Self::D => Color::D,
            Self::L => Color::L,
            Self::B => Color::B,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    CW = 0,
    CCW = 1,
    Double = 2,
}

impl Direction {
    pub const fn inverse(self) -> Self {
        match self {
            Self::CW => Self::CCW,
            Self::CCW => Self::CW,
            Self::Double => Self::Double,
        }
    }

    pub(crate) const fn quarter_turns(self) -> usize {
        match self {
            Self::CW => 1,
            Self::Double => 2,
            Self::CCW => 3,
        }
    }

    pub(crate) const fn suffix(self) -> &'static str {
        match self {
            Self::CW => "",
            Self::CCW => "'",
            Self::Double => "2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub face: Face,
    pub dir: Direction,
}

impl Move {
    pub const fn new(face: Face, dir: Direction) -> Self {
        Self { face, dir }
    }

    pub const fn inverse(self) -> Self {
        Self {
            face: self.face,
            dir: self.dir.inverse(),
        }
    }

    pub fn to_notation(self) -> String {
        format!("{}{}", self.face.as_char(), self.dir.suffix())
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_notation())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MoveSequence(pub Vec<Move>);

impl MoveSequence {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn inverse(&self) -> Self {
        Self(self.0.iter().rev().copied().map(Move::inverse).collect())
    }

    pub fn to_notation(&self) -> String {
        self.0
            .iter()
            .map(|mv| mv.to_notation())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl From<Vec<Move>> for MoveSequence {
    fn from(value: Vec<Move>) -> Self {
        Self(value)
    }
}

impl fmt::Display for MoveSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_notation())
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slice {
    M = 0,
    S = 1,
    E = 2,
}

impl Slice {
    pub(crate) fn from_char(ch: char) -> Option<Self> {
        match ch {
            'M' => Some(Self::M),
            'S' => Some(Self::S),
            'E' => Some(Self::E),
            _ => None,
        }
    }

    pub(crate) const fn as_char(self) -> char {
        match self {
            Self::M => 'M',
            Self::S => 'S',
            Self::E => 'E',
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub(crate) fn from_char(ch: char) -> Option<Self> {
        match ch {
            'x' | 'X' => Some(Self::X),
            'y' | 'Y' => Some(Self::Y),
            'z' | 'Z' => Some(Self::Z),
            _ => None,
        }
    }

    pub(crate) const fn as_char(self) -> char {
        match self {
            Self::X => 'x',
            Self::Y => 'y',
            Self::Z => 'z',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExtMove {
    Face(Face, Direction),
    Slice(Slice, Direction),
    Rotation(Axis, Direction),
    Wide(Face, Direction),
}

impl ExtMove {
    pub fn to_notation(self) -> String {
        match self {
            Self::Face(face, dir) => format!("{}{}", face.as_char(), dir.suffix()),
            Self::Slice(slice, dir) => format!("{}{}", slice.as_char(), dir.suffix()),
            Self::Rotation(axis, dir) => format!("{}{}", axis.as_char(), dir.suffix()),
            Self::Wide(face, dir) => format!("{}{}", face.as_lower_char(), dir.suffix()),
        }
    }
}

impl fmt::Display for ExtMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_notation())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ExtMoveSequence(pub Vec<ExtMove>);

impl ExtMoveSequence {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_notation(&self) -> String {
        self.0
            .iter()
            .map(|mv| mv.to_notation())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl From<Vec<ExtMove>> for ExtMoveSequence {
    fn from(value: Vec<ExtMove>) -> Self {
        Self(value)
    }
}

impl fmt::Display for ExtMoveSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_notation())
    }
}
