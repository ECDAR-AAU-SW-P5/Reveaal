#[cfg(test)]

mod test {
    use crate::{
        system::query_failures::{ConsistencyFailure, ConsistencyResult, QueryResult},
        tests::refinement::helper::json_run_query,
    };

    const PATH: &str = "samples/json/ConsistencyTest";

    #[test]
    fn not_consistent_test() {
        let actual = json_run_query(PATH, "consistency: notConsistent").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Consistency(ConsistencyResult::Err(
                ConsistencyFailure::InconsistentFrom { .. }
            ))
        ));
    }
}
