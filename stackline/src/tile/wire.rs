//! Wires and diodes

use super::*;

#[derive(Clone, Debug)]
pub struct Wire(Orientation);

impl Wire {
    pub fn new(orientation: Orientation) -> Self {
        Self(orientation)
    }
}

impl Tile for Wire {
    fn transmit<'b>(&'b self, signal: Rc<Signal>, context: TransmitContext<'b>) {
        for &direction in self.0.into_directions() {
            if direction == signal.direction().opposite() {
                continue;
            }

            if let Some(new_pos) = context.offset(direction.into_offset()) {
                let tile = context.get(new_pos);
                if tile.map(|t| t.accepts_signal(direction)).unwrap_or(false) {
                    context.send(new_pos, signal.clone_move(direction).unwrap());
                }
            }
        }
    }

    fn accepts_signal(&self, direction: Direction) -> bool {
        self.0.contains(direction)
    }
}

#[derive(Clone, Debug)]
pub struct Diode(Direction);

impl Diode {
    pub fn new(direction: Direction) -> Self {
        Self(direction)
    }
}

// impl Tile for Diode {
//     fn update(&mut self, world: &mut World, signal: &mut Option<Rc<Signal>>, pos: (usize, usize)) {
//         if let Some(signal) = std::mem::take(signal) {
//             if let Some(new_pos) = world.offset(pos, self.0.into_offset()) {
//                 let tile = world.get(new_pos).unwrap();
//                 if tile.borrow().accepts_signal(self.0) {
//                     world.send_signal(new_pos, (*signal).clone_with_dir(self.0)).unwrap();
//                 }
//             }
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wire_transmit() {
        use crate::Orientation::*;

        let mut pane = test_tile_setup!(3, 2, [
            Wire::new(Horizontal), Wire::new(Any), Wire::new(Horizontal),
            (), Wire::new(Vertical), ()
        ]);

        // Test the signal going from left to right
        test_set_signal!(pane, (0, 0), Direction::Right);

        pane.transmit_all();

        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 1));

        pane.transmit_all();

        assert_signal!(pane, (2, 0));
        assert_signal!(pane, (1, 1));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));

        pane.transmit_all();
        for (_, _, tile) in pane.tiles() {
            assert!(tile.signal().is_none());
        }

        // Test the signal going from right to left
        test_set_signal!(pane, (2, 0), Direction::Left);

        pane.transmit_all();

        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 1));

        pane.transmit_all();

        assert_signal!(pane, (0, 0));
        assert_signal!(pane, (1, 1));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 0));
    }
}
