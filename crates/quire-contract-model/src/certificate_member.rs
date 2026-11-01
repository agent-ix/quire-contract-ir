// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Certificate member names for the malformed certificate rejection locus (FR-070).

/// A member of a probability or zone certificate named by a malformed-certificate rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateMember {
    /// The `identity` member.
    Identity,
    /// The `node_id` member.
    NodeId,
    /// The `model` member.
    Model,
    /// The `ordinal` member.
    Ordinal,
    /// The `objective` member.
    Objective,
    /// The `kind` member.
    Kind,
    /// The `values` member.
    Values,
    /// The `ranking` member.
    Ranking,
    /// The `policy` member.
    Policy,
    /// The `components` member.
    Components,
    /// The `scale` member.
    Scale,
    /// The `bounds` member.
    Bounds,
    /// The `nodes` member.
    Nodes,
    /// The `edges` member.
    Edges,
    /// The `reasons` member.
    Reasons,
    /// The `divergence` member.
    Divergence,
    /// The `initials` member.
    Initials,
    /// The `admitted` member.
    Admitted,
    /// The `refinement` member.
    Refinement,
    /// The `refinement_node` member.
    RefinementNode,
    /// The `concrete_package` member.
    ConcretePackage,
    /// The `abstract_package` member.
    AbstractPackage,
    /// The `positions` member.
    Positions,
    /// The `taken` member.
    Taken,
    /// The `locked` member.
    Locked,
    /// The `start` member.
    Start,
    /// The `nested` member.
    Nested,
    /// The `task_product` member.
    TaskProduct,
    /// The `abstract_states` member.
    AbstractStates,
    /// The `automaton` member.
    Automaton,
    /// The `bias` member.
    Bias,
    /// The `bound` member.
    Bound,
    /// The `choice` member.
    Choice,
    /// The `clock` member.
    Clock,
    /// The `concrete` member.
    Concrete,
    /// The `constraint` member.
    Constraint,
    /// The `gain` member.
    Gain,
    /// The `history` member.
    History,
    /// The `lower` member.
    Lower,
    /// The `node` member.
    Node,
    /// The `rank` member.
    Rank,
    /// The `set` member.
    Set,
    /// The `state` member.
    State,
    /// The `strict` member.
    Strict,
    /// The `target` member.
    Target,
    /// The `transition` member.
    Transition,
    /// The `transitions` member.
    Transitions,
    /// The `type` member.
    Type,
    /// The `upper` member.
    Upper,
    /// The `value` member.
    Value,
    /// The `zone` member.
    Zone,
}

impl CertificateMember {
    /// Every member in the published schema sequence.
    pub const ALL: &'static [Self] = &[
        Self::Identity,
        Self::NodeId,
        Self::Model,
        Self::Ordinal,
        Self::Objective,
        Self::Kind,
        Self::Values,
        Self::Ranking,
        Self::Policy,
        Self::Components,
        Self::Scale,
        Self::Bounds,
        Self::Nodes,
        Self::Edges,
        Self::Reasons,
        Self::Divergence,
        Self::Initials,
        Self::Admitted,
        Self::Refinement,
        Self::RefinementNode,
        Self::ConcretePackage,
        Self::AbstractPackage,
        Self::Positions,
        Self::Taken,
        Self::Locked,
        Self::Start,
        Self::Nested,
        Self::TaskProduct,
        Self::AbstractStates,
        Self::Automaton,
        Self::Bias,
        Self::Bound,
        Self::Choice,
        Self::Clock,
        Self::Concrete,
        Self::Constraint,
        Self::Gain,
        Self::History,
        Self::Lower,
        Self::Node,
        Self::Rank,
        Self::Set,
        Self::State,
        Self::Strict,
        Self::Target,
        Self::Transition,
        Self::Transitions,
        Self::Type,
        Self::Upper,
        Self::Value,
        Self::Zone,
    ];

    /// The exact member name in the backend-provider certificate schema.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::NodeId => "node_id",
            Self::Model => "model",
            Self::Ordinal => "ordinal",
            Self::Objective => "objective",
            Self::Kind => "kind",
            Self::Values => "values",
            Self::Ranking => "ranking",
            Self::Policy => "policy",
            Self::Components => "components",
            Self::Scale => "scale",
            Self::Bounds => "bounds",
            Self::Nodes => "nodes",
            Self::Edges => "edges",
            Self::Reasons => "reasons",
            Self::Divergence => "divergence",
            Self::Initials => "initials",
            Self::Admitted => "admitted",
            Self::Refinement => "refinement",
            Self::RefinementNode => "refinement_node",
            Self::ConcretePackage => "concrete_package",
            Self::AbstractPackage => "abstract_package",
            Self::Positions => "positions",
            Self::Taken => "taken",
            Self::Locked => "locked",
            Self::Start => "start",
            Self::Nested => "nested",
            Self::TaskProduct => "task_product",
            Self::AbstractStates => "abstract_states",
            Self::Automaton => "automaton",
            Self::Bias => "bias",
            Self::Bound => "bound",
            Self::Choice => "choice",
            Self::Clock => "clock",
            Self::Concrete => "concrete",
            Self::Constraint => "constraint",
            Self::Gain => "gain",
            Self::History => "history",
            Self::Lower => "lower",
            Self::Node => "node",
            Self::Rank => "rank",
            Self::Set => "set",
            Self::State => "state",
            Self::Strict => "strict",
            Self::Target => "target",
            Self::Transition => "transition",
            Self::Transitions => "transitions",
            Self::Type => "type",
            Self::Upper => "upper",
            Self::Value => "value",
            Self::Zone => "zone",
        }
    }
}
