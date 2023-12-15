use crate::DataReader::component_loader::ComponentLoader;
use crate::ModelObjects::State;
use crate::System::reachability;
use crate::System::refine;
use crate::System::save_component::combine_components;
use crate::TransitionSystems::TransitionSystemPtr;
use edbm::util::constraints::ClockIndex;
use std::cmp;

use super::query_failures::PathFailure;
use super::query_failures::QueryResult;
use super::save_component::PruningStrategy;
use super::specifics::SpecificDecision;

impl QueryResult {
    pub fn print_result(&self, query_str: &str) {
        match self {
            QueryResult::Refinement(Ok(_)) => satisfied(query_str),
            QueryResult::Refinement(Err(failure)) => {
                not_satisfied(query_str);
                println!("\nGot failure: {}", failure);
            }

            QueryResult::Reachability(path) => match path {
                Ok(path) => {
                    satisfied(query_str);
                    print_path(&path.path);
                }
                Err(PathFailure::Unreachable) => {
                    not_satisfied(query_str);
                }
            },

            QueryResult::Consistency(Ok(_)) => satisfied(query_str),
            QueryResult::Consistency(Err(_)) => not_satisfied(query_str),

            QueryResult::Determinism(Ok(_)) => satisfied(query_str),
            QueryResult::Determinism(Err(_)) => not_satisfied(query_str),

            QueryResult::GetComponent(_) => {
                println!("{} -- Component succesfully created", query_str)
            }
            QueryResult::CustomError(_) => println!("{} -- Failed", query_str),
            QueryResult::RecipeFailure(_) => not_satisfied(query_str),
        };
    }
}

fn satisfied(query_str: &str) {
    println!("{} -- Property is satisfied", query_str);
}

fn not_satisfied(query_str: &str) {
    println!("{} -- Property is NOT satisfied", query_str);
}

fn print_path(path: &Vec<SpecificDecision>) {
    println!("Edges that have been taken:");
    for SpecificDecision {
        source_state,
        action,
        ..
    } in path
    {
        println!("{} from {}", action, source_state);
    }
}

pub trait ExecutableQuery {
    fn execute(self: Box<Self>) -> QueryResult;
    fn get_dim(&self) -> ClockIndex;
}

pub struct RefinementExecutor {
    pub sys1: TransitionSystemPtr,
    pub sys2: TransitionSystemPtr,
}

impl ExecutableQuery for RefinementExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        let (sys1, sys2) = (self.sys1, self.sys2);

        refine::check_refinement(sys1, sys2).into()
    }

    fn get_dim(&self) -> ClockIndex {
        cmp::max(self.sys1.get_dim(), self.sys2.get_dim())
    }
}

/// Used to store input for the reachability checker
pub struct ReachabilityExecutor {
    // sys represents the transition system
    pub transition_system: TransitionSystemPtr,

    // s_state is the start state
    pub start_state: State,

    // e_state is the end state, where we want to see whether end state is reachable from start state
    pub end_state: State,
}
impl ExecutableQuery for ReachabilityExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        reachability::find_specific_path(self.start_state, self.end_state, &self.transition_system)
            .into()
    }

    fn get_dim(&self) -> ClockIndex {
        self.transition_system.get_dim()
    }
}

pub struct GetComponentExecutor<'a> {
    pub system: TransitionSystemPtr,
    pub comp_name: String,
    pub component_loader: &'a mut dyn ComponentLoader,
}

impl<'a> ExecutableQuery for GetComponentExecutor<'a> {
    fn execute(self: Box<Self>) -> QueryResult {
        let mut comp = combine_components(&self.system, PruningStrategy::Reachable);
        comp.name = self.comp_name;

        comp.remake_edge_ids();

        self.component_loader.save_component(comp.clone());

        QueryResult::GetComponent(comp)
    }

    fn get_dim(&self) -> ClockIndex {
        self.system.get_dim()
    }
}

pub struct ConsistencyExecutor {
    pub system: TransitionSystemPtr,
}

impl ExecutableQuery for ConsistencyExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        self.system.precheck_sys_rep().into()
    }

    fn get_dim(&self) -> ClockIndex {
        self.system.get_dim()
    }
}

pub struct DeterminismExecutor {
    pub system: TransitionSystemPtr,
}

impl ExecutableQuery for DeterminismExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        self.system.check_determinism().into()
    }

    fn get_dim(&self) -> ClockIndex {
        self.system.get_dim()
    }
}
