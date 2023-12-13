use std::{collections::HashSet, fmt};

use crate::model_objects::{Component, State, StatePair};
use crate::transition_systems::{CompositionType, TransitionSystem, TransitionSystemPtr};

use super::specifics::{SpecificPath, SpecificState};

/// Represents how a system is composed at the highest level
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemType {
    /// A refinement between two systems
    Refinement,
    /// A quotient of two systems
    Quotient,
    /// A composition of two systems
    Composition,
    /// A conjunction of two systems
    Conjunction,
    /// A simple component, not composed with anything else
    Simple,
}

impl SystemType {
    /// Returns the string representation of the operator for this composition type
    pub fn operator(&self) -> String {
        match self {
            Self::Quotient => r"\\",
            Self::Refinement => "<=",
            Self::Composition => "||",
            Self::Conjunction => "&&",
            Self::Simple => unreachable!(),
        }
        .to_string()
    }
}

impl fmt::Display for SystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quotient => write!(f, "Quotient"),
            Self::Refinement => write!(f, "Refinement"),
            Self::Composition => write!(f, "Composition"),
            Self::Conjunction => write!(f, "Conjunction"),
            Self::Simple => write!(f, "Component"),
        }
    }
}

/// Represents a system of components as a [String] `name` and the type of the highest level composition `sys_type`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct System {
    pub name: String,
    pub sys_type: SystemType,
}
impl System {
    /// Creates a new refinement system from two systems, `sys1` and `sys2`
    pub fn refinement(sys1: &dyn TransitionSystem, sys2: &dyn TransitionSystem) -> Self {
        Self {
            name: format!("{} <= {}", sys1.to_string(), sys2.to_string()),
            sys_type: SystemType::Refinement,
        }
    }
    /// Creates a new system from a single [TransitionSystem]
    pub fn from(sys: &dyn TransitionSystem) -> Self {
        Self {
            name: sys.to_string(),
            sys_type: sys.get_composition_type().into(),
        }
    }
    /// Creates a new system from two [TransitionSystem]s, `sys1` and `sys2`, and the type of the composition `sys_type`
    pub fn from_composite_system(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        sys_type: SystemType,
    ) -> Self {
        Self {
            name: format!(
                "{} {} {}",
                sys1.to_string(),
                sys_type.operator(),
                sys2.to_string()
            ),
            sys_type,
        }
    }
}

/// Represents a set of actions in a system of components. The system is represented by a [String] `system`,
/// along with the `actions` and whether the actions are all inputs (`is_input`) or all outputs (`!is_input`).
///
/// For representing a single action, see [Action].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionSet {
    pub system: String,
    pub actions: HashSet<String>,
    pub is_input: bool,
}

impl fmt::Display for ActionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let action_type = if self.is_input { "Input" } else { "Output" };

        write!(
            f,
            "{} set {:?} in system '{}'",
            action_type, self.actions, self.system
        )
    }
}

/// Represents a single action in a system of components. The system is represented by a [String] `system`,
/// along with the `action` and whether the action is an input (`is_input`).
///
/// For representing a set of actions, see [ActionSet].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub name: String,
    pub is_input: bool,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_input {
            write!(f, "Input \"{}\"", self.name)
        } else {
            write!(f, "Output \"{}\"", self.name)
        }
    }
}

impl Action {
    /// Creates a new [Action] from a [String] `name` and whether the action is an input (`is_input`).
    pub fn new(name: String, is_input: bool) -> Self {
        Self { name, is_input }
    }
}

/// Represents the different types of results that can be returned from a query
#[derive(Clone, Debug)]
pub enum QueryResult {
    /// A query failed because the recipe was invalid. e.g. a conjunction was empty or actions mismatched in a composition.
    RecipeFailure(SystemRecipeFailure),
    /// A reachability query returned a path or failure, see [PathResult].
    Reachability(PathResult),
    /// A refinement query returned a success or failure, see [RefinementResult].
    Refinement(RefinementResult),
    /// A consistency query returned a success or failure, see [ConsistencyResult].
    Consistency(ConsistencyResult),
    /// A syntax query returned a success or failure, see [SyntaxResult].
    Syntax(SyntaxResult),
    /// A determinism query returned a success or failure, see [DeterminismResult].
    Determinism(DeterminismResult),
    /// A get components query returned a new component.
    GetComponent(Component),
    /// The query resulted in an unclassified error.
    CustomError(String),
}

pub type PathResult = Result<SpecificPath, PathFailure>;

//TODO: add refinement Ok result
pub type RefinementResult = Result<(), RefinementFailure>;

