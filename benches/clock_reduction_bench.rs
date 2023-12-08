use std::ops::Deref;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicU64, Ordering};
use criterion::{criterion_group, criterion_main, Criterion};
use reveaal::ComponentLoader;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
use jemalloc_ctl::{stats, epoch};

mod bench_helper;
use reveaal::extract_system_rep::create_executable_query;
use reveaal::parse_queries::parse_to_query;

// const QUERY: &str = "refinement: (((((Adm2 && HalfAdm1 && HalfAdm2) || Machine || Researcher) && ((Adm2 && HalfAdm1) || Machine || Researcher) && ((Adm2 && HalfAdm2) || Machine || Researcher) && ((HalfAdm1 && HalfAdm2) || Machine || Researcher) && (Adm2 || Machine || Researcher)) // (Adm2 && HalfAdm1 && HalfAdm2)) // Researcher) <= (((((Adm2 && HalfAdm1 && HalfAdm2) || Machine || Researcher) && ((Adm2 && HalfAdm1) || Machine || Researcher) && ((Adm2 && HalfAdm2) || Machine || Researcher) && ((HalfAdm1 && HalfAdm2) || Machine || Researcher) && (Adm2 || Machine || Researcher)) // (Adm2 && HalfAdm1 && HalfAdm2)) // Researcher)";
const REFINEMENT_QUERY: &str = "refinement: Researcher <= Spec // Administration // Machine";
const REACHABILITY_QUERY: &str = "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4 && Researcher.L9";
const CONSISTENCY_QUERY: &str = "consistency: Machine || Researcher";
const SYNTAX_QUERY: &str = "syntax: Researcher";
const GETCOMPONENT_QUERY: &str = "get-component: Adm2 || Machine save-as get_component_test";
const DETERMINISM_QUERY: &str = "determinism: Researcher && Machine";
const LONG_QUERY: &str = "refinement: (((((Adm2 && HalfAdm1 && HalfAdm2) || Machine || Researcher) && ((Adm2 && HalfAdm1) || Machine || Researcher) && ((Adm2 && HalfAdm2) || Machine || Researcher) && ((HalfAdm1 && HalfAdm2) || Machine || Researcher) && (Adm2 || Machine || Researcher)) // (Adm2 && HalfAdm1 && HalfAdm2)) // Researcher) <= (((((Adm2 && HalfAdm1 && HalfAdm2) || Machine || Researcher) && ((Adm2 && HalfAdm1) || Machine || Researcher) && ((Adm2 && HalfAdm2) || Machine || Researcher) && ((HalfAdm1 && HalfAdm2) || Machine || Researcher) && (Adm2 || Machine || Researcher)) // (Adm2 && HalfAdm1 && HalfAdm2)) // Researcher)";

/// This bench runs `REFINEMENT QUERY` with and without clock reduction such that you can compare the results. It also runs other queries
fn bench_clock_reduction(c: &mut Criterion) {
    // Set up the bench.
    let mut loader = bench_helper::get_uni_loader();
    let mut group = c.benchmark_group("Clock Reduction");

    epoch::advance().unwrap();
    let start = stats::resident::read().unwrap();
    group.bench_function("Refinement check - No reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = false;
        b.iter(|| clock_reduction_helper(&mut loader, REFINEMENT_QUERY));
    });
    epoch::advance().unwrap();
    let end = stats::resident::read().unwrap();
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident. Total allocation change {}", allocated, resident, end - start);

    epoch::advance().unwrap();
    let start = stats::resident::read().unwrap();
    group.bench_function("Refinement check - With reduction", |b| {
        loader.get_settings_mut().disable_clock_reduction = true;
        b.iter(|| clock_reduction_helper(&mut loader, REFINEMENT_QUERY));
    });
    let end = stats::resident::read().unwrap();
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident. Total allocation change {}", allocated, resident, end - start);

    group.finish();
}

fn clock_reduction_helper(loader: &mut Box<dyn ComponentLoader>, input: &str) {
    let query = parse_to_query(input);
    let executable_query = create_executable_query(query.get(0).unwrap(), loader.as_mut())
        .unwrap();
    executable_query.execute();
}

criterion_group! {
    name = clock_reduction_bench;
    config = Criterion::default().sample_size(50);
    targets = bench_clock_reduction
}
criterion_main!(clock_reduction_bench);
