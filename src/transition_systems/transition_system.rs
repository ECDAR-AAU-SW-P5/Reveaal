use super::ComponentInfo;
use super::{CompositionType, LocationID, LocationTree};
use crate::edge_eval::updater::CompiledUpdate;
use crate::model_objects::{Component, Declarations, State, Transition};
use crate::parse_queries::parse_to_system_expr;
use crate::system::query_failures::{ConsistencyResult, DeterminismResult};
use crate::system::specifics::SpecificLocation;
use crate::{
    data_reader::component_loader::ComponentContainer, extract_system_rep::get_system_recipe,
    ComponentLoader,
};
use dyn_clone::{clone_trait_object, DynClone};
use edbm::util::{bounds::Bounds, constraints::ClockIndex};
use std::collections::hash_map::Entry;
use std::collections::vec_deque::VecDeque;
use std::collections::{hash_set::HashSet, HashMap};
use std::hash::Hash;
use std::rc::Rc;

pub type TransitionSystemPtr = Box<dyn TransitionSystem>;
pub type Action = String;
pub type EdgeTuple = (Action, Transition);
pub type EdgeIndex = (LocationID, usize);

pub enum ComponentInfoTree<'a> {
    Info(&'a ComponentInfo),
    Composition(Box<ComponentInfoTree<'a>>, Box<ComponentInfoTree<'a>>),
}

impl<'a> ComponentInfoTree<'a> {
    pub fn iter(&'a self) -> Box<dyn Iterator<Item = &'a ComponentInfo> + '_> {
        match self {
            ComponentInfoTree::Info(info) => Box::new(std::iter::once(*info)),
            ComponentInfoTree::Composition(left, right) => {
                Box::new(left.iter().chain(right.iter()))
            }
        }
    }

    pub fn split(self) -> (ComponentInfoTree<'a>, ComponentInfoTree<'a>) {
        match self {
            ComponentInfoTree::Composition(left, right) => (*left, *right),
            ComponentInfoTree::Info(_) => {
                unreachable!("Cannot split a ComponentInfoTree with only one ComponentInfo")
            }
        }
    }

    pub fn info(&self) -> &ComponentInfo {
        match self {
            ComponentInfoTree::Info(info) => info,
            ComponentInfoTree::Composition(_, _) => {
                unreachable!(
                    "Cannot get info from a ComponentInfoTree with more than one ComponentInfo"
                )
            }
        }
    }
}

pub trait TransitionSystem: DynClone {
    fn get_local_max_bounds(&self, loc: &LocationTree) -> Bounds;
    fn get_dim(&self) -> ClockIndex;

    fn next_transitions_if_available(
        &self,
        location: Rc<LocationTree>,
        action: &str,
    ) -> Vec<Transition> {
        if self.actions_contain(action) {
            self.next_transitions(location, action)
        } else {
            vec![]
        }
    }

