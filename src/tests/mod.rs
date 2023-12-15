use crate::protobuf_server::services::query_request::Settings;

pub mod edge_ids;
pub mod failure_message;
pub mod grpc;
pub mod model_objects;
pub mod reachability;
pub mod refinement;
pub mod sample;
pub mod save_component;
pub mod simulation;
pub mod system_recipe;

/// The default settings for Testing
pub const TEST_SETTINGS: Settings = Settings {
    disable_clock_reduction: false,
};
