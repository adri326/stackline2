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
    fn transmit<'b>(&'b self, signal: Rc<Signal>, mut context: TransmitContext<'b>) {
        for &direction in self.0.into_directions() {
            if direction == signal.direction().opposite() {
                continue;
            }

            if let Some(pos) = context.offset(direction.into_offset()) {
                if context.accepts_signal(pos, direction) {
                    context.send(pos, signal.clone_move(direction).unwrap_or_else(|| unreachable!()));
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

impl Tile for Diode {
    fn transmit<'b>(&'b self, signal: Rc<Signal>, mut context: TransmitContext<'b>) {
        if let Some(pos) = context.offset(self.0.into_offset()) {
            if context.accepts_signal(pos, self.0) {
                context.send(pos, signal.clone_move(self.0).unwrap_or_else(|| unreachable!()));
            }
        }
    }
}

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

        pane.update_all();
        pane.transmit_all();

        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 1));

        pane.update_all();
        pane.transmit_all();

        assert_signal!(pane, (2, 0));
        assert_signal!(pane, (1, 1));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (1, 0));

        pane.update_all();
        pane.transmit_all();
        for (_, _, tile) in pane.tiles() {
            assert!(tile.signal().is_none());
        }

        // Let the simulation cool down
        pane.update_all();
        pane.update_all();

        // Test the signal going from right to left
        test_set_signal!(pane, (2, 0), Direction::Left);

        pane.update_all();
        pane.transmit_all();

        assert_signal!(pane, (1, 0));
        assert_no_signal!(pane, (0, 0));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 1));

        pane.update_all();
        pane.transmit_all();

        assert_signal!(pane, (0, 0));
        assert_signal!(pane, (1, 1));
        assert_no_signal!(pane, (2, 0));
        assert_no_signal!(pane, (1, 0));
    }
}
