#[cfg(test)]
mod reachability_search_algorithm_test {
    use crate::transition_systems::TransitionID;

    use test_case::test_case;

    #[test_case(TransitionID::Conjunction(
        Box::new(TransitionID::Simple("a".to_string())),
        Box::new(TransitionID::Simple("b".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string()))];
    "Simple conjunction")]
    #[test_case(TransitionID::Composition(
        Box::new(TransitionID::Simple("a".to_string())),
        Box::new(TransitionID::Simple("b".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string()))];
    "Simple composition")]
    #[test_case(TransitionID::Conjunction(
        Box::new(TransitionID::Conjunction(
            Box::new(TransitionID::Simple("a".to_string())),
            Box::new(TransitionID::Simple("b".to_string()))
        )),
        Box::new(TransitionID::Simple("c".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string())), vec!(TransitionID::Simple("c".to_string()))];
    "Simple nesting")]
    #[test_case(TransitionID::Composition(
        Box::new(TransitionID::Conjunction(
            Box::new(TransitionID::Simple("a".to_string())),
            Box::new(TransitionID::Composition(
                Box::new(TransitionID::Simple("b".to_string())),
                Box::new(TransitionID::Simple("c".to_string()))
            ))
        )),
        Box::new(TransitionID::Composition(
            Box::new(TransitionID::Simple("d".to_string())),
            Box::new(TransitionID::Simple("e".to_string()))
        ))
    ),
    vec![
        vec!(TransitionID::Simple("a".to_string())), 
        vec!(TransitionID::Simple("b".to_string())), 
        vec!(TransitionID::Simple("c".to_string())), 
        vec!(TransitionID::Simple("d".to_string())), 
        vec!(TransitionID::Simple("e".to_string()))];
    "Multiple conjunction and composition")]
    #[test_case(TransitionID::Quotient(
        vec!(TransitionID::Simple("a".to_string())),
        vec!(TransitionID::Simple("b".to_string()))
    ),
    vec![vec!(TransitionID::Simple("a".to_string())), vec!(TransitionID::Simple("b".to_string()))];
    "simple quotient")]
    #[test_case(TransitionID::Quotient(
        vec!(TransitionID::Simple("a".to_string()), TransitionID::Simple("b".to_string())),
        vec!(TransitionID::Simple("c".to_string()), TransitionID::Simple("d".to_string()), TransitionID::Simple("e".to_string()))
    ),
    vec![
        vec!(TransitionID::Simple("a".to_string()), TransitionID::Simple("b".to_string())), 
        vec!(TransitionID::Simple("c".to_string()), TransitionID::Simple("d".to_string()), TransitionID::Simple("e".to_string()))];
    "quotient with vec")]
    #[test_case(
        TransitionID::Conjunction(
            Box::new(
                TransitionID::Quotient(
                    vec![
                        TransitionID::Conjunction(
                            Box::new(TransitionID::Simple("a".to_string())),
                            Box::new(TransitionID::Simple("b".to_string())), 
                        ),
                        TransitionID::Conjunction(
                            Box::new(TransitionID::Simple("c".to_string())),
                            Box::new(TransitionID::Simple("d".to_string())), 
                        )
                    ],
                    vec![TransitionID::Simple("e".to_string()), TransitionID::Simple("f".to_string())]
                )
            ),
            Box::new(TransitionID::Simple("g".to_string()))
        ),
        vec![
            vec!(TransitionID::Simple("a".to_string()), TransitionID::Simple("c".to_string())), 
            vec!(TransitionID::Simple("b".to_string()), TransitionID::Simple("d".to_string())),
            vec!(TransitionID::Simple("e".to_string()), TransitionID::Simple("f".to_string())),
            vec!(TransitionID::Simple("g".to_string()))];
        "Complex quotient")]
    fn get_leaves_returns_correct_vector(id: TransitionID, expected: Vec<Vec<TransitionID>>) {
        assert_eq!(id.get_leaves(), expected);
    }
}
