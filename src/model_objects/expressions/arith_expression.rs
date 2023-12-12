use colored::Colorize;
use edbm::util::constraints::ClockIndex;

use serde::Deserialize;

use crate::model_objects::ClockReduceError;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ArithExpression {
    Difference(Box<ArithExpression>, Box<ArithExpression>),
    Addition(Box<ArithExpression>, Box<ArithExpression>),
    Multiplication(Box<ArithExpression>, Box<ArithExpression>),
    Division(Box<ArithExpression>, Box<ArithExpression>),
    Modulo(Box<ArithExpression>, Box<ArithExpression>),
    Clock(ClockIndex),
    VarName(String),
    Int(i32),
}

impl ArithExpression {
    pub fn get_evaluated_int(&self) -> Result<i32, ClockReduceError> {
        match self {
            ArithExpression::Difference(left, right) => {
                Ok(left.get_evaluated_int()? - right.get_evaluated_int()?)
            }
            ArithExpression::Addition(left, right) => {
                Ok(left.get_evaluated_int()? + right.get_evaluated_int()?)
            }
            ArithExpression::Multiplication(left, right) => {
                Ok(left.get_evaluated_int()? * right.get_evaluated_int()?)
            }
            ArithExpression::Division(left, right) => {
                let divide_with = right.get_evaluated_int()?;
                if divide_with == 0 {
                    Err(ClockReduceError::EvaluationError(
                        "Division with zero".to_string(),
                    ))
                } else {
                    Ok(left.get_evaluated_int()? / divide_with)
                }
            }
            ArithExpression::Modulo(left, right) => {
                let modulo_with = right.get_evaluated_int()?;
                if modulo_with == 0 {
                    Err(ClockReduceError::EvaluationError(
                        "Modulo with zero".to_string(),
                    ))
                } else {
                    Ok(left.get_evaluated_int()? % modulo_with)
                }
            }
            ArithExpression::Clock(_) => Err(ClockReduceError::EvaluationError(
                "This function cant work with clock_index".to_string(),
            )),
            ArithExpression::VarName(_) => Err(ClockReduceError::EvaluationError(
                "This function cant work with clock_names".to_string(),
            )),
            ArithExpression::Int(value) => Ok(*value),
        }
    }

