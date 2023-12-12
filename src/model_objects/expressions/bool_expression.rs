use colored::Colorize;
use edbm::util::constraints::{ClockIndex, Conjunction, Constraint, Disjunction};

use serde::Deserialize;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops;

use super::ArithExpression;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum BoolExpression {
    AndOp(Box<BoolExpression>, Box<BoolExpression>),
    OrOp(Box<BoolExpression>, Box<BoolExpression>),
    LessEQ(Box<ArithExpression>, Box<ArithExpression>),
    GreatEQ(Box<ArithExpression>, Box<ArithExpression>),
    LessT(Box<ArithExpression>, Box<ArithExpression>),
    GreatT(Box<ArithExpression>, Box<ArithExpression>),
    EQ(Box<ArithExpression>, Box<ArithExpression>),
    Bool(bool),
}

impl BoolExpression {
    pub fn swap_clock_names(
        &self,
        from_vars: &HashMap<String, ClockIndex>,
        to_vars: &HashMap<ClockIndex, String>,
    ) -> BoolExpression {
        match self {
            BoolExpression::AndOp(left, right) => BoolExpression::AndOp(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::OrOp(left, right) => BoolExpression::OrOp(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::LessEQ(left, right) => BoolExpression::LessEQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::LessT(left, right) => BoolExpression::LessT(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::EQ(left, right) => BoolExpression::EQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::GreatEQ(left, right) => BoolExpression::GreatEQ(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::GreatT(left, right) => BoolExpression::GreatT(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            BoolExpression::Bool(val) => BoolExpression::Bool(*val),
        }
    }

    pub fn encode_expr(&self) -> String {
        match self {
            BoolExpression::AndOp(left, right) => [
                left.encode_expr(),
                String::from(" && "),
                right.encode_expr(),
            ]
            .concat(),
            BoolExpression::OrOp(left, right) => [
                left.encode_expr(),
                String::from(" || "),
                right.encode_expr(),
            ]
            .concat(),
            BoolExpression::LessEQ(left, right) => {
                [left.encode_expr(), String::from("<="), right.encode_expr()].concat()
            }
            BoolExpression::GreatEQ(left, right) => {
                [left.encode_expr(), String::from(">="), right.encode_expr()].concat()
            }
            BoolExpression::LessT(left, right) => {
                [left.encode_expr(), String::from("<"), right.encode_expr()].concat()
            }
            BoolExpression::GreatT(left, right) => {
                [left.encode_expr(), String::from(">"), right.encode_expr()].concat()
            }
            BoolExpression::EQ(left, right) => {
                [left.encode_expr(), String::from("=="), right.encode_expr()].concat()
            }
            BoolExpression::Bool(boolean) => boolean.to_string(),
        }
    }

    pub fn from_disjunction(
        disjunction: &Disjunction,
        naming: &HashMap<String, ClockIndex>,
    ) -> Option<Self> {
        let naming = naming
            .iter()
            .map(|(name, index)| (*index, name.clone()))
            .collect();
        if disjunction.conjunctions.is_empty() {
            Some(BoolExpression::Bool(false))
        } else if disjunction.conjunctions.len() == 1 {
            BoolExpression::from_conjunction(&disjunction.conjunctions[0], &naming)
        } else {
            let mut result = None;

            for conjunction in &disjunction.conjunctions {
                // If any is None (true), the disjuntion is None (true) so we use ?
                let expr = BoolExpression::from_conjunction(conjunction, &naming)?;

                match result {
                    None => result = Some(expr),
                    Some(res) => result = Some(BoolExpression::OrOp(Box::new(res), Box::new(expr))),
                }
            }

            result
        }
    }

    pub fn from_conjunction(
        conjunction: &Conjunction,
        naming: &HashMap<ClockIndex, String>,
    ) -> Option<Self> {
        if conjunction.constraints.is_empty() {
            //BoolExpression::Bool(true)
            None
        } else if conjunction.constraints.len() == 1 {
            Some(BoolExpression::from_constraint(
                &conjunction.constraints[0],
                naming,
            ))
        } else {
            let mut result = None;

            for constraint in &conjunction.constraints {
                let expr = BoolExpression::from_constraint(constraint, naming);
                match result {
                    None => result = Some(expr),
                    Some(res) => {
                        result = Some(BoolExpression::AndOp(Box::new(res), Box::new(expr)))
                    }
                }
            }

            result
        }
    }

    pub fn from_constraint(constraint: &Constraint, naming: &HashMap<ClockIndex, String>) -> Self {
        let ineq = constraint.ineq();
        let is_strict = ineq.is_strict();
        let bound = ineq.bound();
        let i = constraint.i;
        let j = constraint.j;

        match (i, j) {
            (0, 0) => {
                unreachable!("Constraint with i == 0 and j == 0 is not allowed");
            }
            (0, j) => {
                // negated lower bound
                match is_strict {
                    true => {
                        BoolExpression::GreatT(var_from_naming(naming, j), arith_from_int(-bound))
                    }
                    false => {
                        BoolExpression::GreatEQ(var_from_naming(naming, j), arith_from_int(-bound))
                    }
                }
            }
            (i, 0) => {
                // upper bound
                match is_strict {
                    true => {
                        BoolExpression::LessT(var_from_naming(naming, i), arith_from_int(bound))
                    }
                    false => {
                        BoolExpression::LessEQ(var_from_naming(naming, i), arith_from_int(bound))
                    }
                }
            }
            (i, j) => {
                // difference
                if bound == 0 {
                    // i-j<=0 -> i <= 0+j
                    match is_strict {
                        true => BoolExpression::LessT(
                            var_from_naming(naming, i),
                            var_from_naming(naming, j),
                        ),
                        false => BoolExpression::LessEQ(
                            var_from_naming(naming, i),
                            var_from_naming(naming, j),
                        ),
                    }
                } else {
                    match is_strict {
                        true => BoolExpression::LessT(
                            var_diff_from_naming(naming, i, j),
                            arith_from_int(bound),
                        ),
                        false => BoolExpression::LessEQ(
                            var_diff_from_naming(naming, i, j),
                            arith_from_int(bound),
                        ),
                    }
                }
            }
        }
    }

    pub fn get_max_constant(&self, clock: ClockIndex, clock_name: &str) -> i32 {
        let mut new_constraint = 0;

        self.iterate_constraints(&mut |left, right| {
            //Start by matching left and right operands to get constant, this might fail if it does we skip constraint defaulting to 0
            let constant = ArithExpression::get_constant(left, right, clock, clock_name);

            if new_constraint < constant {
                new_constraint = constant;
            }
        });

        new_constraint
    }

    pub fn swap_var_name(&mut self, from_name: &str, to_name: &str) {
        match self {
            BoolExpression::AndOp(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::OrOp(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::LessEQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::GreatT(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::GreatEQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::LessT(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::EQ(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            BoolExpression::Bool(_) => {}
        }
    }

    pub fn conjunction(guards: &mut Vec<BoolExpression>) -> BoolExpression {
        let num_guards = guards.len();

        if let Some(guard) = guards.pop() {
            if num_guards == 1 {
                guard
            } else {
                BoolExpression::AndOp(
                    Box::new(guard),
                    Box::new(BoolExpression::conjunction(guards)),
                )
            }
        } else {
            BoolExpression::Bool(false)
        }
    }

    pub fn iterate_constraints<F>(&self, function: &mut F)
    where
        F: FnMut(&ArithExpression, &ArithExpression),
    {
        match self {
            BoolExpression::AndOp(left, right) => {
                left.iterate_constraints(function);
                right.iterate_constraints(function);
            }
            BoolExpression::OrOp(left, right) => {
                left.iterate_constraints(function);
                right.iterate_constraints(function);
            }
            BoolExpression::GreatEQ(left, right) => function(left, right),
            BoolExpression::LessEQ(left, right) => function(left, right),
            BoolExpression::LessT(left, right) => function(left, right),
            BoolExpression::GreatT(left, right) => function(left, right),
            BoolExpression::EQ(left, right) => function(left, right),
            _ => (),
        }
    }

    pub fn simplify(&mut self) {
        while self.simplify_helper() {}
    }

    fn simplify_helper(&mut self) -> bool {
        let mut changed = false;
        let mut value = None;
        match self {
            BoolExpression::AndOp(left, right) => {
                changed |= left.simplify_helper();
                changed |= right.simplify_helper();
                match **left {
                    BoolExpression::Bool(false) => value = Some(BoolExpression::Bool(false)),
                    BoolExpression::Bool(true) => value = Some((**right).clone()),
                    _ => {}
                }
                match **right {
                    BoolExpression::Bool(false) => value = Some(BoolExpression::Bool(false)),
                    BoolExpression::Bool(true) => value = Some((**left).clone()),
                    _ => {}
                }
            }
            BoolExpression::OrOp(left, right) => {
                changed |= left.simplify_helper();
                changed |= right.simplify_helper();
                match **left {
                    BoolExpression::Bool(true) => value = Some(BoolExpression::Bool(true)),
                    BoolExpression::Bool(false) => value = Some((**right).clone()),
                    _ => {}
                }
                match **right {
                    BoolExpression::Bool(true) => value = Some(BoolExpression::Bool(true)),
                    BoolExpression::Bool(false) => value = Some((**left).clone()),
                    _ => {}
                }
            }

            BoolExpression::LessEQ(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x <= y))
                    }
                }
            }
            BoolExpression::GreatEQ(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x >= y))
                    }
                }
            }
            BoolExpression::LessT(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x < y))
                    }
                }
            }
            BoolExpression::GreatT(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x > y))
                    }
                }
            }
            BoolExpression::EQ(l, r) => {
                **l = l.simplify().expect("Can't simplify");
                **r = r.simplify().expect("Can't simplify");
                if let ArithExpression::Int(x) = **l {
                    if let ArithExpression::Int(y) = **r {
                        value = Some(BoolExpression::Bool(x == y))
                    }
                }
            }
            BoolExpression::Bool(_) => {}
        }

        if let Some(new_val) = value {
            *self = new_val;
            true
        } else {
            changed
        }
    }

    /// Checks if the clock name is used in the expression.
    pub fn has_var_name(&self, name: &String) -> bool {
        match self {
            BoolExpression::AndOp(p1, p2) | BoolExpression::OrOp(p1, p2) => {
                p1.has_var_name(name) || p2.has_var_name(name)
            }
            BoolExpression::LessEQ(a1, a2)
            | BoolExpression::GreatEQ(a1, a2)
            | BoolExpression::LessT(a1, a2)
            | BoolExpression::GreatT(a1, a2)
            | BoolExpression::EQ(a1, a2) => a1.has_var_name(name) || a2.has_var_name(name),
            BoolExpression::Bool(_) => false,
        }
    }

    /// Finds the clocks used in the expression and put them into result_clocks.
    pub fn get_var_names(&self) -> Vec<String> {
        let mut vec = vec![];
        self.get_var_names_rec(&mut vec);
        vec
    }
    fn get_var_names_rec(&self, result_clocks: &mut Vec<String>) {
        match self {
            BoolExpression::AndOp(ref left, ref right)
            | BoolExpression::OrOp(ref left, ref right) => {
                left.get_var_names_rec(result_clocks);
                right.get_var_names_rec(result_clocks);
            }
            BoolExpression::LessEQ(ref left, ref right)
            | BoolExpression::GreatEQ(ref left, ref right)
            | BoolExpression::LessT(ref left, ref right)
            | BoolExpression::GreatT(ref left, ref right)
            | BoolExpression::EQ(ref left, ref right) => {
                left.get_var_names_rec(result_clocks);
                right.get_var_names_rec(result_clocks);
            }
            BoolExpression::Bool(_) => (),
        }
    }

    /// Replaces all occurrences of `ArithExpression::VarName(old)` with `new`

    /// # Arguments
    /// `old`: The `var name` to be replaced

    /// `new`: The new var name
    pub fn replace_var_name(&mut self, old: &String, new: &String) {
        match self {
            BoolExpression::AndOp(e1, e2) | BoolExpression::OrOp(e1, e2) => {
                e1.replace_var_name(old, new);
                e2.replace_var_name(old, new);
            }
            BoolExpression::LessEQ(e1, e2)
            | BoolExpression::GreatEQ(e1, e2)
            | BoolExpression::LessT(e1, e2)
            | BoolExpression::GreatT(e1, e2)
            | BoolExpression::EQ(e1, e2) => {
                e1.replace_var_name(old, new);
                e2.replace_var_name(old, new);
            }
            BoolExpression::Bool(_) => (),
        }
    }

    pub fn b_less_eq(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::LessEQ(Box::new(left), Box::new(right))
    }
    pub fn b_less_t(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::LessT(Box::new(left), Box::new(right))
    }
    pub fn b_great_eq(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::GreatEQ(Box::new(left), Box::new(right))
    }
    pub fn b_great_t(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::GreatT(Box::new(left), Box::new(right))
    }
    pub fn b_eq(left: ArithExpression, right: ArithExpression) -> BoolExpression {
        BoolExpression::EQ(Box::new(left), Box::new(right))
    }
    pub fn b_par(inner: BoolExpression) -> BoolExpression {
        inner
    }
}

