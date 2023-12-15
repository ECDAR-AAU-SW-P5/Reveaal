use crate::data_reader::component_loader::ComponentLoader;
use crate::model_objects::expressions::{QueryExpression, SaveExpression, SystemExpression};
use crate::model_objects::{Component, Query, State};
use crate::system::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    ReachabilityExecutor, RefinementExecutor,
};
use crate::system::extract_state::get_state;

use crate::transition_systems::{
    CompiledComponent, Composition, Conjunction, Quotient, TransitionSystemPtr,
};

use super::executable_query::SyntaxExecutor;
use super::query_failures::{SyntaxResult, SystemRecipeFailure};
use crate::system::pruning;
use edbm::util::constraints::ClockIndex;
use log::debug;
use simple_error::bail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableQueryError {
    SystemRecipeFailure(SystemRecipeFailure),
    Custom(String),
}

impl From<Box<SystemRecipeFailure>> for ExecutableQueryError {
    fn from(failure: Box<SystemRecipeFailure>) -> Self {
        ExecutableQueryError::SystemRecipeFailure(*failure)
    }
}

impl<T: Into<String>> From<T> for ExecutableQueryError {
    fn from(failure: T) -> Self {
        ExecutableQueryError::Custom(failure.into())
    }
}

/// This function fetches the appropriate components based on the structure of the query and makes the enum structure match the query
/// this function also handles setting up the correct indices for clocks based on the amount of components in each system representation
pub fn create_executable_query<'a>(
    full_query: &Query,
    component_loader: &'a mut (dyn ComponentLoader + 'static),
) -> Result<Box<dyn ExecutableQuery + 'a>, ExecutableQueryError> {
    let mut dim: ClockIndex = 0;

    if let Some(query) = full_query.get_query() {
        match query {
            QueryExpression::Refinement(left_side, right_side) => {
                let mut quotient_index = None;

                let left =
                    get_system_recipe(left_side, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();
                let right =
                    get_system_recipe(right_side, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                let mut component_index = 0;

                Ok(Box::new(RefinementExecutor {
                    sys1: left.compile_with_index(dim, &mut component_index)?,
                    sys2: right.compile_with_index(dim, &mut component_index)?,
                }))
            }
            QueryExpression::Reachability { system, from, to } => {
                let machine =
                    get_system_recipe(system, component_loader, &mut dim, &mut None).unwrap();
                let transition_system = machine.clone().compile(dim)?;

                // Assign the start state to the initial state of the transition system if no start state is given by the query
                let start_state: State = if let Some(state) = from.as_ref() {
                    let state = get_state(state, &machine, &transition_system)
                        .map_err(|err| format!("Invalid Start state: {}", err))?;
                    if state.decorated_locations.id.is_partial_location() {
                        return Err("Start state is a partial state, which it must not be".into());
                    }
                    state
                } else {
                    match transition_system.get_initial_state() {
                        Some(state) => state,
                        None => return Err("No start state in the transition system".into()),
                    }
                };

                let end_state: State = get_state(to, &machine, &transition_system)
                    .map_err(|err| format!("Invalid End state: {}", err))?;

                Ok(Box::new(ReachabilityExecutor {
                    transition_system,
                    start_state,
                    end_state,
                }))
            }
            QueryExpression::Consistency(query_expression) => {
                let mut quotient_index = None;
                let recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                )
                .unwrap();

                Ok(Box::new(ConsistencyExecutor {
                    system: recipe.compile(dim)?,
                }))
            }
            QueryExpression::Syntax(query_expression) => {
                let mut quotient_index = None;
                let recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                );

                Ok(Box::new(SyntaxExecutor { result: recipe }))
            }
            QueryExpression::Determinism(query_expression) => {
                let mut quotient_index = None;
                let recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                )
                .unwrap();

                Ok(Box::new(DeterminismExecutor {
                    system: recipe.compile(dim)?,
                }))
            }
            QueryExpression::GetComponent(SaveExpression { system, name }) => {
                let mut quotient_index = None;
                let recipe =
                    get_system_recipe(system, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                Ok(Box::new(GetComponentExecutor {
                    system: recipe.compile(dim)?,
                    comp_name: name.clone().unwrap_or("Unnamed".to_string()),
                    component_loader,
                }))
            }
            QueryExpression::Prune(SaveExpression { system, name }) => {
                let mut quotient_index = None;
                let recipe =
                    get_system_recipe(system, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                Ok(Box::new(GetComponentExecutor {
                    system: pruning::prune_system(recipe.compile(dim)?, dim),
                    comp_name: name.clone().unwrap_or("Unnamed".to_string()),
                    component_loader,
                }))
            }

            // Should handle consistency, Implementation, determinism and specification here, but we cant deal with it atm anyway
            _ => bail!("Not yet setup to handle query"),
        }
    } else {
        bail!("No query was supplied for extraction")
    }
}

