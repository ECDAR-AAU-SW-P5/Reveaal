use crate::data_reader::parse_edge;
use crate::data_reader::parse_edge::Update;
use crate::model_objects::{
    Component, Declarations, Edge, Location, LocationType, Query, SyncType, SystemDeclarations,
    SystemSpecification,
};
use edbm::util::constraints::ClockIndex;
use elementtree::{Element, FindChildren};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

pub fn is_xml_project<P: AsRef<Path>>(project_path: P) -> bool {
    project_path
        .as_ref()
        .extension()
        .is_some_and(|ext| ext == "xml")
}

///Used to parse systems described in xml
pub(crate) fn parse_xml_from_file<P: AsRef<Path>>(
    file_name: P,
) -> (Vec<Component>, SystemDeclarations, Vec<Query>) {
    //Open file and read xml
    let file = File::open(file_name).unwrap();
    let reader = BufReader::new(file);

    parse_xml(reader)
}

pub(crate) fn parse_xml_from_str(xml: &str) -> (Vec<Component>, SystemDeclarations, Vec<Query>) {
    let reader = BufReader::new(xml.as_bytes());

    parse_xml(reader)
}

fn parse_xml<R: Read>(xml_data: R) -> (Vec<Component>, SystemDeclarations, Vec<Query>) {
    let root = Element::from_reader(xml_data).unwrap();

    //storage of components
    let mut xml_components: Vec<Component> = vec![];

    for xml_comp in root.find_all("template") {
        let declarations = match xml_comp.find("declaration") {
            Some(e) => parse_declarations(e.text()),
            None => parse_declarations(""),
        };
        let edges = collect_edges(xml_comp.find_all("transition"));
        let comp = Component {
            name: xml_comp.find("name").unwrap().text().parse().unwrap(),
            declarations,
            locations: collect_locations(
                xml_comp.find_all("location"),
                xml_comp
                    .find("init")
                    .expect("No initial location")
                    .get_attr("ref")
                    .unwrap(),
            ),
            edges,
            special_id: None,
        };
        xml_components.push(comp);
    }

    let system_declarations = SystemDeclarations {
        //name: "".to_string(),
        declarations: decode_sync_type(root.find("system").unwrap().text()),
    };

    (xml_components, system_declarations, vec![])
}

fn collect_locations(xml_locations: FindChildren, initial_id: &str) -> Vec<Location> {
    let mut locations: Vec<Location> = vec![];
    for loc in xml_locations {
        let location = Location {
            id: loc.get_attr("id").unwrap().parse().unwrap(),
            invariant: match loc.find("label") {
                Some(x) => match parse_edge::parse_guard(x.text()) {
                    Ok(edge_attribute) => Some(edge_attribute),
                    Err(e) => panic!("Could not parse invariant {} got error: {:?}", x.text(), e),
                },
                _ => None,
            },
            location_type: match loc.get_attr("id").unwrap().eq(initial_id) {
                true => LocationType::Initial,
                false => LocationType::Normal,
            },
            urgency: "".to_string(),
        };
        locations.push(location);
    }

    locations
}

fn collect_edges(xml_edges: FindChildren) -> Vec<Edge> {
    let mut edges: Vec<Edge> = vec![];
    for e in xml_edges {
        let mut guard: Option<crate::model_objects::expressions::BoolExpression> = None;
        let mut updates: Option<Vec<Update>> = None;
        let mut sync: String = "".to_string();
        for label in e.find_all("label") {
            match label.get_attr("kind").unwrap() {
                "guard" => match parse_edge::parse_guard(label.text()) {
                    Ok(guard_res) => {
                        guard = Some(guard_res);
                    }
                    Err(e) => panic!("Could not parse {} got error: {:?}", label.text(), e),
                },
                "synchronisation" => {
                    sync = label.text().to_string();
                }
                "assignment" => match parse_edge::parse_updates(label.text()) {
                    Ok(updates_res) => updates = Some(updates_res),
                    Err(e) => panic!("Could not parse {} got error: {:?}", label.text(), e),
                },
                _ => {}
            }
        }
        let edge = Edge {
            id: "NotImplemented".to_string(), // We do not support edge IDs for XML right now.
            source_location: e
                .find("source")
                .expect("source edge not found")
                .get_attr("ref")
                .expect("no source edge ID")
                .to_string(),
            target_location: e
                .find("target")
                .expect("target edge not found")
                .get_attr("ref")
                .expect("no target edge ID")
                .to_string(),
            sync_type: match sync.contains('?') {
                true => SyncType::Input,
                false => SyncType::Output,
            },
            guard,
            update: updates,
            sync: sync.replace(['!', '?'], ""),
        };
        edges.push(edge);
    }

    edges
}

