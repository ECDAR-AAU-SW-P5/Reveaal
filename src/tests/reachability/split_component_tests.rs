#[cfg(test)]
mod reachability_search_algorithm_test {
    use crate::transition_systems::TransitionID;

    use test_case::test_case;
    #[test_case(
        vec![],
        vec![];
    "Empty path")]
    #[test_case(
        vec![TransitionID::Simple("a".to_string())],
        vec![vec![vec![TransitionID::Simple("a".to_string())]]];
    "Simplest path")]
    #[test_case(
        vec![
            TransitionID::Simple("a".to_string()),
            TransitionID::None
        ],
        vec![
            // component 1
            vec![
                // transition 1
                vec![TransitionID::Simple("a".to_string())],
                vec![]
            ]
        ];
    "Has none")]
    #[test_case(
        vec![
            TransitionID::Conjunction(
                Box::new(TransitionID::Simple("a".to_string())),
                Box::new(TransitionID::Simple("b".to_string()))
            )
        ],
        vec![
            // component 1
            vec![
                vec![TransitionID::Simple("a".to_string())]
            ],
            // component 2
            vec![
                vec![TransitionID::Simple("b".to_string())]
            ]
        ];
    "One conjunction")]
    #[test_case(
        vec![
            TransitionID::Conjunction(
                Box::new(TransitionID::Simple("a".to_string())),
                Box::new(TransitionID::Simple("b".to_string()))
            ),
            TransitionID::Conjunction(
                Box::new(TransitionID::Simple("c".to_string())),
                Box::new(TransitionID::Simple("d".to_string()))
            ),
            TransitionID::Conjunction(
                Box::new(TransitionID::Simple("e".to_string())),
                Box::new(TransitionID::Simple("f".to_string()))
            )
        ],
        vec![
            // component 1
            vec![
                vec![TransitionID::Simple("a".to_string())],
                vec![TransitionID::Simple("c".to_string())],
                vec![TransitionID::Simple("e".to_string())]
            ],
            // component 2
            vec![
                vec![TransitionID::Simple("b".to_string())],
                vec![TransitionID::Simple("d".to_string())],
                vec![TransitionID::Simple("f".to_string())]
            ]
        ];
    "Path")]
    fn split_component_test(path: Vec<TransitionID>, expected: Vec<Vec<Vec<TransitionID>>>) {
        assert_eq!(
            TransitionID::split_into_component_lists(&path),
            Ok(expected)
        );
    }

    #[test_case(
        vec![
            TransitionID::Simple("a".to_string()),
            TransitionID::Conjunction(
                Box::new(TransitionID::Simple("b".to_string())),
                Box::new(TransitionID::Simple("c".to_string()))
            )
        ];
    "Different structures")]
    #[test_case(
        vec![
            TransitionID::Conjunction(
                Box::new(TransitionID::Simple("b".to_string())),
                Box::new(TransitionID::Simple("c".to_string()))
            ),
            TransitionID::Simple("a".to_string())
        ];
    "Different structures 2")]
    fn split_component_invalid_input(path: Vec<TransitionID>) {
        assert!(
            TransitionID::split_into_component_lists(&path).is_err(),
            "Expected error"
        )
    }
}
