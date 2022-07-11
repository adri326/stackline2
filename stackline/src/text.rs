use super::*;
use palette::{FromColor, Srgb};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextChar {
    pub ch: char,
    pub fg: Srgb<u8>,
    pub bg: Option<Srgb<u8>>,
}

impl TextChar {
    #[inline]
    pub fn new<C1, C2>(ch: char, fg: C1, bg: Option<C2>) -> Self
    where
        Srgb<u8>: FromColor<C1>,
        Srgb<u8>: FromColor<C2>,
    {
        Self {
            ch,
            fg: Srgb::from_color(fg),
            bg: bg.map(|x| Srgb::from_color(x)),
        }
    }

    #[inline]
    pub fn from_char(ch: impl Into<char>) -> Self {
        Self {
            ch: ch.into(),
            fg: Srgb::new(255, 255, 255),
            bg: None,
        }
    }

    #[inline]
    pub fn from_state(ch: impl Into<char>, state: State) -> Self {
        Self {
            ch: ch.into(),
            fg: match state {
                State::Idle => Srgb::new(128, 128, 128),
                State::Active => Srgb::new(220, 255, 255),
                State::Dormant => Srgb::new(100, 60, 60),
            },
            bg: None,
        }
    }
}

impl Default for TextChar {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Srgb::new(255, 255, 255),
            bg: None,
        }
    }
}

pub struct TextSurface {
    width: usize,
    height: usize,

    chars: Vec<TextChar>,
}

impl TextSurface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,

            chars: vec![TextChar::default(); width * height],
        }
    }

    /// Returns the [`TextChar`] at `(x, y)`, if it exists.
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let surface = TextSurface::new(1, 1);
    ///
    /// assert_eq!(surface.get(0, 0), Some(TextChar::default()));
    /// ```
    pub fn get(&self, x: usize, y: usize) -> Option<TextChar> {
        if self.in_bounds(x, y) {
            Some(self.chars[y * self.width + x])
        } else {
            None
        }
    }

    /// Sets the [`TextChar`] at `(x, y)` to `c`, if `(x, y)` is within the bounds of this `TextSurface`.
    /// Returns `None` if `(x, y)` is outside of the bounds.
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let mut surface = TextSurface::new(2, 2);
    ///
    /// surface.set(0, 0, TextChar::from_char('a')).unwrap();
    /// assert_eq!(surface.get(0, 0,), Some(TextChar::from_char('a')));
    /// ```
    pub fn set(&mut self, x: usize, y: usize, c: TextChar) -> Option<()> {
        if self.in_bounds(x, y) {
            self.chars[y * self.width + x] = c;
            Some(())
        } else {
            None
        }
    }

    /// Returns `true` iff `(x, y)` is within the bounds of this `TextSurface`.
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    // TODO: resize
}

impl std::fmt::Display for TextSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use colored::Colorize;

        for y in 0..self.height {
            for x in 0..self.width {
                let ch = self.chars[y * self.width + x];
                let mut string = String::from(ch.ch).truecolor(ch.fg.red, ch.fg.green, ch.fg.blue);

                if let Some(bg) = ch.bg {
                    string = string.on_truecolor(bg.red, bg.green, bg.blue);
                }

                write!(f, "{}", string)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
