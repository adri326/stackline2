use serde::{Serialize, Deserialize};

/// Represents one or many undirected orientation(s), since we are in a 2D grid,
/// this may either be [Horizontal](Orientation::Horizontal), [Vertical](Orientation::Vertical) or both ([Any](Orientation::Any))
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Orientation {
    Horizontal,
    Vertical,
    Any,
}

/// Represents one directed orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Represents the state that a cell may be in. The usual state transition schema is `Idle → Active → Dormant → Idle`.
/// A tile will only be [`update`d](crate::Tile::update) if it is in the `Active` or `Dormant` state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum State {
    Idle,
    Active,
    Dormant,
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

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Any
    }
}

impl Direction {
    /// Converts a [Direction] in a pair `(Δx, Δy)`, with [Up](Direction::Up) being equal to `(0, -1)`
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

impl Default for Direction {
    fn default() -> Self {
        Direction::Up
    }
}

impl State {
    /// Rotates the state:
    /// - `Idle → Active`
    /// - `Active → Dormant`
    /// - `Dormant → Idle`
    pub fn next(self) -> Self {
        match self {
            State::Idle => State::Active,
            State::Active => State::Dormant,
            State::Dormant => State::Idle,
        }
    }

    /// Returns true if `Idle`
    pub fn accepts_signal(self) -> bool {
        match self {
            State::Idle => true,
            _ => false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Idle
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

    #[test]
    fn test_state_next_rotate_3() {
        let state = State::default();

        assert_eq!(state, State::Idle);
        assert!(state.accepts_signal());

        let state = state.next();
        assert_eq!(state, State::Active);
        assert!(!state.accepts_signal());

        let state = state.next();
        assert_eq!(state, State::Dormant);
        assert!(!state.accepts_signal());

        let state = state.next();
        assert_eq!(state, State::Idle);
    }
}
