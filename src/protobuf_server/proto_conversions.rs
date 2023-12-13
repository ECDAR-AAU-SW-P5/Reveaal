use crate::protobuf_server::services::query_response::{
    ConsistencyFailure as ProtobufConsistencyFailure,
    DeterminismFailure as ProtobufDeterminismFailure, ModelFailure, ReachabilityFailure,
    ReachabilityPath, RefinementFailure as ProtobufRefinementFailure,
    SyntaxFailure as ProtobufSyntaxFailure,
};
use crate::protobuf_server::services::{
    self, clock::Clock as ProtoClockEnum, clock::ComponentClock as ProtoComponentClock,
    clock::SystemClock as ProtoSystemClock, ActionFailure as ProtobufActionFailure,
    BinaryLocationOperator, Clock as ProtoClock, ComponentInstance as ProtoSpecificComponent,
    Conjunction as ProtoConjunction, Constraint as ProtoConstraint,
    Disjunction as ProtoDisjunction, LeafLocation, LocationTree, State as ProtoState,
};
use crate::system::query_failures::*;
use crate::system::specifics::{
    SpecialLocation, SpecificClock, SpecificClockVar, SpecificComp, SpecificConjunction,
    SpecificConstraint, SpecificDecision, SpecificDisjunction, SpecificEdge, SpecificLocation,
    SpecificPath, SpecificState,
};

impl From<SpecificState> for ProtoState {
    fn from(state: SpecificState) -> Self {
        ProtoState {
            location_tree: Some(state.locations.into()),
            zone: Some(state.constraints.into()),
        }
    }
}

impl From<SpecificLocation> for LocationTree {
    fn from(loc: SpecificLocation) -> Self {
        use services::location_tree::NodeType;
        match loc {
            SpecificLocation::BranchLocation(left, right, op) => LocationTree {
                node_type: Some(NodeType::BinaryLocationOp(Box::new(
                    BinaryLocationOperator {
                        left: Some(Box::new((*left).into())),
                        right: Some(Box::new((*right).into())),
                        operator: match op {
                            SystemType::Conjunction => 0,
                            SystemType::Composition => 1,
                            SystemType::Quotient => 2,
                            SystemType::Refinement => 3,
                            _ => unreachable!(),
                        },
                    },
                ))),
            },
            SpecificLocation::ComponentLocation { comp, location_id } => LocationTree {
                node_type: Some(NodeType::LeafLocation(LeafLocation {
                    id: location_id,
                    component_instance: Some(comp.into()),
                })),
            },
            SpecificLocation::SpecialLocation(kind) => LocationTree {
                node_type: Some(NodeType::SpecialLocation(match kind {
                    SpecialLocation::Universal => 0,
                    SpecialLocation::Error => 1,
                })),
            },
        }
    }
}

impl From<LocationTree> for SpecificLocation {
    fn from(loc: LocationTree) -> Self {
        use services::location_tree::NodeType;
        match loc.node_type.unwrap() {
            NodeType::BinaryLocationOp(branch) => {
                use services::binary_location_operator::Operator;
                let sys_type = match branch.operator() {
                    Operator::Conjunction => SystemType::Conjunction,
                    Operator::Composition => SystemType::Composition,
                    Operator::Quotient => SystemType::Quotient,
                    Operator::Refinement => SystemType::Refinement,
                };

                let left = Box::new((*branch.left.unwrap()).into());
                let right = Box::new((*branch.right.unwrap()).into());

                SpecificLocation::BranchLocation(left, right, sys_type)
            }
            NodeType::LeafLocation(leaf) => SpecificLocation::ComponentLocation {
                comp: leaf.component_instance.unwrap().into(),
                location_id: leaf.id,
            },
            NodeType::SpecialLocation(special) => match special {
                0 => SpecificLocation::SpecialLocation(SpecialLocation::Universal),
                1 => SpecificLocation::SpecialLocation(SpecialLocation::Error),
                _ => panic!("Invalid special location id"),
            },
        }
    }
}

