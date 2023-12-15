#[cfg(test)]
mod reachability_parse_partial_state {
    use crate::{
        extract_system_rep::{self, ExecutableQueryError},
        model_objects::expressions::SystemExpression,
        parse_queries, system,
        tests::reachability::helper_functions::reachability_test_helper_functions,
        JsonProjectLoader,
    };
    use test_case::test_case;

    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";

    #[test_case("true", true;
    "State gets parsed as partial")]
    #[test_case("Adm2.L20", false;
    "State gets parsed as not partial")]
    fn query_parser_checks_invalid_locations(location_str: &str, expect_partial: bool) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);

        let (machine, system) =
            reachability_test_helper_functions::create_system_recipe_and_machine(
                mock_model,
                FOLDER_PATH,
            );

        let mock_state = reachability_test_helper_functions::string_to_state_expr(location_str);

        let result = system::extract_state::get_state(&mock_state, &machine, &system);

        if let Ok(end_state) = result {
            assert_eq!(
                end_state.decorated_locations.id.is_partial_location(),
                expect_partial
            );
        } else {
            panic!("Could not get end state");
        }
    }

    #[test_case("reachability: Adm2 @ true -> Adm2.L20";
    "partial start state and one component")]
    #[test_case("reachability: Adm2[1] && Adm2[2] @ Adm2[1].L21 -> Adm2[1].L20 && Adm2[2].L21";
    "partial start state and two components")]
    #[test_case("reachability: Adm2[1] && Adm2[2] && Adm2[3] && Adm2[4] && Adm2[5] @ Adm2[1].L20 -> Adm2[2].L21";
    "partial start state and complex composition")]
    fn query_parser_reject_partial_start(parser_input: &str) {
        let mut comp_loader =
            JsonProjectLoader::new_loader(String::from(FOLDER_PATH), crate::tests::TEST_SETTINGS)
                .to_comp_loader();
        // Make query:
        let q = parse_queries::parse_to_query(parser_input);
        let queries = q.first().unwrap();

        let result = extract_system_rep::create_executable_query(queries, &mut *comp_loader);
        if let Err(e) = result {
            assert_eq!(
                e,
                ExecutableQueryError::Custom(
                    "Start state is a partial state, which it must not be".to_string()
                )
            );
        } else {
            panic!("No error was returned")
        }
    }
}
