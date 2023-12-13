#[cfg(test)]
pub mod util {
    use crate::data_reader::component_loader::JsonProjectLoader;
    use crate::data_reader::parse_queries;
    use crate::model_objects::expressions::QueryExpression;
    use crate::system::extract_system_rep;
    use crate::system::extract_system_rep::SystemRecipe;
    use crate::system::query_failures::ConsistencyResult;
    use crate::system::refine;
    use crate::system::save_component::combine_components;
    use crate::system::save_component::PruningStrategy;
    use edbm::util::constraints::ClockIndex;

    pub fn json_reconstructed_component_refines_base_self(input_path: &str, system: &str) {
        let project_loader =
            JsonProjectLoader::new_loader(String::from(input_path), crate::tests::TEST_SETTINGS);

        //This query is not executed but simply used to extract an UncachedSystem so the tests can just give system expressions
        let str_query = format!("get-component: {} save-as test", system);
        let query = parse_queries::parse_to_expression_tree(str_query.as_str())
            .unwrap()
            .remove(0);

        let mut dim: ClockIndex = 0;
        let (base_system, new_system) = if let QueryExpression::GetComponent(expr) = &query {
            let mut comp_loader = project_loader.to_comp_loader();
            (
                extract_system_rep::get_system_recipe(
                    &expr.system,
                    &mut *comp_loader,
                    &mut dim,
                    &mut None,
                )
                .unwrap(),
                extract_system_rep::get_system_recipe(
                    &expr.system,
                    &mut *comp_loader,
                    &mut dim,
                    &mut None,
                )
                .unwrap(),
            )
        } else {
            panic!("Failed to create system")
        };

        let new_comp = new_system.compile(dim);
        //TODO:: Return the SystemRecipeFailure if new_comp is a failure
        if new_comp.is_err() {
            return;
        }
        let new_comp = combine_components(&new_comp.unwrap(), PruningStrategy::NoPruning);

        let new_comp = SystemRecipe::Component(Box::new(new_comp))
            .compile(dim)
            .unwrap();
        //TODO:: if it can fail unwrap should be replaced.
        let base_system = base_system.compile(dim).unwrap();

        let base_precheck = base_system.precheck_sys_rep();
        let new_precheck = new_comp.precheck_sys_rep();
        assert_eq!(helper(&base_precheck), helper(&new_precheck));

        //Only do refinement check if both pass precheck
        if helper(&base_precheck) && helper(&new_precheck) {
            assert!(matches!(
                refine::check_refinement(new_comp.clone(), base_system.clone()),
                Ok(())
            ));
            assert!(matches!(
                refine::check_refinement(base_system.clone(), new_comp.clone()),
                Ok(())
            ));
        }
    }

    fn helper(a: &ConsistencyResult) -> bool {
        a.is_ok()
    }
}
