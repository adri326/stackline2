//! Wires and diodes

use crate::tile::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Wire(Orientation);

impl Wire {
    pub fn new(orientation: Orientation) -> Self {
        Self(orientation)
    }
}

impl Tile for Wire {
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        if let Some(signal) = context.take_signal() {
            for &direction in self.0.into_directions() {
                if direction == signal.direction().opposite() {
                    continue;
                }

                if let Some(pos) = context.accepts_direction(direction) {
                    context.force_send(pos, signal.clone_move(direction)).unwrap_or_else(|_| unreachable!());
                }
            }
        }

        if context.state() != State::Idle {
            context.next_state();
        }
    }

    fn accepts_signal(&self, direction: Direction) -> bool {
        self.0.contains(direction)
    }

    fn draw(&self, x: usize, y: usize, state: State, surface: &mut TextSurface) {
        let ch = match self.0 {
            Orientation::Horizontal => '-',
            Orientation::Vertical => '|',
            Orientation::Any => '+',
        };

        surface.set(x, y, TextChar::from_state(ch, state));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Diode(Direction);

impl Diode {
    pub fn new(direction: Direction) -> Self {
        Self(direction)
    }
}

impl Tile for Diode {
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        if let Some(signal) = context.take_signal() {
            // Block signals coming from where the diode is looking
            if signal.direction().opposite() == self.0 {
                return;
            }

            if let Some(pos) = context.offset(self.0.into_offset()) {
                let _ = context.send(pos, self.0, signal);
            }
        }

        if context.state() != State::Idle {
            context.next_state();
        }
    }

    fn accepts_signal(&self, direction: Direction) -> bool {
        direction.opposite() != self.0
    }

    fn draw(&self, x: usize, y: usize, state: State, surface: &mut TextSurface) {
        let ch = match self.0 {
            Direction::Up => '^',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Right => '>',
        };

        surface.set(x, y, TextChar::from_state(ch, state));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Resistor {
    direction: Direction,
    signal: Option<Signal>,
}

impl Resistor {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            signal: None,
        }
    }
}

impl Tile for Resistor {
    fn update<'b>(&'b mut self, mut context: UpdateContext<'b>) {
        if let Some(signal) = std::mem::take(&mut self.signal) {
            if let Some(pos) = context.offset(self.direction.into_offset()) {
                let _ = context.send(pos, self.direction, signal);
            }
        }

        if let Some(signal) = context.take_signal() {
            self.signal = Some(signal);
            context.set_state(State::Active);
        } else {
            if context.state() != State::Idle {
                context.next_state();
            }
        }
    }

    fn draw(&self, x: usize, y: usize, state: State, surface: &mut TextSurface) {
        let ch = match self.direction {
            Direction::Up => '\u{2191}',    // Upwards Arrow
            Direction::Down => '\u{2193}',  // Downwards Arrow
            Direction::Left => '\u{2190}',  // Leftwards Arrow
            Direction::Right => '\u{2192}', // Rightwards Arrow
        };

        surface.set(x, y, TextChar::from_state(ch, state));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wire_transmit() {
        use crate::Orientation::*;

        let mut pane = test_tile_setup!(
            3,
            2,
            [
                Wire::new(Horizontal),
                Wire::new(Any),
                Wire::new(Horizontal),
                (),
                Wire::new(Vertical),
                ()
            ]
        );

        // Test the signal going from left to right
        test_set_signal!(pane, (0, 0), Direction::Right);

        pane.step();

        println!("{:#?}", pane);

        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 1));

        pane.step();

        assert_signal!(pane, (2, 0));
        assert_signal!(pane, (1, 1));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));

        pane.step();
        for (_, _, tile) in pane.tiles() {
            assert!(tile.signal().is_none());
        }

        // Let the simulation cool down
        pane.step();
        pane.step();

        // Test the signal going from right to left
        test_set_signal!(pane, (2, 0), Direction::Left);

        pane.step();

        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 1));

        pane.step();

        assert_signal!(pane, (0, 0));
        assert_signal!(pane, (1, 1));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 0));
    }

    #[test]
    fn test_diode_transmit() {
        use crate::Direction::*;

        let mut pane = test_tile_setup!(
            3,
            2,
            [
                Diode::new(Right),
                Diode::new(Right),
                Diode::new(Down),
                (),
                Diode::new(Up),
                Diode::new(Left)
            ]
        );

        // Test the signal going from left to right
        test_set_signal!(pane, (0, 0), Direction::Right);

        pane.step();
        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));

        let positions = [(2, 0), (2, 1), (1, 1), (1, 0)];

        for &pos in positions.iter().cycle().take(16) {
            pane.step();
            println!("{:#?}", pane);
            assert_signal!(pane, pos);
            for &pos2 in positions.iter() {
                if pos == pos2 {
                    continue;
                }
                assert_no_signal!(pane, pos2);
            }
        }
    }

    #[test]
    fn test_resistor_transmit() {
        use crate::Direction::*;

        let mut pane = test_tile_setup!(
            4,
            1,
            [
                Diode::new(Right),
                Resistor::new(Right),
                Resistor::new(Right),
                Diode::new(Right)
            ]
        );

        test_set_signal!(pane, (0, 0), Direction::Right);

        pane.step();
        assert_no_signal!(pane, (0, 0));
        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (3, 0));

        pane.step();
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (3, 0));

        pane.step();
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));
        assert_signal!(pane, (2, 0));
        assert_no_signal!(pane, (3, 0));

        pane.step();
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (3, 0));

        pane.step();
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));
        assert_no_signal!(pane, (2, 0));
        assert_signal!(pane, (3, 0));

        pane.step();
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (3, 0));
    }
}