    pub fn swap_clock_names(
        &self,
        from_vars: &HashMap<String, ClockIndex>,
        to_vars: &HashMap<ClockIndex, String>,
    ) -> ArithExpression {
        match self {
            ArithExpression::Difference(left, right) => ArithExpression::Difference(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Addition(left, right) => ArithExpression::Addition(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Multiplication(left, right) => ArithExpression::Multiplication(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Division(left, right) => ArithExpression::Division(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Modulo(left, right) => ArithExpression::Modulo(
                Box::new(left.swap_clock_names(from_vars, to_vars)),
                Box::new(right.swap_clock_names(from_vars, to_vars)),
            ),
            ArithExpression::Clock(_) => panic!("Did not expect clock index in boolexpression, cannot swap clock names in misformed bexpr"),
            ArithExpression::VarName(name) => {
                let index = from_vars.get(name).unwrap();
                let new_name = to_vars[index].clone();
                ArithExpression::VarName(new_name)
            },
            ArithExpression::Int(val) => ArithExpression::Int(*val),
        }
    }

    pub fn encode_expr(&self) -> String {
        match self {
            ArithExpression::Difference(left, right) => {
                [left.encode_expr(), String::from("-"), right.encode_expr()].concat()
            }
            ArithExpression::Addition(left, right) => {
                [left.encode_expr(), String::from("+"), right.encode_expr()].concat()
            }
            ArithExpression::Multiplication(left, right) => {
                [left.encode_expr(), String::from("*"), right.encode_expr()].concat()
            }
            ArithExpression::Division(left, right) => {
                [left.encode_expr(), String::from("/"), right.encode_expr()].concat()
            }
            ArithExpression::Modulo(left, right) => {
                [left.encode_expr(), String::from("%"), right.encode_expr()].concat()
            }
            ArithExpression::Clock(_) => [String::from("??")].concat(),
            ArithExpression::VarName(var) => var.clone(),
            ArithExpression::Int(num) => num.to_string(),
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

        new_constraint // * 2 + 1 // This should not actually be a dbm_raw, as it is converted from bound to raw in the c code
    }

    pub fn swap_var_name(&mut self, from_name: &str, to_name: &str) {
        match self {
            ArithExpression::Difference(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Addition(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Multiplication(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Division(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Modulo(left, right) => {
                left.swap_var_name(from_name, to_name);
                right.swap_var_name(from_name, to_name);
            }
            ArithExpression::Clock(_) => {
                //Assuming ids are correctly offset we dont have to do anything here
            }
            ArithExpression::VarName(name) => {
                if *name == from_name {
                    *name = to_name.to_string();
                }
            }
            ArithExpression::Int(_) => {}
        }
    }

    pub fn get_constant(left: &Self, right: &Self, clock: ClockIndex, clock_name: &str) -> i32 {
        match left {
            ArithExpression::Clock(clock_id) => {
                if *clock_id == clock {
                    if let ArithExpression::Int(constant) = right {
                        return *constant;
                    }
                }
            }
            ArithExpression::VarName(name) => {
                if name.eq(clock_name) {
                    if let ArithExpression::Int(constant) = right {
                        return *constant;
                    }
                }
            }
            ArithExpression::Int(constant) => match right {
                ArithExpression::Clock(clock_id) => {
                    if *clock_id == clock {
                        return *constant;
                    }
                }
                ArithExpression::VarName(name) => {
                    if name.eq(clock_name) {
                        return *constant;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        0
    }

    pub fn iterate_constraints<F>(&self, function: &mut F)
    where
        F: FnMut(&ArithExpression, &ArithExpression),
    {
        match self {
            ArithExpression::Difference(left, right) => function(left, right),
            ArithExpression::Addition(left, right) => function(left, right),
            ArithExpression::Multiplication(left, right) => function(left, right),
            ArithExpression::Division(left, right) => function(left, right),
            ArithExpression::Modulo(left, right) => function(left, right),
            ArithExpression::Clock(_) => {}
            ArithExpression::VarName(_) => {}
            ArithExpression::Int(_) => {}
        }
    }

    pub fn simplify(&self) -> Result<ArithExpression, String> {
        let mut out = self.clone();
        let mut diffs: Vec<(ArithExpression, Operation)> = vec![];
        let mut op = Operation::None;
        while let Some(x) = out.move_clock_and_vars(op)? {
            op = x.1.clone();
            diffs.push(x);
        }
        while let Some((val, op)) = diffs.pop() {
            match op {
                Operation::Dif(right) => {
                    out = match right {
                        true => ArithExpression::a_dif(out, val),
                        false => ArithExpression::a_dif(val, out),
                    }
                }
                Operation::Add(right) => {
                    out = match right {
                        true => ArithExpression::a_add(out, val),
                        false => ArithExpression::a_add(val, out),
                    }
                }
                Operation::Mul(right) => {
                    out = match right {
                        true => ArithExpression::a_mul(out, val),
                        false => ArithExpression::a_mul(val, out),
                    }
                }
                Operation::Div(right) => {
                    out = match right {
                        true => ArithExpression::a_div(out, val),
                        false => ArithExpression::a_div(val, out),
                    }
                }
                Operation::Mod(right) => {
                    out = match right {
                        true => ArithExpression::a_mod(out, val),
                        false => ArithExpression::a_mod(val, out),
                    }
                }
                Operation::None => out = val,
            }
        }
        while out.simplify_helper() {}
        Ok(out)
    }

    fn move_clock_and_vars(
        &mut self,
        prev_op: Operation,
    ) -> Result<Option<(ArithExpression, Operation)>, String> {
        let mut switch: Option<ArithExpression> = None;
        let out = match self {
            ArithExpression::Clock(x) => {
                switch = Some(ArithExpression::Int(0));
                Some((ArithExpression::Clock(*x), prev_op))
            }
            ArithExpression::VarName(string) => {
                switch = Some(ArithExpression::Int(0));
                Some((ArithExpression::VarName(string.clone()), prev_op))
            }
            ArithExpression::Int(_) => None,
            ArithExpression::Difference(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(l, r, None)?;
                    l.move_clock_and_vars(Operation::Dif(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(r, l, None)?;
                    r.move_clock_and_vars(Operation::Dif(true))?
                } else {
                    None
                }
            }
            ArithExpression::Addition(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(l, r, None)?;
                    l.move_clock_and_vars(Operation::Add(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(r, l, None)?;
                    r.move_clock_and_vars(Operation::Add(true))?
                } else {
                    None
                }
            }
            ArithExpression::Multiplication(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        l,
                        r,
                        Some("Can't parse multiplication with clocks"),
                    )?;
                    l.move_clock_and_vars(Operation::Mul(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        r,
                        l,
                        Some("Can't parse multiplication with clocks"),
                    )?;
                    r.move_clock_and_vars(Operation::Mul(true))?
                } else {
                    None
                }
            }
            ArithExpression::Division(l, r) => {
                if l.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        l,
                        r,
                        Some("Can't parse division with clocks"),
                    )?;
                    l.move_clock_and_vars(Operation::Div(false))?
                } else if r.clock_var_count() > 0 {
                    switch = ArithExpression::clone_expr(
                        r,
                        l,
                        Some("Can't parse division with clocks"),
                    )?;
                    r.move_clock_and_vars(Operation::Div(true))?
                } else {
                    None
                }
            }
            ArithExpression::Modulo(l, r) => {
                if l.clock_var_count() > 0 {
                    switch =
                        ArithExpression::clone_expr(l, r, Some("Can't parse modulo with clocks"))?;
                    l.move_clock_and_vars(Operation::Mod(false))?
                } else if r.clock_var_count() > 0 {
                    switch =
                        ArithExpression::clone_expr(r, l, Some("Can't parse modulo with clocks"))?;
                    r.move_clock_and_vars(Operation::Mod(true))?
                } else {
                    None
                }
            }
        };

        if let Some(x) = switch {
            *self = x;
        }
        Ok(out)
    }

    fn clone_expr(
        checker: &ArithExpression,
        cloner: &ArithExpression,
        err_msg: Option<&str>,
    ) -> Result<Option<ArithExpression>, String> {
        if let ArithExpression::Clock(_) = *checker {
            if let Some(e) = err_msg {
                Err(e.to_string())
            } else {
                Ok(Some(cloner.clone()))
            }
        } else if let ArithExpression::VarName(_) = *checker {
            Ok(Some(cloner.clone()))
        } else {
            Ok(None)
        }
    }

    fn simplify_helper(&mut self) -> bool {
        let mut changed = false;
        let mut value: Option<ArithExpression> = None;
        match self {
            ArithExpression::Difference(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x - y));
                }
            }
            ArithExpression::Addition(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x + y));
                }
            }
            ArithExpression::Multiplication(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x * y));
                }
            }
            ArithExpression::Division(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x / y));
                }
            }
            ArithExpression::Modulo(l, r) => {
                changed = l.simplify_helper() | r.simplify_helper();
                if let (ArithExpression::Int(x), ArithExpression::Int(y)) = (l.as_ref(), r.as_ref())
                {
                    value = Some(ArithExpression::Int(x % y));
                }
            }
            ArithExpression::Clock(_) => {}
            ArithExpression::VarName(_) => {}
            ArithExpression::Int(_) => {}
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
            ArithExpression::Difference(a1, a2)
            | ArithExpression::Addition(a1, a2)
            | ArithExpression::Multiplication(a1, a2)
            | ArithExpression::Division(a1, a2)
            | ArithExpression::Modulo(a1, a2) => a1.has_var_name(name) || a2.has_var_name(name),
            ArithExpression::Clock(_) | ArithExpression::Int(_) => false,
            ArithExpression::VarName(n) => name == n,
        }
    }