pub type ConsistencyResult = Result<(), ConsistencyFailure>;

pub type SyntaxResult = Result<(), SyntaxFailure>;

pub type DeterminismResult = Result<(), DeterminismFailure>;

/// Represents the different ways that a reachability query can fail
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathFailure {
    /// The target state was unreachable from the initial state
    Unreachable,
}

/// Represents the different ways that a refinement query can fail
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinementFailure {
    /// The refinement failed for `system` because the right side cannot match left sides delay after taking `action` from `state`.
    CutsDelaySolutions {
        system: System,
        action: Action,
        state: SpecificState,
    },
    /// The refinement failed for `system` because one side could not match the `action` from `state`.
    CannotMatch {
        system: System,
        action: Action,
        state: SpecificState,
    },
    /// The refinement failed on a precondition, see [RefinementPrecondition].
    Precondition(RefinementPrecondition),
}

/// Represents the different preconditions that a refinement check can fail on
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinementPrecondition {
    /// The refinement `system` failed because the `child` had no initial location.
    EmptyChild { child: String, system: System },
    /// The refinement `system` failed because the initial state was empty.
    EmptyInitialState { system: System },
    /// The refinement `system` failed because of a `ConsistencyFailure`, see [ConsistencyFailure].
    InconsistentChild(ConsistencyFailure, System),
    /// The refinement `system` failed because of an `ActionFailure`, see [ActionFailure].
    ActionMismatch(ActionFailure, System),
}

impl RefinementFailure {
    /// Creates a new [RefinementFailure] that failed because the `left` or `!left` system was empty.
    pub fn empty_child(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        left: bool,
    ) -> RefinementResult {
        Err(RefinementFailure::Precondition(
            RefinementPrecondition::EmptyChild {
                child: if left {
                    sys1.to_string()
                } else {
                    sys2.to_string()
                },
                system: System::refinement(sys1, sys2),
            },
        ))
    }
    /// Creates a new [RefinementFailure] that failed because the initial state was empty.
    pub fn empty_initial(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> RefinementResult {
        Err(RefinementFailure::Precondition(
            RefinementPrecondition::EmptyInitialState {
                system: System::refinement(sys1, sys2),
            },
        ))
    }

    /// Creates a new [RefinementFailure] that failed because a system could not match an `action` from a [state pair](StatePair).
    pub fn cannot_match(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        action: impl Into<String>,
        state: &StatePair,
    ) -> RefinementResult {
        let action: String = action.into();
        let is_input = sys1.inputs_contain(&action) || sys2.inputs_contain(&action);
        Err(RefinementFailure::CannotMatch {
            system: System::refinement(sys1, sys2),
            action: Action::new(action, is_input),
            state: SpecificState::from_state_pair(state, sys1, sys2),
        })
    }

    /// Creates a new [RefinementFailure] that failed because of a cut delay solution.
    pub fn cuts_delays(
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
        action: impl Into<String>,
        state: &StatePair,
    ) -> RefinementResult {
        let action: String = action.into();
        let is_input = sys1.inputs_contain(&action) || sys2.inputs_contain(&action);
        Err(RefinementFailure::CutsDelaySolutions {
            system: System::refinement(sys1, sys2),
            action: Action::new(action, is_input),
            state: SpecificState::from_state_pair(state, sys1, sys2),
        })
    }
}

/// Represents the different ways that actions can mismatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionFailure {
    /// The actions in the first [ActionSet] are not a subset of the actions in the second [ActionSet].
    NotSubset(ActionSet, ActionSet),
    /// The actions in the first [ActionSet] are not disjoint from the actions in the second [ActionSet].
    NotDisjoint(ActionSet, ActionSet),
}
impl ActionFailure {
    /// Creates a new [Result]<T, [ActionFailure]> that failed because the actions in `actions1` from `sys1` are not a disjoint from the actions in `actions2` from `sys2`.
    pub fn not_disjoint<T>(
        (sys1, actions1): (&dyn TransitionSystem, HashSet<String>),
        (sys2, actions2): (&dyn TransitionSystem, HashSet<String>),
    ) -> Result<T, Box<ActionFailure>> {
        let is_input1 = sys1.get_input_actions() == actions1;
        let is_input2 = sys2.get_input_actions() == actions2;

        debug_assert!(is_input1 || sys1.get_output_actions() == actions1);
        debug_assert!(is_input2 || sys2.get_output_actions() == actions2);

        Err(Box::new(ActionFailure::NotDisjoint(
            ActionSet {
                system: sys1.to_string(),
                actions: actions1,
                is_input: is_input1,
            },
            ActionSet {
                system: sys2.to_string(),
                actions: actions2,
                is_input: is_input2,
            },
        )))
    }

