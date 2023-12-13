use criterion::{criterion_group, criterion_main, Criterion};
mod bench_helper;
pub mod flamegraph;
use flamegraph::flamegraph_profiler::FlamegraphProfiler;
use reveaal::data_reader::json_writer::component_to_json;
use reveaal::model_objects::Component;
use reveaal::protobuf_server::services::component::Rep::Json;
use reveaal::protobuf_server::services::{Component as ProtoComp, ComponentsInfo, SimulationInfo};
use reveaal::{
    data_reader::component_loader::ModelCache,
    protobuf_server::{
        services::{SimulationStartRequest, SimulationStepRequest},
        ConcreteEcdarBackend,
    },
};
use tonic::Response;

fn construct_sim_info(components: &[Component], comp: &str, id: i32) -> SimulationInfo {
    SimulationInfo {
        user_id: id,
        component_composition: comp.to_string(),
        components_info: Some(ComponentsInfo {
            components: components
                .iter()
                .map(|c| ProtoComp {
                    rep: Some(Json(component_to_json(c))),
                })
                .collect(),
            components_hash: id as u32,
        }),
    }
}

fn construct_start_request(
    components: &[Component],
    comp: &str,
    id: i32,
) -> SimulationStartRequest {
    SimulationStartRequest {
        simulation_info: Some(construct_sim_info(components, comp, id)),
    }
}

fn construct_step_request(
    components: &[Component],
    comp: &str,
    id: i32,
    last_response: &SimulationStartRequest,
) -> SimulationStepRequest {
    let cache = ModelCache::default();
    let s = ConcreteEcdarBackend::handle_start_simulation(last_response.clone(), cache)
        .map(Response::new)
        .unwrap();
    SimulationStepRequest {
        simulation_info: Some(construct_sim_info(components, comp, id)),
        chosen_decision: Some(s.into_inner().new_decision_points[0].clone()),
    }
}

fn start_simulation(c: &mut Criterion, id: &str, request: SimulationStartRequest) {
    let cache = ModelCache::default();
    c.bench_function(id, |b| {
        b.iter(|| ConcreteEcdarBackend::handle_start_simulation(request.to_owned(), cache.clone()))
    });
}

fn take_simulation_step(c: &mut Criterion, id: &str, request: SimulationStepRequest) {
    let cache = ModelCache::default();
    c.bench_function(id, |b| {
        b.iter(|| ConcreteEcdarBackend::handle_take_simulation_step(request.clone(), cache.clone()))
    });
}

fn simulation(c: &mut Criterion) {
    let mut loader = bench_helper::get_uni_loader();

    let start_request_1 = construct_start_request(
        &[loader.get_component("Machine").unwrap().clone()],
        "(Machine)",
        1,
    );

    let start_request_2 = construct_start_request(
        &[
            loader.get_component("HalfAdm1").unwrap().clone(),
            loader.get_component("HalfAdm2").unwrap().clone(),
        ],
        "(HalfAdm1 && HalfAdm2)",
        2,
    );

    let start_request_3 = construct_start_request(
        &[
            loader.get_component("Machine").unwrap().clone(),
            loader.get_component("Administration").unwrap().clone(),
            loader.get_component("Researcher").unwrap().clone(),
        ],
        "(Administration || Machine || Researcher)",
        3,
    );

    let start_request_4 = construct_start_request(
        &[
            loader.get_component("Machine").unwrap().clone(),
            loader.get_component("HalfAdm1").unwrap().clone(),
            loader.get_component("HalfAdm2").unwrap().clone(),
            loader.get_component("Researcher").unwrap().clone(),
        ],
        "((HalfAdm1 && HalfAdm2) || Machine || Researcher)",
        4,
    );

    let step_request_1 = construct_step_request(
        &[loader.get_component("Machine").unwrap().clone()],
        "(Machine)",
        1,
        &start_request_1,
    );

    let step_request_2 = construct_step_request(
        &[
            loader.get_component("HalfAdm1").unwrap().clone(),
            loader.get_component("HalfAdm2").unwrap().clone(),
        ],
        "(HalfAdm1 && HalfAdm2)",
        2,
        &start_request_2,
    );

    let step_request_3 = construct_step_request(
        &[
            loader.get_component("Machine").unwrap().clone(),
            loader.get_component("Administration").unwrap().clone(),
            loader.get_component("Researcher").unwrap().clone(),
        ],
        "(Administration || Machine || Researcher)",
        3,
        &start_request_3,
    );

    let step_request_4 = construct_step_request(
        &[
            loader.get_component("Machine").unwrap().clone(),
            loader.get_component("HalfAdm1").unwrap().clone(),
            loader.get_component("HalfAdm2").unwrap().clone(),
            loader.get_component("Researcher").unwrap().clone(),
        ],
        "((HalfAdm1 && HalfAdm2) || Machine || Researcher)",
        4,
        &start_request_4,
    );

    start_simulation(c, "start simulation for (Machine)", start_request_1);
    start_simulation(
        c,
        "start simulation for (HalfAdm1 && HalfAdm2)",
        start_request_2,
    );
    start_simulation(
        c,
        "start simulation for (Administration || Machine || Researcher)",
        start_request_3,
    );
    start_simulation(
        c,
        "start simulation for ((HalfAdm1 && HalfAdm2) || Machine || Researcher)",
        start_request_4,
    );

    take_simulation_step(c, "take simulation step for (Machine)", step_request_1);
    take_simulation_step(
        c,
        "take simulation step for (HalfAdm1 && HalfAdm2)",
        step_request_2,
    );
    take_simulation_step(
        c,
        "take simulation step for (Administration || Machine || Researcher)",
        step_request_3,
    );
    take_simulation_step(
        c,
        "take simulation step for ((HalfAdm1 && HalfAdm2) || Machine || Researcher)",
        step_request_4,
    );
}

criterion_group! {
  name = simulation_benches;
  config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
  targets = simulation
}

criterion_main!(simulation_benches);
