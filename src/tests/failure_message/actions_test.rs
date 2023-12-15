#[cfg(test)]

mod test {

    use crate::system::query_failures::{
        ConsistencyFailure, DeterminismFailure, DeterminismResult, QueryResult, RefinementFailure,
        RefinementPrecondition,
    };
    use crate::system::specifics::SpecificLocation;
    use crate::tests::refinement::helper::json_run_query;
    const PATH: &str = "samples/json/Actions";

    #[test]
    fn determinism_test() {
        let expected_action = String::from("1");
        let expected_location = SpecificLocation::new("NonDeterministic1", "L1", 0); //LocationID::Simple(String::from("L1"));
        if let QueryResult::Determinism(DeterminismResult::Err(DeterminismFailure {
            state: actual_state,
            action: actual_action,
            system: actual_system,
        })) = json_run_query(PATH, "determinism: NonDeterministic1").unwrap()
        {
            let actual_location = actual_state.locations;
            assert_eq!(
                (expected_location, expected_action),
                (actual_location, actual_action.name)
            );
            assert_eq!(actual_system, "NonDeterministic1");
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn not_consistent_from_test() {
        let expected_location = SpecificLocation::new("NonConsistent", "L17", 0);
        if let QueryResult::Consistency(Err(ConsistencyFailure::InconsistentFrom {
            state: actual_state,
            system: actual_system,
        })) = json_run_query(PATH, "consistency: NonConsistent").unwrap()
        {
            let actual_location = actual_state.locations;
            assert_eq!((expected_location), (actual_location));
            assert_eq!(actual_system, "NonConsistent");
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn refinement_determinism_test() {
        let expected_action = String::from("1");
        let expected_location = SpecificLocation::new("NonDeterministic1", "L1", 0);
        if let QueryResult::Refinement(Err(RefinementFailure::Precondition(
            RefinementPrecondition::InconsistentChild(
                ConsistencyFailure::NotDeterministic(DeterminismFailure {
                    state: actual_state,
                    action: actual_action,
                    system: actual_system,
                }),
                _,
            ),
        ))) = json_run_query(PATH, "refinement: NonDeterministic1 <= NonDeterministic2").unwrap()
        {
            let actual_location = actual_state.locations;
            assert_eq!(
                (expected_location, expected_action),
                (actual_location, actual_action.name)
            );
            assert_eq!(actual_system, "NonDeterministic1"); // Assuming left child is checked first
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }

    #[test]
    fn refinement_consistency_test() {
        let expected_location = SpecificLocation::new("NonConsistent", "L17", 0);
        if let QueryResult::Refinement(Err(RefinementFailure::Precondition(
            RefinementPrecondition::InconsistentChild(
                ConsistencyFailure::InconsistentFrom {
                    state: actual_state,
                    system: actual_system,
                },
                _,
            ),
        ))) = json_run_query(PATH, "refinement: NonConsistent <= CorrectComponent").unwrap()
        {
            let actual_location = actual_state.locations;
            assert_eq!((expected_location), (actual_location));
            assert_eq!(actual_system, "NonConsistent");
        } else {
            panic!("Models in samples/action have been changed, REVERT!");
        }
    }
}
