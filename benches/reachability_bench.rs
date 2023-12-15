use criterion::{criterion_group, criterion_main, Criterion};
pub mod bench_helper;
pub mod flamegraph;
use flamegraph::flamegraph_profiler::FlamegraphProfiler;
use reveaal::extract_system_rep::create_executable_query;
use reveaal::ModelObjects::Query;
use reveaal::{parse_queries, ComponentLoader};

fn bench_reachability(c: &mut Criterion, query: &str, loader: &mut Box<dyn ComponentLoader>) {
    c.bench_function(query, |b| {
        b.iter(|| {
            let query = parse_queries::parse_to_expression_tree(query)
                .unwrap()
                .remove(0);
            let q = Query {
                query: Option::from(query),
                comment: "".to_string(),
            };

            let query = create_executable_query(&q, loader.as_mut()).unwrap();

            query.execute()
        })
    });
}

fn reachability_benchmarking(c: &mut Criterion) {
    let mut loader = bench_helper::get_uni_loader(false);

    bench_reachability(
        c,
        "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4 && Researcher.L9",
    &mut loader);
    bench_reachability(
        c,
        "reachability: Administration || Machine || Researcher @ Administration.L3 && Machine.L5 && Researcher.L9 -> Administration.L0 && Machine.L5 && Researcher.U0",
    &mut loader);
    bench_reachability(
        c,
        "reachability: Administration || Machine || Researcher @ Administration.L0 && Machine.L5 && Researcher.U0 -> Administration.L3 && Machine.L5 && Researcher.L9",
    &mut loader);
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 && Machine.y<6 -> Machine.L4 && Machine.y<=6",
        &mut loader,
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 -> Machine.L4 && Machine.y>7",
        &mut loader,
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L4 && Machine.y<=6 -> Machine.L5 && Machine.y>=4",
        &mut loader,
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 && Machine.y<1 -> Machine.L5 && Machine.y<2",
        &mut loader,
    );
    bench_reachability(
        c,
        "reachability: Machine @ Machine.L5 -> Machine.L5",
        &mut loader,
    );
    bench_reachability(
        c,
        "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Machine.L5 && Researcher.L7",
    &mut loader);
    bench_reachability(
        c,
        "reachability: Researcher @ Researcher.U0 -> Researcher.L7",
        &mut loader,
    );
}

criterion_group! {
  name = reachability_benches;
  config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
  targets = reachability_benchmarking
}

criterion_main!(reachability_benches);
