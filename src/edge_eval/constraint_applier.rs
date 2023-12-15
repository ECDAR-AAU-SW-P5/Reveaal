use edbm::util::constraints::{ClockIndex, Inequality};
use edbm::zones::OwnedFederation;

use crate::model_objects::expressions::{ArithExpression, BoolExpression, Clock};
use crate::model_objects::Declarations;

pub fn apply_constraints_to_state(
    guard: &BoolExpression,
    decls: &Declarations,
    fed: OwnedFederation,
) -> Result<OwnedFederation, String> {
    apply_constraints_to_state_helper(guard, decls, fed)
}

fn apply_constraints_to_state_helper(
    guard: &BoolExpression,
    decls: &Declarations,
    fed: OwnedFederation,
) -> Result<OwnedFederation, String> {
    if fed.is_empty() {
        return Ok(fed);
    }
    use Inequality::*;
    match guard {
        BoolExpression::AndOp(left, right) => {
            let fed = apply_constraints_to_state_helper(left, decls, fed)?;
            apply_constraints_to_state_helper(right, decls, fed)
        }
        BoolExpression::OrOp(left, right) => {
            let clone = fed.clone();
            let fed1 = apply_constraints_to_state_helper(left, decls, fed)?;
            let fed2 = apply_constraints_to_state_helper(right, decls, clone)?;
            Ok(fed1 + fed2)
        }
        BoolExpression::LessEQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls)?;
            // i-j<=c
            Ok(fed.constrain(i, j, LE(c)))
        }
        BoolExpression::GreatEQ(left, right) => {
            let (i, j, c) = get_indices(right, left, decls)?;
            // j-i <= -c -> c <= i-j
            Ok(fed.constrain(i, j, LE(c)))
        }
        BoolExpression::EQ(left, right) => {
            let (i, j, c) = get_indices(left, right, decls)?;
            // i-j <= c && j-i <= -c -> c <= i-j

            // TODO: maybe use fed.constrain_many(...)
            Ok(fed.constrain(i, j, LE(c)).constrain(j, i, LE(-c)))
        }
        BoolExpression::LessT(left, right) => {
            let (i, j, c) = get_indices(left, right, decls)?;
            // i-j < c
            Ok(fed.constrain(i, j, LS(c)))
        }
        BoolExpression::GreatT(left, right) => {
            let (i, j, c) = get_indices(right, left, decls)?;
            // j-i < -c -> c < i-j
            Ok(fed.constrain(i, j, LS(c)))
        }
        BoolExpression::Bool(val) => {
            if !*val {
                return Ok(fed.set_empty());
            }
            Ok(fed)
        }
    }
}

/// Assumes that the constraint is of the form left <?= right
fn get_indices(
    left: &ArithExpression,
    right: &ArithExpression,
    d: &Declarations,
) -> Result<(ClockIndex, ClockIndex, i32), String> {
    let left = &(replace_vars(left, d)?.simplify())?;
    let right = &(replace_vars(right, d)?.simplify())?;
    let (clocks_left, clocks_right) = (left.clock_var_count(), right.clock_var_count());

    if clocks_left + clocks_right == 0 {
        return Err(String::from(
            "expressions must use clocks to get indices, this should be unreachable",
        ));
    } else if clocks_left + clocks_right > 2 {
        return Err(String::from("Too many clocks"));
    }

    let (left_const, right_const) = (get_const(left, d), get_const(right, d));
    let constant = right_const - left_const;

    let result: Result<(ClockIndex, ClockIndex, i32), String> = match (clocks_left, clocks_right) {
        (1, 1) => {
            let (c1, c2) = (
                get_clock_val(left, 1, false)?.0,
                get_clock_val(right, 1, false)?.0,
            );
            combine_clocks(c1, c2, constant, true)
        }
        (2, 0) => {
            let (c1, c2) = get_clock_val(left, 2, false)?;
            combine_clocks(c1, c2.unwrap(), constant, false)
        }
        (0, 2) => {
            let (mut c1, c2) = get_clock_val(right, 2, false)?;
            let mut c2 = c2.unwrap();
            c1.invert();
            c2.invert();
            combine_clocks(c1, c2, constant, false)
        }
        (1, 0) => {
            let c = get_clock_val(left, 1, false)?.0;
            if c.negated {
                Ok((0, c.value, constant))
            } else {
                Ok((c.value, 0, constant))
            }
        }
        (0, 1) => {
            let c = get_clock_val(right, 1, false)?.0;
            if !c.negated {
                Ok((0, c.value, constant))
            } else {
                Ok((c.value, 0, constant))
            }
        }
        _ => unreachable!(),
    };
    result
}

