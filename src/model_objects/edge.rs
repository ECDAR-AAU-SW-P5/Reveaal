use crate::data_reader::parse_edge;
use crate::data_reader::serialization::{
    decode_guard, decode_sync, decode_sync_type, decode_update, DummyEdge,
};
use crate::edge_eval::constraint_applier::apply_constraints_to_state;
use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::Declarations;
use edbm::zones::OwnedFederation;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(into = "DummyEdge")]
pub struct Edge {
    /// Uniquely identifies the edge within its component
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
    pub guard: Option<BoolExpression>,
    #[serde(
        deserialize_with = "decode_update",
        serialize_with = "encode_opt_updates"
    )]
    pub update: Option<Vec<parse_edge::Update>>,
    #[serde(deserialize_with = "decode_sync")]
    pub sync: String,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Edge {{{}-({}{})->{}, Guard: {}, Update: {:?}}}",
            self.source_location,
            self.sync,
            match self.sync_type {
                SyncType::Input => "?",
                SyncType::Output => "!",
            },
            self.target_location,
            self.guard.as_ref().unwrap_or(&TRUE),
            self.update
        ))?;
        Ok(())
    }
}

impl Edge {
    pub fn apply_update(
        &self,
        decl: &Declarations, //Will eventually be mutable
        mut fed: OwnedFederation,
    ) -> OwnedFederation {
        if let Some(updates) = self.get_update() {
            for update in updates {
                fed = update.compiled(decl).apply(fed);
            }
        }

        fed
    }

    pub fn apply_guard(&self, decl: &Declarations, mut fed: OwnedFederation) -> OwnedFederation {
        if let Some(guards) = self.get_guard() {
            fed = apply_constraints_to_state(guards, decl, fed).expect("Failed to apply guard");
        };

        fed
    }

    pub fn get_source_location(&self) -> &String {
        &self.source_location
    }

    pub fn get_target_location(&self) -> &String {
        &self.target_location
    }

    pub fn get_sync_type(&self) -> &SyncType {
        &self.sync_type
    }

    pub fn get_guard(&self) -> &Option<BoolExpression> {
        &self.guard
    }

    pub fn get_update(&self) -> &Option<Vec<parse_edge::Update>> {
        &self.update
    }

    pub fn get_sync(&self) -> &String {
        &self.sync
    }

    pub fn get_update_clocks(&self) -> Vec<&str> {
        let mut clock_vec = vec![];
        if let Some(updates) = self.get_update() {
            for u in updates {
                clock_vec.push(u.get_variable_name())
            }
        }

        clock_vec
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SyncType {
    Input,
    Output,
}

const TRUE: BoolExpression = BoolExpression::Bool(true);
