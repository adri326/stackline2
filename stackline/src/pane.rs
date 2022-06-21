use super::*;

#[derive(Debug)]
pub struct Pane {
    tiles: Vec<FullTile>,
    width: NonZeroUsize,
    height: NonZeroUsize,

    pub(crate) signals: Vec<(usize, usize)>,
}

impl Pane {
    pub fn empty(width: usize, height: usize) -> Option<Self> {
        // TODO: check that width * height is a valid usize
        let length = width.checked_mul(height)?;

        Some(Self {
            width: width.try_into().ok()?,
            height: height.try_into().ok()?,
            tiles: vec![FullTile::default(); length],

            signals: Vec::new(),
        })
    }

    /// Returns `Some((x + Δx, y + Δy))` iff `(x + Δx, y + Δy)` is inside the world
    #[inline]
    pub fn offset(&self, position: (usize, usize), offset: (i8, i8)) -> Option<(usize, usize)> {
        if offset.0 < 0 && (-offset.0) as usize > position.0
            || offset.1 < 0 && (-offset.1) as usize > position.1
        {
            return None;
        }

        // TODO: check that position and position + offset are valid isize values
        let new_pos = (
            (position.0 as isize + offset.0 as isize) as usize,
            (position.1 as isize + offset.1 as isize) as usize,
        );

        if new_pos.0 < self.width.get() && new_pos.1 < self.height.get() {
            Some(new_pos)
        } else {
            None
        }
    }

    // TODO: Have a Result instead of an Option
    #[inline]
    pub fn get<'b>(&'b self, position: (usize, usize)) -> Option<&'b FullTile> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles.get(position.1 * self.width.get() + position.0)
    }

    #[inline]
    pub fn get_mut<'b>(&'b mut self, position: (usize, usize)) -> Option<&'b mut FullTile> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles.get_mut(position.1 * self.width.get() + position.0)
    }

    #[inline]
    pub fn get_state(&self, position: (usize, usize)) -> Option<State> {
        self.get(position).map(|x| x.state().clone())
    }

    /// Sets the signal for the tile at `position` to `signal`.
    /// Returns `Some` iff:
    /// - the tile exists
    /// - the tile accepts a signal (ie. it isn't empty)
    #[inline]
    pub fn set_signal(&mut self, position: (usize, usize), mut signal: Signal) -> Option<()> {
        signal.set_position(position);
        self.get_mut(position)?.set_signal(Some(signal))?;
        self.signals.push(position);
        Some(())
    }

    #[inline]
    pub fn in_bounds(&self, position: (usize, usize)) -> bool {
        position.0 < self.width.get() && position.1 < self.height.get()
    }

    #[inline]
    fn update(&mut self, position: (usize, usize), commit: &mut UpdateCommit) -> Option<()> {
        let (ctx, tile) = UpdateContext::new(self, position, commit)?;

        tile.update(ctx);

        Some(())
    }

    // TODO: document
    pub fn step(&mut self) {
        let mut commit = UpdateCommit::new();

        for position in std::mem::replace(&mut self.signals, Vec::new()) {
            let _ = self.update(position, &mut commit);
        }

        for y in 0..self.height.get() {
            for x in 0..self.width.get() {
                if self.get_state((x, y)).unwrap() != State::Idle {
                    let _ = self.update((x, y), &mut commit);
                }
            }
        }

        commit.apply(self);
    }

    /// Returns an iterator over the tiles and their coordinates
    #[inline]
    pub fn tiles<'b>(&'b self) -> impl Iterator<Item=(usize, usize, &FullTile)> + 'b {
        self.tiles.iter().enumerate().filter_map(move |(i, v)| {
            Some((i % self.width, i / self.width, v))
        })
    }
}