fn replace_vars(expr: &ArithExpression, decls: &Declarations) -> Result<ArithExpression, String> {
    //let mut out = expr.clone();
    match expr {
        ArithExpression::Difference(l, r) => Ok(ArithExpression::a_dif(
            replace_vars(l, decls)?,
            replace_vars(r, decls)?,
        )),
        ArithExpression::Addition(l, r) => Ok(ArithExpression::a_add(
            replace_vars(l, decls)?,
            replace_vars(r, decls)?,
        )),
        ArithExpression::Multiplication(l, r) => Ok(ArithExpression::a_mul(
            replace_vars(l, decls)?,
            replace_vars(r, decls)?,
        )),
        ArithExpression::Division(l, r) => Ok(ArithExpression::a_div(
            replace_vars(l, decls)?,
            replace_vars(r, decls)?,
        )),
        ArithExpression::Modulo(l, r) => Ok(ArithExpression::a_mod(
            replace_vars(l, decls)?,
            replace_vars(r, decls)?,
        )),
        ArithExpression::Clock(x) => Ok(ArithExpression::Clock(*x)),
        ArithExpression::VarName(name) => {
            if let Some(x) = decls.clocks.get(name.as_str()).copied() {
                Ok(ArithExpression::Clock(x))
            } else if let Some(x) = decls.ints.get(name.as_str()).copied() {
                Ok(ArithExpression::Int(x))
            } else {
                Err(name.to_string())
            }
        }
        ArithExpression::Int(i) => Ok(ArithExpression::Int(*i)),
    }
}

fn get_const(expr: &ArithExpression, decls: &Declarations) -> i32 {
    match expr {
        ArithExpression::Int(x) => *x,
        ArithExpression::Clock(_) => 0,
        ArithExpression::VarName(name) => decls.ints.get(name).copied().unwrap_or(0),
        ArithExpression::Difference(l, r) => get_const(l, decls) - get_const(r, decls),
        ArithExpression::Addition(l, r) => get_const(l, decls) + get_const(r, decls),
        ArithExpression::Multiplication(l, r) => get_const(l, decls) * get_const(r, decls),
        ArithExpression::Division(l, r) => get_const(l, decls) / get_const(r, decls),
        ArithExpression::Modulo(l, r) => get_const(l, decls) % get_const(r, decls),
    }
}

fn combine_clocks(
    c1: Clock,
    c2: Clock,
    constant: i32,
    same_sign: bool,
) -> Result<(ClockIndex, ClockIndex, i32), String> {
    if (same_sign && c1.negated != c2.negated) || (!same_sign && c1.negated == c2.negated) {
        Err(String::from("Same sign"))
    } else if !c1.negated {
        Ok((c1.value, c2.value, constant))
    } else {
        Ok((c2.value, c1.value, constant))
    }
}

