#[cfg(test)]

mod test {
    use std::collections::HashSet;

    use crate::extract_system_rep::ExecutableQueryError;
    use crate::{
        system::query_failures::{ActionFailure, SystemRecipeFailure},
        tests::refinement::helper::json_run_query,
    };

    const PATH: &str = "samples/json/SystemRecipe/Composition";

    #[test]
    fn compostion1_fails_correctly() {
        let actual =
            json_run_query(PATH, "consistency: LeftComposition1 || RightComposition1").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn composition1_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(PATH, "consistency: LeftComposition1 || RightComposition1").err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compostion2_fails_correctly() {
        let actual =
            json_run_query(PATH, "consistency: LeftComposition2 || RightComposition2").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn composition2_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output1".to_string(), "Output2".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(PATH, "consistency: LeftComposition2 || RightComposition2").err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn compostion3_fails_correctly() {
        let actual =
            json_run_query(PATH, "consistency: LeftComposition3 || RightComposition3").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn composition3_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Output2".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(PATH, "consistency: LeftComposition3 || RightComposition3").err()
        {
            assert_eq!(
                left.actions
                    .intersection(&right.actions)
                    .cloned()
                    .collect::<HashSet<_>>(),
                expected_actions
            );
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }
}
