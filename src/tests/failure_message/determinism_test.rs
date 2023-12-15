#[cfg(test)]

mod test {
    use crate::{
        system::query_failures::{
            ConsistencyFailure, QueryResult, RefinementFailure, RefinementPrecondition,
        },
        tests::refinement::helper::json_run_query,
    };

    const PATH: &str = "samples/json/Determinism";

    #[test]
    fn determinism_failure_test() {
        let actual = json_run_query(PATH, "determinism: NonDeterminismCom").unwrap();

        assert!(matches!(actual, QueryResult::Determinism(Err(_))));
    }

    #[test]
    fn determinism_failure_in_refinement_test() {
        let actual = json_run_query(PATH, "refinement: NonDeterminismCom <= Component2").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Refinement(Err(RefinementFailure::Precondition(
                RefinementPrecondition::InconsistentChild(
                    ConsistencyFailure::NotDeterministic(_),
                    _
                )
            )))
        )); // TODO: check the child name
    }
}
