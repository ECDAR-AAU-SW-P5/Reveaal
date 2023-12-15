use crate::data_reader::parse_edge;
use crate::model_objects::expressions;
use crate::model_objects::{Component, Declarations, Edge, Location, LocationType, SyncType};
use crate::simulation::graph_layout::layout_dummy_component;
use edbm::util::constraints::ClockIndex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::ops::Add;

#[derive(Serialize)]
pub struct DummyNail {
    pub x: f32,
    pub y: f32,
    pub property_type: String,
    pub property_x: f32,
    pub property_y: f32,
}

impl DummyNail {
    pub fn new(p_type: &str) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            property_type: p_type.to_string(),
            property_x: 10.0,
            property_y: -10.0,
        }
    }
}

#[derive(Serialize)]
pub struct DummyEdge {
    pub id: String,
    #[serde(rename = "sourceLocation")]
    pub source_location: String,
    #[serde(rename = "targetLocation")]
    pub target_location: String,
    #[serde(
        deserialize_with = "decode_sync_type",
        serialize_with = "encode_sync_type",
        rename = "status"
    )]
    pub sync_type: SyncType,
    #[serde(
        deserialize_with = "decode_guard",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub guard: Option<expressions::BoolExpression>,
    #[serde(
        deserialize_with = "decode_update",
        serialize_with = "encode_opt_updates"
    )]
    pub update: Option<Vec<parse_edge::Update>>,
    #[serde(deserialize_with = "decode_sync")]
    pub sync: String,
    pub select: String,
    pub nails: Vec<DummyNail>,
}

impl From<Edge> for DummyEdge {
    fn from(item: Edge) -> Self {
        let mut nails = vec![];
        if item.guard.is_some() {
            nails.push(DummyNail::new("GUARD"));
        }

        if item.update.is_some() {
            nails.push(DummyNail::new("UPDATE"));
        }

        nails.push(DummyNail::new("SYNCHRONIZATION"));

        if nails.len() < 2 && item.source_location == item.target_location {
            nails.push(DummyNail::new("NONE"));
        }

        DummyEdge {
            id: item.id,
            source_location: item.source_location,
            target_location: item.target_location,
            sync_type: item.sync_type,
            guard: item.guard,
            update: item.update,
            sync: item.sync,
            select: "".to_string(),
            nails,
        }
    }
}

#[derive(Serialize)]
pub struct DummyComponent {
    pub name: String,

    #[serde(
        deserialize_with = "decode_declarations",
        serialize_with = "encode_declarations"
    )]
    pub declarations: Declarations,
    pub locations: Vec<DummyLocation>,
    pub edges: Vec<DummyEdge>,

    pub description: String,
    pub include_in_periodic_check: bool,
    pub color: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<Component> for DummyComponent {
    fn from(item: Component) -> Self {
        let mut comp = DummyComponent {
            name: item.name,
            declarations: item.declarations,
            locations: item.locations.into_iter().map(|l| l.into()).collect(),
            edges: item.edges.into_iter().map(|l| l.into()).collect(),
            description: "".to_string(),
            include_in_periodic_check: false,
            color: 6.to_string(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };

        layout_dummy_component(&mut comp);

        comp
    }
}

#[derive(Serialize)]
pub struct DummyLocation {
    pub id: String,
    #[serde(
        //deserialize_with = "decode_invariant",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub invariant: Option<expressions::BoolExpression>,
    #[serde(
        //deserialize_with = "decode_location_type",
        serialize_with = "encode_location_type",
        rename = "type"
    )]
    pub location_type: LocationType,
    pub urgency: String,
    pub nickname: String,
    pub x: f32,
    pub y: f32,
    pub color: u32,
    pub nickname_x: f32,
    pub nickname_y: f32,
    pub invariant_x: f32,
    pub invariant_y: f32,
}

impl From<Location> for DummyLocation {
    fn from(item: Location) -> Self {
        DummyLocation {
            id: item.id,
            invariant: item.invariant,
            location_type: item.location_type,
            urgency: item.urgency,
            nickname: "".to_string(),
            x: 100.0,
            y: 100.0,
            color: 6,
            nickname_x: 30.0,
            nickname_y: -10.0,
            invariant_x: 30.0,
            invariant_y: 10.0,
        }
    }
}