    fn next_transitions(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition>;

    fn next_outputs(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_output_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn next_inputs(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_input_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn get_input_actions(&self) -> HashSet<String>;

    fn inputs_contain(&self, action: &str) -> bool {
        self.get_input_actions().contains(action)
    }

    fn get_output_actions(&self) -> HashSet<String>;

    fn outputs_contain(&self, action: &str) -> bool {
        self.get_output_actions().contains(action)
    }

    fn get_actions(&self) -> HashSet<String>;

    fn actions_contain(&self, action: &str) -> bool {
        self.get_actions().contains(action)
    }

    fn get_initial_location(&self) -> Option<Rc<LocationTree>>;

    /// Function to get all locations from a [`TransitionSystem`]
    /// #### Warning
    /// This function utilizes a lot of memory. Use with caution
    fn get_all_locations(&self) -> Vec<Rc<LocationTree>>;

    fn get_location(&self, id: &LocationID) -> Option<Rc<LocationTree>> {
        self.get_all_locations()
            .iter()
            .find(|loc| loc.id == *id)
            .cloned()
    }

    fn get_decls(&self) -> Vec<&Declarations>;

    fn precheck_sys_rep(&self) -> ConsistencyResult {
        self.check_determinism()?;
        self.check_local_consistency()
    }

    fn check_determinism(&self) -> DeterminismResult;

    fn check_local_consistency(&self) -> ConsistencyResult;

    fn get_initial_state(&self) -> Option<State>;

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;

    fn comp_infos(&'_ self) -> ComponentInfoTree<'_> {
        let (left, right) = self.get_children();
        let left_info = left.comp_infos();
        let right_info = right.comp_infos();
        ComponentInfoTree::Composition(Box::new(left_info), Box::new(right_info))
    }

    fn to_string(&self) -> String {
        if self.get_composition_type() == CompositionType::Simple {
            panic!("Simple Transition Systems should implement to_string() themselves.")
        }
        let (left, right) = self.get_children();
        let comp = match self.get_composition_type() {
            CompositionType::Conjunction => "&&",
            CompositionType::Composition => "||",
            CompositionType::Quotient => r"\\",
            CompositionType::Simple => unreachable!(),
        };
        format!("({} {} {})", left.to_string(), comp, right.to_string())
    }

    /// Returns a [`Vec`] of all component names in a given [`TransitionSystem`].
    fn component_names(&self) -> Vec<&str> {
        let children = self.get_children();
        let left_child = children.0;
        let right_child = children.1;
        left_child
            .component_names()
            .into_iter()
            .chain(right_child.component_names())
            .collect()
    }

    ///Constructs a [ClockAnalysisGraph],
    ///where nodes represents locations and Edges represent transitions
    fn get_analysis_graph(&self) -> ClockAnalysisGraph {
        let mut graph: ClockAnalysisGraph = ClockAnalysisGraph::from_dim(self.get_dim());
        self.find_edges_and_nodes(self.get_initial_location().unwrap(), &mut graph);

        graph
    }

    ///Helper function to recursively traverse all transitions in a transitions system
    ///in order to find all transitions and location in the transition system, and
    ///saves these as [ClockAnalysisEdge]s and [ClockAnalysisNode]s in the [ClockAnalysisGraph]
    fn find_edges_and_nodes(
        &self,
        init_location: Rc<LocationTree>,
        graph: &mut ClockAnalysisGraph,
    ) {
        let mut worklist: VecDeque<Rc<LocationTree>> = VecDeque::from([init_location]);
        let actions = self.get_actions();
        while let Some(location) = worklist.pop_front() {
            //Constructs a node to represent this location and add it to the graph.
            let mut node: ClockAnalysisNode = ClockAnalysisNode {
                invariant_dependencies: HashSet::new(),
                id: location.id.get_unique_string(),
            };

            //Finds clocks used in invariants in this location.
            if let Some(invariant) = &location.invariant {
                let conjunctions = invariant.minimal_constraints().conjunctions;
                for conjunction in conjunctions {
                    for constraint in conjunction.iter() {
                        node.invariant_dependencies.insert(constraint.i);
                        node.invariant_dependencies.insert(constraint.j);
                    }
                }
            }
            graph.nodes.insert(node.id.clone(), node);

            //Constructs an edge to represent each transition from this graph and add it to the graph.
            for action in &actions {
                for transition in self.next_transitions_if_available(Rc::clone(&location), action) {
                    let mut edge = ClockAnalysisEdge {
                        from: location.id.get_unique_string(),
                        to: transition.target_locations.id.get_unique_string(),
                        guard_dependencies: HashSet::new(),
                        updates: transition.updates,
                        edge_type: action.to_string(),
                    };

                    //Finds clocks used in guards in this transition.
                    let conjunctions = transition.guard_zone.minimal_constraints().conjunctions;
                    for conjunction in &conjunctions {
                        for constraint in conjunction.iter() {
                            edge.guard_dependencies.insert(constraint.i);
                            edge.guard_dependencies.insert(constraint.j);
                        }
                    }

                    graph.edges.push(edge);

                    if !graph
                        .nodes
                        .contains_key(&transition.target_locations.id.get_unique_string())
                    {
                        worklist.push_back(transition.target_locations);
                    }
                }
            }
        }
    }

    fn find_redundant_clocks(&self) -> Vec<ClockReductionInstruction> {
        self.get_analysis_graph().find_clock_redundancies()
    }

    fn construct_location_tree(&self, target: SpecificLocation)
        -> Result<Rc<LocationTree>, String>;
}

/// Returns a [`TransitionSystemPtr`] equivalent to a `composition` of some `components`.
pub fn components_to_transition_system(
    components: Vec<Component>,
    composition: &str,
) -> TransitionSystemPtr {
    let mut component_container = ComponentContainer::from(components);
    component_loader_to_transition_system(&mut component_container, composition)
}

/// Returns a [`TransitionSystemPtr`] equivalent to a `composition` of some components in a [`ComponentLoader`].
pub fn component_loader_to_transition_system(
    loader: &mut dyn ComponentLoader,
    composition: &str,
) -> TransitionSystemPtr {
    let mut dimension = 0;
    let sys_expr = parse_to_system_expr(composition).unwrap();
    get_system_recipe(&sys_expr, loader, &mut dimension, &mut None)
        .unwrap()
        .compile(dimension)
        .unwrap()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClockReductionInstruction {
    RemoveClock {
        clock_index: ClockIndex,
    },
    ReplaceClocks {
        clock_index: ClockIndex,
        clock_indices: HashSet<ClockIndex>,
    },
}

#[derive(Debug)]
pub struct ClockAnalysisNode {
    pub invariant_dependencies: HashSet<ClockIndex>,
    pub id: String,
}

#[derive(Debug)]
pub struct ClockAnalysisEdge {
    pub from: String,
    pub to: String,
    pub guard_dependencies: HashSet<ClockIndex>,
    pub updates: Vec<CompiledUpdate>,
    pub edge_type: String,
}

#[derive(Debug)]
pub struct ClockAnalysisGraph {
    pub nodes: HashMap<String, ClockAnalysisNode>,
    pub edges: Vec<ClockAnalysisEdge>,
    pub dim: ClockIndex,
}

impl ClockAnalysisGraph {
    pub fn from_dim(dim: usize) -> ClockAnalysisGraph {
        ClockAnalysisGraph {
            nodes: HashMap::new(),
            edges: vec![],
            dim,
        }
    }

    pub fn find_clock_redundancies(self) -> Vec<ClockReductionInstruction> {
        //First we find the used clocks
        let used_clocks = self.find_used_clocks();

        //Then we instruct the caller to remove the unused clocks, we start at 1 since the 0 clock is not a real clock
        let mut unused_clocks = (1..self.dim).collect::<HashSet<ClockIndex>>();
        for used_clock in &used_clocks {
            unused_clocks.remove(used_clock);
        }

        let mut rv: Vec<ClockReductionInstruction> = Vec::new();
        for unused_clock in &unused_clocks {
            rv.push(ClockReductionInstruction::RemoveClock {
                clock_index: *unused_clock,
            });
        }

        let mut equivalent_clock_groups = self.find_equivalent_clock_groups(&used_clocks);

        for equivalent_clock_group in &mut equivalent_clock_groups {
            let lowest_clock = *equivalent_clock_group.iter().min().unwrap();
            equivalent_clock_group.remove(&lowest_clock);
            rv.push(ClockReductionInstruction::ReplaceClocks {
                clock_index: lowest_clock,
                clock_indices: equivalent_clock_group.clone(),
            });
        }

        rv
    }

    fn find_used_clocks(&self) -> HashSet<ClockIndex> {
        let mut used_clocks = HashSet::new();

        //First we find the used clocks
        for edge in &self.edges {
            for guard_dependency in &edge.guard_dependencies {
                used_clocks.insert(*guard_dependency);
            }
        }

        for node in &self.nodes {
            for invariant_dependency in &node.1.invariant_dependencies {
                used_clocks.insert(*invariant_dependency);
            }
        }

        //Clock index 0 is not a real clock therefore it is removed
        used_clocks.remove(&0);

        used_clocks
    }

    fn find_equivalent_clock_groups(
        &self,
        used_clocks: &HashSet<ClockIndex>,
    ) -> Vec<HashSet<ClockIndex>> {
        if used_clocks.len() < 2 || self.edges.is_empty() {
            return Vec::new();
        }

        //This function works by maintaining the loop invariant that equivalent_clock_groups contains
        //groups containing clocks where all clocks contained are equivalent in all edges we have iterated
        //through. We also have to make sure that each clock are only present in one group at a time.
        //This means that for the first iteration all clocks are equivalent. We do not include
        //unused clocks since they are all equivalent and will removed completely in another stage.
        let mut equivalent_clock_groups: Vec<HashSet<ClockIndex>> = vec![used_clocks.clone()];

        for edge in &self.edges {
            //First the clocks which are equivalent in this edge are found. This is defined by every
            //clock in their respective group are set to the same value. This is done in a HashMap
            //where each clock group has their own unique u32, the clock indices
            //with the same value are in the same group
            let mut locally_equivalent_clock_groups: HashMap<ClockIndex, u32> = HashMap::new();

            //Then we create the groups in the hashmap
            for update in edge.updates.iter() {
                locally_equivalent_clock_groups.insert(update.clock_index, update.value as u32);
            }

            //Then the locally equivalent clock groups will be combined with the globally equivalent
            //clock groups to identify the new globally equivalent clocks
            let mut new_groups: HashMap<usize, HashSet<ClockIndex>> = HashMap::new();
            let mut group_offset: usize = u32::MAX as usize;

            //For each of the existing clock groups we will remove the clocks from the groups
            //that are locally equivalent, this means that each global group will now be
            //updated to uphold the loop invariant.
            //This is done by giving each globally equivalent clock group a group offset
            //So all groups in the locally equivalent clock groups will be partitioned
            //by the group they are in, in their globally equivalent group
            for (old_group_index, equivalent_clock_group) in
                equivalent_clock_groups.iter_mut().enumerate()
            {
                for clock in equivalent_clock_group.iter() {
                    if let Some(group_id) = locally_equivalent_clock_groups.get(clock) {
                        ClockAnalysisGraph::get_or_insert(
                            &mut new_groups,
                            group_offset + *group_id as usize,
                        )
                        .insert(*clock);
                    } else {
                        ClockAnalysisGraph::get_or_insert(&mut new_groups, old_group_index)
                            .insert(*clock);
                    }
                }
                group_offset += (u32::MAX as usize) * 2;
            }

            //Then we just have to take each of the values in the map and collect them into a vec
            equivalent_clock_groups = new_groups
                .into_iter()
                .map(|pair| pair.1)
                .filter(|group| group.len() > 1)
                .collect();
        }
        equivalent_clock_groups
    }

    fn get_or_insert<K: Eq + Hash, V: Default>(map: &'_ mut HashMap<K, V>, key: K) -> &'_ mut V {
        match map.entry(key) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(V::default()),
        }
    }
}

clone_trait_object!(TransitionSystem);
