//! Transmission tiles: allow for inter-Pane communication

use crate::prelude::*;

/// Instantly sends any incomming signals to `coordinates`
#[derive(Clone, Debug)]
pub struct Teleporter {
    pub coordinates: (String, usize, usize),
}

impl Teleporter {
    pub fn new(name: String, x: usize, y: usize) -> Self {
        Self {
            coordinates: (name, x, y),
        }
    }
}

impl Tile for Teleporter {
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

/// Sends a signal through a virtual wire towards `coordinates`.
#[derive(Clone, Debug)]
pub struct Sender {
    pub coordinates: (String, usize, usize),
    pub path: Vec<(i32, i32)>, // x, y
    pub length: usize,
    pub signals: Vec<(Signal, usize)>,
}

impl Sender {
    pub fn new(name: String, x: usize, y: usize) -> Self {
        Self {
            coordinates: (name, x, y),
            path: Vec::new(),
            length: 0,
            signals: Vec::new(),
        }
    }

    // TODO: implement WorldMask, calculate_path and a method of Tile to call this method automatically
    // pub fn calculate_path(&mut self, mask: &WorldMask) {
    //
    // }
}

impl Tile for Sender {
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        let mut needs_sending = false;

        for (_signal, ref mut time) in self.signals.iter_mut() {
            *time += 1;

            if *time >= self.length {
                needs_sending = true;
            }
        }

        if let Some(signal) = context.take_signal() {
            self.signals.push((signal, 0));
            if self.length == 0 {
                needs_sending = true;
            }
        }

        if needs_sending {
            for (signal, _time) in self.signals.drain_filter(|(_, time)| *time >= self.length) {
                context.send_outbound(self.coordinates.clone(), signal);
            }
        }

        if context.state() == State::Active {
            context.next_state();
        } else if context.state() == State::Dormant && self.signals.len() == 0 {
            context.next_state();
        }
    }

    // TODO: read self.signals to determine the state of each char
    // TODO: automated test
    fn draw(&self, x: usize, y: usize, _state: State, surface: &mut TextSurface) {
        for (prev, next) in self.path.iter().zip(self.path.iter().skip(1)) {
            if prev.0 != next.0 {
                // Draw the diode of the corner
                let ch = if next.0 > prev.0 { '>' } else { '<' };
                surface.set(
                    (x as i32 + prev.0) as usize,
                    (y as i32 + prev.1) as usize,
                    TextChar::from_state(ch, State::Idle),
                );

                // Draw the horizontal line

                for dx in (prev.0 + 1)..(next.0) {
                    surface.set(
                        (x as i32 + dx) as usize,
                        (y as i32 + prev.1) as usize,
                        TextChar::from_state('-', State::Idle),
                    );
                }
            } else {
                // Draw the diode of the corner
                let ch = if next.1 > prev.1 { 'v' } else { '^' };
                surface.set(
                    (x as i32 + prev.0) as usize,
                    (y as i32 + prev.1) as usize,
                    TextChar::from_state(ch, State::Idle),
                );

                // Draw the vertical line

                for dy in (prev.1 + 1)..(next.1) {
                    surface.set(
                        (y as i32 + prev.0) as usize,
                        (y as i32 + dy) as usize,
                        TextChar::from_state('|', State::Idle),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use veccell::VecRef;

    #[test]
    fn test_teleporter_transmit_same_pane() {
        use crate::{Diode, Wire};
        use Direction::*;
        use Orientation::*;

        let mut main_pane = test_tile_setup!(
            3,
            3,
            [
                Diode::new(Right),
                Teleporter::new(String::from("main"), 2, 2),
                (),
                (),
                (),
                (),
                (),
                (),
                Wire::new(Any)
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
    fn test_teleporter_transmit_other_pane() {
        use crate::{Diode, Wire};
        use Direction::*;
        use Orientation::*;

        let mut main_pane = test_tile_setup!(
            2,
            1,
            [
                Diode::new(Right),
                Teleporter::new(String::from("sub"), 0, 0),
            ]
        );

        test_set_signal!(main_pane, (0, 0), Right);

        let sub_pane = test_tile_setup!(1, 1, [Wire::new(Any),]);

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
    fn test_teleporter_transmit_self() {
        use Direction::*;

        let mut main_pane = test_tile_setup!(1, 1, [Teleporter::new(String::from("main"), 0, 0),]);

        test_set_signal!(main_pane, (0, 0), Right);

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);

        for _ in 0..5 {
            world.step();

            assert_signal!(world.get_pane("main").unwrap(), (0, 0));
        }
    }

    #[test]
    fn test_sender_instantaneous() {
        use crate::Wire;
        use Direction::*;
        use Orientation::*;

        let mut main_pane = test_tile_setup!(
            1,
            3,
            [Sender::new(String::from("main"), 0, 2), (), Wire::new(Any)]
        );

        test_set_signal!(main_pane, (0, 0), Right);

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);

        world.step();

        assert_signal!(world.get_pane("main").unwrap(), (0, 2));
    }

    #[test]
    fn test_sender_delay() {
        use crate::Wire;
        use Direction::*;
        use Orientation::*;

        let mut sender = Sender::new(String::from("main"), 0, 2);
        sender.length = 2;

        let mut main_pane = test_tile_setup!(1, 3, [sender, (), Wire::new(Any)]);

        test_set_signal!(main_pane, (0, 0), Right);

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);

        for n in 0..2 {
            world.step();

            // TODO: pane.get_as::<Sender>(coords)
            let sender: VecRef<'_, Sender> = VecRef::map(
                world.get_pane("main").unwrap().get((0, 0)).unwrap(),
                |tile| tile.get().unwrap().try_into().unwrap(),
            );

            assert!(sender.signals.len() == 1);
            assert!(sender.signals[0].1 == n);

            drop(sender);

            assert_no_signal!(world.get_pane("main").unwrap(), (0, 0));
            assert_no_signal!(world.get_pane("main").unwrap(), (0, 2));
        }

        world.step();

        let sender: VecRef<'_, Sender> = VecRef::map(
            world.get_pane("main").unwrap().get((0, 0)).unwrap(),
            |tile| tile.get().unwrap().try_into().unwrap(),
        );

        assert!(sender.signals.len() == 0);

        assert_no_signal!(world.get_pane("main").unwrap(), (0, 0));
        assert_signal!(world.get_pane("main").unwrap(), (0, 2));
    }
}
