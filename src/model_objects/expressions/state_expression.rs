use super::{super::Component, ArithExpression, BoolExpression};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum StateExpression {
    LEQ(OperandExpression, OperandExpression),
    GEQ(OperandExpression, OperandExpression),
    EQ(OperandExpression, OperandExpression),
    LT(OperandExpression, OperandExpression),
    GT(OperandExpression, OperandExpression),
    AND(Vec<StateExpression>),
    OR(Vec<StateExpression>),
    Location(ComponentVariable),
    NOT(Box<StateExpression>),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum OperandExpression {
    Number(i32),
    Clock(ComponentVariable),
    Difference(Box<OperandExpression>, Box<OperandExpression>),
    Sum(Box<OperandExpression>, Box<OperandExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentVariable {
    /// Fx. `"A[Temp].x"` -> `"A"`
    pub component: String,
    /// Fx. `"A[Temp].x"` -> `Some("Temp")` or `"A.x"` -> `None`
    pub special_id: Option<String>,
    /// Fx. `"A[Temp].x"` -> `"x"`
    pub variable: String,
}

impl Display for ComponentVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.special_id {
            Some(id) => write!(f, "{}[{}].{}", self.component, id, self.variable),
            None => write!(f, "{}.{}", self.component, self.variable),
        }
    }
}

impl OperandExpression {
    pub fn to_arith_expression(&self, comps: &Vec<&Component>) -> Result<ArithExpression, String> {
        match self {
            OperandExpression::Number(n) => Ok(ArithExpression::Int(*n)),
            OperandExpression::Clock(ComponentVariable {
                component,
                special_id,
                variable,
            }) => {
                let comp = comps
                    .iter()
                    .find(|c| c.name == *component && c.special_id == *special_id)
                    .ok_or_else(|| format!("Component '{}' not found", component))?;
                let clock_id = comp
                    .declarations
                    .get_clock_index_by_name(variable)
                    .ok_or_else(|| {
                        format!(
                            "Clock '{}' not found for component '{}'",
                            variable, component
                        )
                    })?;

                Ok(ArithExpression::Clock(*clock_id))
            }
            OperandExpression::Difference(left, right) => Ok(ArithExpression::Difference(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            OperandExpression::Sum(left, right) => Ok(ArithExpression::Addition(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
        }
    }
}

impl Display for OperandExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandExpression::Number(n) => write!(f, "{}", n),
            OperandExpression::Clock(var) => write!(f, "{}", var),
            OperandExpression::Difference(left, right) => {
                write!(f, "{} - {}", left, right)
            }
            OperandExpression::Sum(left, right) => write!(f, "{} + {}", left, right),
        }
    }
}

impl StateExpression {
    pub fn to_bool_expression(&self, comps: &Vec<&Component>) -> Result<BoolExpression, String> {
        match self {
            StateExpression::LEQ(left, right) => Ok(BoolExpression::LessEQ(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::GEQ(left, right) => Ok(BoolExpression::GreatEQ(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::EQ(left, right) => Ok(BoolExpression::EQ(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::LT(left, right) => Ok(BoolExpression::LessT(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::GT(left, right) => Ok(BoolExpression::GreatT(
                Box::new(left.to_arith_expression(comps)?),
                Box::new(right.to_arith_expression(comps)?),
            )),
            StateExpression::AND(exprs) => {
                let mut exprs = exprs
                    .iter()
                    .map(|e| e.to_bool_expression(comps))
                    .collect::<Result<Vec<BoolExpression>, _>>()?
                    .into_iter();
                let first = exprs
                    .next()
                    .expect("AND expression must have at least one operand");
                Ok(exprs.fold(first, |acc, e| {
                    BoolExpression::AndOp(Box::new(acc), Box::new(e))
                }))
            }
            StateExpression::OR(exprs) => {
                let mut exprs = exprs
                    .iter()
                    .map(|e| e.to_bool_expression(comps))
                    .collect::<Result<Vec<BoolExpression>, _>>()?
                    .into_iter();
                let first = exprs
                    .next()
                    .expect("OR expression must have at least one operand");
                Ok(exprs.fold(first, |acc, e| {
                    BoolExpression::OrOp(Box::new(acc), Box::new(e))
                }))
            }
            StateExpression::NOT(_expr) => {
                unimplemented!("NOT expressions are not supported yet")
            }
            StateExpression::Location(_) => {
                // Locations here should just be ignored
                Ok(BoolExpression::Bool(true))
            }
            StateExpression::Bool(b) => Ok(BoolExpression::Bool(*b)),
        }
    }
}

impl Display for StateExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateExpression::LEQ(left, right) => write!(f, "{} <= {}", left, right),
            StateExpression::GEQ(left, right) => write!(f, "{} >= {}", left, right),
            StateExpression::EQ(left, right) => write!(f, "{} == {}", left, right),
            StateExpression::LT(left, right) => write!(f, "{} < {}", left, right),
            StateExpression::GT(left, right) => write!(f, "{} > {}", left, right),
            StateExpression::AND(exprs) => {
                let mut s = "(".to_string();
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        s.push_str(" && ");
                    }
                    s.push_str(&expr.to_string());
                }
                s.push(')');
                write!(f, "{}", s)
            }
            StateExpression::OR(exprs) => {
                let mut s = "(".to_string();
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        s.push_str(" || ");
                    }
                    s.push_str(&expr.to_string());
                }
                s.push(')');
                write!(f, "{}", s)
            }
            StateExpression::Location(var) => write!(f, "{}", var),
            StateExpression::NOT(expr) => write!(f, "!({})", expr),
            StateExpression::Bool(b) => write!(f, "{}", b),
        }
    }
}
