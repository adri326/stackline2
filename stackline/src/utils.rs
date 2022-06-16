/// Represents one or many undirected orientation(s), since we are in a 2D grid, this may either be [Horizontal], [Vertical] or both ([Any])
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Orientation {
    Horizontal,
    Vertical,
    Any,
}

/// Represents one directed orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

const HORIZONTAL: [Direction; 2] = [Direction::Left, Direction::Right];
const VERTICAL: [Direction; 2] = [Direction::Up, Direction::Down];
const ANY: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

impl Orientation {
    /// Converts an [Orientation] into an array of [Direction]s
    #[inline]
    pub fn into_directions(self) -> &'static [Direction] {
        match self {
            Orientation::Horizontal => &HORIZONTAL,
            Orientation::Vertical => &VERTICAL,
            Orientation::Any => &ANY,
        }
    }

    /// Returns true iff `dir ∈ self`
    #[inline]
    pub fn contains(&self, dir: Direction) -> bool {
        match (self, dir) {
            (Orientation::Vertical, Direction::Up) => true,
            (Orientation::Vertical, Direction::Down) => true,
            (Orientation::Horizontal, Direction::Left) => true,
            (Orientation::Horizontal, Direction::Right) => true,
            (Orientation::Any, _) => true,
            _ => false,
        }
    }
}

impl Direction {
    /// Converts a [Direction] in a pair `(Δx, Δy)`, with [Up] being equal to `(0, -1)`
    #[inline]
    pub fn into_offset(self) -> (i8, i8) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// Returns the opposite direction
    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_orientation_contains_self() {
        let list = [
            Orientation::Horizontal,
            Orientation::Vertical,
            Orientation::Any,
        ];
        for orientation in list.into_iter() {
            for direction in orientation.into_directions() {
                assert!(orientation.contains(*direction));
            }
        }
    }
}
