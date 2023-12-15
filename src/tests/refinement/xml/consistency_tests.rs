#[cfg(test)]
mod test {
    use crate::{system::query_failures::QueryResult, tests::refinement::helper::xml_run_query};

    const PATH: &str = "samples/xml/ConsTests.xml";

    fn convert_to_bool(result: QueryResult) -> bool {
        matches!(result, QueryResult::Consistency(Ok(())))
    }

    #[test]
    fn test_g1() {
        let result = xml_run_query(PATH, "consistency: G1");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }
    #[test]
    fn test_g2() {
        let result = xml_run_query(PATH, "consistency: G2");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g3() {
        let result = xml_run_query(PATH, "consistency: G3");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g4() {
        let result = xml_run_query(PATH, "consistency: G4");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g5() {
        let result = xml_run_query(PATH, "consistency: G5");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g6() {
        let result = xml_run_query(PATH, "consistency: G6");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g7() {
        let result = xml_run_query(PATH, "consistency: G7");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g8() {
        let result = xml_run_query(PATH, "consistency: G8");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g9() {
        let result = xml_run_query(PATH, "consistency: G9");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g10() {
        let result = xml_run_query(PATH, "consistency: G10");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g11() {
        let result = xml_run_query(PATH, "consistency: G11");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g12() {
        let result = xml_run_query(PATH, "consistency: G12");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g13() {
        let result = xml_run_query(PATH, "consistency: G13");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g14() {
        let result = xml_run_query(PATH, "consistency: G14");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g15() {
        let result = xml_run_query(PATH, "consistency: G15");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g16() {
        let result = xml_run_query(PATH, "consistency: G16");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g17() {
        let result = xml_run_query(PATH, "consistency: G17");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g18() {
        let result = xml_run_query(PATH, "consistency: G18");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g19() {
        let result = xml_run_query(PATH, "consistency: G19");

        if let QueryResult::Consistency(_) = &result {
            assert!(!convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g20() {
        let result = xml_run_query(PATH, "consistency: G20");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }

    #[test]
    fn test_g21() {
        let result = xml_run_query(PATH, "consistency: G21");

        if let QueryResult::Consistency(_) = &result {
            assert!(convert_to_bool(result));
        } else {
            panic!("Not consistency check");
        }
    }
}
