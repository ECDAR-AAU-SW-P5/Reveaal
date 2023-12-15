#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/Conjunction";

    #[test]
    fn t1_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test1 <= Test1"));
    }

    #[test]
    fn t2_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test2 <= Test2"));
    }

    #[test]
    fn t3_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test3 <= Test3"));
    }

    #[test]
    fn t4_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test4 <= Test4"));
    }

    #[test]
    fn t5_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Test5 <= Test5"));
    }

    #[test]
    fn t1_conj_t2_refines_t3() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test2 <= Test3"
        ));
    }

    #[test]
    fn t2_conj_t3_refines_t1() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test2 && Test3 <= Test1"
        ));
    }

    #[test]
    fn t1_conj_t3_refines_t2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test3 <= Test2"
        ));
    }

    #[test]
    fn t1_conj_t2_conj_t4_refines_t5() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test1 && Test2 && Test4 <= Test5"
        ));
    }

    #[test]
    fn t3_conj_t4_refines_t5() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test3 && Test4 <= Test5"
        ));
    }

    #[test]
    fn t6_conj_t7_refines_t8() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test6 && Test7 <= Test8"
        ));
    }

    #[test]
    fn test1_nested_conj_refines_t12() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Test9 && Test10 && Test11 <= Test12"
        ));
    }
}
