use crate::data_reader::component_loader::{ComponentContainer, ModelCache};
use crate::data_reader::json_writer::component_to_json;
use crate::data_reader::parse_queries;
use crate::extract_system_rep::ExecutableQueryError;
use crate::model_objects::Query;
use crate::protobuf_server::ecdar_requests::request_util::insert_model;
use crate::protobuf_server::services::component::Rep;
use crate::protobuf_server::services::query_response::{
    Error as InnerError, Result as ProtobufResult, Success,
};
use crate::protobuf_server::services::{
    query_response, Component as ProtobufComponent, QueryRequest, QueryResponse,
};
use crate::protobuf_server::ConcreteEcdarBackend;
use crate::system::query_failures::{
    ConsistencyFailure, DeterminismFailure, PathFailure, QueryResult, RefinementFailure,
    SyntaxFailure, SystemRecipeFailure,
};

use crate::system::extract_system_rep;

use log::trace;
use tonic::Status;

fn string_error(error: impl Into<String>) -> ProtobufResult {
    ProtobufResult::Error(InnerError {
        error: error.into(),
    })
}

impl ConcreteEcdarBackend {
    pub fn handle_send_query(
        query_request: QueryRequest,
        mut model_cache: ModelCache,
    ) -> Result<QueryResponse, Status> {
        trace!("Received query: {:?}", query_request);
        let components_info = query_request.components_info.as_ref().unwrap();
        let proto_components = &components_info.components;

        // Model already in cache
        if let Some(model) =
            model_cache.get_model(query_request.user_id, components_info.components_hash)
        {
            send_query(model, query_request)
        }
        // Model not in cache but included in request
        else if !proto_components.is_empty() {
            let model = insert_model(
                &mut model_cache,
                query_request.user_id,
                components_info.components_hash,
                proto_components,
            );
            send_query(model, query_request)
        }
        // Model not in cache nor included in request
        else {
            Ok(QueryResponse {
                query_id: query_request.query_id,
                info: vec![],
                result: Some(query_response::Result::ComponentsNotInCache(
                    Default::default(),
                )),
            })
        }
    }
}

fn send_query(
    mut model: ComponentContainer,
    query_request: QueryRequest,
) -> Result<QueryResponse, Status> {
    let query = parse_query(&query_request)?;

    model.set_settings(query_request.settings.unwrap_or(crate::DEFAULT_SETTINGS));

    match extract_system_rep::create_executable_query(&query, &mut model) {
        Ok(query) => {
            let result = query.execute();
            Ok(QueryResponse {
                query_id: query_request.query_id,
                info: vec![], // TODO: Should be logs
                result: Some(result.into()),
            })
        }
        Err(ExecutableQueryError::Custom(e)) => Err(Status::invalid_argument(format!(
            "Creation of query failed: {}",
            e
        ))),
        Err(ExecutableQueryError::SystemRecipeFailure(failure)) => {
            Ok(QueryResponse {
                query_id: query_request.query_id,
                info: vec![], // TODO: Should be logs
                result: Some(failure.into()),
            })
        }
    }
}

fn parse_query(query_request: &QueryRequest) -> Result<Query, Status> {
    let mut queries = parse_queries::parse_to_query(&query_request.query);

    if queries.len() != 1 {
        Err(Status::invalid_argument(
            "This procedure takes in exactly 1 query",
        ))
    } else {
        Ok(queries.remove(0))
    }
}

impl From<QueryResult> for ProtobufResult {
    fn from(result: QueryResult) -> ProtobufResult {
        match result {
            QueryResult::Reachability(Ok(path)) => ProtobufResult::ReachabilityPath(path.into()),
            QueryResult::Refinement(Ok(_))
            | QueryResult::Consistency(Ok(_))
            | QueryResult::Syntax(Ok(_))
            | QueryResult::Determinism(Ok(_)) => ProtobufResult::Success(Success {}),
            QueryResult::Refinement(Err(fail)) => fail.into(),
            QueryResult::Consistency(Err(fail)) => fail.into(),
            QueryResult::Determinism(Err(fail)) => fail.into(),
            QueryResult::Reachability(Err(fail)) => fail.into(),
            QueryResult::Syntax(Err(fail)) => fail.into(),

            QueryResult::GetComponent(comp) => ProtobufResult::Component(ProtobufComponent {
                rep: Some(Rep::Json(component_to_json(&comp))),
            }),

            QueryResult::RecipeFailure(recipe) => recipe.into(),
            QueryResult::CustomError(custom) => string_error(custom),
        }
    }
}

impl From<SystemRecipeFailure> for ProtobufResult {
    fn from(fail: SystemRecipeFailure) -> ProtobufResult {
        ProtobufResult::Model(fail.into())
    }
}

impl From<DeterminismFailure> for ProtobufResult {
    fn from(fail: DeterminismFailure) -> ProtobufResult {
        ProtobufResult::Determinism(fail.into())
    }
}

impl From<ConsistencyFailure> for ProtobufResult {
    fn from(fail: ConsistencyFailure) -> ProtobufResult {
        ProtobufResult::Consistency(fail.into())
    }
}

impl From<SyntaxFailure> for ProtobufResult {
    fn from(fail: SyntaxFailure) -> ProtobufResult {
        ProtobufResult::Syntax(fail.into())
    }
}

impl From<RefinementFailure> for ProtobufResult {
    fn from(fail: RefinementFailure) -> ProtobufResult {
        ProtobufResult::Refinement(fail.into())
    }
}

impl From<PathFailure> for ProtobufResult {
    fn from(fail: PathFailure) -> ProtobufResult {
        ProtobufResult::Reachability(fail.into())
    }
}
