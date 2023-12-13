#[cfg(test)]
pub mod test {
    use crate::data_reader::json_reader::read_json_component;
    use crate::extract_system_rep::SystemRecipe;
    use crate::model_objects::Component;
    use crate::system::input_enabler;
    use crate::transition_systems::transition_system::ClockReductionInstruction;
    use crate::transition_systems::TransitionSystemPtr;
    use crate::{JsonProjectLoader, DEFAULT_SETTINGS};
    use edbm::util::constraints::ClockIndex;
    use std::collections::{HashMap, HashSet};
    use std::path::Path;

    /// Reads and processes a component.
    pub fn read_json_component_and_process(project_path: &str, component_name: &str) -> Component {
        let mut component = read_json_component(project_path, component_name).unwrap();
        let inputs = component.get_input_actions();
        input_enabler::make_input_enabled(&mut component, &inputs);
        component
    }

    /// Assert that a [`vec<&ClockReductionInstruction>`] contains an instruction that `clock` should
    /// be removed.
    pub(crate) fn assert_unused_clock_in_clock_reduction_instruction_vec(
        redundant_clocks: Vec<ClockReductionInstruction>,
        clock: ClockIndex,
    ) {
        assert!(redundant_clocks
            .iter()
            .any(|instruction| match instruction {
                ClockReductionInstruction::RemoveClock { clock_index } => {
                    println!("Found {}, searching for {}", clock_index, clock);
                    *clock_index == clock
                }
                _ => false,
            }));
    }
    /// Assert that a [`vec<&ClockReductionInstruction>`] contains an instruction that `clock` is a
    /// duplicate of the clocks in `clocks`.
    pub(crate) fn assert_duplicate_clock_in_clock_reduction_instruction_vec(
        redundant_clocks: Vec<ClockReductionInstruction>,
        clock: ClockIndex,
        clocks: &HashSet<ClockIndex>,
    ) {
        assert!(redundant_clocks
            .iter()
            .any(|instruction| match instruction {
                ClockReductionInstruction::RemoveClock { .. } => {
                    false
                }
                ClockReductionInstruction::ReplaceClocks {
                    clock_index,
                    clock_indices,
                } => {
                    *clock_index == clock && clock_indices == clocks
                }
            }));
    }

    pub(crate) fn get_conjunction_transition_system(
        path: &Path,
        comp1: &str,
        comp2: &str,
    ) -> TransitionSystemPtr {
        let (dim, system_recipe) = get_conjunction_system_recipe(path, comp1, comp2);
        system_recipe.compile(dim).unwrap()
    }

    pub(crate) fn get_conjunction_system_recipe(
        path: &Path,
        comp1: &str,
        comp2: &str,
    ) -> (ClockIndex, SystemRecipe) {
        let project_loader =
            JsonProjectLoader::new_loader(path.to_string_lossy().to_string(), DEFAULT_SETTINGS);

        let mut component_loader = project_loader.to_comp_loader();

        let mut next_clock_index: usize = 0;
        let mut component1 = component_loader.get_component(comp1).unwrap().clone();
        let mut component2 = component_loader.get_component(comp2).unwrap().clone();

        component1.set_clock_indices(&mut next_clock_index);
        component2.set_clock_indices(&mut next_clock_index);

        let dimensions =
            component1.declarations.clocks.len() + component2.declarations.clocks.len();

        let sr_component1 = Box::new(SystemRecipe::Component(Box::new(component1)));
        let sr_component2 = Box::new(SystemRecipe::Component(Box::new(component2)));

        let conjunction = SystemRecipe::Conjunction(sr_component1, sr_component2);
        (dimensions, conjunction)
    }

    pub(crate) fn get_composition_transition_system(
        path: &Path,
        comp1: &str,
        comp2: &str,
    ) -> TransitionSystemPtr {
        let project_loader =
            JsonProjectLoader::new_loader(path.to_string_lossy().to_string(), DEFAULT_SETTINGS);

        let mut component_loader = project_loader.to_comp_loader();

        let mut next_clock_index: usize = 0;
        let mut component1 = component_loader.get_component(comp1).unwrap().clone();
        let mut component2 = component_loader.get_component(comp2).unwrap().clone();

        component1.set_clock_indices(&mut next_clock_index);
        component2.set_clock_indices(&mut next_clock_index);

        let dimensions =
            component1.declarations.clocks.len() + component2.declarations.clocks.len();

        let sr_component1 = Box::new(SystemRecipe::Component(Box::new(component1)));
        let sr_component2 = Box::new(SystemRecipe::Component(Box::new(component2)));

        let conjunction = SystemRecipe::Composition(sr_component1, sr_component2);

        conjunction.compile(dimensions).unwrap()
    }

    pub(crate) fn create_clock_name_to_index(
        transition_system: &TransitionSystemPtr,
    ) -> HashMap<String, ClockIndex> {
        let mut clock_name_to_index: HashMap<String, ClockIndex> = HashMap::new();

        for (i, declaration) in transition_system.get_decls().iter().enumerate() {
            for (clock_name, clock_index) in &declaration.clocks {
                clock_name_to_index.insert(format!("component{}:{}", i, clock_name), *clock_index);
            }
        }
        clock_name_to_index
    }
}
