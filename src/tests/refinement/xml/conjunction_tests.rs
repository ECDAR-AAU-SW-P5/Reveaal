#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::xml_refinement_check;

    const PATH: &str = "samples/xml/conjun.xml";

    #[test]
    fn p0_conj_p1_ref_p2() {
        assert!(!xml_refinement_check(PATH, "refinement: P0 && P1 <= P2"));
    }

    #[test]
    fn p7_conj_p8_conj_p9_ref_p10() {
        assert!(!xml_refinement_check(
            PATH,
            "refinement: P7 && P8 && P9 <= P10"
        ));
    }

    #[test]
    fn p11_conj_p12_ref_p13() {
        assert!(!xml_refinement_check(PATH, "refinement: P11 && P12 <= P13"));
    }
}
