use std::rc::Rc;

use edbm::{util::constraints::ClockIndex, zones::OwnedFederation};

use crate::edge_eval::constraint_applier::apply_constraints_to_state;
use crate::model_objects::{Declarations, Location, LocationType};

use super::LocationID;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum CompositionType {
    Conjunction,
    Composition,
    Quotient,
    Simple,
}

#[derive(Clone, Debug)]
pub struct LocationTree {
    pub id: LocationID,
    /// The invariant for the `Location`
    pub invariant: Option<OwnedFederation>,
    loc_type: LocationType,
    left: Option<Rc<LocationTree>>,
    right: Option<Rc<LocationTree>>,
}

impl PartialEq for LocationTree {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.loc_type == other.loc_type
    }
}

impl LocationTree {
    pub fn universal() -> Rc<Self> {
        Rc::new(LocationTree {
            id: LocationID::Special(crate::system::specifics::SpecialLocation::Universal),
            invariant: None,
            loc_type: LocationType::Universal,
            left: None,
            right: None,
        })
    }

    pub fn error(dim: ClockIndex, quotient_clock_index: ClockIndex) -> Rc<Self> {
        let inv = OwnedFederation::universe(dim).constrain_eq(quotient_clock_index, 0);

        Rc::new(LocationTree {
            id: LocationID::Special(crate::system::specifics::SpecialLocation::Error),
            invariant: Some(inv),
            loc_type: LocationType::Inconsistent,
            left: None,
            right: None,
        })
    }

    pub fn simple(location: &Location, decls: &Declarations, dim: ClockIndex) -> Rc<Self> {
        let invariant = if let Some(inv) = &location.invariant {
            let mut fed = OwnedFederation::universe(dim);
            fed = apply_constraints_to_state(inv, decls, fed).unwrap();
            Some(fed)
        } else {
            None
        };
        Rc::new(LocationTree {
            id: LocationID::Simple(location.id.clone()),
            invariant,
            loc_type: location.location_type,
            left: None,
            right: None,
        })
    }
    /// This method is used to a build partial [`LocationTree`].
    /// A partial [`LocationTree`] means it has a [`LocationID`] that is [`LocationID::AnyLocation`].
    /// A partial [`LocationTree`] has `None` in the field `invariant` since a partial [`LocationTree`]
    /// covers more than one location, and therefore there is no specific `invariant`
    pub fn build_any_location_tree() -> Rc<Self> {
        Rc::new(LocationTree {
            id: LocationID::AnyLocation,
            invariant: None,
            loc_type: LocationType::Any,
            left: None,
            right: None,
        })
    }

    //Merge two locations keeping the invariants seperate
    pub fn merge_as_quotient(left: Rc<Self>, right: Rc<Self>) -> Rc<Self> {
        let id = LocationID::Quotient(Box::new(left.id.clone()), Box::new(right.id.clone()));

        let loc_type = left.loc_type.combine(right.loc_type);

        Rc::new(LocationTree {
            id,
            invariant: None,
            loc_type,
            left: Some(Rc::clone(&left)),
            right: Some(Rc::clone(&right)),
        })
    }

    //Compose two locations intersecting the invariants
    pub fn compose(left: Rc<Self>, right: Rc<Self>, comp: CompositionType) -> Rc<Self> {
        let id = match comp {
            CompositionType::Conjunction => {
                LocationID::Conjunction(Box::new(left.id.clone()), Box::new(right.id.clone()))
            }
            CompositionType::Composition => {
                LocationID::Composition(Box::new(left.id.clone()), Box::new(right.id.clone()))
            }
            _ => panic!("Invalid composition type {:?}", comp),
        };

        let invariant = if let Some(inv1) = &left.invariant {
            if let Some(inv2) = &right.invariant {
                Some(inv1.clone().intersection(inv2))
            } else {
                Some(inv1.clone())
            }
        } else {
            right.invariant.clone()
        };

        let loc_type = left.loc_type.combine(right.loc_type);

        Rc::new(LocationTree {
            id,
            invariant,
            loc_type,
            left: Some(Rc::clone(&left)),
            right: Some(Rc::clone(&right)),
        })
    }

    pub fn get_invariants(&self) -> Option<&OwnedFederation> {
        self.invariant.as_ref()
    }

    pub fn apply_invariants(&self, fed: OwnedFederation) -> OwnedFederation {
        if let Some(inv) = &self.invariant {
            fed.intersection(inv)
        } else {
            fed
        }
    }

    pub fn get_left(&self) -> Rc<LocationTree> {
        Rc::clone(self.left.as_ref().unwrap())
    }

    pub fn get_right(&self) -> Rc<LocationTree> {
        Rc::clone(self.right.as_ref().unwrap())
    }

    pub fn is_initial(&self) -> bool {
        self.loc_type == LocationType::Initial
    }

    pub fn is_universal(&self) -> bool {
        self.loc_type == LocationType::Universal
    }

    pub fn is_inconsistent(&self) -> bool {
        self.loc_type == LocationType::Inconsistent
    }

    /// This function is used when you want to compare [`LocationTree`]s that can contain partial locations.
    pub fn compare_partial_locations(&self, other: Rc<LocationTree>) -> bool {
        match (&self.id, &other.id) {
            (LocationID::Composition(..), LocationID::Composition(..))
            | (LocationID::Conjunction(..), LocationID::Conjunction(..))
            | (LocationID::Quotient(..), LocationID::Quotient(..)) => {
                self.get_left().compare_partial_locations(other.get_left())
                    && self
                        .get_right()
                        .compare_partial_locations(other.get_right())
            }
            (LocationID::AnyLocation, LocationID::Simple { .. })
            | (LocationID::Simple { .. }, LocationID::AnyLocation)
            | (LocationID::AnyLocation, LocationID::AnyLocation) => true,
            (LocationID::Simple(loc_id_1), LocationID::Simple(loc_id_2)) => loc_id_1 == loc_id_2,
            (_, _) => false,
        }
    }
}
