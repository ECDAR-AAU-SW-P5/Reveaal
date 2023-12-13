use crate::model_objects::{Component, Query, SystemDeclarations};
use crate::system::query_failures::{SyntaxFailure, SyntaxResult};
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_system_declarations<P: AsRef<Path>>(project_path: P) -> Option<SystemDeclarations> {
    let sysdecl_path = project_path.as_ref().join("SystemDeclarations.json");

    if !Path::new(&sysdecl_path).exists() {
        return None;
    }

    match read_json::<SystemDeclarations, _>(&sysdecl_path) {
        Ok(sys_decls) => Some(sys_decls),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to component",
            error,
            sysdecl_path.display()
        ),
    }
}

pub fn read_json_component<P: AsRef<Path>>(
    project_path: P,
    component_name: &str,
) -> Result<Component, SyntaxResult> {
    let component_path = project_path
        .as_ref()
        .join("Components")
        .join(format!("{}.json", component_name));

    let component: Result<Component, SyntaxResult> = match read_json(&component_path) {
        Ok(json) => Ok(json),
        Err(error) => Err(SyntaxFailure::unparsable(
            error.to_string(),
            component_path.display().to_string(),
        )),
    };

    component
}

/// Opens a file and reads it.
/// If the file is read successfully,
/// a Result object which contains a DeserializeOwned JSON object is returned.
/// More information: https://doc.rust-lang.org/std/result/
///
/// # Arguments
///
/// * `filename` - A path to the json file
pub fn read_json<T: DeserializeOwned, P: AsRef<Path>>(filename: P) -> serde_json::Result<T> {
    let mut file = File::open(&filename)
        .unwrap_or_else(|_| panic!("Could not find file {}", filename.as_ref().display()));
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    serde_json::from_str(&data)
}

pub fn json_to_component(json_str: &str) -> Result<Component, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Transforms JSON into a Query type
///
/// # Arguments
///
/// * `project_path` - A path to the project
pub fn read_queries<P: AsRef<Path>>(project_path: P) -> Option<Vec<Query>> {
    let queries_path = project_path.as_ref().join("Queries.json");

    if !Path::new(&queries_path).exists() {
        return None;
    }

    match read_json(&queries_path) {
        Ok(json) => Some(json),
        Err(error) => panic!(
            "We got error {}, and could not parse json file {} to query",
            error,
            queries_path.display()
        ),
    }
}
