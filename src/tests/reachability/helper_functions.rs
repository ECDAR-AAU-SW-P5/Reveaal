pub mod reachability_test_helper_functions {
    use edbm::util::constraints::ClockIndex;

    use crate::extract_system_rep::get_system_recipe;
    use crate::extract_system_rep::SystemRecipe;
    use crate::model_objects::expressions::StateExpression;
    use crate::model_objects::expressions::SystemExpression;
    use crate::parse_queries::parse_to_state_expr;
    use crate::transition_systems::TransitionSystem;
    use crate::xml_parser;
    use crate::JsonProjectLoader;
    use crate::XmlProjectLoader;

    /// Helper function which converts a string to an option<box<BoolExpression>> by replacing ',' with "&&" and using the invariant parser.
    pub fn string_to_state_expr(string: &str) -> StateExpression {
        parse_to_state_expr(string).unwrap()
    }

    /// Helper function to create a transition system and a machine (system recipe)
    pub fn create_system_recipe_and_machine(
        model: SystemExpression,
        folder_path: &str,
    ) -> (Box<SystemRecipe>, Box<dyn TransitionSystem>) {
        let mut comp_loader = if xml_parser::is_xml_project(folder_path) {
            XmlProjectLoader::new_loader(folder_path, crate::tests::TEST_SETTINGS)
        } else {
            JsonProjectLoader::new_loader(folder_path, crate::tests::TEST_SETTINGS)
        }
        .to_comp_loader();
        let mut dim: ClockIndex = 0;
        let mut quotient_index = None;
        let machine =
            get_system_recipe(&model, &mut (*comp_loader), &mut dim, &mut quotient_index).unwrap();
        //TODO:: - unwrap might not be the best way to handle this
        let system = machine.clone().compile(dim).unwrap();
        (machine, system)
    }
}
