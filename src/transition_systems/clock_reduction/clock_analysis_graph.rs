use crate::transition_systems::compiled_update::CompiledUpdate;
use crate::transition_systems::{LocationTree, TransitionSystemPtr};
use edbm::util::constraints::ClockIndex;
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug)]
pub struct ClockAnalysisGraph {
    pub nodes: HashMap<String, ClockAnalysisNode>,
    pub edges: Vec<ClockAnalysisEdge>,
    pub dim: ClockIndex,
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

/// Main way to use ClockAnalysisGraph
pub fn find_redundant_clocks(
    system: &TransitionSystemPtr,
) -> (Vec<ClockIndex>, Vec<HashSet<ClockIndex>>) {
    ClockAnalysisGraph::from_system(system).find_clock_redundancies()
}

impl ClockAnalysisGraph {
    /// Constructs a [ClockAnalysisGraph] where nodes represent locations and edges represent transitions
    pub fn from_system(system: &TransitionSystemPtr) -> ClockAnalysisGraph {
        let mut graph: ClockAnalysisGraph = ClockAnalysisGraph::from_dim(system.get_dim());
        graph.find_edges_and_nodes(system, system.get_initial_location().unwrap());

        graph
    }

    pub fn from_dim(dim: usize) -> ClockAnalysisGraph {
        ClockAnalysisGraph {
            nodes: HashMap::new(),
            edges: vec![],
            dim,
        }
    }

    pub fn find_clock_redundancies(self) -> (Vec<ClockIndex>, Vec<HashSet<ClockIndex>>) {
        // First we find the used clocks
        let used_clocks = self.find_used_clocks();

        // Then we instruct the caller to remove the unused clocks. We start at 1 since the 0 clock is not a real clock
        let unused_clocks = (1..self.dim)
            .filter(|clock| !used_clocks.contains(clock))
            .sorted()
            .collect::<Vec<ClockIndex>>();

        let equivalent_clock_groups = self.find_equivalent_clock_groups(&used_clocks);

        (unused_clocks, equivalent_clock_groups)
    }

    fn find_used_clocks(&self) -> HashSet<ClockIndex> {
        let mut used_clocks = HashSet::new();

        // First we find the used clocks
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

        // Clock index 0 is not a real clock. Therefore, it is removed
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

        // This function works by maintaining the loop invariant that equivalent_clock_groups contains
        // groups containing clocks where all clocks contained are equivalent in all edges we have iterated
        // through. We also have to make sure that each clock are only present in one group at a time.
        // This means that for the first iteration all clocks are equivalent. We do not include
        // unused clocks since they are all equivalent and will removed completely in another stage.
        let mut equivalent_clock_groups: Vec<HashSet<ClockIndex>> = vec![used_clocks.clone()];

        for edge in &self.edges {
            // First the clocks which are equivalent in this edge are found. This is defined by every
            // clock in their respective group are set to the same value. This is done in a HashMap
            // where each clock group has their own unique u32, the clock indices
            // with the same value are in the same group
            let mut locally_equivalent_clock_groups: HashMap<ClockIndex, u32> = HashMap::new();

            // Then we create the groups in the hashmap
            for update in edge.updates.iter() {
                locally_equivalent_clock_groups.insert(update.clock_index, update.value as u32);
            }

            // Then the locally equivalent clock groups will be combined with the globally equivalent
            // clock groups to identify the new globally equivalent clocks
            let mut new_groups: HashMap<usize, HashSet<ClockIndex>> = HashMap::new();
            let mut group_offset: usize = u32::MAX as usize;

            // For each of the existing clock groups we will remove the clocks from the groups
            // that are locally equivalent, this means that each global group will now be
            // updated to uphold the loop invariant.
            // This is done by giving each globally equivalent clock group a group offset
            // So all groups in the locally equivalent clock groups will be partitioned
            // by the group they are in, in their globally equivalent group
            for (old_group_index, equivalent_clock_group) in
                equivalent_clock_groups.iter_mut().enumerate()
            {
                for clock in equivalent_clock_group.iter() {
                    if let Some(group_id) = locally_equivalent_clock_groups.get(clock) {
                        ClockAnalysisGraph::get_or_insert(
                            &mut new_groups,
                            group_offset + ((*group_id) as usize),
                        )
                        .insert(*clock);
                    } else {
                        ClockAnalysisGraph::get_or_insert(&mut new_groups, old_group_index)
                            .insert(*clock);
                    }
                }
                group_offset += (u32::MAX as usize) * 2;
            }

            // Then we just have to take each of the values in the map and collect them into a vec
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

    /// Helper function to recursively traverse all transitions in a transitions system
    /// in order to find all transitions and location in the transition system, and
    /// saves these as [ClockAnalysisEdge]s and [ClockAnalysisNode]s in the [ClockAnalysisGraph]
    fn find_edges_and_nodes(
        &mut self,
        system: &TransitionSystemPtr,
        init_location: Rc<LocationTree>,
    ) {
        let mut worklist = VecDeque::from([init_location]);
        let actions = system.get_actions();
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
            self.nodes.insert(node.id.clone(), node);

            //Constructs an edge to represent each transition from this graph and add it to the graph.
            for action in &actions {
                for transition in system.next_transitions_if_available(Rc::clone(&location), action)
                {
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

                    self.edges.push(edge);

                    if !self
                        .nodes
                        .contains_key(&transition.target_locations.id.get_unique_string())
                    {
                        worklist.push_back(transition.target_locations);
                    }
                }
            }
        }
    }
}
