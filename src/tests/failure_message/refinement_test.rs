#[cfg(test)]

mod test {
    use crate::{
        system::query_failures::{
            ActionFailure, QueryResult, RefinementFailure, RefinementPrecondition,
        },
        tests::refinement::helper::json_run_query,
    };

    const PATH: &str = "samples/json/RefinementTests";

    #[test]
    fn not_empty_result_test() {
        let actual = json_run_query(PATH, "refinement: A <= B").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::CannotMatch { .. }))
        ));
    }

    #[test]
    fn empty_transition2s_test() {
        let actual = json_run_query(PATH, "refinement: A <= A2").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::CannotMatch { .. }))
        ));
    }

    #[test]
    fn cuts_delay_solutions_test() {
        let actual = json_run_query(PATH, "refinement: A2 <= B2").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::CutsDelaySolutions { .. }))
        ));
    }

    #[test]
    fn initial_state_test() {
        let actual = json_run_query(PATH, "refinement: C <= D").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::Precondition(
                RefinementPrecondition::EmptyInitialState { .. },
            )))
        ));
    }

    #[test]
    fn not_disjoint_and_not_subset_test() {
        let actual = json_run_query(
            PATH,
            "refinement: notDisjointAndNotSubset1 <= notDisjointAndNotSubset2",
        )
        .unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::Precondition(
                RefinementPrecondition::ActionMismatch(ActionFailure::NotDisjoint(_, _), _),
            )))
        ));
    }

    #[test]
    fn not_subset_test() {
        let actual = json_run_query(PATH, "refinement: notSubset1 <= notSubset2")
            .ok()
            .unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::Precondition(
                RefinementPrecondition::ActionMismatch(ActionFailure::NotSubset(_, _), _),
            )))
        ));
    }

    #[test]
    fn not_disjoint_test() {
        let actual = json_run_query(PATH, "refinement: disJoint2 <= disJoint1").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::Precondition(
                RefinementPrecondition::ActionMismatch(ActionFailure::NotDisjoint(_, _), _),
            ))),
        ));
    }
}
