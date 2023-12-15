#[cfg(test)]
mod test {
    use crate::tests::refinement::helper::json_refinement_check;

    const PATH: &str = "samples/json/Unspec";

    #[test]
    fn test_arefines_self() {
        assert!(json_refinement_check(PATH, "refinement: A <= A"));
    }

    #[test]
    fn test_aa_refines_self() {
        assert!(json_refinement_check(PATH, "refinement: AA <= AA"));
    }

    #[test]
    fn test_brefines_self() {
        assert!(json_refinement_check(PATH, "refinement: B <= B"));
    }
}
