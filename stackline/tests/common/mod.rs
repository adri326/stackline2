#[macro_export]
macro_rules! load_test {
    ( $path:expr ) => {{
        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), $path);
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("Couldn't load {}: {}", &path, err);
        });
        let world: stackline::prelude::World =
            serde_json::from_str(&raw).expect("Couldn't parse World");
        world
    }};
}

#[macro_export]
macro_rules! run {
    ( $world:expr ) => {
        $world.step();
    };

    ( $world:expr, $steps:expr ) => {
        for _step in 0..$steps {
            $world.step();
        }
    };
}

#[macro_export]
macro_rules! assert_signal {
    ( $world:expr, $x:expr, $y:expr ) => {{
        let guard = $world
            .get(($x, $y))
            .expect(&format!("Couldn't get tile at {}:{}", $x, $y));
        let signal = guard.signal();
        assert!(
            signal.is_some(),
            "Expected signal at {}:{}!\n{}",
            $x,
            $y,
            $world
        );
        signal
    }};

    ( $world:expr, $x:expr, $y:expr, [ $( $data:expr ),* ] ) => {{
        let signal = assert_signal!($pane, $x, $y);
        // TODO: check that signal.data == data
    }};
}

#[macro_export]
macro_rules! assert_no_signal {
    ( $world:expr, $x:expr, $y:expr ) => {{
        let guard = $world
            .get(($x, $y))
            .expect(&format!("Couldn't get tile at {}:{}", $x, $y));
        let signal = guard.signal();
        assert!(
            signal.is_none(),
            "Expected no signal at {}:{}!\n{}",
            $x,
            $y,
            $world
        );
    }};
}
