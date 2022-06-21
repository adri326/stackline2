use super::*;

#[derive(Debug)]
pub struct Pane {
    tiles: Vec<FullTile>,
    width: NonZeroUsize,
    height: NonZeroUsize,

    signals: Vec<(usize, usize)>,
}

impl Pane {
    pub fn empty(width: usize, height: usize) -> Option<Self> {
        // TODO: check that width * height is a valid usize
        let length = width.checked_mul(height)?;

        Some(Self {
            width: width.try_into().ok()?,
            height: height.try_into().ok()?,
            tiles: vec![FullTile::default(); length],

            signals: vec![],
        })
    }

    /// Returns `Some((x + Δx, y + Δy))` iff `(x + Δx, y + Δy)` is inside the world
    // SAFETY: this function may *not* access `self.signals`, `∀x, self.tiles[x].cell` or `∀x, self.tiles[x].signal`
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

    // SAFETY: this function may not access `self.signals`, nor may it read the contents of `self.tiles[position]`
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

    /// Sets the signal for the tile at `position` to `signal`.
    /// Returns `Some` iff:
    /// - the tile exists
    /// - the tile accepts a signal (ie. it isn't empty)
    // SAFETY: may only access `self[pos].signal` and `self.signals`
    #[inline]
    pub fn set_signal(&mut self, position: (usize, usize), mut signal: Signal) -> Option<()> {
        signal.set_position(position);
        self.get_mut(position)?.set_signal(signal)?;
        self.signals.push(position);
        Some(())
    }

    #[inline]
    pub fn in_bounds(&self, position: (usize, usize)) -> bool {
        position.0 < self.width.get() && position.1 < self.height.get()
    }

    #[inline]
    pub fn update(&mut self, position: (usize, usize)) -> Option<()> {
        let (ctx, tile) = UpdateContext::new(self, position)?;

        tile.update(ctx);

        Some(())
    }

    /// Calls [`Pane::update`] on all non-empty, non-idle tiles
    fn update_all(&mut self) {
        for y in 0..self.height.get() {
            for x in 0..self.width.get() {
                if let Some((ctx, tile)) = UpdateContext::new(self, (x, y)) {
                    if ctx.state() != State::Idle {
                        tile.update(ctx);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn transmit(&mut self, position: (usize, usize)) -> Option<()> {
        let (ctx, tile, signal) = TransmitContext::new(self, position)?;

        tile.transmit(signal, ctx);

        Some(())
    }

    /// Calls [`Pane::transmit`] on all tiles with a signal
    fn transmit_all(&mut self) {
        // TODO: store a second buffer and perform swap reads
        for position in std::mem::replace(&mut self.signals, vec![]) {
            let _ = self.transmit(position); // May return None if the signal was aliased
        }
    }

    /// Runs a single simulation step, which consists of:
    /// - an update phase, which mutates the inner state of [active](State::Active)] cells by calling [`Tile::update`]
    /// - a transmit phase, which mutates and moves signals between cells by calling [`Tile::transmit`]
    pub fn step(&mut self) {
        self.update_all();
        self.transmit_all();
    }

    /// Returns an iterator over the tiles and their coordinates
    #[inline]
    pub fn tiles<'b>(&'b self) -> impl Iterator<Item=(usize, usize, &'b FullTile)> + 'b {
        self.tiles.iter().enumerate().map(move |(i, v)| (i % self.width, i / self.width, v))
    }

    /// Returns a mutable iterator over the tiles and their coordinates
    #[inline]
    pub fn tiles_mut<'b>(&'b mut self) -> impl Iterator<Item=(usize, usize, &'b mut FullTile)> + 'b {
        let width = self.width;
        self.tiles.iter_mut().enumerate().map(move |(i, v)| (i % width, i / width, v))
    }
}
