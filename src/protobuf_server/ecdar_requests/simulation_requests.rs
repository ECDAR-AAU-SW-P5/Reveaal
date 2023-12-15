use tonic::Status;

use crate::model_objects::Decision;
use crate::{
    data_reader::{component_loader::ModelCache, proto_reader::proto_decision_to_decision},
    protobuf_server::{
        services::{SimulationStartRequest, SimulationStepRequest, SimulationStepResponse},
        ConcreteEcdarBackend,
    },
    system::specifics::{SpecificDecision, SpecificState},
};

use super::request_util::simulation_info_to_transition_system;

impl ConcreteEcdarBackend {
    /// Handles a start simulation request: Responding with the initial decision point in the transition system given in the `request`.
    pub fn handle_start_simulation(
        request: SimulationStartRequest,
        mut cache: ModelCache,
    ) -> Result<SimulationStepResponse, Status> {
        let simulation_info = request.simulation_info.unwrap();

        let transition_system = simulation_info_to_transition_system(&simulation_info, &mut cache);

        // Get the decisions from the initial state and convert them to proto
        let decisions = Decision::get_initial_decisions(&transition_system)
            .into_iter()
            .map(|i| SpecificDecision::from_decision(&i, &*transition_system).into())
            .collect();

        let state = transition_system.get_initial_state();

        let full_state =
            state.map(|state| SpecificState::from_state(&state, &*transition_system).into());

        Ok(SimulationStepResponse {
            full_state,
            new_decision_points: decisions,
        })
    }

    /// Handles a take simulation step request:
    /// Given a `decision` and transition system in the `request`, walk along the decided edge and respond with the resulting decision points.
    pub fn handle_take_simulation_step(
        request: SimulationStepRequest,
        mut cache: ModelCache,
    ) -> Result<SimulationStepResponse, Status> {
        let request_message = request;
        let simulation_info = request_message.simulation_info.unwrap();

        let system = simulation_info_to_transition_system(&simulation_info, &mut cache);

        let chosen_decision = request_message.chosen_decision.unwrap();

        let full_state = chosen_decision.destination.clone();

        let chosen_decision = proto_decision_to_decision(chosen_decision, &system);

        let decision_points = chosen_decision.resolve(&system);

        let decision_points = decision_points
            .into_iter()
            .map(|i| SpecificDecision::from_decision(&i, &*system).into())
            .collect();

        Ok(SimulationStepResponse {
            full_state,
            new_decision_points: decision_points,
        })
    }
}
