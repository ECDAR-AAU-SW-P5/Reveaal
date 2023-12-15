#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/AG";

    #[test]
    fn a_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: A <= A"));
    }

    #[test]
    fn g_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: G <= G"));
    }

    #[test]
    fn q_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Q <= Q"));
    }

    #[test]
    fn imp_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Imp <= Imp"));
    }

    #[test]
    fn aa_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: AA <= AA"));
    }

    #[test]
    fn a_g_not_refines_a_imp() {
        assert!(!json_refinement_check(PATH, "refinement: A||G <= A||Imp"));
        // should fail because left side has more inputs
    }

    #[test]
    fn g_not_refines_imp() {
        assert!(!json_refinement_check(PATH, "refinement: G <= Imp"));
        // should fail because right side has more outputs
    }

    #[test]
    fn imp_refines_g() {
        assert!(json_refinement_check(PATH, "refinement: Imp <= G"));
    }

    #[test]
    fn g_refines_q() {
        assert!(json_refinement_check(PATH, "refinement: G <= Q"));
    }

    #[test]
    fn q_refines_g() {
        assert!(json_refinement_check(PATH, "refinement: Q <= G"));
    }

    #[test]
    fn q_not_refines_imp() {
        // should fail because right side has more outputs
        assert!(!json_refinement_check(PATH, "refinement: Q <= Imp"));
    }

    #[test]
    fn imp_refines_q() {
        assert!(json_refinement_check(PATH, "refinement: Imp <= Q"));
    }
}
