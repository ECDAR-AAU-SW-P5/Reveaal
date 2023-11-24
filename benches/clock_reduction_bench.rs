use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use reveaal::ComponentLoader;

mod bench_helper;
use reveaal::extract_system_rep::create_executable_query;
use reveaal::parse_queries::parse_to_query;

const REFINEMENT_QUERY: &str = "refinement: Researcher <= Spec // Administration // Machine";
const REACHABILITY_QUERY: &str = "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4 && Researcher.L9";
const CONSISTENCY_QUERY: &str = "consistency: Machine";
const GETCOMPONENT_QUERY: &str = "get-component: Adm2 || Machine save-as test";

/// This bench runs `REFINEMENT QUERY` with and without clock reduction such that you can compare the results. It also runs other queries
fn bench_clock_reduction(c: &mut Criterion) {
    // Set up the bench.
    let mut loader = bench_helper::get_uni_loader();
    let mut group = c.benchmark_group("Clock Reduction");
    group.bench_function("Refinement check - No reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = true;
        b.iter(|| clock_reduction_helper(&mut loader, REFINEMENT_QUERY));
    });
    group.bench_function("Refinement check - With reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = false;
        b.iter(|| clock_reduction_helper(&mut loader, REFINEMENT_QUERY));
    });
    group.bench_function("Reachability check - With reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = false;
        b.iter(|| clock_reduction_helper(&mut loader,REACHABILITY_QUERY));
    });
    group.bench_function("Consistency check - With reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = false;
        b.iter(|| clock_reduction_helper(&mut loader,CONSISTENCY_QUERY));
    });
    group.bench_function("GetComponent check - With reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = false;
        b.iter(|| clock_reduction_helper(&mut loader, GETCOMPONENT_QUERY));
    });

    group.finish();
}

fn clock_reduction_helper(loader: &mut Box<dyn ComponentLoader>, input: &str) {
    let query = parse_to_query(input);
    create_executable_query(query.get(0).unwrap(), loader.as_mut())
        .unwrap()
        .execute();
}

criterion_group! {
    name = clock_reduction_bench;
    config = Criterion::default().sample_size(10).warm_up_time(Duration::new(10,0));
    targets = bench_clock_reduction
}
criterion_main!(clock_reduction_bench);