    /// Creates a new [Result]<T, [ActionFailure]> that failed because the actions in `inputs` are not a disjoint from the actions in `outputs`.
    pub fn not_disjoint_io(
        name: impl Into<String>,
        inputs: HashSet<String>,
        outputs: HashSet<String>,
    ) -> Result<(), Box<ActionFailure>> {
        let system = name.into();
        Err(Box::new(ActionFailure::NotDisjoint(
            ActionSet {
                system: system.clone(),
                actions: inputs,
                is_input: true,
            },
            ActionSet {
                system,
                actions: outputs,
                is_input: false,
            },
        )))
    }

    /// Creates a new [Result]<T, [ActionFailure]> that failed because the actions in `actions1` from `sys1` are not a subset of the actions in `actions2` from `sys2`.
    pub fn not_subset(
        (sys1, actions1): (&dyn TransitionSystem, HashSet<String>),
        (sys2, actions2): (&dyn TransitionSystem, HashSet<String>),
    ) -> Result<(), Box<ActionFailure>> {
        let is_input1 = sys1.get_input_actions() == actions1;
        let is_input2 = sys2.get_input_actions() == actions2;

        debug_assert!(is_input1 || sys1.get_output_actions() == actions1);
        debug_assert!(is_input2 || sys2.get_output_actions() == actions2);

        Err(Box::new(ActionFailure::NotSubset(
            ActionSet {
                system: sys1.to_string(),
                actions: actions1,
                is_input: is_input1,
            },
            ActionSet {
                system: sys2.to_string(),
                actions: actions2,
                is_input: is_input2,
            },
        )))
    }

    /// Converts this [ActionFailure] into a [RefinementPrecondition] given the two [TransitionSystem]s that failed.
    pub fn to_precondition(
        self,
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> Box<RefinementPrecondition> {
        Box::new(RefinementPrecondition::ActionMismatch(
            self,
            System::refinement(sys1, sys2),
        ))
    }

    /// Converts this [ActionFailure] into a [SystemRecipeFailure] given the [TransitionSystem] that failed.
    pub fn to_recipe_failure(self, sys: &dyn TransitionSystem) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(self, System::from(sys))
    }

    /// Converts this [ActionFailure] into a [SystemRecipeFailure] given the name of the system `sys` that failed.
    pub fn to_simple_failure(self, sys: impl Into<String>) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(
            self,
            System {
                name: sys.into(),
                sys_type: SystemType::Simple,
            },
        )
    }

    /// Converts this [ActionFailure] that occured during the construction of a [Quotient](crate::transition_systems::Quotient) into a [SystemRecipeFailure] given the two [TransitionSystem]s that failed.
    pub fn to_rfq(self, t: &TransitionSystemPtr, s: &TransitionSystemPtr) -> SystemRecipeFailure {
        SystemRecipeFailure::Action(
            self,
            System::from_composite_system(t.as_ref(), s.as_ref(), SystemType::Quotient),
        )
    }

    /// Converts this [ActionFailure] that occured during the construction of a [Composition](crate::transition_systems::Composition) into a [SystemRecipeFailure] given the two [TransitionSystem]s that failed.
    pub fn to_rfcomp(
        self,
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
    ) -> Box<SystemRecipeFailure> {
        Box::new(SystemRecipeFailure::Action(
            self,
            System::from_composite_system(left.as_ref(), right.as_ref(), SystemType::Composition),
        ))
    }

    /// Converts this [ActionFailure] that occured during the construction of a [Conjunction](crate::transition_systems::Conjunction) into a [SystemRecipeFailure] given the two [TransitionSystem]s that failed.
    pub fn to_rfconj(
        self,
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
    ) -> Box<SystemRecipeFailure> {
        Box::new(SystemRecipeFailure::Action(
            self,
            System::from_composite_system(left.as_ref(), right.as_ref(), SystemType::Conjunction),
        ))
    }
}

/// A query failed because the recipe was invalid. e.g. a conjunction was empty or actions mismatched in a composition
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemRecipeFailure {
    /// The recipe failed because of an action mismatch, see [ActionFailure].
    Action(ActionFailure, System),
    /// The recipe failed because a conjunction in the system was empty (and therefore inconsistent).
    Inconsistent(ConsistencyFailure, System),
}

/// Represents the different ways that clock reduction can fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockReductionFailure {}

