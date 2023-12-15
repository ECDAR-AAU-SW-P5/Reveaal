use crate::protobuf_server::services::ecdar_backend_server::EcdarBackend;

use crate::data_reader::component_loader::ModelCache;
use crate::protobuf_server::services::{
    QueryRequest, QueryResponse, SimulationStartRequest, SimulationStepRequest,
    SimulationStepResponse, UserTokenResponse,
};
use futures::executor::block_on;
use futures::FutureExt;
use std::panic::UnwindSafe;
use std::sync::atomic::{AtomicI32, Ordering};
use tonic::{Request, Response, Status};

use rayon::{ThreadPool, ThreadPoolBuilder};

#[derive(Debug)]
pub struct ConcreteEcdarBackend {
    thread_pool: ThreadPool,
    model_cache: ModelCache,
    num: AtomicI32,
}

impl ConcreteEcdarBackend {
    pub fn new(thread_count: usize, cache_size: usize) -> Self {
        ConcreteEcdarBackend {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .unwrap(),
            model_cache: ModelCache::new(cache_size),
            num: AtomicI32::new(1),
        }
    }
}

impl Default for ConcreteEcdarBackend {
    fn default() -> Self {
        ConcreteEcdarBackend {
            thread_pool: ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .build()
                .unwrap(),

            model_cache: ModelCache::default(),
            num: AtomicI32::new(1),
        }
    }
}

async fn catch_unwind<T, O>(future: T) -> Result<Response<O>, Status>
where
    T: UnwindSafe + futures::Future<Output = Result<O, Status>>,
{
    fn downcast_to_string(e: Box<dyn std::any::Any + Send>) -> String {
        match e.downcast::<String>() {
            Ok(v) => *v,
            Err(e) => match e.downcast::<&str>() {
                Ok(v) => v.to_string(),
                _ => "Unknown Source of Error".to_owned(),
            },
        }
    }

    match future.catch_unwind().await {
        Ok(response) => response,
        Err(e) => Err(Status::internal(format!(
            "{}, please report this bug to the developers",
            downcast_to_string(e)
        ))),
    }
    .map(Response::new)
}

impl ConcreteEcdarBackend {}

#[tonic::async_trait]
impl EcdarBackend for ConcreteEcdarBackend {
    async fn get_user_token(
        &self,
        _request: Request<()>,
    ) -> Result<Response<UserTokenResponse>, Status> {
        let id = self.num.fetch_add(1, Ordering::SeqCst);
        let token_response = UserTokenResponse { user_id: id };
        Result::Ok(Response::new(token_response))
    }

    async fn send_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        async fn async_query(
            request: QueryRequest,
            cache: ModelCache,
        ) -> Result<QueryResponse, Status> {
            ConcreteEcdarBackend::handle_send_query(request, cache)
        }
        let cache = self.model_cache.clone();

        self.thread_pool
            .install(|| block_on(catch_unwind(async_query(request.into_inner(), cache))))

        // TODO: Test whether there is a large performance difference between block_on and the non-catching commented out code below
        // self.thread_pool.install(|| {
        //     ConcreteEcdarBackend::handle_send_query(request.into_inner(), cache).map(Response::new)
        // })
    }

    async fn start_simulation(
        &self,
        request: Request<SimulationStartRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        async fn async_start_simulation(
            request: SimulationStartRequest,
            cache: ModelCache,
        ) -> Result<SimulationStepResponse, Status> {
            ConcreteEcdarBackend::handle_start_simulation(request, cache)
        }

        catch_unwind(async_start_simulation(
            request.into_inner(),
            self.model_cache.clone(),
        ))
        .await
    }

    async fn take_simulation_step(
        &self,
        request: Request<SimulationStepRequest>,
    ) -> Result<Response<SimulationStepResponse>, Status> {
        async fn async_simulation_step(
            request: SimulationStepRequest,
            cache: ModelCache,
        ) -> Result<SimulationStepResponse, Status> {
            ConcreteEcdarBackend::handle_take_simulation_step(request, cache)
        }

        catch_unwind(async_simulation_step(
            request.into_inner(),
            self.model_cache.clone(),
        ))
        .await
    }
}
