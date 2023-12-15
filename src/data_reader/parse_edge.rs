extern crate pest;

use crate::edge_eval::updater::CompiledUpdate;

use crate::model_objects::expressions::{ArithExpression, BoolExpression};

use crate::{data_reader::serialization::encode_arithexpr, model_objects::Declarations};
use edbm::util::constraints::ClockIndex;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use pest::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//This file handles parsing the edges based on the abstract syntax described in the .pest files in the grammar folder
//For clarification see documentation on pest crate
#[derive(Parser)]
#[grammar = "data_reader/grammars/edge_grammar.pest"]
pub struct EdgeParser;

lazy_static! {
    static ref PRATT: PrattParser<Rule> = PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left)
            | Op::infix(Rule::div, Assoc::Left)
            | Op::infix(Rule::r#mod, Assoc::Left))
        .op(Op::infix(Rule::and, Assoc::Left))
        .op(Op::infix(Rule::or, Assoc::Left));
}

#[derive(Debug, Clone, Deserialize)]
pub enum EdgeAttribute {
    Updates(Vec<Update>),
    Guard(BoolExpression),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Update {
    pub variable: String,
    #[serde(serialize_with = "encode_arithexpr")]
    pub expression: ArithExpression,
}

impl Update {
    pub fn get_expression(&self) -> &ArithExpression {
        &self.expression
    }

    pub fn get_variable_name(&self) -> &str {
        self.variable.as_str()
    }

    pub fn swap_var_name(&mut self, from_name: &str, to_name: &str) {
        if self.variable == from_name {
            self.variable = to_name.to_string();
        }

        self.expression.swap_var_name(from_name, to_name);
    }

    pub fn swap_clock_names(
        &mut self,
        from_vars: &HashMap<String, ClockIndex>,
        to_vars: &HashMap<ClockIndex, String>,
    ) {
        if let Some(index) = from_vars.get(&self.variable) {
            self.variable = to_vars[index].clone();
            self.expression = self.expression.swap_clock_names(from_vars, to_vars);
        }
    }

    pub fn compiled(&self, decl: &Declarations) -> CompiledUpdate {
        CompiledUpdate::compile(self, decl)
    }
}

/// Parses a guard string `input` into a BoolExpression
pub fn parse_guard(input: &str) -> Result<BoolExpression, String> {
    let mut pairs = match EdgeParser::parse(Rule::guard, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let guard = pairs.next().unwrap();

    // Check if there are any constraints
    let result = match guard.into_inner().next() {
        Some(bool_expr) => parse_bool_expr(bool_expr),
        None => BoolExpression::Bool(true),
    };

    Ok(result)
}

/// Parses an update string `input` into a vector of Updates
pub fn parse_updates(input: &str) -> Result<Vec<Update>, String> {
    let mut pairs = match EdgeParser::parse(Rule::update, input) {
        Ok(pairs) => pairs,
        Err(e) => return Err(format!("Could not parse as rule with error: {}", e)),
    };

    let update = pairs.next().unwrap();

    // Check if there are any assignments
    if let Some(assignments) = update.into_inner().next() {
        let mut updates = Vec::new();
        for assignment in assignments.into_inner() {
            match assignment.as_rule() {
                Rule::assignment => updates.push(parse_update(assignment)),
                _ => unreachable!("Unable to match: {:?} as rule, updates", assignment),
            }
        }
        Ok(updates)
    } else {
        Ok(vec![])
    }
}

fn parse_update(pair: pest::iterators::Pair<Rule>) -> Update {
    let mut inner_pairs = pair.into_inner();
    let variable = inner_pairs.next().unwrap().as_str().to_string();
    let expression = parse_arith_expr(inner_pairs.next().unwrap())
        .simplify()
        .expect("Error simplifying update");

    Update {
        variable,
        expression,
    }
}

fn parse_bool_expr(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    PRATT
        .map_primary(|pair| match pair.as_rule() {
            Rule::boolExpr => parse_bool_expr(pair),
            Rule::bool_true => BoolExpression::Bool(true),
            Rule::bool_false => BoolExpression::Bool(false),
            Rule::comparison => parse_comparison(pair),
            _ => unreachable!("Unable to match: {:?} as rule, bool_expr", pair),
        })
        .map_infix(|left, op, right| match op.as_rule() {
            Rule::and => BoolExpression::AndOp(Box::new(left), Box::new(right)),
            Rule::or => BoolExpression::OrOp(Box::new(left), Box::new(right)),
            _ => unreachable!("Unable to match operation: {:?}, bool_expr", op),
        })
        .parse(pair.into_inner())
}

fn parse_comparison(pair: pest::iterators::Pair<Rule>) -> BoolExpression {
    let mut inner_pairs = pair.into_inner();
    let left_pair = inner_pairs.next().unwrap();
    let op = inner_pairs.next().unwrap();
    let right_pair = inner_pairs.next().unwrap();

    let left = Box::new(parse_arith_expr(left_pair));
    let right = Box::new(parse_arith_expr(right_pair));

    match op.as_rule() {
        Rule::eq => BoolExpression::EQ(left, right),
        Rule::lt => BoolExpression::LessT(left, right),
        Rule::leq => BoolExpression::LessEQ(left, right),
        Rule::gt => BoolExpression::GreatT(left, right),
        Rule::geq => BoolExpression::GreatEQ(left, right),
        _ => unreachable!("Unable to match: {:?} as rule, comparison", op),
    }
}

fn parse_arith_expr(pair: pest::iterators::Pair<Rule>) -> ArithExpression {
    PRATT
        .map_primary(|pair| match pair.as_rule() {
            Rule::arithExpr => parse_arith_expr(pair),
            Rule::int => ArithExpression::Int(pair.as_str().parse().unwrap()),
            Rule::variable => ArithExpression::VarName(pair.as_str().to_string()),
            _ => panic!("Unable to match: {:?} as rule, arith", pair),
        })
        .map_infix(|left, op, right| {
            let left = Box::new(left);
            let right = Box::new(right);
            match op.as_rule() {
                Rule::add => ArithExpression::Addition(left, right),
                Rule::sub => ArithExpression::Difference(left, right),
                Rule::mul => ArithExpression::Multiplication(left, right),
                Rule::div => ArithExpression::Division(left, right),
                Rule::r#mod => ArithExpression::Modulo(left, right),
                _ => unreachable!("Unable to match: {:?} as rule, arith", op),
            }
        })
        .parse(pair.into_inner())
}