impl Default for BoolExpression {
    fn default() -> Self {
        BoolExpression::Bool(true)
    }
}

fn var_from_naming(
    naming: &HashMap<ClockIndex, String>,
    index: ClockIndex,
) -> Box<ArithExpression> {
    Box::new(ArithExpression::VarName(
        naming.get(&index).unwrap().to_string(),
    ))
}

fn var_diff_from_naming(
    naming: &HashMap<ClockIndex, String>,
    i: ClockIndex,
    j: ClockIndex,
) -> Box<ArithExpression> {
    Box::new(ArithExpression::Difference(
        var_from_naming(naming, i),
        var_from_naming(naming, j),
    ))
}

fn arith_from_int(value: i32) -> Box<ArithExpression> {
    Box::new(ArithExpression::Int(value))
}

impl ops::BitAnd for BoolExpression {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        BoolExpression::AndOp(Box::new(self), Box::new(other))
    }
}

impl ops::BitOr for BoolExpression {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        BoolExpression::OrOp(Box::new(self), Box::new(other))
    }
}

impl Display for BoolExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolExpression::AndOp(left, right) => {
                // And(eq(a,b), And(eq(b,c), And(eq(c,d), And(...)))) -> a=b=c=d
                match &**left {
                    BoolExpression::EQ(a, b)
                    | BoolExpression::LessEQ(a, b)
                    | BoolExpression::LessT(a, b) => match &**right {
                        BoolExpression::AndOp(op, _) => {
                            if let BoolExpression::EQ(b1, _c)
                            | BoolExpression::LessEQ(b1, _c)
                            | BoolExpression::LessT(b1, _c) = &**op
                            {
                                if **b == **b1 {
                                    write!(f, "{}{}{}", a, get_op(left).unwrap(), right)?;
                                    return Ok(());
                                }
                            }
                        }
                        BoolExpression::EQ(b1, _c)
                        | BoolExpression::LessEQ(b1, _c)
                        | BoolExpression::LessT(b1, _c) => {
                            if **b == **b1 {
                                write!(f, "{}{}{}", a, get_op(left).unwrap(), right)?;
                                return Ok(());
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

                let l = match **left {
                    BoolExpression::OrOp(_, _) => format!("({})", left),
                    _ => format!("{}", left),
                };
                let r = match **right {
                    BoolExpression::OrOp(_, _) => format!("({})", right),
                    _ => format!("{}", right),
                };
                write!(f, "{} && {}", l, r)?;
            }
            BoolExpression::OrOp(left, right) => {
                let l = match **left {
                    BoolExpression::AndOp(_, _) => format!("({})", left),
                    _ => format!("{}", left),
                };
                let r = match **right {
                    BoolExpression::AndOp(_, _) => format!("({})", right),
                    _ => format!("{}", right),
                };
                write!(f, "{} || {}", l, r)?;
            }
            BoolExpression::GreatEQ(left, right) => {
                write!(f, "{}≥{}", left, right)?;
            }
            BoolExpression::LessEQ(left, right) => {
                write!(f, "{}≤{}", left, right)?;
            }
            BoolExpression::LessT(left, right) => {
                write!(f, "{}<{}", left, right)?;
            }
            BoolExpression::GreatT(left, right) => {
                write!(f, "{}>{}", left, right)?;
            }
            BoolExpression::EQ(left, right) => {
                write!(f, "{}={}", left, right)?;
            }
            BoolExpression::Bool(val) => {
                if *val {
                    write!(f, "{}", val.to_string().green())?;
                } else {
                    write!(f, "{}", val.to_string().red())?;
                }
            }
        }
        Ok(())
    }
}

fn get_op(exp: &BoolExpression) -> Option<String> {
    match exp {
        BoolExpression::EQ(_, _) => Some("=".to_string()),
        BoolExpression::LessEQ(_, _) => Some("≤".to_string()),
        BoolExpression::LessT(_, _) => Some("<".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::data_reader::parse_edge::parse_guard;
    use test_case::test_case;

    #[test_case("0>4", vec ! [], true; "No clocks")]
    #[test_case("x<=5", vec ! ["x".to_string()], true; "A single clock using leq")]
    #[test_case("x <= 5", vec ! ["x".to_string()], true; "A single clock with spaces")]
    #[test_case("x==5", vec ! ["x".to_string()], true; "A single clock using eq")]
    #[test_case("x>=5", vec ! ["x".to_string()], true; "A single clock using geq")]
    #[test_case("x>=xx", vec ! ["x".to_string(), "xx".to_string()], true; "Two clocks with similar names")]
    #[test_case("x<5&&x>0", vec ! ["x".to_string(), "x".to_string()], true; "Two occurrences of the same clock")]
    #[test_case("x<y", vec ! ["x".to_string(), "y".to_string()], true; "Two different clocks")]
    #[test_case("alpha<5", vec ! ["alpha".to_string()], true; "Longer clock names")]
    #[test_case("x>2&&y+1<=6", vec ! ["x".to_string(), "y".to_string()], true; "Two different clocks in two different expressions")]
    #[test_case("x<5&&b>1", vec ! ["x".to_string(), "y".to_string()], false; "Two clocks, should fail")]
    #[test_case("x<5+4", vec ! ["x".to_string()], true; "A single clock with arithmetic expressions")]
    #[test_case("x<5+4||y==6*4", vec ! ["x".to_string(), "y".to_string()], true; "Two clocks with arithmetic expressions")]
    pub fn test_get_clocks_bool(expression: &str, expected: Vec<String>, verdict: bool) {
        // Arrange
        // parse_guard is used to parse a boolean expression, as guards are just boolean expressions.
        match parse_guard(expression) {
            Ok(input_expr) => {
                // Act
                let results = input_expr.get_var_names();
                // Assert
                assert_eq!((expected == results), verdict);
            }
            Err(err) => {
                panic!("Test failed: {}", err);
            }
        };
    }
}