#[derive(Clone, Debug)]
pub enum SystemRecipe {
    Composition(Box<SystemRecipe>, Box<SystemRecipe>),
    Conjunction(Box<SystemRecipe>, Box<SystemRecipe>),
    Quotient(Box<SystemRecipe>, Box<SystemRecipe>, ClockIndex),
    Component(Box<Component>),
}

impl SystemRecipe {
    pub fn compile(self, dim: ClockIndex) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        let mut component_index = 0;
        self._compile(dim + 1, &mut component_index)
    }

    pub fn compile_with_index(
        self,
        dim: ClockIndex,
        component_index: &mut u32,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        self._compile(dim + 1, component_index)
    }

    fn _compile(
        self,
        dim: ClockIndex,
        component_index: &mut u32,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        match self {
            SystemRecipe::Composition(left, right) => Composition::new_ts(
                left._compile(dim, component_index)?,
                right._compile(dim, component_index)?,
                dim,
            ),
            SystemRecipe::Conjunction(left, right) => Conjunction::new_ts(
                left._compile(dim, component_index)?,
                right._compile(dim, component_index)?,
                dim,
            ),
            SystemRecipe::Quotient(left, right, clock_index) => Quotient::new_ts(
                left._compile(dim, component_index)?,
                right._compile(dim, component_index)?,
                clock_index,
                dim,
            ),
            SystemRecipe::Component(comp) => {
                CompiledComponent::compile(*comp, dim, component_index)
                    .map(|comp| comp as TransitionSystemPtr)
            }
        }
    }

    /// Gets the number of `Components`s in the `SystemRecipe`
    pub fn get_component_count(&self) -> usize {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                left.get_component_count() + right.get_component_count()
            }
            SystemRecipe::Component(_) => 1,
        }
    }

    pub fn get_components(&self) -> Vec<&Component> {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                let mut o = left.get_components();
                o.extend(right.get_components());
                o
            }
            SystemRecipe::Component(c) => vec![c],
        }
    }
}

pub fn get_system_recipe(
    side: &SystemExpression,
    component_loader: &mut dyn ComponentLoader,
    clock_index: &mut ClockIndex,
    quotient_index: &mut Option<ClockIndex>,
) -> Result<Box<SystemRecipe>, SyntaxResult> {
    match side {
        SystemExpression::Composition(left, right) => Ok(Box::new(SystemRecipe::Composition(
            get_system_recipe(left, component_loader, clock_index, quotient_index)?,
            get_system_recipe(right, component_loader, clock_index, quotient_index)?,
        ))),
        SystemExpression::Conjunction(left, right) => Ok(Box::new(SystemRecipe::Conjunction(
            get_system_recipe(left, component_loader, clock_index, quotient_index)?,
            get_system_recipe(right, component_loader, clock_index, quotient_index)?,
        ))),
        SystemExpression::Quotient(left, right) => {
            let left = get_system_recipe(left, component_loader, clock_index, quotient_index)?;
            let right = get_system_recipe(right, component_loader, clock_index, quotient_index)?;

            let q_index = match quotient_index {
                Some(q_i) => *q_i,
                None => {
                    *clock_index += 1;
                    debug!("Quotient clock index: {}", *clock_index);

                    quotient_index.replace(*clock_index);
                    quotient_index.unwrap()
                }
            };

            Ok(Box::new(SystemRecipe::Quotient(left, right, q_index)))
        }
        SystemExpression::Component(name, id) => {
            let mut component = component_loader.get_component(name)?.clone();
            component.set_clock_indices(clock_index);
            component.special_id = id.clone();
            // Logic for locations
            debug!("{} Clocks: {:?}", name, component.declarations.clocks);

            Ok(Box::new(SystemRecipe::Component(Box::new(component))))
        }
    }
}