    pub fn get_var_names(&self) -> Vec<String> {
        let mut vec = vec![];
        self.get_var_names_rec(&mut vec);
        vec
    }

    /// Finds the clocks used in the expression and put them into result_clocks.
    pub fn get_var_names_rec(&self, result_clocks: &mut Vec<String>) {
        match self {
            ArithExpression::Difference(ref left, ref right)
            | ArithExpression::Addition(ref left, ref right)
            | ArithExpression::Multiplication(ref left, ref right)
            | ArithExpression::Division(ref left, ref right)
            | ArithExpression::Modulo(ref left, ref right) => {
                left.get_var_names_rec(result_clocks);
                right.get_var_names_rec(result_clocks);
            }
            ArithExpression::Clock(_) => (),
            ArithExpression::VarName(ref name) => result_clocks.push(name.clone()),
            ArithExpression::Int(_) => (),
        }
    }

    /// Replaces all occurrences of `ArithExpression::VarName(old)` with `new`

    /// # Arguments
    /// `old`: The `var name` to be replaced

    /// `new`: The new var name
    pub fn replace_var_name(&mut self, old: &String, new: &String) {
        match self {
            ArithExpression::Difference(a1, a2)
            | ArithExpression::Addition(a1, a2)
            | ArithExpression::Multiplication(a1, a2)
            | ArithExpression::Division(a1, a2)
            | ArithExpression::Modulo(a1, a2) => {
                a1.replace_var_name(old, new);
                a2.replace_var_name(old, new);
            }
            ArithExpression::Clock(_) | ArithExpression::Int(_) => (),
            ArithExpression::VarName(name) => {
                if *name == *old {
                    *name = new.to_string();
                }
            }
        }
    }

