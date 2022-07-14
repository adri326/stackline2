use stackline::prelude::*;
mod common;

#[test]
fn test_wire_loop() {
    let mut world = load_test!("tests/wire/loop.json");

    run!(world, 6);
    assert_no_signal!(world, 0, 0);
    assert_signal!(world, 3, 3);

    run!(world, 6);
    assert_signal!(world, 0, 0);
    assert_no_signal!(world, 3, 3);
}

#[test]
fn test_diode_loop() {
    let mut world = load_test!("tests/wire/diode-loop.json");

    run!(world, 2);
    assert_no_signal!(world, 0, 0);
    assert_signal!(world, 1, 1);

    run!(world, 4);
    assert_signal!(world, 1, 1);
}

#[test]
fn test_display_oob() {
    let world = load_test!("tests/wire/diode-loop.json");

    println!("{}", world);

    let mut surface = TextSurface::new(0, 0);
    world.draw(0, 0, &mut surface);
    world.draw(-1000, -1000, &mut surface);
}
