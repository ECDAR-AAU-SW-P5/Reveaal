#[cfg(test)]
mod reachability_partial_states_test {
    use std::rc::Rc;

    use crate::model_objects::{Declarations, Location, LocationType};
    use crate::transition_systems::CompositionType;
    use crate::transition_systems::LocationTree;
    use test_case::test_case;

    fn build_location_tree_helper(id: &str, location_type: LocationType) -> Rc<LocationTree> {
        LocationTree::simple(
            &Location {
                id: id.to_string(),
                invariant: None,
                location_type,
                urgency: "".to_string(),
            },
            &Declarations::empty(),
            0,
        )
    }
    #[test_case(LocationTree::build_any_location_tree(),
                build_location_tree_helper("L9", LocationType::Normal);
                "_ == L9")]
    #[test_case(build_location_tree_helper("L0", LocationType::Initial),
                LocationTree::build_any_location_tree();
                "L0 == _")]
    #[test_case(build_location_tree_helper("L5", LocationType::Normal),
                build_location_tree_helper("L5", LocationType::Normal);
                "L5 == L5")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L5", LocationType::Normal), LocationTree::build_any_location_tree()),
                LocationTree::merge_as_quotient(build_location_tree_helper("L5", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal));
                "L5//_ == L5//L1")]
    #[test_case(LocationTree::compose(build_location_tree_helper("L5", LocationType::Normal), LocationTree::build_any_location_tree(), CompositionType::Conjunction),
                LocationTree::compose(LocationTree::build_any_location_tree(), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
                "L5&&_ == _&&L1")]
    #[test_case(LocationTree::compose(build_location_tree_helper("L7", LocationType::Normal), LocationTree::build_any_location_tree(), CompositionType::Composition),
                LocationTree::compose(build_location_tree_helper("L7", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
                "L7||_ == L7||L1")]
    #[test_case(LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Composition),
                LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
                "_||_ == L2||L1")]
    #[test_case(LocationTree::compose(LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Composition),
                LocationTree::compose(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Composition);
                "_||_||L2 == L2||L1||L2")]
    #[test_case(build_location_tree_helper("L_35", LocationType::Normal),
                build_location_tree_helper("L_35", LocationType::Normal);
                "L_35 == L_35")]
    fn checks_cmp_locations_returns_true(loc1: Rc<LocationTree>, loc2: Rc<LocationTree>) {
        assert!(loc1.compare_partial_locations(loc2));
    }

    #[test_case(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L5", LocationType::Normal), CompositionType::Composition),
                LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
                "L2||L5 != L2||L1")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L6", LocationType::Normal)),
                LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition);
                "L2//L6 != L2||L1")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L7", LocationType::Normal), build_location_tree_helper("L6", LocationType::Normal)),
                LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
                "L7//L6 != L2&&L1")]
    #[test_case(LocationTree::merge_as_quotient(build_location_tree_helper("L8", LocationType::Normal), LocationTree::build_any_location_tree()),
                LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
                "L8//_ != L2&&L1")]
    #[test_case(LocationTree::build_any_location_tree(),
                LocationTree::compose(build_location_tree_helper("L6", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Conjunction);
                "_ != L6&&L1")]
    #[test_case(LocationTree::build_any_location_tree(),
                LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Conjunction);
                "anylocation _ != _&&_")]
    #[test_case(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L4", LocationType::Normal), CompositionType::Conjunction),
                LocationTree::merge_as_quotient(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L4", LocationType::Normal));
                "L2&&L4 != L2\\L4")]
    #[test_case(LocationTree::compose(LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Conjunction),
                LocationTree::compose(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), build_location_tree_helper("L1", LocationType::Normal), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Composition);
                "_||_&&L2 == L2||L1||L2")]
    #[test_case(LocationTree::compose(LocationTree::compose(build_location_tree_helper("L2", LocationType::Normal), LocationTree::build_any_location_tree(), CompositionType::Composition),build_location_tree_helper("L2", LocationType::Normal), CompositionType::Conjunction),
                LocationTree::compose(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree(), CompositionType::Conjunction);
                "L2||_&&L2 == _&&_")]
    #[test_case(build_location_tree_helper("L7", LocationType::Normal),
                build_location_tree_helper("L5", LocationType::Normal);
                "L7 != L5")]
    #[test_case(LocationTree::merge_as_quotient(LocationTree::build_any_location_tree(), LocationTree::build_any_location_tree()),
                LocationTree::compose(build_location_tree_helper("L6", LocationType::Normal), build_location_tree_helper("L25", LocationType::Normal), CompositionType::Conjunction);
                "_//_ != L6&&L25")]
    #[test_case(build_location_tree_helper("_L1", LocationType::Normal),
                build_location_tree_helper("L1", LocationType::Normal);
                "_L1 != L1")]
    #[test_case(build_location_tree_helper("__", LocationType::Normal),
                build_location_tree_helper("L7", LocationType::Normal);
                "__ != L7")]
    fn checks_cmp_locations_returns_false(loc1: Rc<LocationTree>, loc2: Rc<LocationTree>) {
        assert!(!loc1.compare_partial_locations(loc2));
    }
}
