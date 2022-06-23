use super::*;

#[derive(Clone, Debug)]
pub struct Signal {
    direction: Direction,
    position: (usize, usize),
}

impl Signal {
    pub fn empty(position: (usize, usize), direction: Direction) -> Self {
        Self {
            direction,
            position,
        }
    }

    pub fn clone_move(&self, direction: Direction) -> Self {
        let mut res = self.clone();
        res.direction = direction;

        res
    }

    pub fn moved(mut self, direction: Direction) -> Self {
        self.direction = direction;

        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    pub(crate) fn set_position(&mut self, position: (usize, usize)) {
        self.position = position;
    }
}