    pub fn clock_var_count(&self) -> u32 {
        match self {
            ArithExpression::Clock(_) => 1,
            ArithExpression::VarName(_) => 1,
            ArithExpression::Difference(l, r)
            | ArithExpression::Addition(l, r)
            | ArithExpression::Multiplication(l, r)
            | ArithExpression::Division(l, r)
            | ArithExpression::Modulo(l, r) => l.clock_var_count() + r.clock_var_count(),
            _ => 0,
        }
    }

    pub fn a_par(inner: ArithExpression) -> ArithExpression {
        inner
    }

    pub fn a_dif(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if let ArithExpression::Int(0) = right {
            return left;
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i - j);
            }
        }

        ArithExpression::Difference(Box::new(left), Box::new(right))
    }

    pub fn a_add(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if let ArithExpression::Int(0) = right {
            return left;
        } else if let ArithExpression::Int(0) = left {
            return right;
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i + j);
            }
        }

        ArithExpression::Addition(Box::new(left), Box::new(right))
    }

    pub fn a_mul(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if right == ArithExpression::Int(0) || left == ArithExpression::Int(0) {
            return ArithExpression::Int(0);
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i * j);
            }
        }

        ArithExpression::Multiplication(Box::new(left), Box::new(right))
    }

    pub fn a_div(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if right == ArithExpression::Int(0) || left == ArithExpression::Int(0) {
            return ArithExpression::Int(0);
        }

        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i / j);
            }
        }

        ArithExpression::Division(Box::new(left), Box::new(right))
    }

    pub fn a_mod(left: ArithExpression, right: ArithExpression) -> ArithExpression {
        if let ArithExpression::Int(i) = left {
            if let ArithExpression::Int(j) = right {
                return ArithExpression::Int(i % j);
            }
        }

        ArithExpression::Modulo(Box::new(left), Box::new(right))
    }
}

