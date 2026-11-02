// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Certificate member names for the malformed certificate rejection locus (FR-070).

macro_rules! certificate_members {
    ($( $variant:ident => $spelling:literal, )+) => {
        /// A member of a probability or zone certificate named by a malformed-certificate rejection.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum CertificateMember {
            $( #[doc = concat!("The `", $spelling, "` member.")] $variant, )+
        }

        impl CertificateMember {
            /// Every member in the published schema sequence.
            pub const ALL: &'static [Self] = &[$( Self::$variant, )+];

            /// The exact member name in the backend-provider certificate schema.
            pub fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $spelling, )+
                }
            }
        }
    };
}

certificate_members! {
    Identity => "identity",
    NodeId => "node_id",
    Model => "model",
    Ordinal => "ordinal",
    Objective => "objective",
    Kind => "kind",
    Values => "values",
    Ranking => "ranking",
    Policy => "policy",
    Components => "components",
    Scale => "scale",
    Bounds => "bounds",
    Nodes => "nodes",
    Edges => "edges",
    Reasons => "reasons",
    Divergence => "divergence",
    Initials => "initials",
    Admitted => "admitted",
    Refinement => "refinement",
    RefinementNode => "refinement_node",
    ConcretePackage => "concrete_package",
    AbstractPackage => "abstract_package",
    Positions => "positions",
    Taken => "taken",
    Locked => "locked",
    Start => "start",
    Nested => "nested",
    TaskProduct => "task_product",
    AbstractStates => "abstract_states",
    Automaton => "automaton",
    Bias => "bias",
    Bound => "bound",
    Choice => "choice",
    Clock => "clock",
    Concrete => "concrete",
    Constraint => "constraint",
    Gain => "gain",
    History => "history",
    Lower => "lower",
    Node => "node",
    Rank => "rank",
    Set => "set",
    State => "state",
    Strict => "strict",
    Target => "target",
    Transition => "transition",
    Transitions => "transitions",
    Type => "type",
    Upper => "upper",
    Value => "value",
    Zone => "zone",
}
