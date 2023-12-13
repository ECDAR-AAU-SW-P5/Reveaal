use crate::data_reader::component_loader::ComponentLoader;
use crate::model_objects::expressions::{QueryExpression, SaveExpression, SystemExpression};
use crate::model_objects::{Component, Query, State};
use crate::system::executable_query::{
    ConsistencyExecutor, DeterminismExecutor, ExecutableQuery, GetComponentExecutor,
    ReachabilityExecutor, RefinementExecutor,
};
use crate::system::extract_state::get_state;
use std::collections::HashMap;

use crate::transition_systems::{
    CompiledComponent, Composition, Conjunction, Quotient, TransitionSystemPtr,
};

use super::executable_query::SyntaxExecutor;
use super::query_failures::{SyntaxResult, SystemRecipeFailure};
use crate::system::pruning;
use crate::transition_systems::transition_system::ClockReductionInstruction;
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

                let mut left =
                    get_system_recipe(left_side, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();
                let mut right =
                    get_system_recipe(right_side, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                if !component_loader.get_settings().disable_clock_reduction {
                    clock_reduction::clock_reduce(
                        &mut left,
                        Some(&mut right),
                        &mut dim,
                        quotient_index,
                    )?;
                }

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
                let mut recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                )
                .unwrap();

                if !component_loader.get_settings().disable_clock_reduction {
                    clock_reduction::clock_reduce(&mut recipe, None, &mut dim, quotient_index)?;
                }

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
                let mut recipe = get_system_recipe(
                    query_expression,
                    component_loader,
                    &mut dim,
                    &mut quotient_index,
                )
                .unwrap();

                if !component_loader.get_settings().disable_clock_reduction {
                    clock_reduction::clock_reduce(&mut recipe, None, &mut dim, quotient_index)?;
                }

                Ok(Box::new(DeterminismExecutor {
                    system: recipe.compile(dim)?,
                }))
            }
            QueryExpression::GetComponent(SaveExpression { system, name }) => {
                let mut quotient_index = None;
                let mut recipe =
                    get_system_recipe(system, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                if !component_loader.get_settings().disable_clock_reduction {
                    clock_reduction::clock_reduce(&mut recipe, None, &mut dim, quotient_index)?;
                }

                Ok(Box::new(GetComponentExecutor {
                    system: recipe.compile(dim)?,
                    comp_name: name.clone().unwrap_or("Unnamed".to_string()),
                    component_loader,
                }))
            }
            QueryExpression::Prune(SaveExpression { system, name }) => {
                let mut quotient_index = None;
                let mut recipe =
                    get_system_recipe(system, component_loader, &mut dim, &mut quotient_index)
                        .unwrap();

                if !component_loader.get_settings().disable_clock_reduction {
                    clock_reduction::clock_reduce(&mut recipe, None, &mut dim, quotient_index)?;
                }

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

    ///Applies the clock-reduction
    fn reduce_clocks(&mut self, clock_instruction: Vec<ClockReductionInstruction>) {
        let mut comps = self.get_components_mut();
        for redundant in clock_instruction {
            match redundant {
                ClockReductionInstruction::RemoveClock { clock_index } => comps
                    .iter_mut()
                    .find(|c| c.declarations.clocks.values().any(|ci| *ci == clock_index))
                    .unwrap_or_else(|| {
                        panic!(
                            "A component could not be found for clock index {}",
                            clock_index
                        )
                    })
                    .remove_clock(clock_index),
                ClockReductionInstruction::ReplaceClocks {
                    clock_indices,
                    clock_index,
                } => comps
                    .iter_mut()
                    .for_each(|c| c.replace_clock(clock_index, &clock_indices)),
            }
        }
    }

    /// Gets all components in `SystemRecipe`
    fn get_components_mut(&mut self) -> Vec<&mut Component> {
        match self {
            SystemRecipe::Composition(left, right)
            | SystemRecipe::Conjunction(left, right)
            | SystemRecipe::Quotient(left, right, _) => {
                let mut o = left.get_components_mut();
                o.extend(right.get_components_mut());
                o
            }
            SystemRecipe::Component(c) => vec![c],
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

    fn change_quotient(&mut self, index: ClockIndex) {
        match self {
            SystemRecipe::Composition(l, r) | SystemRecipe::Conjunction(l, r) => {
                l.change_quotient(index);
                r.change_quotient(index);
            }
            SystemRecipe::Quotient(l, r, q) => {
                *q = index;
                l.change_quotient(index);
                r.change_quotient(index);
            }
            SystemRecipe::Component(_) => (),
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
            debug!("{} Clocks: {:?}", name, component.declarations.clocks);

            Ok(Box::new(SystemRecipe::Component(Box::new(component))))
        }
    }
}

/// Module containing a "safer" function for clock reduction, along with some helper functions
pub(crate) mod clock_reduction {
    use super::*;

    /// Function for a "safer" clock reduction that handles both the dimension of the DBM and the quotient index if needed be
    /// # Arguments
    /// `lhs`: The (main) [`SystemRecipe`] to clock reduce\n
    /// `rhs`: An optional [`SystemRecipe`] used for multiple operands (Refinement)\n
    /// `dim`: A mutable reference to the DBMs dimension for updating\n
    /// `quotient_clock`: The clock for the quotient (This is not reduced)
    /// # Returns
    /// A `Result` used if the [`SystemRecipe`](s) fail during compilation
    pub fn clock_reduce(
        lhs: &mut Box<SystemRecipe>,
        rhs: Option<&mut Box<SystemRecipe>>,
        dim: &mut usize,
        quotient_clock: Option<ClockIndex>,
    ) -> Result<(), Box<SystemRecipeFailure>> {
        if *dim == 0 {
            return Ok(());
        } else if rhs.is_none() {
            return clock_reduce_single(lhs, dim, quotient_clock);
        }
        let rhs = rhs.unwrap();

        let (l_clocks, r_clocks) = filter_redundant_clocks(
            lhs.clone().compile(*dim)?.find_redundant_clocks(),
            rhs.clone().compile(*dim)?.find_redundant_clocks(),
            quotient_clock,
            lhs.get_components_mut()
                .iter()
                .flat_map(|c| c.declarations.clocks.values().cloned())
                .max()
                .unwrap_or_default(),
        );

        debug!("Clocks to be reduced: {l_clocks:?} + {l_clocks:?}");
        *dim -= l_clocks
            .iter()
            .chain(r_clocks.iter())
            .fold(0, |acc, c| acc + c.clocks_removed_count());
        debug!("New dimension: {dim}");

        rhs.reduce_clocks(r_clocks);
        lhs.reduce_clocks(l_clocks);
        compress_component_decls(lhs.get_components_mut(), Some(rhs.get_components_mut()));
        if quotient_clock.is_some() {
            lhs.change_quotient(*dim);
            rhs.change_quotient(*dim);
        }

        Ok(())
    }

    /// Clock reduces a "single_expression", such as consistency
    /// # Arguments
    ///
    /// * `sys`: The [`SystemRecipe`] to clock reduce
    /// * `dim`: the dimension of the system
    /// * `quotient_clock`: The clock for the quotient (This is not reduced)
    ///
    /// returns: Result<(), SystemRecipeFailure>
    fn clock_reduce_single(
        sys: &mut Box<SystemRecipe>,
        dim: &mut usize,
        quotient_clock: Option<ClockIndex>,
    ) -> Result<(), Box<SystemRecipeFailure>> {
        let mut clocks = sys.clone().compile(*dim)?.find_redundant_clocks();
        clocks.retain(|ins| ins.get_clock_index() != quotient_clock.unwrap_or_default());
        debug!("Clocks to be reduced: {clocks:?}");
        *dim -= clocks
            .iter()
            .fold(0, |acc, c| acc + c.clocks_removed_count());
        debug!("New dimension: {dim}");
        sys.reduce_clocks(clocks);
        compress_component_decls(sys.get_components_mut(), None);
        if quotient_clock.is_some() {
            sys.change_quotient(*dim);
        }
        Ok(())
    }

    fn filter_redundant_clocks(
        lhs: Vec<ClockReductionInstruction>,
        rhs: Vec<ClockReductionInstruction>,
        quotient_clock: Option<ClockIndex>,
        split_index: ClockIndex,
    ) -> (
        Vec<ClockReductionInstruction>,
        Vec<ClockReductionInstruction>,
    ) {
        fn get_unique_redundant_clocks<P: Fn(ClockIndex) -> bool>(
            l: Vec<ClockReductionInstruction>,
            r: Vec<ClockReductionInstruction>,
            quotient: ClockIndex,
            bound_predicate: P,
        ) -> Vec<ClockReductionInstruction> {
            l.into_iter()
                // Takes clock instructions that also occur in the rhs system
                // This is done because the lhs also finds the redundant clocks from the rhs,
                // so to ensure that it should be removed, we check if it occurs on both sides
                // which would mean it can be removed
                // e.g "A <= B", we can find clocks from B that are not used in A, so they are marked as remove
                .filter(|ins| r.contains(ins))
                // Takes all the clocks within the bounds of the given system
                // This is done to ensure that we don't try to remove a clock from the rhs system
                .filter(|ins| bound_predicate(ins.get_clock_index()))
                // Removes the quotient clock
                .filter(|ins| ins.get_clock_index() != quotient)
                .collect()
        }
        let quotient_clock = quotient_clock.unwrap_or_default();
        (
            get_unique_redundant_clocks(lhs.clone(), rhs.clone(), quotient_clock, |c| {
                c <= split_index
            }),
            get_unique_redundant_clocks(rhs, lhs, quotient_clock, |c| c > split_index),
        )
    }

    fn compress_component_decls(
        mut comps: Vec<&mut Component>,
        other: Option<Vec<&mut Component>>,
    ) {
        let mut seen: HashMap<ClockIndex, ClockIndex> = HashMap::new();
        let mut l: Vec<&mut ClockIndex> = comps
            .iter_mut()
            .flat_map(|c| c.declarations.clocks.values_mut())
            .collect();
        let mut temp = other.unwrap_or_default();
        l.extend(
            temp.iter_mut()
                .flat_map(|c| c.declarations.clocks.values_mut()),
        );
        l.sort();
        let mut index = 1;
        for clock in l {
            if let Some(val) = seen.get(clock) {
                *clock = *val;
            } else {
                seen.insert(*clock, index);
                *clock = index;
                index += 1;
            }
        }
    }
}
