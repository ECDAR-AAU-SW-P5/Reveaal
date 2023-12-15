#[cfg(test)]
mod reachability_search_algorithm_test {

    use crate::system::query_failures::QueryResult;
    use crate::tests::refinement::helper::json_run_query;
    use test_case::test_case;

    const PATH: &str = "samples/json/EcdarUniversity";
    const PATH2: &str = "samples/json/AutomatonTestReachability";

    #[test_case(PATH, "reachability: Machine @ Machine.L5 && Machine.y<6 -> Machine.L4 && Machine.y<=6", true; "Existing states and with right clocks")]
    #[test_case(PATH, "reachability: Machine @ Machine.L5 -> Machine.L4 && Machine.y>7", false; "Exisiting locations but not possible with the clocks")]
    #[test_case(PATH, "reachability: Machine @ Machine.L4 && Machine.y<=6 -> Machine.L5 && Machine.y>=4", true; "Switched the two states and with right clocks")]
    #[test_case(PATH, "reachability: Machine @ Machine.L5 && Machine.y<1 -> Machine.L5 && Machine.y<2", true; "Same location, different clocks")]
    #[test_case(PATH, "reachability: Machine @ Machine.L5 -> Machine.L5", true; "Same location, no clocks")]
    #[test_case(PATH, "reachability: Machine @ Machine.L5 -> true", true; "Trivially reachable because the end state is true which means any location")]
    #[test_case(PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4 && Researcher.L9", true; "Composition between Machine & Researcher, with existing locations and not clocks")]
    #[test_case(PATH, "reachability: Machine || Researcher @  Machine.L5 && Researcher.U0 -> Machine.L5 && Researcher.L7", false; "No valid path from the two states")]
    #[test_case(PATH, "reachability: Researcher @ Researcher.U0 -> Researcher.L7", false; "No possible path between to locations, locations exists in Researcher")]
    #[test_case(PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Machine.L4", true; "Machine || Researcher with Partial end state")]
    #[test_case(PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.L6 -> Researcher.L9", true; "Machine || Researcher with Partial end state 2")]
    #[test_case(PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Machine.L5", true; "Machine || Researcher reachable with partial end state")]
    #[test_case(PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Machine.L4", true; "Machine || Researcher reachable with partial end state 2")]
    #[test_case(PATH, "reachability: Machine || Researcher @ Machine.L5 && Researcher.U0 -> Researcher.L7", false; "Machine || Researcher not reachable with partial end state")]
    #[test_case(PATH, "reachability: Researcher[1] && Researcher[2] @ init -> Researcher[1].L7", true; "Machine || Researcher with partial state reachable from intial")]
    #[test_case(PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[1].U0 && Researcher[2].U0", true; "Trivially reachable")]
    #[test_case(PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[1].U0 && Researcher[2].U0 && Researcher[1].x>5", true; "Trivially reachable but with clocks")]
    #[test_case(PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[1].L6 && Researcher[2].U0", false; "Trivially unreachable")]
    #[test_case(PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0 -> Researcher[2].U0", true; "Trivially reachable because _ is U0")]
    fn search_algorithm_returns_result_university(path: &str, query: &str, expected: bool) {
        match json_run_query(path, query).unwrap() {
            QueryResult::Reachability(path) => assert_eq!(path.is_ok(), expected),
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }

    #[test_case(PATH2, "reachability: Component1 @ Component1.L1 -> Component1.L3", false; "False due to invariants")]
    #[test_case(PATH2, "reachability: Component2 @ Component2.L4 -> Component2.L5", false; "False due to invariants, like the other")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L8", false; "False due to guards on the last transition")]
    #[test_case(PATH2, "reachability: Component1 @ Component1.L0 -> Component1.L2", true; "It is possible to travel from L0 to L2 without specifiying guards")]
    #[test_case(PATH2, "reachability: Component4 @ Component4.L9 -> Component4.L10", false; "False due to start state invariant and guard")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L7", true; "It is possible to travel from L6 to L7 without specifiying guards")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L7 -> Component3.L8", true; "It is possible to travel from L7 to L8 without specifiying guards")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L7 && Component3.x<5", false; "It is not possible to travel from L6 to L7 due to specified guards")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L7 && Component3.x>4 -> Component3.L8", false; "It is not possible to travel from L7 to L8 due to specified guards")]
    #[test_case(PATH2, "reachability: Component5 @ Component5.L11 -> Component5.L12", true; "It is possible to travel from L11 to L12 due to update")]
    #[test_case(PATH2, "reachability: Component6 @ Component6.L13 -> Component6.L15", true; "It is possible to travel from L13 to L15 due to the updates at L14")]
    #[test_case(PATH2, "reachability: Component7 @ Component7.L16 -> Component7.L19", true; "Overwrite state of location once to reach end state")]
    #[test_case(PATH2, "reachability: Component8 @ Component8.L20 -> Component8.L22", true; "Reset clock to reach end state")]
    #[test_case(PATH2, "reachability: Component7 @ Component7.L16 -> Component7.L19 && Component7.y<2", false; "Unreachable due to second clock")]
    #[test_case(PATH2, "reachability: Component3[1] && Component3[2] @ Component3[1].L6 && Component3[2].L6 -> Component3[1].L7 && Component3[2].L7", true; "Simple conjunction")]
    fn search_algorithm_returns_result(path: &str, query: &str, expected: bool) {
        match json_run_query(path, query).unwrap() {
            QueryResult::Reachability(path) => assert_eq!(path.is_ok(), expected),
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }

    #[test_case(PATH2, "reachability: Component1 @ Component1.L0 -> Component1.L2", vec!["E3", "E2"]; "Path in Component1 from L0 to L2")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L6 -> Component3.L7", vec!["E5"]; "Path in Component3 from L6 to L7")]
    #[test_case(PATH2, "reachability: Component3 @ Component3.L7 -> Component3.L8", vec!["E6"]; "Path in Component3 from L7 to L8")]
    #[test_case(PATH2, "reachability: Component5 @ Component5.L11 -> Component5.L12", vec!["E8"]; "Path in Component5 from L11 to L12")]
    #[test_case(PATH2, "reachability: Component6 @ Component6.L13 -> Component6.L15", vec!["E12", "E11", "E9", "E10", "E13"]; "Path in Component6 from L13 to L15")]
    #[test_case(PATH2, "reachability: Component7 @ Component7.L16 -> Component7.L19", vec!["E11", "E12", "E10"]; "Path in Component7 from L16 to L19")]
    #[test_case(PATH2, "reachability: Component8 @ Component8.L20 -> Component8.L22", vec!["E13", "E15", "E14"]; "Path in Component8 from L20 to L22")]
    #[test_case(PATH2, "reachability: Component9 @ Component9.L23 && Component9.x>5 -> Component9.L26", vec!["E17", "E18"]; "Path in Component9 from L23 x gt 5 to L26")]
    #[test_case(PATH2, "reachability: Component9 @ Component9.L23 && Component9.x<5 -> Component9.L26", vec!["E16", "E19"]; "Path in Component9 from L23 x lt 5 to L26")]
    fn path_gen_test_correct_path(folder_path: &str, query: &str, expected_path: Vec<&str>) {
        match json_run_query(folder_path, query).unwrap() {
            QueryResult::Reachability(actual_path) => {
                let actual_path = actual_path.unwrap_or_else(|_| {
                    panic!(
                        "Query: {}\nEnd state is not reachable from start state \n",
                        query
                    )
                });
                let path = actual_path.path;
                assert!(expected_path.len() == path.len(), "Query: {}\nThe length of the actual and expected are not the same.\nexpected_path.len = {}\nactual_path.len = {} \n", query, expected_path.len(),path.len());
                for i in 0..path.len() {
                    let edges: Vec<_> = path[i].edges.iter().map(|e| e.edge_id.clone()).collect();
                    assert_eq!(
                        1,
                        edges.len(),
                        "Query: {}\nThere should only be one edge in the path \n",
                        query
                    );
                    assert!(
                        expected_path[i] == edges[0],
                        "Query: {}\nThe actual and expected is not the same \n",
                        query
                    );
                }
            }
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }

    #[test_case(PATH2, "reachability: Component3[1] && Component3[2] @ Component3[1].L6 && Component3[2].L6  -> Component3[1].L7 && Component3[2].L7", vec![vec!["E5","E5"]]; "Path in Component3 && Component3 from L6 && L6 to L7 && L7")]
    #[test_case(PATH2, "reachability: Component3[1] && Component3[2] && Component3[3] @ Component3[1].L6 && Component3[2].L6 && Component3[3].L6  -> Component3[1].L7 && Component3[2].L7 && Component3[3].L7", vec![vec!["E5","E5", "E5"]]; "Path in Component3 && Component3 && Component3 from L6 && L6 && L6 to L7 && L7 && L7")]
    #[test_case(PATH, "reachability: Researcher[1] && Researcher[2] @ Researcher[1].U0 && Researcher[2].U0  -> Researcher[2].U0", vec![]; "Path in Researcher && Researcher from universal state to partial universal state")]
    fn path_gen_test_correct_path_vecvec(
        folder_path: &str,
        query: &str,
        expected_path: Vec<Vec<&str>>,
    ) {
        match json_run_query(folder_path, query).unwrap() {
            QueryResult::Reachability(actual_path) => {
                let actual_path = actual_path.unwrap_or_else(|_| {
                    panic!(
                        "Query: {}\nEnd state is not reachable from start state \n",
                        query
                    )
                });
                let path = actual_path.path;
                assert!(expected_path.len() == path.len(), "Query: {}\nThe length of the actual and expected are not the same.\nexpected_path.len = {}\nactual_path.len = {} \n", query, expected_path.len(),path.len());
                for i in 0..path.len() {
                    let edges: Vec<_> = path[i].edges.iter().map(|e| e.edge_id.clone()).collect();
                    assert_eq!(
                        expected_path[i].len(),
                        edges.len(),
                        "Query: {}\nThere should only be one edge in the path \n",
                        query
                    );
                    assert!(
                        expected_path[i] == edges,
                        "Query: {}\nThe actual and expected is not the same \n",
                        query
                    );
                }
            }
            _ => panic!("Inconsistent query result, expected Reachability"),
        }
    }
}
