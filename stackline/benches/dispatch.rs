use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stackline::prelude::*;
use stackline::tile::*;

fn benchmark_step(c: &mut Criterion) {
    c.bench_function("Pane::step", |b| {
        let mut pane = Pane::empty(4, 4).unwrap();

        pane.set_tile((0, 0), Diode::new(Direction::Right));
        pane.set_tile((3, 0), Diode::new(Direction::Down));
        pane.set_tile((3, 3), Diode::new(Direction::Left));
        pane.set_tile((0, 3), Diode::new(Direction::Up));

        for n in 1..3 {
            pane.set_tile((n, 0), Wire::new(Orientation::Horizontal));
            pane.set_tile((n, 3), Wire::new(Orientation::Horizontal));
            pane.set_tile((0, n), Wire::new(Orientation::Vertical));
            pane.set_tile((3, n), Wire::new(Orientation::Vertical));
        }

        pane.set_signal((0, 0), stackline::signal!(
            (0, 0),
            Direction::Right,
            []
        ));

        pane.set_signal((3, 0), stackline::signal!(
            (3, 0),
            Direction::Down,
            []
        ));

        pane.set_signal((3, 3), stackline::signal!(
            (3, 3),
            Direction::Left,
            []
        ));

        pane.set_signal((0, 3), stackline::signal!(
            (0, 3),
            Direction::Up,
            []
        ));

        b.iter(|| pane.step());
    });
}

criterion_group!(benches, benchmark_step);
criterion_main!(benches);
