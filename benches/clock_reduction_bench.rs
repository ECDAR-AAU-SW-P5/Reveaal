use std::ops::Deref;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicU64, Ordering};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup};
use reveaal::ComponentLoader;

pub struct Trallocator<A: GlobalAlloc>{
    alloc: A,
    allocated: AtomicU64,
    freed: AtomicU64,
    max_size: AtomicU64,
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for Trallocator<A> {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        self.allocated.fetch_add(l.size() as u64, Ordering::SeqCst);
        self.calc_size();
        self.alloc.alloc(l)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
        self.alloc.dealloc(ptr, l);
        self.freed.fetch_add(l.size() as u64, Ordering::SeqCst);
        self.calc_size();
    }
}

impl<A: GlobalAlloc> Trallocator<A> {
    pub const fn new(a: A) -> Self {
        Trallocator{
            alloc: a,
            allocated: AtomicU64::new(0),
            freed: AtomicU64::new(0),
            max_size: AtomicU64::new(0),
        }
    }

    pub fn reset(&self) {
        self.freed.store(0, Ordering::SeqCst);
        self.max_size.store(0, Ordering::SeqCst);
        self.allocated.store(0, Ordering::SeqCst);
    }
    pub fn get_allocated(&self) -> u64 {
        self.allocated.load(Ordering::SeqCst)
    }

    pub fn get_freed(&self) -> u64 {
        self.freed.load(Ordering::SeqCst)
    }

    fn calc_size(&self) {
        if let Some(size) = self.get_allocated().checked_sub(self.get_freed()) {
            if size > self.get_max_size() {
                self.max_size.store(size, Ordering::SeqCst);
            }
        }
    }

    pub fn get_max_size(&self) -> u64 {
        self.max_size.load(Ordering::SeqCst)
    }

    pub fn get_current_size(&self) -> u64 {
        self.get_allocated() - self.get_freed()
    }
}
use std::alloc::System;
use std::fmt::format;
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
    println!("{id} | {} allocated | {} freed | {} max size", format_bytes(GLOBAL.get_allocated()), format_bytes(GLOBAL.get_freed()),  format_bytes(GLOBAL.get_max_size()));
    group.bench_function(id, |b| {
        loader.get_settings_mut().disable_clock_reduction = disable_clock_reduction;
        b.iter(|| clock_reduction_helper(loader, input));
    });
    println!("{id} | {} allocated | {} freed | {} current size | {} max size", format_bytes(GLOBAL.get_allocated()), format_bytes(GLOBAL.get_freed()), format_bytes(GLOBAL.get_current_size()),  format_bytes(GLOBAL.get_max_size()));

}

fn format_bytes(bytes: u64) -> String {
    match bytes.checked_ilog2() {
        Some(0..=9) => format!("{} B", bytes),
        Some(10..=19) => format!("{} KiB", bytes / 1024),
        Some(20..=29) => format!("{} MiB", bytes / (1024 * 1024)),
        Some(30..=39) => format!("{} GiB", bytes / (1024 * 1024 * 1024)),
        Some(40..=49) => format!("{} TiB", bytes / (1024 * 1024 * 1024 * 1024)),
        Some(50..=59) => format!("{} PiB", bytes / (1024 * 1024 * 1024 * 1024 * 1024)),
        Some(_) => format!("{bytes} B"),
        None => format!("{bytes} B"),
    }
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
