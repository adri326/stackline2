use super::*;
use std::collections::HashMap;

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

    pub fn get_pane<'b>(&'b self, name: &str) -> Option<&'b Pane> {
        self.panes.get(name)
    }

    pub fn get_pane_mut<'b>(&'b mut self, name: &str) -> Option<&'b mut Pane> {
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