fn get_clock_val(
    expression: &ArithExpression,
    count: i32,
    negated: bool,
) -> Result<(Clock, Option<Clock>), String> {
    let mut nxt_expr: Option<&ArithExpression> = None;
    let mut nxt_negated = false;
    let val = match expression {
        ArithExpression::Difference(l, r) => {
            if let ArithExpression::Clock(x) = **l {
                nxt_expr = Some(&**r);
                nxt_negated = true;
                Clock::pos(x)
            } else if let ArithExpression::Clock(x) = **r {
                nxt_expr = Some(&**l);
                Clock::neg(x)
            } else {
                return Err(String::from("No Clocks"));
            }
        }
        ArithExpression::Addition(l, r) => {
            if let ArithExpression::Clock(x) = **l {
                nxt_expr = Some(&**r);
                Clock::pos(x)
            } else if let ArithExpression::Clock(x) = **r {
                nxt_expr = Some(&**l);
                Clock::pos(x)
            } else {
                return Err(String::from("No Clocks"));
            }
        }
        ArithExpression::Multiplication(_, _)
        | ArithExpression::Division(_, _)
        | ArithExpression::Modulo(_, _) => {
            return Err("Multiplication with clock is illegal".to_string());
        }
        ArithExpression::Clock(x) => Clock::new(*x, negated),
        _ => return Err(String::from("No Clocks")),
    };

    if count > 1 {
        Ok((
            val,
            Some(get_clock_val(nxt_expr.unwrap(), count - 1, nxt_negated)?.0),
        ))
    } else {
        Ok((val, None))
    }
}

#[cfg(test)]
mod test {
    use super::get_indices;
    use crate::model_objects::expressions::ArithExpression;
    use crate::model_objects::Declarations;
    use std::collections::HashMap;

