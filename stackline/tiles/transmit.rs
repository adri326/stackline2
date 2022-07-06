//! Transmission tiles: allow for inter-Pane communication

use crate::prelude::*;

/// Instantly sends any incomming signals to `coordinates`
#[derive(Clone, Debug)]
pub struct Portal {
    pub coordinates: (String, usize, usize),
}

impl Portal {
    pub fn new(name: String, x: usize, y: usize) -> Self {
        Self {
            coordinates: (name, x, y)
        }
    }
}

impl Tile for Portal {
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        if let Some(signal) = context.take_signal() {
            context.send_outbound(self.coordinates.clone(), signal);
        }

        if context.state() != State::Idle {
            context.next_state();
        }
    }

    fn draw(&self, x: usize, y: usize, state: State, surface: &mut TextSurface) {
        surface.set(x, y, TextChar::from_state('P', state));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_portal_transmit_same_pane() {
        use crate::{Diode, Wire};
        use Direction::*;
        use Orientation::*;

        let mut main_pane = test_tile_setup!(
            3,
            3,
            [
                Diode::new(Right), Portal::new(String::from("main"), 2, 2), (),
                (), (), (),
                (), (), Wire::new(Any)
            ]
        );

        test_set_signal!(main_pane, (0, 0), Right);

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);

        world.step();

        assert_signal!(world.get_pane("main").unwrap(), (1, 0));

        world.step();

        assert_no_signal!(world.get_pane("main").unwrap(), (1, 0));
        assert_signal!(world.get_pane("main").unwrap(), (2, 2));

        world.step();

        assert_no_signal!(world.get_pane("main").unwrap(), (0, 0));
        assert_no_signal!(world.get_pane("main").unwrap(), (1, 0));
        assert_no_signal!(world.get_pane("main").unwrap(), (2, 2));
    }

    #[test]
    fn test_portal_transmit_other_pane() {
        use crate::{Diode, Wire};
        use Direction::*;
        use Orientation::*;

        let mut main_pane = test_tile_setup!(
            2,
            1,
            [
                Diode::new(Right), Portal::new(String::from("sub"), 0, 0),
            ]
        );

        test_set_signal!(main_pane, (0, 0), Right);

        let sub_pane = test_tile_setup!(
            1,
            1,
            [
                Wire::new(Any),
            ]
        );

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);
        world.set_pane(String::from("sub"), sub_pane);

        world.step();

        assert_signal!(world.get_pane("main").unwrap(), (1, 0));

        world.step();

        assert_no_signal!(world.get_pane("main").unwrap(), (1, 0));
        assert_signal!(world.get_pane("sub").unwrap(), (0, 0));

        world.step();

        assert_no_signal!(world.get_pane("main").unwrap(), (0, 0));
        assert_no_signal!(world.get_pane("main").unwrap(), (1, 0));
        assert_no_signal!(world.get_pane("sub").unwrap(), (0, 0));
    }

    #[test]
    fn test_portal_transmit_self() {
        use Direction::*;

        let mut main_pane = test_tile_setup!(
            1,
            1,
            [
                Portal::new(String::from("main"), 0, 0),
            ]
        );

        test_set_signal!(main_pane, (0, 0), Right);

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);

        for _ in 0..5 {
            world.step();

            assert_signal!(world.get_pane("main").unwrap(), (0, 0));
        }
    }
}
