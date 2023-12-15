// These tests ensure the parser/grammar can parse strings of the reachability syntax,
// which is the parse_queries::parse_to_expression_tree() function.
#[cfg(test)]
mod reachability_grammar_test {
    use crate::parse_queries;
    use test_case::test_case;

    #[test_case("reachability: Hi @ Hi.L1 && Hi.y<3 -> Hi.L2 && Hi.y<2"; "only 1 machine, start/end location and clock restriction")]
    #[test_case("reachability: Hi[1] && Hi[2] @ Hi[1].L1 && Hi[2].L1 && Hi[1].y<3 -> Hi[1].L2 && Hi[1].y<2"; "2 machine, start/end location and clock restriction")]
    fn query_grammar_test_valid_queries(parser_input: &str) {
        // This tests that the grammar accepts this string, and does not panic:
        assert!(parse_queries::parse_to_expression_tree(parser_input).is_ok());
    }

    #[test_case("reachability: Hi @ L1 && Hi.y<3 -> L2 && Hi.y<2"; "No component prefix on location")]
    #[test_case("reachability: Hi @ Hi.L1 && y<3 -> Hi.L2 && y<2"; "No component prefix on clock")]
    fn query_grammar_test_invalid_queries(parser_input: &str) {
        // This tests that the grammar does NOT accept this string and panics:
        assert!(parse_queries::parse_to_expression_tree(parser_input).is_err());
    }
}
