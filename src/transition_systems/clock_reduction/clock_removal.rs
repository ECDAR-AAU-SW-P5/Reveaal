use crate::transition_systems::LocationTree;
use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;
use edbm::zones::OwnedFederation;
use std::rc::Rc;

pub fn remove_clocks_from_federation(
    mut federation: OwnedFederation,
    clocks: &[ClockIndex],
    shrink_expand_src: &[bool],
    shrink_expand_dst: &[bool],
) -> OwnedFederation {
    'outer: for conj in federation.minimal_constraints().conjunctions {
        for constraint in conj.constraints {
            if clocks.contains(&constraint.i) || clocks.contains(&constraint.j) {
                federation = federation.set_empty();
                break 'outer;
            }
        }
    }
    federation
        .shrink_expand(&shrink_expand_src.to_vec(), &shrink_expand_dst.to_vec())
        .0
}
pub fn remove_clocks_from_location(
    loc: &mut Rc<LocationTree>,
    clocks: &[ClockIndex],
    shrink_expand_src: &[bool],
    shrink_expand_dst: &[bool],
) {
    // Remove from Invariant
    if let Some(federation) = &loc.invariant {
        let mut new_loc = loc.as_ref().clone();
        new_loc.invariant = Some(remove_clocks_from_federation(
            federation.clone(),
            clocks,
            shrink_expand_src,
            shrink_expand_dst,
        ));
        *loc = Rc::new(new_loc);
    }
}
pub fn rebuild_bounds(old_bounds: &Bounds, dim: ClockIndex, clocks: &[ClockIndex]) -> Bounds {
    let mut b = Bounds::new(dim - clocks.len());
    let mut j = 0;
    for i in 0..dim {
        if clocks.contains(&i) {
            continue;
        }
        match old_bounds.get_upper(i) {
            None => {}
            Some(bound) => {
                if bound > 0 {
                    b.add_upper(j, bound);
                }
            }
        }
        match old_bounds.get_lower(i) {
            None => {}
            Some(bound) => {
                if bound > 0 {
                    b.add_lower(j, bound);
                }
            }
        }
        j += 1;
    }
    b
}
