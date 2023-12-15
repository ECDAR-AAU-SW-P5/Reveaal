#[cfg(test)]
mod refinements {
    use crate::protobuf_server::services::component::Rep;
    use crate::protobuf_server::services::ecdar_backend_server::EcdarBackend;
    use crate::protobuf_server::services::query_response;
    use crate::protobuf_server::services::Component;
    use crate::protobuf_server::services::ComponentsInfo;
    use crate::protobuf_server::services::QueryRequest;
    use crate::protobuf_server::ConcreteEcdarBackend;
    use tonic::Request;

    //const CONJUN: &str = "samples/xml/conjun.xml";
    const ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[tokio::test]
    async fn send_self_refinement_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("refinement: Machine <= Machine");

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_ok());

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_consistency_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("consistency: Machine");

        let query_response = backend.send_query(query_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_syntax_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("syntax: Machine");

        let query_response = backend.send_query(query_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_determinism_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("determinism: Machine");

        let query_response = backend.send_query(query_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_query_using_cache() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request_for_cache("refinement: Machine <= Machine");

        // Normal query request, including component.
        let query_response = backend.send_query(query_request.0).await;

        let query_result = query_response.unwrap().into_inner();
        let normal_result = query_result.result.unwrap();

        // Component should be cached now.
        // Query without component utilizing cache
        let query_response = backend.send_query(query_request.1).await;

        let query_result = query_response.unwrap().into_inner();
        let cache_result = query_result.result.unwrap();

        // Compare normal and cache response.
        assert_eq!(normal_result, cache_result);
    }

    #[tokio::test]
    async fn send_query_not_in_cache() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request_for_cache("refinement: Machine <= Machine");

        // Cache request
        let query_response = backend.send_query(query_request.1).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();

        match result {
            query_response::Result::ComponentsNotInCache(_) => {}
            _ => panic!("Expected failure, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn send_query_different_users_cache() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request_for_cache("refinement: Machine <= Machine");

        let _ = backend.send_query(query_request.0).await;

        let user_1_request = Request::new(QueryRequest {
            user_id: 1,
            ..query_request.1.into_inner()
        });

        let query_response = backend.send_query(user_1_request).await;

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();

        match result {
            query_response::Result::ComponentsNotInCache(_) => {}
            _ => panic!("Expected failure, got {:?}", result),
        }
    }

    fn construct_query_request_for_cache(
        query: &str,
    ) -> (Request<QueryRequest>, Request<QueryRequest>) {
        let json =
            std::fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();

        let normal_request = Request::new(QueryRequest {
            user_id: 0,
            query_id: 0,
            query: String::from(query),
            components_info: Some(ComponentsInfo {
                components: vec![Component {
                    rep: Some(Rep::Json(json)),
                }],
                components_hash: 1,
            }),
            settings: Some(crate::tests::TEST_SETTINGS),
        });

        //TODO There is some fancy rust syntax to make a clone of the above with minor alterations.
        let empty_component_request = Request::new(QueryRequest {
            user_id: 0,
            query_id: 0,
            query: String::from(query),
            components_info: Some(ComponentsInfo {
                components: vec![],
                components_hash: 1,
            }),
            settings: Some(crate::tests::TEST_SETTINGS),
        });
        (normal_request, empty_component_request)
    }

    fn construct_query_request(query: &str) -> Request<QueryRequest> {
        let json =
            std::fs::read_to_string(format!("{}/Components/Machine.json", ECDAR_UNI)).unwrap();

        Request::new(QueryRequest {
            user_id: 0,
            query_id: 0,
            query: String::from(query),
            components_info: Some(ComponentsInfo {
                components: vec![Component {
                    rep: Some(Rep::Json(json)),
                }],
                components_hash: 0,
            }),
            settings: Some(crate::tests::TEST_SETTINGS),
        })
    }

    /// Ensure that the backend does not crash when a query panics
    #[tokio::test]
    async fn send_panic_query() {
        let backend = ConcreteEcdarBackend::default();
        let query_request = construct_query_request("refinement: Machine | <= Machine");

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_err());
    }

    /// Ensure that the backend can recover from panics entirely
    #[tokio::test]
    async fn send_after_panic_query() {
        let backend = ConcreteEcdarBackend::default();
        for _ in 0..5 {
            let query_request = construct_query_request("refinement: Machine | <= Machine");
            let query_response = backend.send_query(query_request).await;
            assert!(query_response.is_err());
        }
        let query_request = construct_query_request("refinement: Machine <= Machine");

        let query_response = backend.send_query(query_request).await;
        assert!(query_response.is_ok());

        let query_result = query_response.unwrap().into_inner();
        let result = query_result.result.unwrap();
        match result {
            query_response::Result::Success(_) => {}
            _ => panic!("Expected success, got {:?}", result),
        }
    }
}
