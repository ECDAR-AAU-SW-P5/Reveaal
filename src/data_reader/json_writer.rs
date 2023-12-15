use crate::model_objects::Component;
use std::{fs::File, path::Path};

pub fn component_to_json_file<P: AsRef<Path>>(project_path: P, component: &Component) {
    let path = project_path
        .as_ref()
        .join("Components")
        .join(format!("{}.json", component.name));

    let file = File::create(path).expect("Couldnt open file");

    serde_json::to_writer_pretty(&file, component).expect("Failed to serialize component");
}

pub fn component_to_json(component: &Component) -> String {
    serde_json::to_string(component).unwrap()
}
