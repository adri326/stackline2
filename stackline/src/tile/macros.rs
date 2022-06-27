#[macro_export]
macro_rules! test_tile_setup {
    ( $width:expr, $height:expr, [ $( $x:expr ),* ] ) => {{
        assert!($width > 0);
        assert!($height > 0);
        let mut pane = crate::pane::Pane::empty($width, $height).unwrap();
        let mut index = 0;

        $(
            {
                let x = index % $width;
                let y = index / $width;
                *pane.get_mut((x, y)).unwrap() = crate::tile::FullTile::from($x);
                index += 1;
            }
        )*

        assert!(index == $width * $height);

        pane
    }}
}

#[macro_export]
macro_rules! test_set_signal {
    ( $pane:expr, $pos:expr, $dir:expr ) => {
        $pane.set_signal($pos, crate::signal::Signal::empty($pos, $dir)).unwrap();
    };
}

#[macro_export]
macro_rules! assert_signal {
    ( $pane:expr, $pos:expr ) => {{
        let guard = $pane
            .get($pos)
            .expect(&format!("Couldn't get tile at {:?}", $pos));
        let signal = guard.signal();
        assert!(signal.is_some());
        signal
    }};

    ( $pane:expr, $pos:expr, [ $( $data:expr ),* ] ) => {{
        let signal = assert_signal!($pane, $pos);
        // TODO: check that signal.data == data
    }};
}

#[macro_export]
macro_rules! assert_no_signal {
    ( $pane:expr, $pos:expr) => {{
        let guard = $pane
            .get($pos)
            .expect(&format!("Couldn't get tile at {:?}", $pos));
        let signal = guard.signal();
        assert!(signal.is_none());
    }};
}
