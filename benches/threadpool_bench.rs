use std::vec;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use reveaal::data_reader::json_writer::component_to_json;
use reveaal::protobuf_server::{
    services::{
        component::Rep, ecdar_backend_server::EcdarBackend, Component as ProtoComp, ComponentsInfo,
        QueryRequest,
    },
    ConcreteEcdarBackend,
};
use tonic::Request;

use criterion::async_executor::FuturesExecutor;
use futures::stream::FuturesUnordered;
use futures::StreamExt;

mod bench_helper;
pub mod flamegraph;
use flamegraph::flamegraph_profiler::FlamegraphProfiler;
use reveaal::model_objects::Component;

const NUM_OF_REQUESTS: u32 = 512;

fn send_query_with_components(
    id: String,
    c: &mut Criterion,
    components: &[Component],
    query: &str,
    active_cache: bool,
) {
    c.bench_function(&id, |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let backend = ConcreteEcdarBackend::default();
            let responses = (0..NUM_OF_REQUESTS)
                .map(|hash| {
                    let request = compose_query_request(
                        &components.iter().map(component_to_json).collect::<Vec<_>>(),
                        query,
                        if active_cache { 0 } else { hash },
                    );
                    backend.send_query(request)
                })
                .collect::<FuturesUnordered<_>>();

            _ = black_box(responses.collect::<Vec<_>>().await);
        });
    });
}

fn compose_query_request(json: &[String], query: &str, hash: u32) -> Request<QueryRequest> {
    Request::new(QueryRequest {
        user_id: 0,
        query_id: 0,
        query: String::from(query),
        components_info: Some(ComponentsInfo {
            components: construct_components(json),
            components_hash: hash,
        }),
        settings: None,
    })
}

fn construct_components(json: &[String]) -> Vec<ProtoComp> {
    json.iter()
        .map(|json| ProtoComp {
            rep: Some(Rep::Json(json.clone())),
        })
        .collect()
}

fn threadpool_cache(c: &mut Criterion) {
    let mut loader = bench_helper::get_uni_loader();
    let comps = vec![
        loader.get_component("Administration").unwrap().clone(),
        loader.get_component("Researcher").unwrap().clone(),
        loader.get_component("Machine").unwrap().clone(),
    ];
    let expensive_query = String::from("determinism: Administration || Researcher || Machine");
    let cheap_query = String::from("determinism: Machine");

    send_query_with_components(
        String::from("Expensive queries with identical models"),
        c,
        &comps,
        &expensive_query,
        true,
    );
    send_query_with_components(
        String::from("Expensive queries with different models"),
        c,
        &comps,
        &expensive_query,
        false,
    );
    send_query_with_components(
        String::from("Cheap queries with identical models"),
        c,
        &comps,
        &cheap_query,
        true,
    );
    send_query_with_components(
        String::from("Cheap queries with different models"),
        c,
        &comps,
        &cheap_query,
        false,
    );
}

criterion_group! {
    name = backend_bench;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(100)).sample_size(10);
    targets = threadpool_cache
}

criterion_main!(backend_bench);
