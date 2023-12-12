use crate::data_reader::component_loader::{JsonProjectLoader, XmlProjectLoader};
use crate::data_reader::parse_queries;
use crate::extract_system_rep::ExecutableQueryError;
use crate::logging::setup_logger;
use crate::model_objects::expressions::QueryExpression;
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
    let project_loader = XmlProjectLoader::new_loader(project_path, crate::tests::TEST_SETTINGS);
    let query = parse_queries::parse_to_expression_tree(query)
        .unwrap()
        .remove(0);
    let q = Query {
        query: Option::from(query),
        comment: "".to_string(),
    };

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
    // After implementing clock reduction on component level, a few tests are failing due to
    // inconsistencies with initial state and global clock. Turn boolean true to ignore inconsistencies
    if let Some(query_type) = q.get_query() {
        match query_type {
            QueryExpression::Reachability { .. } => {
                project_loader.get_settings_mut().disable_clock_reduction = true;
            }
            QueryExpression::Refinement(_, _)
            | QueryExpression::Consistency(_)
            | QueryExpression::Implementation(_)
            | QueryExpression::Determinism(_)
            | QueryExpression::Specification(_)
            | QueryExpression::Syntax(_)
            | QueryExpression::BisimMinim(_)
            | QueryExpression::GetComponent(_)
            | QueryExpression::Prune(_) => {
                project_loader.get_settings_mut().disable_clock_reduction = false;
            }
        }
    }

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
