use edbm::util::constraints::{ClockIndex, Conjunction, Constraint, Disjunction, Inequality};
use edbm::zones::{Federation, OwnedFederation};

pub fn remove_clock_from_federation(
    federation: &OwnedFederation,
    remove_clock: &ClockIndex,
    replacing_clock: Option<&ClockIndex>,
) -> OwnedFederation {
    assert_ne!(Some(remove_clock), replacing_clock);

    let mut _replacing_clock: Option<&ClockIndex>;
    if replacing_clock.is_some_and(|replacing_clock_value| replacing_clock_value > remove_clock) {
        _replacing_clock = Some(&(replacing_clock.unwrap() - 1));
    } else {
        _replacing_clock = replacing_clock.clone();
    }

    let old_disjunction = federation.minimal_constraints();

    let new_disjunction = Disjunction::new(
        old_disjunction
            .iter()
            // map to new constraints without clock_index and filter by empty conjunctions
            .filter_map(|conjunction| {
                rebuild_conjunction(conjunction, remove_clock, replacing_clock)
            })
            .collect(),
    );
    Federation::from_disjunction(&new_disjunction, federation.dim() - 1)
}

fn rebuild_conjunction(
    conjunction: &Conjunction,
    remove_clock: &ClockIndex,
    replacing_clock: Option<&ClockIndex>,
) -> Option<Conjunction> {
    let mut new_constraints: Vec<Constraint> = Vec::new();
    for constraint in conjunction
        .iter()
        // Clone constraint
        .filter_map(|constraint| {
            remove_or_replace_constraint(constraint, remove_clock, replacing_clock)
        })
    {
        // only add the tightest constraints
        //debug_assert!(self[(i, j)] > constraint && constraint.as_negated() < self[(j, i)]);

        // self[(i, j)] > constraint
        match new_constraints.iter().position(|new_constraint| {
            new_constraint.i == constraint.i && new_constraint.j == constraint.j
        }) {
            None => {
                new_constraints.push(constraint);
            }
            Some(constraint_index) => {
                let new_constraint = new_constraints.get(constraint_index).unwrap();

                if new_constraint.ineq().bound() < constraint.ineq().bound() {
                    // constraint.as_negated() < self[(j, i)]
                    match new_constraints.iter().position(|new_constraint| {
                        new_constraint.i == constraint.j && new_constraint.j == constraint.i
                    }) {
                        None => {
                            new_constraints.swap_remove(constraint_index);
                            new_constraints.push(constraint);
                        }
                        Some(reverse_index) => {
                            let reverse_constraint = new_constraints.get(reverse_index).unwrap();

                            if reverse_constraint.ineq().negated_bound().bound()
                                > constraint.ineq().negated_bound().bound()
                            {
                                new_constraints.swap_remove(constraint_index);
                                new_constraints.push(constraint);
                            }
                        }
                    }
                }
            }
        }
    }
    if new_constraints.len() == 0 {
        // Remove conjunction constraints using only global clock
        return None;
    }
    let new_conjunction = Conjunction::new(new_constraints);
    Some(new_conjunction)
}

//helper for remove_or_replace_constraint
fn create_constraint(
    i: ClockIndex,
    j: ClockIndex,
    inequality: Inequality,
    clock_index: ClockIndex,
) -> Constraint {
    // Redraw constraints in case their clocks are higher than the clock that is to be removed
    Constraint::new(
        if i > clock_index { i - 1 } else { i },
        if j > clock_index { j - 1 } else { j },
        inequality.into(), //similar to the DBM
    )
}

// Remove/Replace constraint if constraint contains clock_index
// clock can be either i, j or neither
fn remove_or_replace_constraint(
    constraint: &Constraint,
    remove_clock: &ClockIndex,
    replacing_clock: Option<&ClockIndex>,
) -> Option<Constraint> {
    match replacing_clock {
        // remove constraint if there's no replacing clock and either side contains the clock to be removed
        None => {
            if constraint.i == *remove_clock || constraint.j == *remove_clock {
                return None;
            }
        }
        // Replace either left or right side if either side contains the clock to be removed
        Some(new_clock) => {
            if constraint.i == *remove_clock {
                if &constraint.j == new_clock {
                    return None;
                }
                return Some(create_constraint(
                    *new_clock,
                    constraint.j,
                    constraint.ineq(),
                    *remove_clock,
                ));
            } else if constraint.j == *remove_clock {
                if &constraint.i == new_clock {
                    return None;
                }
                return Some(create_constraint(
                    constraint.i,
                    *new_clock,
                    constraint.ineq(),
                    *remove_clock,
                ));
            }
        }
    }
    // If neither side contains the clock rebuild the constraint
    return Some(create_constraint(
        constraint.i,
        constraint.j,
        constraint.ineq(),
        *remove_clock,
    ));
}

