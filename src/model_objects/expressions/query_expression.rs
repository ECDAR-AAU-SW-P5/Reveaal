use std::fmt::{Display, Formatter};

use super::StateExpression;

#[derive(Debug, Clone)]
pub enum QueryExpression {
    Refinement(SystemExpression, SystemExpression),
    Consistency(SystemExpression),
    Reachability {
        system: SystemExpression,
        from: Option<StateExpression>,
        to: StateExpression,
    },
    Implementation(SystemExpression),
    Determinism(SystemExpression),
    Specification(SystemExpression),
    GetComponent(SaveExpression),
    Prune(SaveExpression),
    BisimMinim(SaveExpression),
    Syntax(SystemExpression),
}

#[derive(Debug, Clone)]
pub struct SaveExpression {
    pub system: SystemExpression,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SystemExpression {
    /// Fx. `"A[Temp]"` -> `Component("A", Some("Temp"))`
    /// Fx. `"A"` -> `Component("A", None)`
    Component(String, Option<String>),
    Quotient(Box<SystemExpression>, Box<SystemExpression>),
    Composition(Box<SystemExpression>, Box<SystemExpression>),
    Conjunction(Box<SystemExpression>, Box<SystemExpression>),
}

impl Display for SystemExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemExpression::Component(name, Some(id)) => {
                write!(f, "{}[{}]", name, id)?;
            }
            SystemExpression::Component(name, None) => {
                write!(f, "{}", name)?;
            }
            SystemExpression::Quotient(left, right) => {
                write!(f, "({} \\\\ {})", left, right)?;
            }
            SystemExpression::Composition(left, right) => {
                write!(f, "({} || {})", left, right)?;
            }
            SystemExpression::Conjunction(left, right) => {
                write!(f, "({} && {})", left, right)?;
            }
        }
        Ok(())
    }
}

impl Display for SaveExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} save-as {}", name, self.system),
            None => write!(f, "{}", self.system),
        }
    }
}

impl Display for QueryExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryExpression::Refinement(left, right) => {
                write!(f, "refinement: {} <= {}", left, right)
            }
            QueryExpression::Reachability { system, from, to } => {
                write!(
                    f,
                    "reachability: {} @ {} -> {}",
                    system,
                    match from {
                        Some(expr) => expr.to_string(),
                        None => "init".to_string(),
                    },
                    to
                )
            }
            QueryExpression::Consistency(system) => {
                write!(f, "consistency: {}", system)
            }
            QueryExpression::GetComponent(comp) => {
                write!(f, "get-component: {}", comp)
            }
            QueryExpression::Prune(comp) => {
                write!(f, "prune: {}", comp)
            }
            QueryExpression::BisimMinim(comp) => {
                write!(f, "bisim-minim: {}", comp)
            }
            QueryExpression::Implementation(system) => {
                write!(f, "implementation: {}", system)
            }
            QueryExpression::Determinism(system) => {
                write!(f, "determinism: {}", system)
            }
            QueryExpression::Specification(system) => {
                write!(f, "specification: {}", system)
            }
            QueryExpression::Syntax(system) => {
                write!(f, "syntax: {}", system)
            }
        }
    }
}
