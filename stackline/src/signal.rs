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
            position
        }
    }

    pub fn clone_move(&self, direction: Direction) -> Option<Self> {
        let mut res = self.clone();
        res.direction = direction;

        let (dx, dy) = direction.into_offset();

        res.position.0 = (res.position.0 as isize + dx as isize).try_into().ok()?;
        res.position.1 = (res.position.1 as isize + dy as isize).try_into().ok()?;

        Some(res)
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn position(&self) -> (usize, usize) {
        self.position
    }
}
