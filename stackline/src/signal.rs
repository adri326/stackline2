use super::*;

#[derive(Clone, Debug)]
pub struct Signal {
    pub direction: Direction,
}

impl Signal {
    pub fn clone_with_dir(&self, direction: Direction) -> Self {
        let mut res = self.clone();
        res.direction = direction;
        res
    }
}
