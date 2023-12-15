use reveaal::cli::Args;
use reveaal::logging::setup_logger;
use reveaal::model_objects::Query;
use reveaal::system::query_failures::QueryResult;

use clap::Parser;
use reveaal::protobuf_server::services::query_request::Settings;
use reveaal::{
    extract_system_rep, parse_queries, start_grpc_server_with_tokio, xml_parser, ComponentLoader,
    JsonProjectLoader, ProjectLoader, XmlProjectLoader,
};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    #[cfg(feature = "logging")]
    setup_logger().unwrap();

    match args {
        Args::Serve {
            endpoint,
            thread_count,
            cache_size,
        } => start_grpc_server_with_tokio(&endpoint, cache_size, thread_count)?,
        Args::Query { .. } => start_using_cli(args),
    }

    Ok(())
}

fn start_using_cli(args: Args) {
    let (mut comp_loader, queries) = parse_args(args);

    let mut results = vec![];
    for query in &queries {
        let executable_query = Box::new(
            extract_system_rep::create_executable_query(query, &mut *comp_loader).unwrap(),
        );

        let result = executable_query.execute();

        if let QueryResult::CustomError(err) = result {
            panic!("{}", err);
        }

        results.push(result);
    }

    println!("\nQuery results:");
    for index in 0..queries.len() {
        results[index].print_result(&queries[index].query.as_ref().unwrap().to_string())
    }
}

fn parse_args(args: Args) -> (Box<dyn ComponentLoader>, Vec<Query>) {
    match args {
        Args::Query {
            query,
            input_folder,
            enable_clock_reduction,
            save_refinement_relations,
            //thread_count,
        } => {
            if save_refinement_relations.is_some() {
                unimplemented!("Saving refinement relations is not yet implemented");
            }

            let settings = Settings {
                disable_clock_reduction: !enable_clock_reduction,
            };

            let project_loader = get_project_loader(input_folder, settings);

            let queries = if query.is_empty() {
                project_loader.get_queries().clone()
            } else {
                parse_queries::parse_to_query(&query)
            };

            (project_loader.to_comp_loader(), queries)
        }
        _ => unreachable!("This function should only be called when the args are a query"),
    }
}

fn get_project_loader<P: AsRef<Path>>(
    project_path: P,
    settings: Settings,
) -> Box<dyn ProjectLoader> {
    if xml_parser::is_xml_project(&project_path) {
        XmlProjectLoader::new_loader(project_path, settings)
    } else {
        JsonProjectLoader::new_loader(project_path, settings)
    }
}

pub fn set_working_directory(folder_path: &str) {
    let mut path = std::path::Path::new(folder_path);
    if path.is_file() {
        path = path
            .parent()
            .expect("Failed to find parent directory of input file");
    };
    env::set_current_dir(path).expect("Failed to set working directory to input folder");
}
