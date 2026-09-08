use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Piece {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Piece {
    pub const ALL: [Self; 7] = [
        Self::I,
        Self::O,
        Self::T,
        Self::S,
        Self::Z,
        Self::J,
        Self::L,
    ];

    pub fn size(self) -> usize {
        match self {
            Self::I => 4,
            Self::O => 2,
            _ => 3,
        }
    }

    pub fn cells(self, rotation: u8) -> [(i32, i32); 4] {
        let mut cells = match self {
            Self::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
            Self::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Self::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
            Self::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
            Self::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
            Self::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
            Self::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        };
        if self != Self::O {
            for _ in 0..rotation % 4 {
                for (x, y) in &mut cells {
                    (*x, *y) = (self.size() as i32 - 1 - *y, *x);
                }
            }
        }
        cells
    }
}

/// Ordered kick offsets in Cartesian coordinates: positive y points upward.
/// Board coordinates point downward; the caller subtracts the y offset.
pub fn kicks(piece: Piece, from: u8, to: u8) -> &'static [(i32, i32)] {
    let (from, to) = (from % 4, to % 4);
    if piece == Piece::O {
        return &[(0, 0)];
    }
    if (to + 4 - from) % 4 == 2 {
        return match from {
            0 => &[(0, 0), (0, 1), (1, 1), (-1, 1), (1, 0), (-1, 0)],
            2 => &[(0, 0), (0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)],
            1 => &[(0, 0), (1, 0), (1, 2), (1, 1), (0, 2), (0, 1)],
            _ => &[(0, 0), (-1, 0), (-1, 2), (-1, 1), (0, 2), (0, 1)],
        };
    }
    if piece == Piece::I {
        return match (from, to) {
            (0, 1) => &[(0, 0), (1, 0), (-2, 0), (-2, -1), (1, 2)],
            (0, 3) => &[(0, 0), (-1, 0), (2, 0), (2, -1), (-1, 2)],
            (1, 0) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
            (1, 2) => &[(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            (2, 1) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
            (2, 3) => &[(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (3, 2) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
            (3, 0) => &[(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            _ => &[(0, 0)],
        };
    }
    match (from, to) {
        (0, 1) | (2, 1) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        (1, 0) | (1, 2) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        (2, 3) | (0, 3) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        (3, 2) | (3, 0) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        _ => &[(0, 0)],
    }
}
