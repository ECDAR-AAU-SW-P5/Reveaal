use crate::protobuf_server::services::ecdar_backend_server::EcdarBackendServer;
use crate::protobuf_server::ConcreteEcdarBackend;
use core::time::Duration;
use tokio::runtime;
use tonic::transport::Server;

pub fn start_grpc_server_with_tokio(
    ip_endpoint: &str,
    cache_size: usize,
    thread_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    //For information on switching to a multithreaded server see:
    //https://docs.rs/tokio/1.12.0/tokio/runtime/index.html#multi-thread-scheduler
    let single_threaded_runtime = runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()?;

    single_threaded_runtime
        .block_on(async { start_grpc_server(ip_endpoint, cache_size, thread_number).await })
}

async fn start_grpc_server(
    ip_endpoint: &str,
    cache_size: usize,
    thread_number: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::builder()
        .http2_keepalive_interval(Some(Duration::from_secs(120)))
        .add_service(EcdarBackendServer::new(ConcreteEcdarBackend::new(
            thread_number,
            cache_size,
        )))
        .serve(ip_endpoint.trim().parse()?);
    println!("Started grpc server on '{}'\r", ip_endpoint.trim());

    server.await?;
    Ok(())
}
