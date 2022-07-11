use super::*;
use veccell::{VecCell, VecRef, VecRefMut};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pane {
    tiles: VecCell<FullTile>,
    width: NonZeroUsize,
    height: NonZeroUsize,

    position: (i32, i32),

    pub(crate) signals: Vec<(usize, usize)>,
}

impl Pane {
    /// Creates a new, empty `Pane` with the given dimensions.
    /// If `width == 0` or `height == 0`, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::Wire;
    ///
    /// // Create a new Pane with width 6 and height 4
    /// let mut pane = Pane::empty(6, 4).unwrap();
    ///
    /// // Place a horizontal wire on (2, 1) and (3, 1)
    /// pane.set_tile((2, 1), Wire::new(Orientation::Horizontal));
    /// pane.set_tile((3, 1), Wire::new(Orientation::Horizontal));
    ///
    /// // Put a signal on (2, 1)
    /// pane.set_signal((2, 1), stackline::signal!((2, 1)));
    ///
    /// // Perform a simulation step
    /// pane.step();
    /// ```
    #[ensures(ret.is_some() -> width > 0)]
    #[ensures(ret.is_some() -> height > 0)]
    pub fn empty(width: usize, height: usize) -> Option<Self> {
        // TODO: check that width * height is a valid usize
        let length = width.checked_mul(height)?;
        let mut tiles = VecCell::with_capacity(length);

        for _ in 0..length {
            tiles.push(FullTile::default());
        }

        Some(Self {
            width: width.try_into().ok()?,
            height: height.try_into().ok()?,
            tiles,

            position: (0, 0),

            signals: Vec::new(),
        })
    }

    /// Returns the width of the current pane
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let pane = Pane::empty(4, 7).unwrap();
    ///
    /// assert_eq!(pane.width().get(), 4);
    /// ```
    #[inline]
    pub fn width(&self) -> NonZeroUsize {
        self.width
    }

    /// Returns the height of the current pane
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let pane = Pane::empty(4, 7).unwrap();
    ///
    /// assert_eq!(pane.height().get(), 7);
    /// ```
    #[inline]
    pub fn height(&self) -> NonZeroUsize {
        self.height
    }

    /// Returns the position of the `Pane` in its [`World`].
    /// This property is used for drawing the `Pane` and calculating the distances between cross-pane tiles.
    #[inline]
    pub fn position(&self) -> (i32, i32) {
        self.position
    }

    /// Sets the position of the `Pane` in its [`World`].
    #[inline]
    pub fn set_position(&mut self, position: (i32, i32)) {
        self.position = position;
    }

