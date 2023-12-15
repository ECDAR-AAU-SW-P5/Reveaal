pub mod cli;
pub mod data_reader;
pub mod edge_eval;
pub mod logging;
pub mod model_objects;
pub mod protobuf_server;
pub mod simulation;
pub mod system;
pub mod tests;
pub mod transition_systems;

pub use crate::data_reader::component_loader::{
    ComponentLoader, JsonProjectLoader, ProjectLoader, XmlProjectLoader,
};
pub use crate::data_reader::{parse_queries, xml_parser};
use crate::protobuf_server::services::query_request::Settings;
pub use crate::system::extract_system_rep;
pub use protobuf_server::start_grpc_server_with_tokio;

/// The default settings
pub const DEFAULT_SETTINGS: Settings = Settings {
    disable_clock_reduction: true,
};

#[macro_use]
extern crate pest_derive;
extern crate colored;
extern crate core;
extern crate serde;
extern crate serde_xml_rs;
extern crate simple_error;
extern crate xml;
#[macro_use]
extern crate lazy_static;
