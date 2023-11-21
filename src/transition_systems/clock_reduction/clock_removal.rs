use edbm::util::constraints::{ClockIndex, Constraint, Disjunction, Inequality};
use edbm::zones::{Federation, OwnedFederation};

pub fn remove_clock_from_federation(
    federation: &OwnedFederation,
    remove_clock: &ClockIndex,
    replacing_clock: Option<&ClockIndex>,
) -> OwnedFederation {
    assert_ne!(Some(remove_clock), replacing_clock);
    let old_disjunction = federation.minimal_constraints();
    let mut found_clock = false;

    let new_disjunction = Disjunction::new(
        old_disjunction
            .iter()
            // map to new constraints without clock_index and filter by empty conjunctions
            .filter_map(|conjunction| {
                rebuild_conjunction(conjunction, remove_clock, replacing_clock, &mut found_clock)
            })
            .collect(),
    );
    if !found_clock {
        // clock didn't exist in federation
        return federation.owned_clone();
    }
    Federation::from_disjunction(&new_disjunction, federation.dim() - 1)
}

fn rebuild_conjunction(
    conjunction: &edbm::util::constraints::Conjunction,
    remove_clock: &ClockIndex,
    replacing_clock: Option<&ClockIndex>,
    found_clock: &mut bool,
) -> Option<edbm::util::constraints::Conjunction> {
    let new_constraints: Vec<Constraint> = conjunction
        .iter()
        // Clone constraint
        .filter_map(|constraint| {
            remove_or_replace_constraint(constraint, remove_clock, replacing_clock)
        })
        .collect::<Vec<Constraint>>();
    if new_constraints.len() != conjunction.constraints.len() {
        *found_clock = true;
    }
    if new_constraints.len() == 0 {
        // Remove conjunction constraints using only global clock
        return None;
    }
    let new_conjunction = edbm::util::constraints::Conjunction::new(new_constraints);
    Some(new_conjunction)
}

//helper for remove_or_replace_constraint
fn create_constraint(
    i: ClockIndex,
    j: ClockIndex,
    inequality: Inequality,
    clock_index: ClockIndex,
) -> Constraint {
    // Redraw constraints if their clocks are higher than the clock to be removed
    if j > clock_index {
        Constraint::new(
            i - 1,
            j - 1,
            inequality.into(), //similar to the DBM
        )
    } else if i > clock_index {
        Constraint::new(i - 1, j, inequality.into())
    } else {
        Constraint::new(i, j, inequality.into())
    }
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
                return Some(create_constraint(
                    *new_clock,
                    constraint.j,
                    constraint.ineq(),
                    *remove_clock,
                ));
            } else if constraint.j == *remove_clock {
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
    use edbm::util::constraints::{Constraint, Inequality};
    use edbm::zones::OwnedFederation;

    #[test]
    fn test_rebuild() {
        let mut fed = OwnedFederation::universe(4);
        //TODO: .constrain(2, 1, Inequality::LS(5))
        // remove clock for 2 for 1,2,3
        // remove clock 2 for 1,3
        // remove clock 3 for 1,2,3
        // remove clock 1 for 1,2,3
        // move everything regarding clocks to clock module
        fed = fed
            .constrain(1, 0, Inequality::LS(5)) // It doesnt make sense to have 0 < i
            .constrain(1, 2, Inequality::LS(4))
            .constrain(2, 3, Inequality::LS(3));
        let new_fed = remove_clock_from_federation(&fed, &1, None);
        assert_eq!(new_fed.dim(), 3);
        assert_eq!(
            new_fed
                .minimal_constraints()
                .conjunctions
                .first()
                .unwrap()
                .constraints
                .len(),
            1
        );
        assert_eq!(
            new_fed
                .minimal_constraints()
                .conjunctions
                .first()
                .unwrap()
                .constraints
                .first()
                .unwrap()
                .to_string(),
            Constraint::new(1, 2, Inequality::LS(3).into()).to_string()
        )
    }
}
