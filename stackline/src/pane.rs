use super::*;

#[derive(Debug)]
pub struct Pane {
    tiles: Vec<FullTile>,
    width: NonZeroUsize,
    height: NonZeroUsize,

    signals: Vec<Weak<Signal>>,
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

    // SAFETY: may only access `self[pos].signal` and `self.signals`
    #[inline]
    pub fn set_signal(&mut self, position: (usize, usize), signal: Signal) -> Option<Weak<Signal>> {
        let signal = self.get_mut(position)?.set_signal(signal)?;
        self.signals.push(signal.clone());
        Some(signal)
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

    // TODO: update_all (requires FullTile::state)

    #[inline]
    pub fn transmit(&mut self, position: (usize, usize)) -> Option<()> {
        let (ctx, tile, signal) = TransmitContext::new(self, position)?;

        tile.transmit(signal, ctx);

        Some(())
    }

    pub fn transmit_all(&mut self) {
        // TODO: store a second buffer and perform swap reads
        for signal in std::mem::replace(&mut self.signals, vec![]) {
            if let Some(upgraded) = signal.upgrade() {
                let position = upgraded.position();
                let _ = self.transmit(position); // May return None if the signal was aliased
            }
        }
    }

    /// Returns an iterator over the tiles and their coordinates
    #[inline]
    pub fn tiles<'b>(&'b self) -> impl Iterator<Item=(usize, usize, &'b FullTile)> + 'b {
        self.tiles.iter().enumerate().map(move |(i, v)| (i % self.width, i / self.width, v))
    }
}