use crate::data_reader::component_loader::{JsonProjectLoader, XmlProjectLoader};
use crate::data_reader::parse_queries;
use crate::extract_system_rep::ExecutableQueryError;
use crate::logging::setup_logger;
use crate::model_objects::Query;
use crate::system::extract_system_rep::create_executable_query;
use crate::system::query_failures::QueryResult;
use crate::transition_systems::transition_system::component_loader_to_transition_system;
use crate::transition_systems::TransitionSystemPtr;

fn try_setup_logging() {
    #[cfg(feature = "logging")]
    let _ = setup_logger();
}

pub fn xml_refinement_check(path: &str, query: &str) -> bool {
    try_setup_logging();
    match xml_run_query(path, query) {
        QueryResult::Refinement(Ok(())) => true,
        QueryResult::Refinement(Err(_)) => false,
        QueryResult::CustomError(err) => panic!("{}", err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn json_refinement_check(path: &str, query: &str) -> bool {
    try_setup_logging();

    match json_run_query(path, query).unwrap() {
        QueryResult::Refinement(Ok(())) => true,
        QueryResult::Refinement(Err(_)) => false,
        QueryResult::CustomError(err) => panic!("{}", err),
        _ => panic!("Not a refinement check"),
    }
}

pub fn xml_run_query(path: &str, query: &str) -> QueryResult {
    let project_path = String::from(path);
    let mut project_loader =
        XmlProjectLoader::new_loader(project_path, crate::tests::TEST_SETTINGS);
    let query = parse_queries::parse_to_expression_tree(query)
        .unwrap()
        .remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    project_loader.get_settings_mut().disable_clock_reduction = true;

    let mut comp_loader = project_loader.to_comp_loader();
    let query = create_executable_query(&q, &mut *comp_loader).unwrap();

    query.execute()
}

pub fn json_run_query(path: &str, query: &str) -> Result<QueryResult, ExecutableQueryError> {
    let mut project_loader =
        JsonProjectLoader::new_loader(String::from(path), crate::tests::TEST_SETTINGS);
    let query = parse_queries::parse_to_expression_tree(query)
        .unwrap()
        .remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

    project_loader.get_settings_mut().disable_clock_reduction = true;

    let mut comp_loader = project_loader.to_comp_loader();
    let query = create_executable_query(&q, &mut *comp_loader)?;

    Ok(query.execute())
}

pub fn json_get_system(path: &str, comp: &str) -> TransitionSystemPtr {
    let project_loader =
        JsonProjectLoader::new_loader(String::from(path), crate::tests::TEST_SETTINGS);
    let mut loader = project_loader.to_comp_loader();
    component_loader_to_transition_system(&mut *loader, comp)
}
