use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
}

impl Value {
    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(x) => Some(*x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        self.as_number().map(|x| x as i64)
    }
}

impl From<f64> for Value {
    fn from(x: f64) -> Value {
        Value::Number(x.into())
    }
}

impl From<u32> for Value {
    fn from(x: u32) -> Value {
        Value::Number(x.into())
    }
}

impl From<String> for Value {
    fn from(string: String) -> Value {
        Value::String(string)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(string: &'a str) -> Value {
        Value::String(String::from(string))
    }
}

/// The unit of information that [`Tile`]s transmit between each other.
/// A `Signal` is made up of a [`stack`](Signal::stack), and tracks its [`position`](Signal::position) and [`direction`](Signal::direction).
///
/// ## Creating a signal
///
/// There are multiple ways to create a `Signal`:
///
/// - By cloning it, through [`clone_move`](Signal::clone_move) (recommended) or [`clone`](Signal::clone)
/// - By creating an empty signal, with [`empty`](Signal::empty)
/// - Through the [`stackline::signal!`](crate::signal!) macro
#[derive(Clone, Debug)]
pub struct Signal {
    direction: Direction,
    position: (usize, usize),
    stack: Vec<Value>,
}

impl Signal {
    pub fn empty(position: (usize, usize), direction: Direction) -> Self {
        Self {
            direction,
            position,
            stack: Vec::new(),
        }
    }

    /// Variant of [`moved`](Signal::moved), but clones the signal beforehand.
    ///
    /// See [`moved`](Signal::moved) for more information
    pub fn clone_move(&self, direction: Direction) -> Self {
        let mut res = self.clone();
        res.direction = direction;

        res
    }

    /// Sets the direction of the signal to `direction`, and returns that signal.
    ///
    /// This function or its sister function, [`clone_move`](Signal::clone_move), should always be called before [`send`ing](UpdateContext::send) a signal to another tile.
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// # #[derive(Clone, Debug)]
    /// # struct MyTile;
    /// # impl Tile for MyTile {
    /// fn update<'b>(&'b mut self, mut ctx: UpdateContext<'b>) {
    ///     let direction = Direction::Down;
    ///
    ///     if let Some(signal) = ctx.take_signal() {
    ///         // We have a signal, see if it can be sent down
    ///         if let Some(pos) = ctx.accepts_direction(direction) {
    ///             ctx.send(pos, direction, signal);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn moved(mut self, direction: Direction) -> Self {
        self.direction = direction;

        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    pub(crate) fn set_position(&mut self, position: (usize, usize)) {
        self.position = position;
    }

    /// Pushes a value onto the stack of the signal.
    /// Signals are pushed on top of the stack and can be [`pop`ped](Signal::pop) in reverse order.
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let mut signal = Signal::empty((0, 0), Direction::Down);
    /// assert_eq!(signal.len(), 0);
    ///
    /// signal.push(Value::Number(1.0));
    /// assert_eq!(signal.len(), 1);
    ///
    /// signal.push(Value::Number(2.0));
    /// assert_eq!(signal.len(), 2);
    /// ```
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pops a value from the stack of the signal, returning `Some(value)`
    /// if the stack wasn't empty and `None` otherwise.
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let mut signal = Signal::empty((0, 0), Direction::Down);
    ///
    /// signal.push(Value::Number(1.0));
    /// signal.push(Value::Number(2.0));
    /// assert_eq!(signal.len(), 2);
    ///
    /// // We pushed 2.0 last, so pop() will return that
    /// assert_eq!(signal.pop(), Some(Value::Number(2.0)));
    /// assert_eq!(signal.len(), 1);
    /// ```
    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    /// Returns the number of elements in the stack of the signal.
    ///
    /// ## Example
    ///
    /// ```
    /// # use stackline::prelude::*;
    /// let signal = stackline::signal!((0, 0), Direction::Down, [1.0, -1.5]);
    ///
    /// assert_eq!(signal.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn stack(&self) -> &Vec<Value> {
        &self.stack
    }

    pub fn stack_mut(&mut self) -> &mut Vec<Value> {
        &mut self.stack
    }
}

/// Creates a signal with initial values in its stack.
///
/// The syntax for the macro is `signal!(position, direction, [value1, value2, ...])`, where:
/// - `position`: a pair `(x, y): (usize, usize)`
/// - `direction`: a [`Direction`] *(optional, defaults to [`Direction::default()`])*
/// - `value1`, `value2`, etc.: [`Value`] must implement [`From<T>`](Value#trait-implementations) for each value *(optional)*
///
/// ## Examples
///
/// ```
/// # use stackline::prelude::*;
/// // Creates an empty signal at (1, 2)
/// let signal = stackline::signal!((1, 2));
///
/// // Creates an empty signal going right at (2, 2)
/// let signal = stackline::signal!((2, 2), Direction::Right);
///
/// // Creates a signal with the values 10 and 12 on its stack
/// let signal = stackline::signal!((0, 0), [10, 12]);
/// ```
#[macro_export]
macro_rules! signal {
    ( $pos:expr, $dir:expr, [ $( $x:expr ),* ] ) => {{
        let mut res = Signal::empty($pos, $dir);

        $({
            res.push(Value::from($x));
        })*

        res
    }};

    ( $pos:expr, [ $( $x:expr ),* ] ) => {{
        let mut res = Signal::empty($pos, Direction::default());

        $({
            res.push(Value::from($x));
        })*

        res
    }};

    ( $pos:expr, $dir:expr) => {{
        let mut res = Signal::empty($pos, $dir);

        res
    }};

    ( $pos:expr) => {{
        let mut res = Signal::empty($pos, Direction::default());

        res
    }};
}
