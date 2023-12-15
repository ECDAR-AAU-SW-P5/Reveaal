mod ecdar_backend;
mod ecdar_requests;
mod proto_conversions;
mod server;

pub mod services {
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("ecdar_proto_buf");
}

pub use ecdar_backend::ConcreteEcdarBackend;
pub use server::start_grpc_server_with_tokio;
