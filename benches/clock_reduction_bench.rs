use std::ops::Deref;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicU64, Ordering};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup};
use reveaal::ComponentLoader;

pub struct Trallocator<A: GlobalAlloc>(pub A, AtomicU64);

unsafe impl<A: GlobalAlloc> GlobalAlloc for Trallocator<A> {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        self.1.fetch_add(l.size() as u64, Ordering::SeqCst);
        self.0.alloc(l)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
        self.0.dealloc(ptr, l);
        self.1.fetch_sub(l.size() as u64, Ordering::SeqCst);
    }
}

impl<A: GlobalAlloc> Trallocator<A> {
    pub const fn new(a: A) -> Self {
        Trallocator(a, AtomicU64::new(0))
    }

    pub fn reset(&self) {
        self.1.store(0, Ordering::SeqCst);
    }
    pub fn get(&self) -> u64 {
        self.1.load(Ordering::SeqCst)
    }
}
use std::alloc::System;
use criterion::measurement::WallTime;

#[global_allocator]
static GLOBAL: Trallocator<System> = Trallocator::new(System);


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

    add_benchmark(&mut group, &mut loader, "Refinement - No reduction", REFINEMENT_QUERY, true);
    add_benchmark(&mut group, &mut loader, "Refinement - With reduction", REFINEMENT_QUERY, false);

    group.finish();
}

fn add_benchmark(group: &mut BenchmarkGroup<WallTime>, loader: &mut Box<dyn ComponentLoader>, id: &str, input: &str, disable_clock_reduction: bool) {
    GLOBAL.reset();
    println!("Starting memory {id} | {} bytes", GLOBAL.get());
    group.bench_function(id, |b| {
        loader.get_settings_mut().disable_clock_reduction = disable_clock_reduction;
        b.iter(|| clock_reduction_helper(loader, input));
    });
    println!("Ending memory memory {id} | {} bytes", GLOBAL.get());
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
