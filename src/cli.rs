use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about="Reveaal is a model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems)\nFor more information about ECDAR see https://www.ecdar.net/", long_about = Some("With Reveaal you can either run a single query with the 'query' command or run it as a server with the 'serve' command"))]
pub enum Args {
    /// Start a gRPC server with the protocol defined in the protobuf file
    ///
    /// Examples of usage:
    ///
    /// Reveaal serve 127.0.0.1:4242
    ///
    /// Reveaal serve -t 1 -c 50 127.0.0.1:4242
    Serve {
        /// Ip address and port to serve the gRPC server on
        #[clap(value_name = "IP:PORT")]
        endpoint: String,

        /// The number of threads to use when running queries on the server
        #[arg(short, long, default_value_t = num_cpus::get())]
        thread_count: usize,

        /// The maximal number of component saved in the server cache
        #[arg(short, long, default_value_t = 100)]
        cache_size: usize,
    },
    /// Run a query
    ///
    /// Examples of usage:
    ///
    /// Reveaal query "refinement: Researcher || Machine || Administration <= Spec" -i samples/json/EcdarUniversity
    ///
    /// Reveaal query "consistency: Machine" -i samples/json/EcdarUniversity
    ///
    /// Reveaal query "determinism: Researcher" -i samples/json/EcdarUniversity
    Query {
        /// The query to execute
        #[clap(value_name = "QUERY_TYPE: refinement|consistency|reachability|save-component|syntax", value_parser = query_check)]
        query: String,

        /// File (XML) or folder (JSON) with component definitions
        #[arg(short, long, value_name = "XML|JSON")]
        input_folder: PathBuf,

        /// Whether to enable clock reduction
        #[arg(short, long, default_value_t = false)]
        enable_clock_reduction: bool,

        /// Save file for refinement relations
        #[arg(short, long, value_name = "FILE")]
        save_refinement_relations: Option<PathBuf>,
        // TODO: Maybe add this later
        // /// The number of threads to use when running the query
        // #[arg(short, long, default_value_t = num_cpus::get())]
        // thread_count: usize,
    },
}

fn query_check(arg: &str) -> Result<String, String> {
    crate::parse_queries::parse_to_expression_tree(arg).map(|_| arg.to_string())
}

#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;
    use std::path::PathBuf;
    use std::str::FromStr;
    use test_case::test_case;

    #[test]
    fn serve_command_with_t_and_c_flags() {
        println!("{:?}", PathBuf::from_str(" fucking pis path"));
        // 0th argument does not matter, but it must be present
        let input_args = vec!["", "serve", "-t", "10", "-c", "100", "127.0.0.1:4242"];
        let args_matches = Args::parse_from(input_args);
        check_args(
            args_matches,
            Args::Serve {
                endpoint: "127.0.0.1:4242".to_string(),
                thread_count: 10,
                cache_size: 100,
            },
        );
    }

    #[test_case(
    &["", "query", "-i", "/path/to/system", "-e", "-s", "saved-comp", "refinement: some <= refinement"], Args::Query {
    query: "refinement: some <= refinement".to_string(),
    input_folder: PathBuf::from("/path/to/system"),
    enable_clock_reduction: true,
    save_refinement_relations: Some(PathBuf::from("saved-comp")),
    } ; "All fields"
    )]
    #[test_case(
    &["", "query", "-i", "/path/to/system", "-s", "saved-comp", "refinement: some <= refinement"], Args::Query {
    query: "refinement: some <= refinement".to_string(),
    input_folder: PathBuf::from("/path/to/system"),
    enable_clock_reduction: Default::default(),
    save_refinement_relations: Some(PathBuf::from("saved-comp")),
    } ; "Default clock-reduction"
    )]
    #[test_case(
    &["", "query", "-i", "/path/to/system", "refinement: some <= refinement"], Args::Query {
    query: "refinement: some <= refinement".to_string(),
    input_folder: PathBuf::from("/path/to/system"),
    enable_clock_reduction: Default::default(),
    save_refinement_relations: None,
    } ; "No saved path"
    )]
    fn query_command_tests(input_args: &[&str], expected: Args) {
        check_args(Args::parse_from(input_args), expected);
    }

    #[test_case(&["", "query", "-i", "/path/to/system", "-s", "refinement: some <= refinement"] ; "Not supplying needed argument")]
    #[test_case(&["", "query", "-i", "/path/to/system", "refinement: some  refinement"] ; "Bad query")]
    #[test_case(&["", "serve", "-i", "/path/to/system", "refinement: some <= refinement"] ; "Wrong command")]
    #[should_panic]
    fn query_command_tests_panics(input_args: &[&str]) {
        Args::try_parse_from(input_args).unwrap();
    }

    fn check_args(actual: Args, expected: Args) {
        match (actual, expected) {
            (
                Args::Query {
                    query: qa,
                    input_folder: ia,
                    enable_clock_reduction: da,
                    save_refinement_relations: sa,
                },
                Args::Query {
                    query: qe,
                    input_folder: ie,
                    enable_clock_reduction: de,
                    save_refinement_relations: se,
                },
            ) => {
                assert_eq!(qa, qe);
                assert_eq!(ia, ie);
                assert_eq!(da, de);
                assert_eq!(sa, se);
            }
            (
                Args::Serve {
                    endpoint: ea,
                    thread_count: ta,
                    cache_size: ca,
                },
                Args::Serve {
                    endpoint: ee,
                    thread_count: te,
                    cache_size: ce,
                },
            ) => {
                assert_eq!(ea, ee);
                assert_eq!(ta, te);
                assert_eq!(ca, ce);
            }
            (a, e) => panic!("Not same, expected {:?}, got {:?}", e, a),
        }
    }
}
