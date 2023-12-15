#[cfg(test)]
mod tests {
    use crate::protobuf_server::services::SimulationStartRequest;
    use crate::{
        protobuf_server::{self, services::ecdar_backend_server::EcdarBackend},
        tests::simulation::helper::construct_step_requests,
    };
    use test_case::test_case;
    use tonic::Request;

    #[test_case(
        &["Machine"],
        "samples/json/EcdarUniversity",
        "(Machine)"
    )]
    #[test_case(
        &["HalfAdm1", "HalfAdm2"],
        "samples/json/EcdarUniversity",
        "(HalfAdm1 && HalfAdm2)"
    )]
    #[test_case(
        &["Administration", "Machine", "Researcher"],
        "samples/json/EcdarUniversity",
        "(Administration || Machine || Researcher)"
    )]
    #[test_case(
        &["Spec", "Machine"],
        "samples/json/EcdarUniversity",
        "(Spec // Machine)"
    )]
    #[test_case(
        &["Machine", "Spec", "Researcher"],
        "samples/json/EcdarUniversity",
        "(Spec // Machine // Researcher)"
    )]
    #[test_case(
        &["HalfAdm1", "HalfAdm2", "Machine", "Researcher"],
        "samples/json/EcdarUniversity",
        "((HalfAdm1 && HalfAdm2) || Machine || Researcher)"
    )]
    #[tokio::test]
    async fn start_simulation_then_take_simulation_step(
        component_names: &[&str],
        components_path: &str,
        composition: &str,
    ) {
        // Arrange
        let backend = protobuf_server::ConcreteEcdarBackend::default();
        let request = Request::new(SimulationStartRequest::new(
            component_names,
            components_path,
            composition,
        ));

        // Act
        let response = backend.start_simulation(request).await;

        // Arrange
        for request in
            construct_step_requests(component_names, components_path, composition, response)
        {
            let request = Request::new(request);

            // Act
            let response = backend.take_simulation_step(request).await;

            // Assert
            assert!(response.is_ok(), "Response was not ok: {:?}", response);

            for request2 in
                construct_step_requests(component_names, components_path, composition, response)
            {
                let request2 = Request::new(request2);

                // Act
                let response2 = backend.take_simulation_step(request2).await;

                // Assert
                assert!(response2.is_ok(), "Response was not ok: {:?}", response2);
            }
        }
    }
}