    /// Given a `position = (x, y)` and an `offset = (Δx, Δy)`,
    /// returns `Some((x + Δx, y + Δy))` if `(x + Δx, y + Δy)` is inside the `Pane`.
    ///
    /// If `(x + Δx, y + Δy)` fall outside of the bounds of `Pane`, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// #
    /// let pane = Pane::empty(4, 2).unwrap();
    ///
    /// assert_eq!(pane.offset((1, 0), (2, 1)), Some((3, 1))); // (1 + 2, 0 + 1) = (3, 1), inside
    ///
    /// assert_eq!(pane.offset((1, 0), (-2, 0)), None); // (1 - 2, 0 + 0) = (-1, 0), outside
    ///
    /// assert_eq!(pane.offset((1, 0), (3, 0)), None); // (1 + 3, 0 + 0) = (4, 0), outside
    /// ```
    #[inline]
    #[ensures(ret.is_some() -> position.0 as isize + offset.0 as isize >= 0)]
    #[ensures(ret.is_some() -> position.1 as isize + offset.1 as isize >= 0)]
    #[ensures(ret.is_some() -> position.0 as isize + (offset.0 as isize) < self.width.get() as isize)]
    #[ensures(ret.is_some() -> position.1 as isize + (offset.1 as isize) < self.height.get() as isize)]
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
    /// Returns an immutable reference to the [`FullTile`] at `position`.
    /// If `position` is out of bounds, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::{FullTile, Wire};
    ///
    /// let mut pane = Pane::empty(4, 4).unwrap();
    ///
    /// pane.set_tile((0, 0), Wire::new(Orientation::Horizontal));
    ///
    /// let tile = pane.get((0, 0)).unwrap();
    /// ```
    #[inline]
    #[ensures(self.in_bounds(position) -> ret.is_some())]
    pub fn get<'b>(&'b self, position: (usize, usize)) -> Option<VecRef<'b, FullTile>> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles
            .borrow(position.1 * self.width.get() + position.0)
    }

    /// Returns an immutable reference to the [`Tile`] at `position`.
    /// If `position` is out of bounds, returns `None`.
    /// If the tile at position isn't an instance of `T`, returns `None`.
    ///
    /// `T` may be any [`Tile`] that is a member of [`AnyTile`].
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::{FullTile, Wire};
    ///
    /// let mut pane = Pane::empty(4, 4).unwrap();
    ///
    /// pane.set_tile((0, 0), Wire::new(Orientation::Horizontal));
    ///
    /// let wire = pane.get_as::<Wire>((0, 0)).unwrap();
    /// assert_eq!(wire, Wire::new(Orientation::Horizontal));
    /// ```
    // TODO: error types
    #[inline]
    pub fn get_as<'b, T: Tile>(&'b self, position: (usize, usize)) -> Option<VecRef<'b, T>>
    where
        for<'c> &'c AnyTile: TryInto<&'c T>,
    {
        let tile = self.get(position)?;
        VecRef::try_map(tile, |tile| {
            tile.get()
                .ok_or(())
                .and_then(|anytile: &AnyTile| anytile.try_into().map_err(|_| ()))
        })
        .ok()
    }

    /// Returns a mutable reference to the [`Tile`] at `position`.
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::Wire;
    ///
    /// let mut pane = Pane::empty(4, 4).unwrap();
    ///
    /// pane.set_tile((0, 0), Wire::new(Orientation::Horizontal));
    ///
    /// let mut tile = pane.get_mut((0, 0)).unwrap();
    /// tile.set_state(State::Active);
    /// ```
    #[inline]
    #[ensures(old(self.in_bounds(position)) -> ret.is_some())]
    pub fn get_mut<'b>(&'b mut self, position: (usize, usize)) -> Option<&'b mut FullTile> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles
            .get_mut(position.1 * self.width.get() + position.0)
    }

    /// Returns a mutable reference to the [`Tile`] at `position`.
    /// If `position` is out of bounds, returns `None`.
    /// If the tile at position isn't an instance of `T`, returns `None`.
    ///
    /// `T` may be any [`Tile`] that is a member of [`AnyTile`].
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::{FullTile, Wire};
    ///
    /// let mut pane = Pane::empty(4, 4).unwrap();
    ///
    /// pane.set_tile((0, 0), Wire::new(Orientation::Horizontal));
    ///
    /// let mut wire = pane.get_mut_as::<Wire>((0, 0)).unwrap();
    /// assert_eq!(*wire, Wire::new(Orientation::Horizontal));
    /// *wire = Wire::new(Orientation::Vertical);
    /// ```
    #[inline]
    pub fn get_mut_as<'b, T: Tile>(&'b mut self, position: (usize, usize)) -> Option<&'b mut T>
    where
        for<'c> &'c mut AnyTile: TryInto<&'c mut T>,
    {
        let tile = self.get_mut(position)?.get_mut()?;
        tile.try_into().ok()
    }

    /// Returns a mutable borrow to the [`Tile`] at `position`.
    ///
    /// This function does not need a mutable reference to `self`, and makes use
    /// of [`VecCell`]'s ability to provide interior mutability for one item at a time.
    #[inline]
    pub(crate) fn borrow_mut<'b>(
        &'b self,
        position: (usize, usize),
    ) -> Option<VecRefMut<'b, FullTile>> {
        if !self.in_bounds(position) {
            return None;
        }

        self.tiles
            .borrow_mut(position.1 * self.width.get() + position.0)
    }

    /// Returns a mutable borrow to the [`Tile`] at `position`.
    /// If `position` is out of bounds, returns `None`.
    /// If the tile at position isn't an instance of `T`, returns `None`.
    ///
    /// `T` may be any [`Tile`] that is a member of [`AnyTile`].
    ///
    /// This function does not need a mutable reference to `self`, and makes use
    /// of [`VecCell`]'s ability to provide interior mutability for one item at a time.
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::{FullTile, Wire};
    ///
    /// let mut pane = Pane::empty(4, 4).unwrap();
    ///
    /// pane.set_tile((0, 0), Wire::new(Orientation::Horizontal));
    ///
    /// let pane = pane;
    ///
    /// let mut wire = pane.borrow_mut_as::<Wire>((0, 0)).unwrap();
    /// assert_eq!(wire, Wire::new(Orientation::Horizontal));
    /// *wire = Wire::new(Orientation::Vertical);
    /// ```
    #[inline]
    pub fn borrow_mut_as<'b, T: Tile>(&'b self, position: (usize, usize)) -> Option<VecRefMut<'b, T>>
    where
        for<'c> &'c mut AnyTile: TryInto<&'c mut T>,
    {
        let tile = self.borrow_mut(position)?;
        VecRefMut::try_map(tile, |tile| {
            tile.get_mut()
                .ok_or(())
                .and_then(|anytile: &mut AnyTile| anytile.try_into().map_err(|_| ()))
        })
        .ok()
    }

    /// Sets the tile at `position` to `tile`. `T` must either implement [`Tile`] or be `()`.
    #[inline]
    #[ensures(self.in_bounds(position) -> ret.is_some())]
    pub fn set_tile<T>(&mut self, position: (usize, usize), tile: T) -> Option<()>
    where
        FullTile: From<T>,
    {
        let full_tile = self.get_mut(position)?;

        *full_tile = FullTile::from(tile);

        Some(())
    }

    /// Returns the [`State`] of the tile at `position`, if that tile exists.
    /// If `position` is out of bounds, returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::Wire;
    ///
    /// let mut pane = Pane::empty(1, 1).unwrap();
    ///
    /// // All tiles are initialized with the Idle state
    /// assert_eq!(pane.get_state((0, 0)), Some(State::Idle));
    ///
    /// // Creating a new tile gives it the Idle state
    /// pane.set_tile((0, 0), Wire::new(Orientation::Horizontal));
    /// assert_eq!(pane.get_state((0, 0)), Some(State::Idle));
    ///
    /// // We manually set the state to Dormant and observe the change
    /// pane.get_mut((0, 0)).unwrap().set_state(State::Dormant);
    /// assert_eq!(pane.get_state((0, 0)), Some(State::Dormant));
    /// ```
    #[inline]
    #[ensures(self.in_bounds(position) -> ret.is_some())]
    pub fn get_state(&self, position: (usize, usize)) -> Option<State> {
        self.get(position).map(|x| x.state().clone())
    }

    /// Sets the signal for the tile at `position` to `signal`.
    /// Returns `Some(())` if the tile exists and the tile can have a signal.
    ///
    /// This function does not check the tile's [`accepts_signal`](Tile::accepts_signal) method.
    /// It will also overwrite any signal already present.
    ///
    /// # Example
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::Diode;
    ///
    /// let mut pane = Pane::empty(2, 1).unwrap();
    ///
    /// pane.set_tile((0, 0), Diode::new(Direction::Right));
    /// pane.set_tile((1, 0), Diode::new(Direction::Down));
    ///
    /// // The signal for a tile is initially None
    /// assert!(pane.get((0, 0)).unwrap().signal().is_none());
    ///
    /// // We set it to something else
    /// pane.set_signal((0, 0), stackline::signal!((0, 0), Direction::Right)).unwrap();
    ///
    /// assert!(pane.get((0, 0)).unwrap().signal().is_some());
    /// ```
    #[inline]
    #[ensures(ret.is_some() -> self.in_bounds(position) && (*self.get(position).unwrap()).get().is_some())]
    #[ensures(!self.in_bounds(position) -> ret.is_none())]
    #[allow(unused_mut)]
    pub fn set_signal(&mut self, position: (usize, usize), mut signal: Signal) -> Option<()> {
        signal.set_position(position);
        if let Some(tile) = self.get_mut(position) {
            tile.set_signal(Some(signal))?;
            tile.set_state(State::Active);
            self.signals.push(position);
            Some(())
        } else {
            None
        }
    }

    /// Returns `true` if `position` is within the bounds of the pane.
    /// Returns `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let pane = Pane::empty(2, 3).unwrap();
    ///
    /// assert!(pane.in_bounds((0, 0)));
    /// assert!(pane.in_bounds((1, 2)));
    /// assert!(!pane.in_bounds((10, 10)));
    /// ```
    #[inline]
    pub fn in_bounds(&self, position: (usize, usize)) -> bool {
        position.0 < self.width.get() && position.1 < self.height.get()
    }

    #[inline]
    #[ensures(self.in_bounds(position) -> self.get(position).unwrap().updated)]
    #[ensures(!self.in_bounds(position) -> ret.is_none())]
    fn update(&mut self, position: (usize, usize), commit: &mut UpdateCommit) -> Option<()> {
        // NOTE: Tiles will only be updated once as per UpdateContext::new
        let (ctx, mut tile) = UpdateContext::new(self, position, commit)?;

        (*tile).get_mut()?.update(ctx);

        commit.apply_immediate(&mut *tile);

        Some(())
    }

    /// Performs an update cycle.
    /// Such an update cycle roughly consists of the following:
    ///
    /// - Calls [`Tile::update`] on every tile with a signal
    /// - Calls [`Tile::update`] on every active tile (tiles will only be updated once)
    /// - Applies all signal [`send`s](UpdateContext::send)
    /// - Applies all [state changes](UpdateContext::set_state)
    ///
    /// # Guarantees
    ///
    /// To prevent unwanted behavior, the following properties are upheld by this method and the methods of [`UpdateContext`]:
    ///
    /// - Order-agnostism: [updating](Tile::update) a tile `A` before a tile `B` will result in the same state as if `B` was updated before `A`.
    /// - As a consequence, when a tile `A` is updated, it cannot modify the state of any other tile.
    /// - Any [`Signal`] or [`State`] update will only be carried out after every tile was updated.
    ///   The only exception to this rule is [`UpdateContext::keep`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stackline::prelude::*;
    /// use stackline::tile::Diode;
    ///
    /// let mut pane = Pane::empty(2, 2).unwrap();
    ///
    /// pane.set_tile((0, 0), Diode::new(Direction::Right));
    /// pane.set_tile((1, 0), Diode::new(Direction::Down));
    /// pane.set_tile((1, 1), Diode::new(Direction::Left));
    /// pane.set_tile((0, 1), Diode::new(Direction::Up));
    ///
    /// println!("{:?}", pane);
    /// // >v
    /// // ^<
    ///
    /// // Initialize the circuit with a signal at (0, 0)
    /// pane.set_signal((0, 0), stackline::signal!((0, 0), Direction::Right));
    ///
    /// // Do an update step
    /// pane.step();
    ///
    /// // The signal has now been moved to (1, 0) by the Diode
    /// assert!(pane.get((1, 0)).unwrap().signal().is_some());
    ///
    /// // Do another update step
    /// pane.step();
    ///
    /// // The signal is now at (1, 1)
    /// assert!(pane.get((1, 1)).unwrap().signal().is_some());
    /// ```
    pub fn step(&mut self) -> PaneResult {
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

        commit.apply(self)
    }

    /// Returns an iterator over the tiles and their coordinates
    #[inline]
    pub fn tiles<'b>(&'b self) -> impl Iterator<Item = (usize, usize, VecRef<'b, FullTile>)> + 'b {
        self.tiles
            .iter()
            .enumerate()
            .filter_map(move |(i, v)| Some((i % self.width, i / self.width, v)))
    }

    /// Draws the Pane at `(dx + self.position.0, dy + self.position.1)` on a [`TextSurface`].
    /// Empty tiles will leave the `TextSurface` untouched, but tiles are free to modify the characters around them.
    pub fn draw(&self, dx: i32, dy: i32, surface: &mut TextSurface) {
        for (x, y, tile) in self.tiles() {
            let x = x as i32 + dx + self.position.0 as i32;
            let y = y as i32 + dy + self.position.1 as i32;

            if x >= 0 && y >= 0 {
                tile.draw(x as usize, y as usize, surface);
            }
        }
    }
}

