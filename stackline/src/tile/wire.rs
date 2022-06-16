//! Wires and diodes

use super::*;

#[derive(Clone, Debug)]
pub struct Wire(Orientation);

impl Tile for Wire {
    fn transmit<'b>(&'b self, signal: Rc<Signal>, context: TransmitContext<'b>) {
        for &direction in self.0.into_directions() {
            if direction == signal.direction.opposite() {
                continue;
            }

            if let Some(new_pos) = context.offset(direction.into_offset()) {
                let tile = context.get(new_pos);
                if tile.map(|t| t.accepts_signal(direction)).unwrap_or(false) {
                    context.send(new_pos, (*signal).clone_with_dir(direction));
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
