use std::fs;

use tonic::{Response, Status};

use crate::protobuf_server::services::component::Rep;
use crate::protobuf_server::services::{
    Component as ProtoComponent, ComponentsInfo as ProtoComponentsInfo,
    SimulationInfo as ProtoSimulationInfo, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse,
};

pub trait NewSimulationHelper {
    fn new(component_names: &[&str], components_path: &str, composition: &str) -> Self;
}

impl NewSimulationHelper for SimulationStartRequest {
    fn new(component_names: &[&str], components_path: &str, composition: &str) -> Self {
        let simulation_info =
            ProtoSimulationInfo::new(component_names, components_path, composition);
        SimulationStartRequest {
            simulation_info: Some(simulation_info),
        }
    }
}

pub fn construct_step_requests(
    component_names: &[&str],
    components_path: &str,
    composition: &str,
    last_response: Result<Response<SimulationStepResponse>, Status>,
) -> impl Iterator<Item = SimulationStepRequest> {
    let simulation_info = ProtoSimulationInfo::new(component_names, components_path, composition);
    let last_response = last_response.unwrap().into_inner();
    last_response
        .new_decision_points
        .into_iter()
        .map(move |d| SimulationStepRequest {
            simulation_info: Some(simulation_info.clone()),
            chosen_decision: Some(d),
        })
}

impl NewSimulationHelper for ProtoSimulationInfo {
    fn new(component_names: &[&str], components_path: &str, composition: &str) -> Self {
        let json_components: Vec<_> = component_names
            .iter()
            .map(|component_name| ProtoComponent {
                rep: Some(Rep::Json(
                    fs::read_to_string(format!(
                        "{}/Components/{}.json",
                        components_path, component_name
                    ))
                    .unwrap(),
                )),
            })
            .collect();

        ProtoSimulationInfo {
            user_id: 0,
            component_composition: composition.to_string(),
            components_info: Some(ProtoComponentsInfo {
                components: json_components,
                components_hash: 0,
            }),
        }
    }
}