/// Represents the different ways that a [TransitionSystem] can fail to be consistent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsistencyFailure {
    /// The `system` has no initial state.
    NoInitialState { system: String },
    /// The system is not deterministic.
    NotDeterministic(DeterminismFailure),
    /// The `system` cannot prune an inconsistent locaction `state`.
    InconsistentLoc {
        system: String,
        state: SpecificState,
    },
    /// The `system` cannot prune an inconsistent `state`.
    InconsistentFrom {
        system: String,
        state: SpecificState,
    },
}

impl ConsistencyFailure {
    /// Creates a new [ConsistencyFailure] that failed because the system has no initial state.
    pub fn no_initial_state(system: &dyn TransitionSystem) -> ConsistencyResult {
        Err(ConsistencyFailure::NoInitialState {
            system: system.to_string(),
        })
    }

    /// Creates a new [ConsistencyFailure] that failed because the system cannot prune an inconsistent location `state`.
    pub fn inconsistent(system: &dyn TransitionSystem, state: &State) -> ConsistencyResult {
        Err(ConsistencyFailure::InconsistentLoc {
            system: system.to_string(),
            state: SpecificState::from_state(state, system),
        })
    }

    /// Creates a new [ConsistencyFailure] that failed because the system cannot prune an inconsistent `state`.
    pub fn inconsistent_from(
        system: &dyn TransitionSystem,
        //action: impl Into<String>,
        state: &State,
    ) -> ConsistencyResult {
        //let action: String = action.into();
        //let is_input = system.inputs_contain(&action);
        Err(ConsistencyFailure::InconsistentFrom {
            system: system.to_string(),
            //action: Action::new(action, is_input),
            state: SpecificState::from_state(state, system),
        })
    }

    /// Converts this [ConsistencyFailure] into a [RefinementPrecondition] given the two [TransitionSystem]s that failed.
    pub fn to_precondition(
        self,
        sys1: &dyn TransitionSystem,
        sys2: &dyn TransitionSystem,
    ) -> RefinementPrecondition {
        RefinementPrecondition::InconsistentChild(self, System::refinement(sys1, sys2))
    }

    /// Converts this [ConsistencyFailure] into a [SystemRecipeFailure] given the [TransitionSystem] that failed.
    pub fn to_recipe_failure(self, sys: &dyn TransitionSystem) -> SystemRecipeFailure {
        SystemRecipeFailure::Inconsistent(self, System::from(sys))
    }

    /// Converts this [ConsistencyFailure] that occured during the construction of a [Quotient](crate::transition_systems::Quotient) into a [SystemRecipeFailure] given the two [TransitionSystem]s that failed.
    pub fn to_rfq(self, t: &TransitionSystemPtr, s: &TransitionSystemPtr) -> SystemRecipeFailure {
        SystemRecipeFailure::Inconsistent(
            self,
            System::from_composite_system(t.as_ref(), s.as_ref(), SystemType::Quotient),
        )
    }
}

/// Represents how a [TransitionSystem] named `system` failed to be deterministic for `action` in `state`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterminismFailure {
    pub system: String,
    pub action: Action,
    pub state: SpecificState,
}

impl DeterminismFailure {
    /// Creates a new [DeterminismFailure] from a `system`, `action`, and `state`.
    pub fn from_system_and_action(
        system: &dyn TransitionSystem,
        action: impl Into<String>,
        state: &State,
    ) -> DeterminismResult {
        let action: String = action.into();
        let is_input = system.inputs_contain(&action);
        Err(DeterminismFailure {
            system: system.to_string(),
            action: Action::new(action, is_input),
            state: SpecificState::from_state(state, system),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxFailure {
    Unparsable { msg: String, path: String },
}

impl SyntaxFailure {
    pub fn unparsable(msg: impl Into<String>, path: impl Into<String>) -> SyntaxResult {
        Err(SyntaxFailure::Unparsable {
            msg: msg.into(),
            path: path.into(),
        })
    }
}

// ---------------------------- //
// --- Format Display Impl  --- //
// ---------------------------- //

impl std::fmt::Display for DeterminismFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The system '{}' is not deterministic in state {} for {}",
            self.system, self.state, self.action
        )
    }
}

impl std::fmt::Display for RefinementFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefinementFailure::CutsDelaySolutions {
                system,
                action,
                state,
            } => write!(
                f,
                "The refinement '{}' fails because delay solutions are cut in state {} for {}",
                system.name, state, action
            ),
            RefinementFailure::CannotMatch {
                system,
                action,
                state,
            } => write!(
                f,
                "The refinement '{}' fails in state {} because {} cannot be matched",
                system.name, state, action
            ),
            RefinementFailure::Precondition(precond) => precond.fmt(f),
        }
    }
}