    #[test]
    fn test_get_indices_int_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        let left = ArithExpression::Int(3);
        let right = ArithExpression::Clock(1);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((0, 1, -3)));
    }

    #[test]
    fn test_get_indices_clock_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        let left = ArithExpression::Clock(1);
        let right = ArithExpression::Int(3);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 0, 3)));
    }

    #[test]
    fn test_get_indices_clock_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        // i-j <= 0 -> i can at most be the value of j
        let left = ArithExpression::Clock(1);
        let right = ArithExpression::Clock(2);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 2, 0)));
    }

    #[test]
    fn test_get_indices_diff_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        // i-j < c -> c1-c2 < 3
        let left = ArithExpression::a_dif(ArithExpression::Clock(1), ArithExpression::Clock(2));
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 2, 3)));

        let left = ArithExpression::a_dif(ArithExpression::Clock(1), ArithExpression::Int(2));
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), Some((1, 0, 5)));

        let left = ArithExpression::a_dif(ArithExpression::Int(1), ArithExpression::Clock(2));
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 2, 2)));
    }

    #[test]
    fn test_get_indices_int_diff() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
        let left = ArithExpression::Int(3);
        let right = ArithExpression::a_dif(ArithExpression::Clock(1), ArithExpression::Clock(2));

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 1, -3)));
    }

    #[test]
    fn test_get_indices_add_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Int(4);

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_clock_diff_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Clock(1);
        let right = ArithExpression::a_dif(ArithExpression::Clock(2), ArithExpression::Int(3));
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 2, -3)));

        let left = ArithExpression::a_dif(ArithExpression::Clock(2), ArithExpression::Int(3));
        let right = ArithExpression::Clock(1);
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 1, 3)));

        let left = ArithExpression::a_dif(ArithExpression::Int(2), ArithExpression::Clock(3));
        let right = ArithExpression::Clock(1);
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_clock_add_clock() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 3, 2)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Int(1)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 3, 3)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Int(3)),
            Box::new(ArithExpression::Clock(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 4, 1)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Int(1)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Int(3)),
            Box::new(ArithExpression::Clock(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((2, 4, 2)));
    }

    #[test]
    fn test_get_indices_clock_int_diff() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        // i-j < c -> c1-c2 > 3 -> c2-c1 < -3
        let left = ArithExpression::a_dif(ArithExpression::Clock(1), ArithExpression::Int(2));
        let right = ArithExpression::a_dif(ArithExpression::Clock(3), ArithExpression::Int(4));
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 3, -2)));

        let left = ArithExpression::a_dif(ArithExpression::Int(1), ArithExpression::Clock(2));
        let right = ArithExpression::a_dif(ArithExpression::Int(3), ArithExpression::Clock(4));
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((4, 2, 2)));
    }

    #[test]
    fn test_get_indices_clock_add_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Clock(1);
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        assert_eq!(get_indices(&left, &right, &decl), Ok((1, 3, 4)));

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(3)),
            Box::new(ArithExpression::Int(4)),
        );
        let right = ArithExpression::Clock(1);
        assert_eq!(get_indices(&left, &right, &decl), Ok((3, 1, -4)));
    }

    #[test]
    fn test_get_indices_int_add() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Int(3);
        let right = ArithExpression::Addition(
            Box::new(ArithExpression::Clock(1)),
            Box::new(ArithExpression::Clock(2)),
        );

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_high_operators() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Multiplication(
            Box::new(ArithExpression::Clock(2)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Multiplication(
            Box::new(ArithExpression::Int(3)),
            Box::new(ArithExpression::Clock(2)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Division(
            Box::new(ArithExpression::Clock(2)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Modulo(
            Box::new(ArithExpression::Clock(2)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Int(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::Multiplication(
            Box::new(ArithExpression::Int(4)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 10, -12)));

        let left = ArithExpression::Division(
            Box::new(ArithExpression::Int(4)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 10, -2)));

        let left = ArithExpression::Modulo(
            Box::new(ArithExpression::Int(4)),
            Box::new(ArithExpression::Int(3)),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((0, 10, -1)));
    }

    #[test]
    fn test_get_indices_many_clocks() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::a_dif(ArithExpression::Clock(1), ArithExpression::Clock(2));
        let right = Box::new(ArithExpression::Clock(3));

        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);
    }

    #[test]
    fn test_get_indices_int_int() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::Int(1);
        let right = ArithExpression::Int(2);
        //Testing: left < right
        assert_eq!(
            get_indices(&left, &right, &decl),
            Err(
                "expressions must use clocks to get indices, this should be unreachable"
                    .to_string()
            )
        );

        let left = ArithExpression::Addition(
            Box::new(ArithExpression::Int(1)),
            Box::new(ArithExpression::Int(2)),
        );
        let right = ArithExpression::Int(3);
        //Testing: left < right
        assert_eq!(
            get_indices(&left, &right, &decl),
            Err(
                "expressions must use clocks to get indices, this should be unreachable"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_get_indices_big_expr() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::a_dif(
            // = 4
            ArithExpression::Int(10),
            ArithExpression::a_dif(
                ArithExpression::Int(9),
                ArithExpression::a_dif(
                    ArithExpression::Int(8),
                    ArithExpression::a_dif(
                        ArithExpression::Int(7),
                        ArithExpression::a_dif(
                            ArithExpression::Clock(6),
                            ArithExpression::a_dif(
                                ArithExpression::Int(5),
                                ArithExpression::a_dif(
                                    ArithExpression::Int(4),
                                    ArithExpression::a_dif(
                                        ArithExpression::Int(3),
                                        ArithExpression::Int(2),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        );
        let right = ArithExpression::Int(2);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((6, 0, 6)));
    }

    #[test]
    fn test_get_indices_mix_expr() {
        let decl = Declarations {
            clocks: HashMap::new(),
            ints: HashMap::new(),
        };
        let left = ArithExpression::a_dif(
            ArithExpression::Multiplication(
                Box::new(ArithExpression::Clock(3)),
                Box::new(ArithExpression::Int(3)),
            ),
            ArithExpression::Int(10),
        );
        let right = ArithExpression::Clock(10);
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl).ok(), None);

        let left = ArithExpression::a_dif(
            ArithExpression::Multiplication(
                Box::new(ArithExpression::Int(3)),
                Box::new(ArithExpression::Int(3)),
            ),
            ArithExpression::Clock(10),
        );
        let right = ArithExpression::a_dif(ArithExpression::Int(10), ArithExpression::Clock(10));
        //Testing: left < right
        assert_eq!(get_indices(&left, &right, &decl), Ok((10, 10, 1)));
    }
}