fn parse_declarations(variables: &str) -> Declarations {
    //Split string into vector of strings
    let decls: Vec<String> = variables.split('\n').map(|s| s.into()).collect();
    let mut ints: HashMap<String, i32> = HashMap::new();
    let mut clocks: HashMap<String, ClockIndex> = HashMap::new();
    let mut counter: ClockIndex = 1;
    for string in decls {
        //skip comments
        if string.starts_with("//") || string.is_empty() {
            continue;
        }
        let sub_decls: Vec<String> = string.split(';').map(|s| s.into()).collect();

        for mut sub_decl in sub_decls {
            sub_decl = sub_decl.replace('\r', "");

            if !sub_decl.is_empty() {
                let split_string: Vec<String> = sub_decl.split(' ').map(|s| s.into()).collect();
                let variable_type = split_string[0].as_str();

                if variable_type == "clock" {
                    for split_str in split_string.iter().skip(1) {
                        let comma_split: Vec<String> =
                            split_str.split(',').map(|s| s.into()).collect();
                        for var in comma_split {
                            if !var.is_empty() {
                                clocks.insert(var, counter);
                                counter += 1;
                            }
                        }
                    }
                } else if variable_type == "int" {
                    for split_str in split_string.iter().skip(1) {
                        let comma_split: Vec<String> =
                            split_str.split(',').map(|s| s.into()).collect();
                        for var in comma_split {
                            ints.insert(var, 0);
                        }
                    }
                } else {
                    panic!("not implemented read for type: {}", variable_type);
                }
            }
        }
    }

    Declarations { ints, clocks }
}

fn decode_sync_type(global_decl: &str) -> SystemSpecification {
    let mut first_run = true;
    let decls: Vec<String> = global_decl.split('\n').map(|s| s.into()).collect();
    let mut input_actions: HashMap<String, Vec<String>> = HashMap::new();
    let mut output_actions: HashMap<String, Vec<String>> = HashMap::new();
    let mut components: Vec<String> = vec![];

    let mut component_names: Vec<String> = vec![];

    for declaration in &decls {
        //skip comments
        if declaration.starts_with("//") || declaration.is_empty() {
            continue;
        }

        if !declaration.trim().is_empty() {
            if first_run {
                let component_decls = declaration;

                component_names = component_decls
                    .split(' ')
                    .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect())
                    .collect();

                if component_names[0] == "system" {
                    //do not include element 0 as that is the system keyword
                    for name in component_names.iter_mut().skip(1) {
                        let s = name.replace(',', "");
                        let s_cleaned = s.replace(';', "");
                        *name = s_cleaned.clone();
                        components.push(s_cleaned);
                    }
                    first_run = false;
                } else {
                    panic!("Unexpected format of system declarations. Missing system in beginning of {:?}", component_names)
                }
            }

            let split_string: Vec<String> = declaration.split(' ').map(|s| s.into()).collect();
            if split_string[0].as_str() == "IO" {
                let component_name = split_string[1].clone();

                if component_names.contains(&component_name) {
                    for split_str in split_string.iter().skip(2) {
                        let mut s = split_str.replace('{', "");
                        s = s.replace('\r', "");
                        s = s.replace('\n', "");
                        let p = s.replace('}', "");
                        let comp_actions: Vec<String> = p.split(',').map(|s| s.into()).collect();
                        for action in comp_actions {
                            if action.is_empty() {
                                continue;
                            }
                            if action.ends_with('?') {
                                let r = action.replace('?', "");
                                if let Some(channel_vec) = input_actions.get_mut(&component_name) {
                                    channel_vec.push(r)
                                } else {
                                    let channel_vec = vec![r];
                                    input_actions.insert(component_name.clone(), channel_vec);
                                }
                            } else if action.ends_with('!') {
                                let r = action.replace('!', "");
                                if let Some(channel_vec) = output_actions.get_mut(&component_name) {
                                    channel_vec.push(r.clone())
                                } else {
                                    let channel_vec = vec![r.clone()];
                                    output_actions.insert(component_name.clone(), channel_vec);
                                }
                            } else {
                                panic!("Channel type not defined for Channel {:?}", action)
                            }
                        }
                    }
                } else {
                    panic!("Was not able to find component name: {:?} in declared component names: {:?}", component_name, component_names)
                }
            }
        }
    }
    SystemSpecification {
        components,
        input_actions,
        output_actions,
    }
}
