use {
    criterion::{Criterion, criterion_group, criterion_main},
    std::{rc::Rc, sync::Arc}
};

fn arc_clone(c: &mut Criterion) {
    let x = Arc::new(String::from("test"));
    c.bench_function("Arc::clone()", |b| {
        b.iter(|| {
            let _ = x.clone();
        })
    });
}

fn rc_clone(c: &mut Criterion) {
    let x = Rc::new(String::from("test"));
    c.bench_function("Rc::clone()", |b| {
        b.iter(|| {
            let _ = x.clone();
        })
    });
}

criterion_group!(benches, arc_clone, rc_clone);
criterion_main!(benches);
