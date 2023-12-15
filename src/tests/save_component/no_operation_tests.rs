#[cfg(test)]
mod test {
    use crate::tests::save_component::save_comp_helper::util::json_reconstructed_component_refines_base_self;

    const PATH: &str = "samples/json/Conjunction";
    const ECDAR_UNI: &str = "samples/json/EcdarUniversity";

    #[test]
    fn t1_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test1");
    }
    #[test]
    fn t2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test2");
    }
    #[test]
    fn t3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test3");
    }
    #[test]
    fn t4_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test4");
    }
    #[test]
    fn t5_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test5");
    }
    #[test]
    fn t6_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test6");
    }
    #[test]
    fn t7_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test7");
    }
    #[test]
    fn t8_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test8");
    }
    #[test]
    fn t9_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test9");
    }
    #[test]
    fn t10_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test10");
    }
    #[test]
    fn t11_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test11");
    }
    #[test]
    fn t12_saved_refines_self() {
        json_reconstructed_component_refines_base_self(PATH, "Test12");
    }

    #[test]
    fn adm_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Adm2");
    }

    #[test]
    fn administration_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Administration");
    }

    #[test]
    fn half_adm1saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm1");
    }

    #[test]
    fn half_adm_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "HalfAdm2");
    }

    #[test]
    fn machine_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine");
    }

    #[test]
    fn machine_2_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine2");
    }

    #[test]
    fn machine_3_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Machine3");
    }

    #[test]
    fn researcher_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Researcher");
    }

    #[test]
    fn spec_saved_refines_self() {
        json_reconstructed_component_refines_base_self(ECDAR_UNI, "Spec");
    }
}
