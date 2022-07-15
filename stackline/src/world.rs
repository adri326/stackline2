use super::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use veccell::{VecRef, VecRefMut};

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    panes: HashMap<String, Pane>,
}

impl World {
    pub fn new() -> Self {
        Self {
            panes: HashMap::new(),
        }
    }

    pub fn step(&mut self) {
        let mut outbound_signals = Vec::new();

        for pane in self.panes.values_mut() {
            let mut res = pane.step();
            outbound_signals.append(&mut res.outbound_signals);
        }

        for ((name, x, y), signal) in outbound_signals {
            if let Some(pane) = self.get_pane_mut(&name) {
                let _ = pane.set_signal((x, y), signal); // Errors are ignored
            }
        }
    }

    pub fn set_pane(&mut self, name: String, pane: Pane) {
        self.panes.insert(name, pane);
    }

    pub fn get(&self, (x, y): (i32, i32)) -> Option<VecRef<'_, FullTile>> {
        for pane in self.panes.values() {
            let x2 = x - pane.position().0;
            let y2 = y - pane.position().1;
            if x2 >= 0 && x2 < pane.width().get() as i32 && y2 >= 0 && y2 < pane.height().get() as i32 {
                let x2 = x2 as usize;
                let y2 = y2 as usize;
                if let Some(tile) = pane.get((x2, y2)) {
                    return Some(tile);
                }
            }
        }
        None
    }

    pub fn get_with_pos(&self, (x, y): (i32, i32)) -> Option<(VecRef<'_, FullTile>, usize, usize)> {
        for pane in self.panes.values() {
            let x2 = x - pane.position().0;
            let y2 = y - pane.position().1;
            if x2 >= 0 && x2 < pane.width().get() as i32 && y2 >= 0 && y2 < pane.height().get() as i32 {
                let x2 = x2 as usize;
                let y2 = y2 as usize;
                if let Some(tile) = pane.get((x2, y2)) {
                    return Some((tile, x2, y2));
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, (x, y): (i32, i32)) -> Option<&mut FullTile> {
        for pane in self.panes.values_mut() {
            let x2 = x - pane.position().0;
            let y2 = y - pane.position().1;
            if x2 >= 0 && x2 < pane.width().get() as i32 && y2 >= 0 && y2 < pane.height().get() as i32 {
                let x2 = x2 as usize;
                let y2 = y2 as usize;
                if let Some(tile) = pane.get_mut((x2, y2)) {
                    return Some(tile);
                }
            }
        }
        None
    }

    pub fn get_mut_with_pos(&mut self, (x, y): (i32, i32)) -> Option<(&mut FullTile, usize, usize)> {
        for pane in self.panes.values_mut() {
            let x2 = x - pane.position().0;
            let y2 = y - pane.position().1;
            if x2 >= 0 && x2 < pane.width().get() as i32 && y2 >= 0 && y2 < pane.height().get() as i32 {
                let x2 = x2 as usize;
                let y2 = y2 as usize;
                if let Some(tile) = pane.get_mut((x2, y2)) {
                    return Some((tile, x2, y2));
                }
            }
        }
        None
    }

    pub fn get_pane(&self, name: &str) -> Option<&Pane> {
        self.panes.get(name)
    }

    pub fn get_pane_mut(&mut self, name: &str) -> Option<&mut Pane> {
        self.panes.get_mut(name)
    }

    pub fn in_pane(&self, x: i32, y: i32) -> bool {
        for pane in self.panes.values() {
            if
                x >= pane.position().0
                && y >= pane.position().1
                && x < pane.position().0 + pane.width().get() as i32
                && y < pane.position().1 + pane.height().get() as i32
            {
                return true;
            }
        }

        false
    }

    pub fn draw(&self, dx: i32, dy: i32, surface: &mut TextSurface) {
        for pane in self.panes.values() {
            pane.draw(dx, dy, surface);
        }
    }

    pub fn get_bounds(&self) -> (i32, i32, i32, i32) {
        self.panes.values().fold((0, 0, 0, 0), |acc, act| {
            (
                acc.0.min(act.position().0),
                acc.1.max(act.position().0 + act.width().get() as i32),
                acc.2.min(act.position().1),
                acc.3.max(act.position().1 + act.height().get() as i32),
            )
        })
    }

    pub fn panes(&self) -> &HashMap<String, Pane> {
        &self.panes
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bounds = self.get_bounds();
        let width = (bounds.1 - bounds.0) as usize;
        let height = (bounds.3 - bounds.2) as usize;

        let mut surface = TextSurface::new(width, height);

        for pane in self.panes.values() {
            pane.draw(bounds.0, bounds.2, &mut surface);
        }

        <TextSurface as std::fmt::Display>::fmt(&surface, f)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_world_set_pane() {
        let mut world = World::new();

        world.set_pane(String::from("main"), Pane::empty(10, 10).unwrap());

        assert!(world.get_pane("main").is_some());
    }
}
