use crate::data_reader::serialization::{decode_invariant, decode_location_type, DummyLocation};
use crate::model_objects::expressions::BoolExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Copy)]
pub enum LocationType {
    Normal,
    Initial,
    Universal,
    Inconsistent,
    Any,
}

impl LocationType {
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (LocationType::Initial, LocationType::Initial) => LocationType::Initial,
            _ => LocationType::Normal,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(into = "DummyLocation")]
pub struct Location {
    pub id: String,
    #[serde(
        deserialize_with = "decode_invariant",
        serialize_with = "encode_opt_boolexpr"
    )]
    pub invariant: Option<BoolExpression>,
    #[serde(
        deserialize_with = "decode_location_type",
        serialize_with = "encode_location_type",
        rename = "type"
    )]
    pub location_type: LocationType,
    pub urgency: String,
}

impl Location {
    pub fn get_id(&self) -> &String {
        &self.id
    }
    pub fn get_invariant(&self) -> &Option<BoolExpression> {
        &self.invariant
    }
    pub fn get_location_type(&self) -> LocationType {
        self.location_type
    }
    pub fn get_urgency(&self) -> &String {
        &self.urgency
    }
}
