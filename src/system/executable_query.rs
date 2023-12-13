use crate::data_reader::component_loader::ComponentLoader;
use crate::extract_system_rep::SystemRecipe;
use crate::model_objects::State;
use crate::system::reachability;
use crate::system::refine;
use crate::system::save_component::combine_components;
use crate::transition_systems::TransitionSystemPtr;

use super::query_failures::PathFailure;
use super::query_failures::QueryResult;
use super::query_failures::SyntaxResult;
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

            QueryResult::Syntax(Ok(_)) => println!("{} -- Success!", query_str),
            QueryResult::Syntax(Err(err)) => println!("{} -- Failed!\n{}", query_str, err),

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
}

pub struct ConsistencyExecutor {
    pub system: TransitionSystemPtr,
}

impl ExecutableQuery for ConsistencyExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        self.system.precheck_sys_rep().into()
    }
}

pub struct SyntaxExecutor {
    pub result: Result<Box<SystemRecipe>, SyntaxResult>,
}

impl ExecutableQuery for SyntaxExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        match self.result {
            Ok(_) => QueryResult::Syntax(Ok(())),
            Err(err) => QueryResult::Syntax(Err(err.unwrap_err())),
        }
    }
}

pub struct DeterminismExecutor {
    pub system: TransitionSystemPtr,
}

impl ExecutableQuery for DeterminismExecutor {
    fn execute(self: Box<Self>) -> QueryResult {
        self.system.check_determinism().into()
    }
}
