//! Transmission tiles: allow for inter-Pane communication

use crate::prelude::*;
use crate::tile::prelude::*;

/// Instantly sends any incomming signals to `coordinates`
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
    pub fn calculate_path(&mut self, origin: (i32, i32), world: &World) {
        use pathfinding::directed::astar::astar;

        // A* search
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        struct Pos(i32, i32);

        impl Pos {
            fn neighbors(&self, world: &World) -> [(Pos, i32); 4] {
                let x = self.0;
                let y = self.1;

                [
                    Pos(x + 1, y).with_weight(world),
                    Pos(x - 1, y).with_weight(world),
                    Pos(x, y + 1).with_weight(world),
                    Pos(x, y - 1).with_weight(world),
                ]
            }

            fn with_weight(self, world: &World) -> (Self, i32) {
                if world.in_pane(self.0, self.1) {
                    (self, 100)
                } else {
                    (self, 1)
                }
            }

            fn heuristic(&self, target: Pos) -> i32 {
                (self.0 - target.0).abs() + (self.1 - target.1).abs()
            }
        }

        impl From<Pos> for (i32, i32) {
            fn from(pos: Pos) -> (i32, i32) {
                (pos.0, pos.1)
            }
        }

        impl From<&Pos> for (i32, i32) {
            fn from(pos: &Pos) -> (i32, i32) {
                (pos.0, pos.1)
            }
        }

        if let Some(pane) = world.get_pane(&self.coordinates.0) {
            let target = Pos(
                pane.position().0 + self.coordinates.1 as i32,
                pane.position().1 + self.coordinates.2 as i32,
            );

            if let Some((best_path, _)) = astar(
                &Pos(origin.0, origin.1),
                |node| node.neighbors(world),
                |node| node.heuristic(target),
                |&node| node == target,
            ) {
                self.path = Vec::new();
                self.path.push((best_path[0].0, best_path[0].1));

                for (prev, current) in best_path.iter().zip(best_path.iter().skip(1)).skip(1) {
                    // If self.path.last(), prev, current aren't aligned, push prev to self.path
                    let prev_x_aligned = self.path[self.path.len() - 1].0 == prev.0;
                    let curr_x_aligned = prev.0 == current.0;
                    if prev_x_aligned != curr_x_aligned {
                        self.path.push(prev.into());
                    }
                }

                if self.path[self.path.len() - 1] != best_path[best_path.len() - 1].into() {
                    self.path.push(best_path[best_path.len() - 1].into());
                }

                self.length = best_path.len() - 1;
            }
        }
    }
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
    use veccell::{VecRef, VecRefMut};

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

            let sender: VecRef<'_, Sender> = world
                .get_pane("main")
                .unwrap()
                .get_as::<Sender>((0, 0))
                .unwrap();

            assert!(sender.signals.len() == 1);
            assert!(sender.signals[0].1 == n);

            drop(sender);

            assert_no_signal!(world.get_pane("main").unwrap(), (0, 0));
            assert_no_signal!(world.get_pane("main").unwrap(), (0, 2));
        }

        world.step();

        let sender: VecRef<'_, Sender> = world
            .get_pane("main")
            .unwrap()
            .get_as::<Sender>((0, 0))
            .unwrap();

        assert!(sender.signals.len() == 0);

        assert_no_signal!(world.get_pane("main").unwrap(), (0, 0));
        assert_signal!(world.get_pane("main").unwrap(), (0, 2));
    }

    #[test]
    fn test_sender_pathfinding() {
        use crate::Wire;

        let mut main_pane = test_tile_setup!(1, 1, [Sender::new(String::from("second"), 0, 0),]);

        main_pane.set_position((0, 0));

        let mut second_pane = test_tile_setup!(1, 1, [Wire::new(Orientation::Any)]);

        second_pane.set_position((2, 0));

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);
        world.set_pane(String::from("second"), second_pane);

        let mut tile = world
            .get_pane("main")
            .unwrap()
            .borrow_mut_as::<Sender>((0, 0))
            .unwrap();

        tile.calculate_path((0, 0), &world);

        assert_eq!(tile.path, [(0, 0), (2, 0)]);
        assert_eq!(tile.length, 2);

        drop(tile);

        world.get_pane_mut("second").unwrap().set_position((2, 2));

        let mut tile = world
            .get_pane("main")
            .unwrap()
            .borrow_mut_as::<Sender>((0, 0))
            .unwrap();

        tile.calculate_path((0, 0), &world);

        assert!(tile.path == [(0, 0), (2, 0), (2, 2)] || tile.path == [(0, 0), (0, 2), (2, 2)]);
        assert_eq!(tile.length, 4);
    }

    #[test]
    fn test_sender_pathfinding_penalty() {
        use crate::Wire;

        let mut main_pane = test_tile_setup!(2, 4, [
            Sender::new(String::from("second"), 0, 0),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
        ]);
        main_pane.set_position((0, 0));

        let mut obstacle_pane = test_tile_setup!(4, 1, [(), (), (), ()]);
        obstacle_pane.set_position((2, 1));

        let mut second_pane = test_tile_setup!(1, 1, [Wire::new(Orientation::Any)]);
        second_pane.set_position((2, 2));

        let mut world = World::new();
        world.set_pane(String::from("main"), main_pane);
        world.set_pane(String::from("obstacle"), obstacle_pane);
        world.set_pane(String::from("second"), second_pane);

        let mut tile = world
            .get_pane("main")
            .unwrap()
            .borrow_mut_as::<Sender>((0, 0))
            .unwrap();

        tile.calculate_path((0, 0), &world);

        assert_eq!(tile.path, [
            (0, 0),
            (-1, 0),
            (-1, 4),
            (2, 4),
            (2, 2)
        ]);
    }
}
