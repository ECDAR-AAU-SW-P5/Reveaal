#[cfg(test)]
mod test {
    use crate::tests::save_component::save_comp_helper::util::json_reconstructed_component_refines_base_self;

    //const PATH: &str = "samples/json/Conjunction";
    const ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[test]
    fn adm_2_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine");
    }

    #[test]
    fn adm_2_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine2");
    }

    #[test]
    fn adm_2_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Machine3");
    }

    #[test]
    fn adm_2_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2 || Researcher");
    }

    #[test]
    fn administration_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine");
    }

    #[test]
    fn administration_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine2");
    }

    #[test]
    fn administration_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Machine3");
    }

    #[test]
    fn administration_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration || Researcher");
    }

    #[test]
    fn half_adm_1_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine");
    }

    #[test]
    fn half_adm_1_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine2");
    }

    #[test]
    fn half_adm_1_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Machine3");
    }

    #[test]
    fn half_adm_1_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1 || Researcher");
    }

    #[test]
    fn half_adm_2_machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine");
    }

    #[test]
    fn half_adm_2_machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine2");
    }

    #[test]
    fn half_adm_2_machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Machine3");
    }

    #[test]
    fn half_adm_2_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2 || Researcher");
    }

    #[test]
    fn machine_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine || Researcher");
    }

    #[test]
    fn machine_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine || Spec");
    }

    #[test]
    fn machine_2_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 || Researcher");
    }

    #[test]
    fn machine_2_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2 || Spec");
    }

    #[test]
    fn machine_3_researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 || Researcher");
    }

    #[test]
    fn machine_3_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3 || Spec");
    }

    #[test]
    fn researcher_spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher || Spec");
    }
}
