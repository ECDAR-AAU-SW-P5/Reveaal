#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::xml_refinement_check;

    const PATH: &str = "samples/xml/misc_test.xml";

    #[test]
    fn guard_paran_refines_self() {
        assert!(xml_refinement_check(
            PATH,
            "refinement: GuardParan <= GuardParan"
        ));
    }
}
