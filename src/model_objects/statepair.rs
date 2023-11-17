use edbm::zones::OwnedFederation;

use crate::transition_systems::{LocationTree, TransitionSystemPtr};
use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct StatePair {
    pub locations1: LocationTree,
    pub locations2: LocationTree,
    zone: Rc<OwnedFederation>,
}

impl StatePair {
    pub fn from_locations(
        dimensions: usize,
        locations1: LocationTree,
        locations2: LocationTree,
    ) -> StatePair {
        let mut zone = OwnedFederation::init(dimensions);

        zone = locations1.apply_invariants(zone);
        zone = locations2.apply_invariants(zone);

        StatePair {
            locations1,
            locations2,
            zone: Rc::new(zone),
        }
    }

    pub fn new(
        locations1: LocationTree,
        locations2: LocationTree,
        zone: Rc<OwnedFederation>,
    ) -> Self {
        StatePair {
            locations1,
            locations2,
            zone,
        }
    }

    pub fn get_locations1(&self) -> &LocationTree {
        &self.locations1
    }

    pub fn get_locations2(&self) -> &LocationTree {
        &self.locations2
    }

    //Used to allow borrowing both states as mutable
    pub fn get_mut_locations(
        &mut self,
        is_states1: bool,
    ) -> (&mut LocationTree, &mut LocationTree) {
        if is_states1 {
            (&mut self.locations1, &mut self.locations2)
        } else {
            (&mut self.locations2, &mut self.locations1)
        }
    }

    pub fn get_locations(&self, is_states1: bool) -> (&LocationTree, &LocationTree) {
        if is_states1 {
            (&self.locations1, &self.locations2)
        } else {
            (&self.locations2, &self.locations1)
        }
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

    pub fn extrapolate_max_bounds(
        self,
        sys1: &TransitionSystemPtr,
        sys2: &TransitionSystemPtr,
    ) -> Self {
        let mut bounds = sys1.get_local_max_bounds(&self.locations1);
        bounds.add_bounds(&sys2.get_local_max_bounds(&self.locations2));
        let zone = self.clone_zone().extrapolate_max_bounds(&bounds);

        StatePair {
            locations1: self.locations1,
            locations2: self.locations2,
            zone: Rc::new(zone),
        }
    }
}

impl Display for StatePair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Pair: 1:{} where {}, 2:{} where {}, zone: {}",
            self.locations1.id,
            self.locations1
                .get_invariants()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "no invariant".to_string()),
            self.locations2.id,
            self.locations2
                .get_invariants()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "no invariant".to_string()),
            self.ref_zone()
        ))?;

        Ok(())
    }
}