/// Function used for deserializing declarations
pub fn decode_declarations<'de, D>(deserializer: D) -> Result<Declarations, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    //Split string into vector of strings
    let decls: Vec<String> = s.split('\n').map(|s| s.into()).collect();
    let mut ints: HashMap<String, i32> = HashMap::new();
    let mut clocks: HashMap<String, ClockIndex> = HashMap::new();
    let mut counter: ClockIndex = 1;
    for string in decls {
        //skip comments
        if string.starts_with("//") || string.is_empty() {
            continue;
        }
        let sub_decls: Vec<String> = string.split(';').map(|s| s.into()).collect();

        for sub_decl in sub_decls {
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
                    panic!("Not implemented read for type: \"{}\"", variable_type);
                }
            }
        }
    }

    Ok(Declarations { ints, clocks })
}

/// Function used for deserializing guards
pub fn decode_guard<'de, D>(
    deserializer: D,
) -> Result<Option<expressions::BoolExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    parse_edge::parse_guard(&s).map(Some).map_err(|err| {
        serde::de::Error::custom(format!("Could not parse {} got error: {:?}", s, err))
    })
}

//Function used for deserializing updates
pub fn decode_update<'de, D>(deserializer: D) -> Result<Option<Vec<parse_edge::Update>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }

    parse_edge::parse_updates(&s).map(Some).map_err(|err| {
        serde::de::Error::custom(format!("Could not parse {} got error: {:?}", s, err))
    })
}

//Function used for deserializing invariants
pub fn decode_invariant<'de, D>(
    deserializer: D,
) -> Result<Option<expressions::BoolExpression>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    match parse_edge::parse_guard(&s) {
        Ok(edge_attribute) => Ok(Some(edge_attribute)),
        Err(e) => panic!("Could not parse invariant {} got error: {:?}", s, e),
    }
}

//Function used for deserializing sync types
pub fn decode_sync_type<'de, D>(deserializer: D) -> Result<SyncType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "INPUT" => Ok(SyncType::Input),
        "OUTPUT" => Ok(SyncType::Output),
        _ => panic!("Unknown sync type in status {:?}", s),
    }
}

pub fn encode_sync_type<S>(sync_type: &SyncType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match sync_type {
        SyncType::Input => serializer.serialize_str("INPUT"),
        SyncType::Output => serializer.serialize_str("OUTPUT"),
    }
}

pub fn decode_sync<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.contains('!') {
        let res = s.replace('!', "");
        Ok(res)
    } else if s.contains('?') {
        let res = s.replace('?', "");
        Ok(res)
    } else {
        Ok(s)
    }
}

// Function used for deserializing location types
pub fn decode_location_type<'de, D>(deserializer: D) -> Result<LocationType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "NORMAL" => Ok(LocationType::Normal),
        "INITIAL" => Ok(LocationType::Initial),
        "UNIVERSAL" => Ok(LocationType::Universal),
        "INCONSISTENT" => Ok(LocationType::Inconsistent),
        _ => panic!("Unknown sync type in status {:?}", s),
    }
}

// Function used for deserializing location types
pub fn encode_location_type<S>(
    location_type: &LocationType,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match location_type {
        LocationType::Normal => serializer.serialize_str("NORMAL"),
        LocationType::Initial => serializer.serialize_str("INITIAL"),
        LocationType::Universal => serializer.serialize_str("UNIVERSAL"),
        LocationType::Inconsistent => serializer.serialize_str("INCONSISTENT"),
        LocationType::Any => unreachable!(),
    }
}

pub fn encode_declarations<S>(decls: &Declarations, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut output = String::from("clock ");
    let mut it = decls.clocks.iter();
    if let Some((first_clock, _)) = it.next() {
        output = output.add(first_clock);

        for (clock, _) in it {
            output = output.add(&format!(", {}", clock));
        }
        output = output.add(";");

        return serializer.serialize_str(&output);
    }
    serializer.serialize_str("")
}

pub fn encode_opt_boolexpr<S>(
    opt_expr: &Option<expressions::BoolExpression>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(expr) = opt_expr {
        encode_boolexpr(expr, serializer)
    } else {
        serializer.serialize_str("")
    }
}

pub fn encode_boolexpr<S>(
    expr: &expressions::BoolExpression,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&expr.encode_expr())
}

pub fn encode_arithexpr<S>(
    expr: &expressions::ArithExpression,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&expr.encode_expr())
}

pub fn encode_opt_updates<S>(
    opt_updates: &Option<Vec<parse_edge::Update>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut output = String::new();
    if let Some(updates) = opt_updates {
        for (i, update) in updates.iter().enumerate() {
            output = output.add(
                &[
                    update.get_variable_name(),
                    "=",
                    &update.get_expression().encode_expr(),
                ]
                .concat(),
            );

            if i != updates.len() - 1 {
                output = output.add(", ");
            }
        }
        serializer.serialize_str(&output)
    } else {
        serializer.serialize_str("")
    }
}
