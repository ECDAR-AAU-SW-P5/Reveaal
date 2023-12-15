#[cfg(test)]
mod reachability_edge_test {
    use crate::data_reader::json_reader::read_json_component;
    use test_case::test_case;

    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";

    #[test_case("Machine", vec!["E25".to_string(), "E26".to_string(), "E27".to_string(), "E28".to_string(), "E29".to_string()]; "Edge ID test on Machine from the ECDAR University")]
    fn edge_id_checking(component_name: &str, edge_ids: Vec<String>) {
        let component = read_json_component(FOLDER_PATH, component_name).unwrap();
        for (i, edge) in component.edges.iter().enumerate() {
            assert_eq!(edge.id, edge_ids[i]);
        }

        // Make sure they have the same length as well
        assert_eq!(
            edge_ids.len(),
            component.edges.len(),
            "Expected edges and actual edges do not have the same amount of edges"
        );
    }
}
