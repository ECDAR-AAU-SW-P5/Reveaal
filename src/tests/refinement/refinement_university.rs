#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/EcdarUniversity";

    #[test]
    fn test_adm_2_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Adm2 <= Adm2"));
    }

    #[test]
    fn test_half_1_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement: HalfAdm1 <= HalfAdm1"
        ));
    }

    #[test]
    fn test_half_2_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement: HalfAdm2 <= HalfAdm2"
        ));
    }

    #[test]
    fn test_adm_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration <= Administration"
        ));
    }

    #[test]
    fn test_machine_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine <= Machine"
        ));
    }

    #[test]
    fn test_res_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Researcher <= Researcher"
        ));
    }

    #[test]
    fn test_spec_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: Spec <= Spec"));
    }

    #[test]
    fn test_machine_3_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine3 <= Machine3"
        ));
    }

    #[test]
    fn test_adm_not_refines_machine() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Machine"
        ));
    }

    #[test]
    fn test_adm_not_refines_researcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Researcher"
        ));
    }

    #[test]
    fn test_adm_not_refines_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Spec"
        ));
    }

    #[test]
    fn test_adm_not_refines_machine_3() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Administration <= Machine3"
        ));
    }

    #[test]
    fn test_machine_not_refines_adm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Administration"
        ));
    }

    #[test]
    fn test_machine_not_refines_researcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Researcher"
        ));
    }

    #[test]
    fn test_machine_not_refines_spec() {
        assert!(!json_refinement_check(PATH, "refinement: Machine <= Spec"));
    }

    #[test]
    fn test_machine_not_refines_machine_3() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Machine3"
        ));
    }

    #[test]
    fn test_res_not_refines_adm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Administration"
        ));
    }

    #[test]
    fn test_res_not_refines_machine() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Machine"
        ));
    }

    #[test]
    fn test_res_not_refines_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Spec"
        ));
    }

    #[test]
    fn test_res_not_refines_machine_3() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Machine3"
        ));
    }

    #[test]
    fn test_spec_not_refines_adm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Spec <= Administration"
        ));
    }

    #[test]
    fn test_spec_not_refines_machine() {
        assert!(!json_refinement_check(PATH, "refinement: Spec <= Machine"));
    }

    #[test]
    fn test_spec_not_refines_researcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Spec <= Researcher"
        ));
    }

    #[test]
    fn test_spec_not_refines_machine_3() {
        assert!(!json_refinement_check(PATH, "refinement: Spec <= Machine3"));
    }

    #[test]
    fn test_machine_3_refines_machine() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine3 <= Machine"
        ));
    }

    #[test]
    fn test_machine_3_not_refines_adm() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine3 <= Administration"
        ));
    }

    #[test]
    fn test_machine_3_not_refines_researcher() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine3 <= Researcher"
        ));
    }

    #[test]
    fn test_machine_3_not_refines_spec() {
        assert!(!json_refinement_check(PATH, "refinement: Machine3 <= Spec"));
    }

    #[test]
    fn test_comp_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration || Researcher || Machine <= Spec"
        ));
    }

    #[test]
    fn test_half_comp_not_refines_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: (HalfAdm1 && HalfAdm2) || Researcher || Machine <= Spec"
        ));
    }

    #[test]
    fn test_adm_2_not_refines_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Adm2 <= Spec // Researcher // Machine"
        ));
    }

    #[test]
    fn test_researcher_not_refines_adm_2_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher <= Spec // Adm2 // Machine"
        ));
    }

    #[test]
    fn test_machine_not_refines_adm_2_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine <= Spec // Adm2 // Researcher"
        ));
    }

    #[test]
    fn test_adm_2_researcher_not_refines_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Adm2 || Researcher <= Spec // Machine"
        ));
    }

    #[test]
    fn test_researcher_machine_not_refines_adm_2_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Researcher || Machine <= Spec // Adm2"
        ));
    }

    #[test]
    fn test_machine_adm_2_not_refines_spec() {
        assert!(!json_refinement_check(
            PATH,
            "refinement: Machine || Adm2 <= Spec // Researcher"
        ));
    }

    #[test]
    fn test_admin_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration <= Spec // Researcher // Machine"
        ));
    }

    #[test]
    fn test_researcher_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Researcher <= Spec // Administration // Machine"
        ));
    }

    #[test]
    fn test_machine_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine <= Spec // Administration // Researcher"
        ));
    }

    #[test]
    fn test_admin_researcher_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Administration || Researcher <= Spec // Machine"
        ));
    }

    #[test]
    fn test_researcher_machine_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Researcher || Machine <= Spec // Administration"
        ));
    }

    #[test]
    fn test_machine_admin_refines_spec() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Machine || Administration <= Spec // Researcher"
        ));
    }

    #[test]
    fn test_comp_refines_self() {
        assert!(json_refinement_check(
            PATH,
            "refinement:  Administration || Researcher || Machine <=  Administration || Researcher || Machine"
        ));
    }

    #[test]
    fn test_half_1_and_half_2_refines_adm2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: HalfAdm1 && HalfAdm2 <= Adm2"
        ));
    }

    #[test]
    fn test_adm_2_refines_half_1_and_half2() {
        assert!(json_refinement_check(
            PATH,
            "refinement: Adm2 <= HalfAdm1 && HalfAdm2"
        ));
    }
}
