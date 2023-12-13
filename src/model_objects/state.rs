use std::rc::Rc;

use crate::transition_systems::{LocationTree, TransitionSystem};
use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;
use edbm::zones::OwnedFederation;

/// State is a struct used for initial verification of consistency, and determinism as a state that also holds a dbm
/// This is done as the type used in refinement state pair assumes two sides of an operation
// This should probably be refactored as it causes unnecessary confusion
#[derive(Clone, Debug)]
pub struct State {
    pub decorated_locations: Rc<LocationTree>,
    zone: Rc<OwnedFederation>,
}

impl State {
    pub fn new<Z: Into<Rc<OwnedFederation>>>(
        decorated_locations: Rc<LocationTree>,
        zone: Z,
    ) -> Self {
        State {
            decorated_locations,
            zone: zone.into(),
        }
    }

    pub fn is_contained_in_list(&self, list: &[State]) -> bool {
        list.iter().any(|s| self.is_subset_of(s))
    }

    pub fn from_location(
        decorated_locations: Rc<LocationTree>,
        dimensions: ClockIndex,
    ) -> Option<Self> {
        let mut fed = OwnedFederation::init(dimensions);

        fed = decorated_locations.apply_invariants(fed);
        if fed.is_empty() {
            return None;
        }

        Some(State {
            decorated_locations,
            zone: Rc::new(fed),
        })
    }

    pub fn apply_invariants(&mut self) {
        let fed = self.clone_zone();
        let new_fed = self.decorated_locations.apply_invariants(fed);

        self.set_zone(new_fed);
    }

    pub fn clone_zone(&self) -> OwnedFederation {
        self.zone.as_ref().clone()
    }

    pub fn ref_zone(&self) -> &OwnedFederation {
        self.zone.as_ref()
    }

    pub fn get_zone(&self) -> Rc<OwnedFederation> {
        Rc::clone(&self.zone)
    }

    pub(crate) fn set_zone<Z: Into<Rc<OwnedFederation>>>(&mut self, zone: Z) {
        self.zone = zone.into();
    }

    pub fn update_zone(&mut self, update: impl FnOnce(OwnedFederation) -> OwnedFederation) {
        let fed = self.clone_zone();
        let new_fed = update(fed);
        self.set_zone(new_fed);
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self.decorated_locations != other.decorated_locations {
            return false;
        }

        self.ref_zone().subset_eq(other.ref_zone())
    }

    pub fn get_location(&self) -> &LocationTree {
        &self.decorated_locations
    }

    pub fn extrapolate_max_bounds(&mut self, system: &dyn TransitionSystem) {
        let bounds = system.get_local_max_bounds(self.decorated_locations.as_ref());
        self.update_zone(|zone| zone.extrapolate_max_bounds(&bounds))
    }

    pub fn extrapolate_max_bounds_with_extra_bounds(
        &mut self,
        system: &dyn TransitionSystem,
        extra_bounds: &Bounds,
    ) {
        let mut bounds = system.get_local_max_bounds(self.decorated_locations.as_ref());
        bounds.add_bounds(extra_bounds);
        self.update_zone(|zone| zone.extrapolate_max_bounds(&bounds))
    }
}
