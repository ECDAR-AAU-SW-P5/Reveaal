use std::collections::HashMap;
use std::fmt;

use crate::data_reader::parse_edge;
use crate::model_objects::expressions::ArithExpression;
use crate::model_objects::Declarations;
use colored::Colorize;
use edbm::util::constraints::ClockIndex;
use edbm::zones::OwnedFederation;

#[derive(Debug, Clone)]
pub struct CompiledUpdate {
    pub clock_index: ClockIndex,
    pub value: i32,
}

impl fmt::Display for CompiledUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}:={}",
            format!("c:{}", self.clock_index).magenta(),
            self.value
        ))?;
        Ok(())
    }
}

impl CompiledUpdate {
    pub fn compile(update: &parse_edge::Update, decl: &Declarations) -> CompiledUpdate {
        match *update.get_expression() {
            ArithExpression::Int(val) => {
                if let Some(&clock_index) = decl.get_clock_index_by_name(update.get_variable_name())
                {
                    CompiledUpdate {
                        clock_index,
                        value: val,
                    }
                } else {
                    panic!(
                        "Attempting to compile an update with a clock \"{}\" which is not in decl",
                        update.get_variable_name()
                    )
                }
            }
            _ => {
                panic!("Should not be able to assign to {:?} in update", update)
            }
        }
    }

    pub fn apply(&self, fed: OwnedFederation) -> OwnedFederation {
        fed.update_clock_val(self.clock_index, self.value)
    }

    pub fn as_update(&self, clocks: &HashMap<String, usize>) -> parse_edge::Update {
        let map: HashMap<usize, String> = clocks.clone().into_iter().map(|(l, r)| (r, l)).collect();

        parse_edge::Update {
            variable: map.get(&self.clock_index).unwrap().clone(),
            expression: ArithExpression::Int(self.value),
        }
    }

    pub fn apply_as_free(&self, fed: OwnedFederation) -> OwnedFederation {
        fed.free_clock(self.clock_index)
    }

    pub fn apply_as_guard(&self, fed: OwnedFederation) -> OwnedFederation {
        fed.constrain_eq(self.clock_index, self.value)
    }
}