impl From<SpecificComp> for ProtoSpecificComponent {
    fn from(comp: SpecificComp) -> Self {
        ProtoSpecificComponent {
            component_name: comp.name,
            component_index: comp.id,
        }
    }
}

impl From<ProtoSpecificComponent> for SpecificComp {
    fn from(comp: ProtoSpecificComponent) -> Self {
        SpecificComp {
            name: comp.component_name,
            id: comp.component_index,
        }
    }
}

impl From<SpecificDisjunction> for ProtoDisjunction {
    fn from(disj: SpecificDisjunction) -> Self {
        ProtoDisjunction {
            conjunctions: disj
                .conjunctions
                .into_iter()
                .map(|conj| conj.into())
                .collect(),
        }
    }
}

impl From<SpecificConjunction> for ProtoConjunction {
    fn from(conj: SpecificConjunction) -> Self {
        ProtoConjunction {
            constraints: conj.constraints.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<SpecificConstraint> for ProtoConstraint {
    fn from(constraint: SpecificConstraint) -> Self {
        Self {
            x: Some(constraint.i.into()),
            y: Some(constraint.j.into()),
            strict: constraint.strict,
            c: constraint.c,
        }
    }
}

impl From<SpecificClockVar> for ProtoClock {
    fn from(clock: SpecificClockVar) -> Self {
        use std::convert::TryFrom;
        match clock {
            SpecificClockVar::Zero => Self {
                clock: Some(ProtoClockEnum::ZeroClock(Default::default())),
            },
            SpecificClockVar::ComponentClock(clock) => Self {
                clock: Some(ProtoClockEnum::ComponentClock(clock.into())),
            },
            SpecificClockVar::SystemClock(clock_index) => Self {
                clock: Some(ProtoClockEnum::SystemClock(ProtoSystemClock {
                    clock_index: u32::try_from(clock_index)
                        .expect("Could not fit clock index in u32"),
                })),
            },
        }
    }
}

impl From<SpecificClock> for ProtoComponentClock {
    fn from(clock: SpecificClock) -> Self {
        Self {
            component_instance: Some(clock.comp.into()),
            clock_name: clock.name,
        }
    }
}

fn state_action_to_proto(state: SpecificState, action: Action) -> services::StateAction {
    services::StateAction {
        state: Some(state.into()),
        action: action.name,
    }
}

impl From<ActionSet> for services::action_failure::ActionSet {
    fn from(set: ActionSet) -> Self {
        Self {
            actions: set.actions.into_iter().collect(),
            system: set.system,
            is_input: set.is_input,
        }
    }
}

impl From<ActionFailure> for ProtobufActionFailure {
    fn from(af: ActionFailure) -> Self {
        let enum_id = match af {
            ActionFailure::NotSubset(_, _) => 0, // As defined in the proto file
            ActionFailure::NotDisjoint(_, _) => 1, // As defined in the proto file
        };

        match af {
            ActionFailure::NotSubset(a, b) | ActionFailure::NotDisjoint(a, b) => {
                ProtobufActionFailure {
                    failure: enum_id,
                    action_sets: vec![a.into(), b.into()],
                }
            }
        }
    }
}

impl From<DeterminismFailure> for ProtobufDeterminismFailure {
    fn from(df: DeterminismFailure) -> Self {
        Self {
            system: df.system,
            failure_state: Some(state_action_to_proto(df.state, df.action)),
        }
    }
}

impl From<SyntaxFailure> for ProtobufSyntaxFailure {
    fn from(sf: SyntaxFailure) -> Self {
        match sf {
            SyntaxFailure::Unparsable { msg, path } => ProtobufSyntaxFailure { msg, path },
        }
    }
}

impl From<ConsistencyFailure> for ProtobufConsistencyFailure {
    fn from(cf: ConsistencyFailure) -> Self {
        use services::query_response::consistency_failure::Failure;
        match cf {
            ConsistencyFailure::NoInitialState { system } => ProtobufConsistencyFailure {
                system,
                failure: Some(Failure::NoInitialState(0)),
            },
            ConsistencyFailure::NotDeterministic(det) => {
                let df: ProtobufDeterminismFailure = det.into();
                ProtobufConsistencyFailure {
                    system: df.system.clone(),
                    failure: Some(Failure::Determinism(df)),
                }
            }
            ConsistencyFailure::InconsistentLoc { system, state }
            | ConsistencyFailure::InconsistentFrom { system, state } => {
                ProtobufConsistencyFailure {
                    system,
                    failure: Some(Failure::FailureState(state.into())),
                }
            }
        }
    }
}

impl From<RefinementFailure> for ProtobufRefinementFailure {
    fn from(rf: RefinementFailure) -> Self {
        use services::query_response::refinement_failure::Failure;
        use services::query_response::refinement_failure::RefinementStateFailure;

        match rf {
            RefinementFailure::CutsDelaySolutions {
                action,
                state,
                system,
            } => ProtobufRefinementFailure {
                system: system.name,
                failure: Some(Failure::RefinementState(RefinementStateFailure {
                    unmatched: 1,
                    state: Some(state_action_to_proto(state, action)),
                })),
            },

            RefinementFailure::CannotMatch {
                action,
                state,
                system,
            } => ProtobufRefinementFailure {
                system: system.name,
                failure: Some(Failure::RefinementState(RefinementStateFailure {
                    unmatched: 0,
                    state: Some(state_action_to_proto(state, action)),
                })),
            },

            RefinementFailure::Precondition(precond) => {
                use crate::system::query_failures::RefinementPrecondition::*;
                match precond {
                    EmptyChild { child, system } => ProtobufRefinementFailure {
                        system: system.name,
                        failure: Some(Failure::EmptySystem(child)),
                    },
                    EmptyInitialState { system } => ProtobufRefinementFailure {
                        system: system.name,
                        failure: Some(Failure::NoInitialState(0)),
                    },
                    InconsistentChild(cons, system) => ProtobufRefinementFailure {
                        system: system.name,
                        failure: Some(Failure::InconsistentChild(cons.into())),
                    },
                    ActionMismatch(action, system) => ProtobufRefinementFailure {
                        system: system.name,
                        failure: Some(Failure::ActionMismatch(action.into())),
                    },
                }
            }
        }
    }
}

impl From<SystemRecipeFailure> for ModelFailure {
    fn from(srf: SystemRecipeFailure) -> Self {
        use services::query_response::model_failure::Failure;
        match srf {
            SystemRecipeFailure::Action(action, sys) => ModelFailure {
                system: sys.name,
                failure: Some(Failure::ActionMismatch(action.into())),
            },
            SystemRecipeFailure::Inconsistent(cf, sys) => ModelFailure {
                system: sys.name,
                failure: Some(Failure::InconsistentConjunction(cf.into())),
            },
        }
    }
}

impl From<SpecificPath> for ReachabilityPath {
    fn from(path: SpecificPath) -> Self {
        use services::Path as ProtoPath;

        Self {
            path: Some(ProtoPath {
                decisions: path.path.into_iter().map(|d| d.into()).collect(),
            }),
        }
    }
}

impl From<SpecificDecision> for services::Decision {
    fn from(decision: SpecificDecision) -> Self {
        Self {
            source: Some(decision.source_state.into()),
            action: decision.action,
            edges: decision.edges.into_iter().map(|e| e.into()).collect(),
            destination: Some(decision.destination_state.into()),
        }
    }
}

impl From<PathFailure> for ReachabilityFailure {
    fn from(pf: PathFailure) -> Self {
        match pf {
            PathFailure::Unreachable => Self {
                failure: 0, // As defined in the proto file
            },
        }
    }
}

impl From<SpecificEdge> for services::Edge {
    fn from(edge: SpecificEdge) -> Self {
        Self {
            id: edge.edge_id,
            component_instance: Some(edge.comp.into()),
        }
    }
}
