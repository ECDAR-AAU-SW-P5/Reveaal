#[cfg(test)]

mod test {
    use crate::{
        system::query_failures::{QueryResult, SyntaxFailure, SyntaxResult},
        tests::refinement::helper::json_run_query,
    };

    const PATH: &str = "samples/json/SyntaxTest";

    #[test]
    fn syntax_failure_test() {
        let actual = json_run_query(PATH, "syntax: syntaxFailure").unwrap();
        assert!(matches!(
            actual,
            QueryResult::Syntax(SyntaxResult::Err(SyntaxFailure::Unparsable { .. }))
        ));
    }
}
