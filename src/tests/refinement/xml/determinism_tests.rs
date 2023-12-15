#[cfg(test)]
mod test {
    use crate::{
        system::query_failures::{DeterminismResult, QueryResult},
        tests::refinement::helper::xml_run_query,
    };

    const PATH: &str = "samples/xml/ConsTests.xml";

    fn convert_to_bool(result: DeterminismResult) -> bool {
        matches!(result, Ok(()))
    }

    #[test]
    fn test_g1() {
        let result = xml_run_query(PATH, "determinism: G1");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }
    #[test]
    fn test_g2() {
        let result = xml_run_query(PATH, "determinism: G2");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g3() {
        let result = xml_run_query(PATH, "determinism: G3");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }
    #[test]
    fn test_g4() {
        let result = xml_run_query(PATH, "determinism: G4");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g5() {
        let result = xml_run_query(PATH, "determinism: G5");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }
    #[test]
    fn test_g6() {
        let result = xml_run_query(PATH, "determinism: G6");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g7() {
        let result = xml_run_query(PATH, "determinism: G7");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g8() {
        let result = xml_run_query(PATH, "determinism: G8");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g9() {
        // shouldn't be deterministic
        let result = xml_run_query(PATH, "determinism: G9");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(!convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g10() {
        let result = xml_run_query(PATH, "determinism: G10");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g11() {
        let result = xml_run_query(PATH, "determinism: G11");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g12() {
        let result = xml_run_query(PATH, "determinism: G12");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g13() {
        let result = xml_run_query(PATH, "determinism: G13");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g14() {
        let result = xml_run_query(PATH, "determinism: G14");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(!convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g15() {
        let result = xml_run_query(PATH, "determinism: G15");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g16() {
        let result = xml_run_query(PATH, "determinism: G16");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(!convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g17() {
        let result = xml_run_query(PATH, "determinism: G17");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g22() {
        let result = xml_run_query(PATH, "determinism: G22");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }

    #[test]
    fn test_g23() {
        // shouldn't be deterministic
        let result = xml_run_query(PATH, "determinism: G23");

        if let QueryResult::Determinism(is_deterministic) = result {
            assert!(!convert_to_bool(is_deterministic));
        } else {
            panic!("Not determinism query");
        }
    }
}