/// Stores the results of a [`Pane`]'s update step.
pub struct PaneResult {
    /// Signals to be sent to other panes.
    pub outbound_signals: Vec<((String, usize, usize), Signal)>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pane_draw() {
        use crate::tile::Wire;
        use Orientation::*;

        let mut surface = TextSurface::new(3, 3);
        let mut pane = test_tile_setup!(
            2,
            2,
            [
                Wire::new(Horizontal),
                Wire::new(Vertical),
                Wire::new(Any),
                ()
            ]
        );
        test_set_signal!(pane, (0, 0), Direction::Right);

        pane.draw(0, 0, &mut surface);

        assert_eq!(surface.get(0, 0).unwrap().ch, '-');
        assert_eq!(surface.get(1, 0).unwrap().ch, '|');
        assert_eq!(surface.get(0, 1).unwrap().ch, '+');
        assert_eq!(surface.get(1, 1), Some(TextChar::default()));
        for n in 0..3 {
            assert_eq!(surface.get(2, n), Some(TextChar::default()));
            assert_eq!(surface.get(n, 2), Some(TextChar::default()));
        }

        // With offset (1, 0)
        let mut surface = TextSurface::new(3, 3);
        pane.draw(1, 0, &mut surface);

        assert_eq!(surface.get(1, 0).unwrap().ch, '-');
        assert_eq!(surface.get(2, 0).unwrap().ch, '|');
        assert_eq!(surface.get(1, 1).unwrap().ch, '+');
        assert_eq!(surface.get(2, 1), Some(TextChar::default()));
        for n in 0..3 {
            assert_eq!(surface.get(0, n), Some(TextChar::default()));
            assert_eq!(surface.get(n, 2), Some(TextChar::default()));
        }

        // With offset (0, 1)
        let mut surface = TextSurface::new(3, 3);
        pane.draw(0, 1, &mut surface);

        assert_eq!(surface.get(0, 1).unwrap().ch, '-');
        assert_eq!(surface.get(1, 1).unwrap().ch, '|');
        assert_eq!(surface.get(0, 2).unwrap().ch, '+');
        assert_eq!(surface.get(1, 2), Some(TextChar::default()));
        for n in 0..3 {
            assert_eq!(surface.get(2, n), Some(TextChar::default()));
            assert_eq!(surface.get(n, 0), Some(TextChar::default()));
        }

        // Draw outside of bounds with offset (2, 2)
        let mut surface = TextSurface::new(3, 3);
        pane.draw(2, 2, &mut surface);

        assert_eq!(surface.get(2, 2).unwrap().ch, '-');
        for y in 0..3 {
            for x in 0..3 {
                if (x, y) != (2, 2) {
                    assert_eq!(surface.get(x, y), Some(TextChar::default()));
                }
            }
        }
    }

    #[test]
    fn test_pane_draw_offset() {
        use crate::tile::Wire;
        use Orientation::*;

        let mut pane = test_tile_setup!(
            2,
            2,
            [
                Wire::new(Horizontal),
                Wire::new(Vertical),
                Wire::new(Any),
                ()
            ]
        );

        pane.set_position((2, 1));

        let mut surface = TextSurface::new(9, 9);

        pane.draw(5, 3, &mut surface);

        assert_eq!(surface.get(5 + 2, 3 + 1).unwrap().ch, '-');
        assert_eq!(surface.get(5 + 3, 3 + 1).unwrap().ch, '|');
        assert_eq!(surface.get(5 + 2, 3 + 2).unwrap().ch, '+');
        for y in 0..9 {
            for x in 0..9 {
                if (x, y) != (5 + 2, 3 + 1) && (x, y) != (5 + 3, 3 + 1) && (x, y) != (5 + 2, 3 + 2)
                {
                    assert_eq!(surface.get(x, y), Some(TextChar::default()));
                }
            }
        }
    }
}
