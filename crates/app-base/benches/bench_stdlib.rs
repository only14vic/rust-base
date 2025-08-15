use {
    criterion::{Criterion, criterion_group, criterion_main},
    std::{rc::Rc, sync::Arc}
};

fn arc_vs_rc_clone(c: &mut Criterion) {
    let arc = Arc::new(String::from("test"));
    c.bench_function("Arc::clone()", |b| {
        b.iter(|| {
            let _ = arc.clone();
        })
    });

    let rc = Rc::new(String::from("test"));
    c.bench_function("Rc::clone()", |b| {
        b.iter(|| {
            let _ = rc.clone();
        })
    });
}

criterion_group!(benches, arc_vs_rc_clone);
criterion_main!(benches);