impl std::fmt::Display for ConsistencyFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsistencyFailure::NoInitialState { system } => write!(
                f,
                "The system '{}' is inconsistent because it has no initial state",
                system
            ),
            ConsistencyFailure::NotDeterministic(determ) => determ.fmt(f),
            ConsistencyFailure::InconsistentLoc { system, state }
            | ConsistencyFailure::InconsistentFrom { system, state } => write!(
                f,
                "The system '{}' is inconsistent because there are no saving outputs from state {}",
                system, state
            ),
        }
    }
}

impl std::fmt::Display for SystemRecipeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemRecipeFailure::Action(action, system) => write!(
                f,
                "{} in {} is invalid: {}",
                system.sys_type, system.name, action
            ),
            SystemRecipeFailure::Inconsistent(cf, system) => {
                write!(
                    f,
                    "{} in {} is invalid: {}",
                    system.sys_type, system.name, cf
                )
            }
        }
    }
}

impl std::fmt::Display for ActionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionFailure::NotSubset(a, b) => {
                write!(f, "{} are not a subset of {}", a, b)
            }
            ActionFailure::NotDisjoint(a, b) => {
                write!(f, "{} are not disjoint from {}", a, b)
            }
        }
    }
}

impl std::fmt::Display for ClockReductionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown error occured during clock reduction")
    }
}

impl std::fmt::Display for RefinementPrecondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefinementPrecondition::EmptyChild { child, system } => write!(
                f,
                "The refinement '{}' fails because '{}' is empty",
                system.name, child
            ),
            RefinementPrecondition::EmptyInitialState { system } => write!(
                f,
                "The refinement '{}' fails because it has no initial state",
                system.name
            ),
            RefinementPrecondition::InconsistentChild(cf, system) => write!(
                f,
                "The refinement '{}' fails because of inconsistent system: {}",
                system.name, cf
            ),
            RefinementPrecondition::ActionMismatch(action, system) => {
                write!(
                    f,
                    "The refinement '{}' fails based on actions: {}",
                    system.name, action
                )
            }
        }
    }
}

impl std::fmt::Display for SyntaxFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntaxFailure::Unparsable { msg, path } => {
                write!(f, "The file '{}' could not be parsed: {}", path, msg)
            }
        }
    }
}

// ------------------------------- //
// - Ugly conversions begin here - //
// ----- You have been warned ---- //
// ------------------------------- //
mod conversions {
    use super::*;
    use std::error::Error;
    impl Error for SystemRecipeFailure {}
    impl Error for ClockReductionFailure {}
    impl Error for RefinementFailure {}
    impl Error for ConsistencyFailure {}
    impl Error for DeterminismFailure {}
    impl Error for SyntaxFailure {}

    impl From<RefinementPrecondition> for RefinementFailure {
        fn from(failure: RefinementPrecondition) -> Self {
            RefinementFailure::Precondition(failure)
        }
    }

    impl From<Box<RefinementPrecondition>> for RefinementFailure {
        fn from(failure: Box<RefinementPrecondition>) -> Self {
            RefinementFailure::Precondition(*failure)
        }
    }

    impl From<CompositionType> for SystemType {
        fn from(comp: CompositionType) -> Self {
            match comp {
                CompositionType::Quotient => SystemType::Quotient,
                CompositionType::Composition => SystemType::Composition,
                CompositionType::Conjunction => SystemType::Conjunction,
                CompositionType::Simple => SystemType::Simple,
            }
        }
    }

    impl From<DeterminismFailure> for ConsistencyFailure {
        fn from(failure: DeterminismFailure) -> Self {
            ConsistencyFailure::NotDeterministic(failure)
        }
    }

    impl From<SystemRecipeFailure> for QueryResult {
        fn from(res: SystemRecipeFailure) -> Self {
            QueryResult::RecipeFailure(res)
        }
    }

    impl From<PathResult> for QueryResult {
        fn from(res: PathResult) -> Self {
            QueryResult::Reachability(res)
        }
    }

    impl From<ConsistencyResult> for QueryResult {
        fn from(res: ConsistencyResult) -> Self {
            QueryResult::Consistency(res)
        }
    }

    impl From<SyntaxResult> for QueryResult {
        fn from(res: SyntaxResult) -> Self {
            QueryResult::Syntax(res)
        }
    }

    impl From<DeterminismResult> for QueryResult {
        fn from(res: DeterminismResult) -> Self {
            QueryResult::Determinism(res)
        }
    }

    impl From<RefinementResult> for QueryResult {
        fn from(res: RefinementResult) -> Self {
            QueryResult::Refinement(res)
        }
    }
}
