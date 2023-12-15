#[cfg(test)]

mod test {
    use std::collections::HashSet;

    use crate::{
        system::extract_system_rep::ExecutableQueryError,
        system::query_failures::{
            ActionFailure, ConsistencyFailure, DeterminismFailure, SystemRecipeFailure,
        },
        tests::refinement::helper::json_run_query,
    };

    const PATH: &str = "samples/json/SystemRecipe/Quotient";

    #[test]
    fn quotient1_fails_correctly() {
        let actual =
            json_run_query(PATH, "consistency: LeftQuotient1 // RightQuotient1").unwrap_err();
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
                ActionFailure::NotDisjoint(_, _),
                _
            ))
        ));
    }

    #[test]
    fn quotient1_fails_with_correct_actions() {
        let expected_actions = HashSet::from(["Input1".to_string()]);
        if let Some(ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Action(
            ActionFailure::NotDisjoint(left, right),
            _,
        ))) = json_run_query(PATH, "consistency: LeftQuotient1 // RightQuotient1").err()
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
    fn left_quotient_fails_correctly() {
        let actual = json_run_query(
            PATH,
            "consistency: NotDeterministicQuotientComp // DeterministicQuotientComp",
        )
        .unwrap_err();
        println!("{:?}", actual);
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Inconsistent(
                ConsistencyFailure::NotDeterministic(DeterminismFailure { .. }),
                _
            ))
        ));
    }

    #[test]
    fn right_quotient_fails_correctly() {
        let actual = json_run_query(
            PATH,
            "consistency: DeterministicQuotientComp // NotDeterministicQuotientComp",
        )
        .unwrap_err();
        println!("{:?}", actual);
        assert!(matches!(
            actual,
            ExecutableQueryError::SystemRecipeFailure(SystemRecipeFailure::Inconsistent(
                ConsistencyFailure::NotDeterministic(DeterminismFailure { .. }),
                _
            ))
        ));
    }
}
