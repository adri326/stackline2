use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stackline::prelude::*;
use stackline::tile::*;

fn benchmark_step(c: &mut Criterion) {
    c.bench_function("Pane::step", |b| {
        let mut pane = Pane::empty(3, 3).unwrap();

        pane.set_tile((0, 0), Diode::new(Direction::Right));
        pane.set_tile((2, 0), Diode::new(Direction::Down));
        pane.set_tile((2, 2), Diode::new(Direction::Left));
        pane.set_tile((0, 2), Diode::new(Direction::Up));
        pane.set_tile((1, 0), Wire::new(Orientation::Horizontal));
        pane.set_tile((1, 2), Wire::new(Orientation::Horizontal));
        pane.set_tile((0, 1), Wire::new(Orientation::Vertical));
        pane.set_tile((2, 1), Wire::new(Orientation::Vertical));

        pane.set_signal((0, 0), stackline::signal!(
            (0, 0),
            Direction::Right,
            []
        ));

        b.iter(|| pane.step());
    });
}

criterion_group!(benches, benchmark_step);
criterion_main!(benches);
