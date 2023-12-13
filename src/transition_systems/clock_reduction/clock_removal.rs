use crate::transition_systems::LocationTree;
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