#[cfg(test)]
mod a {
    use crate::transition_systems::clock_reduction::clock_removal::remove_clock_from_federation;
    use edbm::util::constraints::Inequality;
    use edbm::zones::OwnedFederation;

    /// This is to make sure that the Federation and it's underlying DBM works as expected. It is recommended
    /// to understand this test and [build_example_dbm] before looking at the others
    #[test]
    fn example_dbm() {
        // Arrange

        // Act
        let fed = build_example_dbm();

        let disjunction = fed.minimal_constraints();
        let conjunction = disjunction.conjunctions.first().unwrap();
        let bounds = fed.get_bounds();

        // Assert
        assert_eq!(conjunction.constraints.len(), 6); // all 6 constraints should exist

        assert_eq!(bounds.get_lower(0).unwrap(), 0); // lower equal to [clock, 0]
        assert_eq!(bounds.get_upper(0).unwrap(), 0); // upper equal to [0, clock]
        assert_eq!(bounds.get_lower(1).unwrap(), 2); // this means that lower and
        assert_eq!(bounds.get_upper(1).unwrap(), 6); // upper are the left most
        assert_eq!(bounds.get_lower(2).unwrap(), 1); // column and the top
        assert_eq!(bounds.get_upper(2).unwrap(), 5); // horizontal row

        // the last 2 values
        for constraint in &conjunction.constraints {
            match constraint.i {
                1 => match constraint.j {
                    2 => assert_eq!(constraint.ineq().bound(), 3),
                    _ => (),
                },
                2 => match constraint.j {
                    1 => assert_eq!(constraint.ineq().bound(), 1),
                    _ => (),
                },
                _ => (),
            }
        }
    }

    #[test]
    fn remove_clock() {
        // Arrange
        let mut fed = build_example_dbm();

        // Act
        fed = remove_clock_from_federation(&fed, &1, None);
        // equivalent of removing the middle column and middle row. everything else get shifted inwards
        // 0 x 0
        // x x x
        // 0 x 0
        // into
        // 0 0
        // 0 0
        // and because the diagonal constraints don't exist there should only be 2 constraints left,
        // an upper and lower bound of clock x_2, because of the shift, although x_2 is now x_1 instead

        // Assert
        let disjunction = fed.minimal_constraints();
        let conjunction = disjunction.conjunctions.first().unwrap();

        assert_eq!(fed.dim(), 2);
        assert_eq!(conjunction.constraints.len(), 2);

        for constraint in &conjunction.constraints {
            match constraint.i {
                0 => {
                    match constraint.j {
                        1 => {
                            assert_eq!(constraint.ineq().bound(), -1); // top right
                        }
                        _ => {}
                    }
                }
                1 => {
                    match constraint.j {
                        0 => {
                            assert_eq!(constraint.ineq().bound(), 5); //bottom left
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn replace_clock() {
        // Arrange
        let mut fed = build_example_dbm();

        // Act
        fed = remove_clock_from_federation(&fed, &1, Some(&2));
        // equivalent of removing the middle column and middle row. everything else get shifted inwards
        // 0 x 0
        // x x x
        // 0 x 0
        // into
        // 0 0
        // 0 0
        // however this time, the previous constraints are reused
        // 0 y w
        // z x x
        // v x 0
        // into
        // 0 (y/w)
        // (z/v) 0
        // depending on whether y or w is the tightest constraint and z or v is the tightest constraint

        // Assert
        let disjunction = fed.minimal_constraints();
        let conjunction = disjunction.conjunctions.first().unwrap();

        assert_eq!(fed.dim(), 2);
        assert_eq!(conjunction.constraints.len(), 2);
        for constraint in &conjunction.constraints {
            match constraint.i {
                0 => {
                    match constraint.j {
                        1 => {
                            //-2 vs -1
                            assert_eq!(constraint.ineq().bound(), -1);
                        }
                        _ => {}
                    }
                }
                1 => {
                    match constraint.j {
                        0 => {
                            //6 vs 5
                            assert_eq!(constraint.ineq().bound(), 6);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    /// x_0 = x, x_1 = y, x_2 = z
    ///
    /// x-y<=0,  x-y<=-2, x-z<=-1
    ///
    /// y-x<=6,  y-y<=0,  y-z<=3
    ///
    /// z-x<=5,  z-y<=1,  z-z<=0
    ///
    /// https://homes.cs.aau.dk/~adavid/UDBM/materials/UDBMLib.pdf slide 4
    fn build_example_dbm() -> OwnedFederation {
        OwnedFederation::universe(3)
            .constrain(0, 1, Inequality::LE(-2))
            .constrain(0, 2, Inequality::LE(-1))
            .constrain(1, 0, Inequality::LE(6))
            .constrain(1, 2, Inequality::LE(3))
            .constrain(2, 0, Inequality::LE(5))
            .constrain(2, 1, Inequality::LE(1))
    }
}