impl Display for ArithExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithExpression::Clock(id) => {
                write!(f, "{}", format!("c:{}", id).magenta())?;
            }
            ArithExpression::VarName(name) => {
                write!(f, "{}", name.to_string().blue())?;
            }
            ArithExpression::Int(num) => {
                write!(f, "{}", num)?;
            }
            ArithExpression::Difference(left, right) => {
                write!(f, "{}-{}", left, right)?;
            }
            ArithExpression::Addition(left, right) => {
                write!(f, "{}+{}", left, right)?;
            }
            ArithExpression::Multiplication(left, right) => {
                write!(f, "{}*{}", left, right)?;
            }
            ArithExpression::Division(left, right) => {
                write!(f, "{}/{}", left, right)?;
            }
            ArithExpression::Modulo(left, right) => {
                write!(f, "{}%{}", left, right)?;
            }
        }
        Ok(())
    }
}

/// Variants represent whether the clock was on the rhs of an expression or not (true == right)
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum Operation {
    Dif(bool),
    Add(bool),
    Mul(bool),
    Div(bool),
    Mod(bool),
    None,
}

pub struct Clock {
    pub value: ClockIndex,
    pub negated: bool,
}

impl Clock {
    pub fn new(v: ClockIndex, n: bool) -> Clock {
        Clock {
            value: v,
            negated: n,
        }
    }

    pub fn neg(v: ClockIndex) -> Clock {
        Clock {
            value: v,
            negated: true,
        }
    }

    pub fn pos(v: ClockIndex) -> Clock {
        Clock {
            value: v,
            negated: false,
        }
    }

    pub fn invert(&mut self) {
        self.negated = !self.negated;
    }
}

#[cfg(test)]
mod tests {
    use crate::data_reader::parse_edge::parse_guard;
    use crate::JsonProjectLoader;
    use test_case::test_case;
    const PATH: &str = "samples/json/PopulateClocks";

    #[test_case("5",        vec![],                                  true  ; "No clocks")]
    #[test_case("5+x",      vec!["x".to_string()],                   true  ; "A single clock with addition")]
    #[test_case("y-9",      vec!["y".to_string()],                   true  ; "A single clock with difference")]
    #[test_case("zz*6/z",   vec!["zz".to_string(), "z".to_string()], true  ; "2 clocks with similar names")]
    #[test_case("5%alpha",  vec!["alpha".to_string()],               true  ; "Longer clock names")]
    #[test_case("5%alpha",  vec!["x".to_string()],                   false ; "One clock, should fail")]
    fn test_get_clocks_arith(expression: &str, expected: Vec<String>, verdict: bool) {
        // We test arith expressions by converting them into boolean expressions and then running the bool test below.
        let mut expression = expression.to_owned();
        expression.push_str("<0"); // Arrange
                                   // parse_guard is used to parse a boolean expression, as guards are just boolean expressions.
        match parse_guard(&expression) {
            Ok(input_expr) => {
                // Act
                let results: Vec<String> = input_expr.get_var_names();
                // Assert
                assert_eq!((expected == results), verdict);
            }
            Err(err) => {
                panic!("Test failed: {}", err);
            }
        };
    }

    #[test_case("Updates1", vec![0,0] ; "Two updates set to 0")]
    #[test_case("Updates2", vec![3,4] ; "Two updates set to none-zero")]
    #[test_case("Updates3", vec![5,7] ; "Updates with arithmetic expressions")]
    fn test_get_evaluated_int(comp_name: &str, expected: Vec<i32>) {
        let mut project_loader = JsonProjectLoader::new_loader(PATH, crate::tests::TEST_SETTINGS);
        project_loader.get_settings_mut().disable_clock_reduction = true;
        let test_comp = project_loader.get_component(comp_name).unwrap().clone();

        let mut clock_values: Vec<i32> = Vec::new();

        if let Some(edge) = test_comp.edges.iter().find(|edge| edge.id == "E12") {
            if let Some(updates) = &edge.update {
                for update in updates {
                    match update.expression.get_evaluated_int() {
                        Err(_) => panic!("no value evaluated on update"),
                        Ok(value) => clock_values.push(value),
                    };
                }
            }
        }

        assert_eq!(clock_values, expected);
    }
}
