//! The fixed outer body grammar of the twenty finite global protocol operators.
//!
//! The protocol step runs after refinement and timed models, before operations.

use super::model_members::{Budget, ModelOwners};
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedPackageLockV2, CheckedPackageV2,
    CheckedSelectionRole, CheckedTerm, CompositeTypeForm, LiteralKind, ModelForm, ProtocolForm,
    RawSemanticNodeV2, RelationForm, ScalarTypeForm, StateClauseKind, StateForm, TemporalForm,
    ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedProtocolDecisionRead, CheckedProtocolRefusalDetail,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use ix_trace_rs::string_edge;
use num_bigint::BigInt;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The activation selected by an admitted protocol clause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedProtocolActivation {
    /// The protocol activates at its origin.
    Origin,
    /// Each occurrence of the named trigger activates the protocol.
    Each {
        /// The admitted trigger parameter node.
        trigger: CheckedNodeId,
    },
}

/// One required temporal clause for an admitted protocol activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedProtocolActivationRow {
    /// The admitted protocol clause node.
    pub clause: CheckedNodeId,
    /// The clause's authored activation.
    pub activation: CheckedProtocolActivation,
    /// The admitted temporal clause named in `requirements`.
    pub required_temporal: CheckedNodeId,
}

impl CheckedPackageV2 {
    /// Derives every local clause's activation requirements from the admitted graph.
    ///
    /// The rows are ordered by clause and then required temporal node digest.
    /// This read-only view does not charge reader work or change package identity.
    pub fn protocol_activation_relation(&self) -> Vec<CheckedProtocolActivationRow> {
        let mut rows = Vec::new();
        for (position, node) in self.wire.semantic_graph.nodes.iter().enumerate() {
            if self.kinds[position] != CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause) {
                continue;
            }
            let CheckedTerm::Application { arguments, .. } = &self.terms[position] else {
                unreachable!("admitted protocol clause has an application body");
            };
            let activation = binding_value(arguments, "activation")
                .expect("admitted protocol clause has an activation");
            let activation = match group_value(activation, "kind").and_then(text_value) {
                Some("origin") => CheckedProtocolActivation::Origin,
                Some("each") => CheckedProtocolActivation::Each {
                    trigger: reference_target(
                        group_value(activation, "trigger")
                            .expect("admitted each activation has a trigger"),
                    )
                    .expect("admitted each trigger is a reference")
                    .clone(),
                },
                _ => unreachable!("admitted activation has a closed kind"),
            };
            let requirements = binding_value(arguments, "requirements")
                .expect("admitted protocol clause has requirements");
            for required_temporal in listed_targets(requirements) {
                rows.push(CheckedProtocolActivationRow {
                    clause: node.node_id.clone(),
                    activation: activation.clone(),
                    required_temporal: required_temporal.clone(),
                });
            }
        }
        rows.sort_unstable_by(|left, right| {
            left.clause.digest.cmp(&right.clause.digest).then_with(|| {
                left.required_temporal
                    .digest
                    .cmp(&right.required_temporal.digest)
            })
        });
        rows
    }
}

/// A defect in the application envelope or an operator's fixed bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProtocolBodyDefect {
    /// The node is not a protocol application, or its result type differs.
    OperatorIneligible,
    /// The profile operator is absent, unknown, or belongs to another form.
    MemberMismatch,
    /// The required protocol profile law is absent.
    LawMissing,
    /// The law has another role.
    LawMismatch,
    /// The law definition differs from the selected profile.
    LawUnselected,
    /// An argument name, position, or term shape differs from its layout.
    MalformedLayout,
    /// A set repeats an entry or is not in ascending node-key order.
    NoncanonicalSet,
}

/// A malformed protocol wire shape, with the offending enum term when known.
pub(super) enum ProtocolLayoutWireDefect {
    Body,
    NegativeQueueCapacity,
    NegativeBound {
        argument: usize,
        member: usize,
        binding: bool,
    },
    EnumValue {
        argument: usize,
        member: Option<(usize, bool)>,
    },
}

#[derive(Clone, Copy)]
enum Shape {
    Reference,
    ReferenceOrText,
    Term,
    Text,
    Integer,
    Boolean,
    BooleanCandidate,
    References,
    Texts,
    OptionalReference,
    OptionalText,
    OptionalInteger,
    OptionalBooleanCandidate,
    Group,
    Records,
}

#[derive(Clone, Copy)]
struct Binding {
    name: &'static str,
    shape: Shape,
}

const fn field(name: &'static str, shape: Shape) -> Binding {
    Binding { name, shape }
}

const CLAUSE: &[Binding] = &[
    field("over", Shape::Reference),
    field("activation", Shape::Group),
    field("captures", Shape::Records),
    field("roles", Shape::References),
    field("relationships", Shape::References),
    field("channels", Shape::References),
    field("queues", Shape::References),
    field("requirements", Shape::References),
    field("compensations", Shape::References),
    field("terminal", Shape::OptionalReference),
    field("scheduling", Shape::OptionalText),
    field("run", Shape::Reference),
    field("finish", Shape::Reference),
];
const ROLE: &[Binding] = &[
    field("model", Shape::ReferenceOrText),
    field("replication", Shape::Group),
];
const CHANNEL: &[Binding] = &[
    field("from", Shape::Reference),
    field("to", Shape::Reference),
    field("carries", Shape::Reference),
    field("ordering", Shape::Group),
    field("delivery", Shape::Text),
];
const QUEUE: &[Binding] = &[
    field("channel", Shape::Reference),
    field("capacity", Shape::Group),
    field("overflow", Shape::Text),
    field("loss", Shape::OptionalText),
];
const FINISH: &[Binding] = &[
    field("binder", Shape::Reference),
    field("constraint", Shape::BooleanCandidate),
];
const TERMINAL: &[Binding] = &[
    field("mode", Shape::Text),
    field("condition", Shape::OptionalBooleanCandidate),
];
const SEQUENCE: &[Binding] = &[field("children", Shape::References)];
const CHOICE: &[Binding] = &[
    field("owner", Shape::Reference),
    field("visible", Shape::References),
    field("cases", Shape::Records),
];
const PARALLEL: &[Binding] = &[
    field("memory", Shape::OptionalText),
    field("branches", Shape::Records),
    field("join", Shape::Group),
    field("joined", Shape::Texts),
    field("outstanding", Shape::OptionalText),
];
const REPEAT: &[Binding] = &[
    field("owner", Shape::Reference),
    field("visible", Shape::References),
    field("maximum", Shape::OptionalInteger),
    field("proof", Shape::Group),
    field("guard", Shape::BooleanCandidate),
    field("body", Shape::Reference),
    field("exhausted", Shape::OptionalReference),
];
const AWAIT: &[Binding] = &[
    field("after", Shape::Group),
    field("clock", Shape::Text),
    field("within", Shape::Group),
    field("event", Shape::Reference),
    field("then", Shape::Reference),
    field("timeout", Shape::Reference),
];
const CHECK: &[Binding] = &[field("condition", Shape::BooleanCandidate)];
const SEND: &[Binding] = &[
    field("channel", Shape::Reference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::BooleanCandidate),
];
const RECEIVE: &[Binding] = &[
    field("channel", Shape::Reference),
    field("of", Shape::Reference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::BooleanCandidate),
];
const ATTEMPT: &[Binding] = &[
    field("role", Shape::Reference),
    field("operation", Shape::Reference),
    field("ordering", Shape::OptionalText),
    field("contracts", Shape::References),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::BooleanCandidate),
];
const EFFECT: &[Binding] = &[
    field("of", Shape::Reference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::BooleanCandidate),
];
const EVENT: &[Binding] = &[
    field("role", Shape::Reference),
    field("compensation", Shape::OptionalReference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::BooleanCandidate),
];
const COMMIT: &[Binding] = &[
    field("role", Shape::Reference),
    field("binder", Shape::Reference),
    field("constraint", Shape::BooleanCandidate),
];
const FENCE: &[Binding] = &[field("ordering", Shape::Text)];
const COMPENSATE: &[Binding] = &[
    field("forward", Shape::Reference),
    field("forward_binder", Shape::Reference),
    field("role", Shape::Reference),
    field("operation", Shape::Reference),
    field("clock", Shape::Text),
    field("registration_captures", Shape::Records),
    field("trigger", Shape::Group),
    field("activation_captures", Shape::Records),
    field("within", Shape::Group),
    field("attempts", Shape::Group),
    field("retry", Shape::Group),
    field("commit", Shape::OptionalReference),
    field("recovery", Shape::Group),
];

fn layout(form: ProtocolForm, operator: &str) -> Option<&'static [Binding]> {
    Some(match (form, operator) {
        (ProtocolForm::ProtocolClause, "protocol") => CLAUSE,
        (ProtocolForm::Role, "role") => ROLE,
        (ProtocolForm::Channel, "channel") => CHANNEL,
        (ProtocolForm::Queue, "queue") => QUEUE,
        (ProtocolForm::Obligation, "finish") => FINISH,
        (ProtocolForm::Obligation, "terminal") => TERMINAL,
        (ProtocolForm::Control, "sequence") => SEQUENCE,
        (ProtocolForm::Control, "choice") => CHOICE,
        (ProtocolForm::Control, "parallel") => PARALLEL,
        (ProtocolForm::Control, "repeat") => REPEAT,
        (ProtocolForm::Control, "await") => AWAIT,
        (ProtocolForm::Control, "check") => CHECK,
        (ProtocolForm::Control, "send") => SEND,
        (ProtocolForm::Control, "receive") => RECEIVE,
        (ProtocolForm::Control, "attempt") => ATTEMPT,
        (ProtocolForm::Control, "effect") => EFFECT,
        (ProtocolForm::Control, "event") => EVENT,
        (ProtocolForm::Control, "commit") => COMMIT,
        (ProtocolForm::Control, "fence") => FENCE,
        (ProtocolForm::Compensation, "compensate") => COMPENSATE,
        _ => return None,
    })
}

fn members(value: &CheckedTerm) -> Option<&[CheckedTerm]> {
    let CheckedTerm::Aggregate { members } = value else {
        return None;
    };
    Some(members)
}

fn shape_fits(value: &CheckedTerm, shape: Shape) -> bool {
    match shape {
        Shape::Reference => matches!(value, CheckedTerm::Reference { .. }),
        Shape::ReferenceOrText => {
            matches!(value, CheckedTerm::Reference { .. }) || shape_fits(value, Shape::Text)
        }
        Shape::Term => matches!(
            value,
            CheckedTerm::Reference { .. }
                | CheckedTerm::Literal { .. }
                | CheckedTerm::DependencyReference { .. }
        ),
        Shape::Text => matches!(
            value,
            CheckedTerm::Literal {
                value_kind: LiteralKind::Text,
                value,
                ..
            } if value.as_str().is_some()
        ),
        Shape::Integer => matches!(
            value,
            CheckedTerm::Literal {
                value_kind: LiteralKind::Integer,
                ..
            }
        ),
        Shape::Boolean => {
            matches!(value, CheckedTerm::Reference { .. })
                || matches!(value, CheckedTerm::Literal { value_kind: LiteralKind::Boolean, value, .. } if value.as_bool().is_some())
        }
        Shape::BooleanCandidate => matches!(
            value,
            CheckedTerm::Reference { .. } | CheckedTerm::Literal { .. }
        ),
        Shape::References => members(value)
            .is_some_and(|items| items.iter().all(|item| shape_fits(item, Shape::Reference))),
        Shape::Texts => members(value)
            .is_some_and(|items| items.iter().all(|item| shape_fits(item, Shape::Text))),
        Shape::OptionalReference => members(value).is_some_and(|items| {
            items.len() <= 1 && items.iter().all(|item| shape_fits(item, Shape::Reference))
        }),
        Shape::OptionalText => members(value).is_some_and(|items| {
            items.len() <= 1 && items.iter().all(|item| shape_fits(item, Shape::Text))
        }),
        Shape::OptionalInteger => members(value).is_some_and(|items| {
            items.len() <= 1 && items.iter().all(|item| shape_fits(item, Shape::Integer))
        }),
        Shape::OptionalBooleanCandidate => members(value).is_some_and(|items| {
            items.len() <= 1
                && items
                    .iter()
                    .all(|item| shape_fits(item, Shape::BooleanCandidate))
        }),
        Shape::Group => members(value).is_some_and(|items| {
            items
                .iter()
                .all(|item| matches!(item, CheckedTerm::Binding { .. }))
        }),
        Shape::Records => members(value).is_some_and(|items| {
            items.iter().all(|item| {
                members(item).is_some_and(|record| {
                    record
                        .iter()
                        .all(|entry| matches!(entry, CheckedTerm::Binding { .. }))
                })
            })
        }),
    }
}

fn named_fields<'a>(term: &'a CheckedTerm, names: &[&str]) -> Option<Vec<&'a CheckedTerm>> {
    let fields = members(term)?;
    if fields.len() != names.len() {
        return None;
    }
    fields
        .iter()
        .zip(names)
        .map(|(field, expected)| match field {
            CheckedTerm::Binding { name, value } if name.as_ref() == *expected => {
                Some(value.as_ref())
            }
            _ => None,
        })
        .collect()
}

fn text_value(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}

#[string_edge]
fn clause_activation_fits(term: &CheckedTerm) -> bool {
    let Some(fields) = members(term) else {
        return false;
    };
    let Some(CheckedTerm::Binding { name, value }) = fields.first() else {
        return false;
    };
    if name.as_ref() != "kind" {
        return false;
    }
    match text_value(value) {
        Some("origin") => fields.len() == 1,
        Some("each") => {
            let expected = if fields.len() == 2 {
                &["kind", "trigger"][..]
            } else {
                &["kind", "trigger", "guard"][..]
            };
            named_fields(term, expected).is_some_and(|values| {
                shape_fits(values[1], Shape::Reference)
                    && (values.len() == 2 || shape_fits(values[2], Shape::Boolean))
            })
        }
        _ => false,
    }
}

fn clause_details_fit(arguments: &[CheckedTerm]) -> bool {
    let Some(activation) = argument(arguments, 1) else {
        return false;
    };
    let captures_fit = argument(arguments, 2)
        .and_then(members)
        .is_some_and(|captures| {
            captures.iter().all(|record| {
                group_fields_fit(
                    record,
                    &[("binder", Shape::Reference), ("initializer", Shape::Term)],
                )
            })
        });
    let scheduling_fit = argument(arguments, 10)
        .and_then(members)
        .is_some_and(|values| {
            matches!(values, [] | [CheckedTerm::Literal { .. }])
                && values
                    .first()
                    .is_none_or(|value| text_value(value) == Some("adversarial"))
        });
    clause_activation_fits(activation) && captures_fit && scheduling_fit
}

fn reference_set_is_ordered(term: &CheckedTerm) -> bool {
    members(term).is_some_and(|items| {
        items.windows(2).all(|pair| match pair {
            [CheckedTerm::Reference { target: left }, CheckedTerm::Reference { target: right }] => {
                left.digest.as_bytes() < right.digest.as_bytes()
            }
            _ => false,
        })
    })
}

fn related_records_are_ordered(term: &CheckedTerm, bytes: u64) -> bool {
    let Some(records) = members(term) else {
        return false;
    };
    let mut previous: Option<(String, Vec<u8>, Vec<u8>)> = None;
    for record in records {
        let Some([relationship, from, to]) = named_fields(record, &["relationship", "from", "to"])
            .and_then(|fields| <[_; 3]>::try_from(fields).ok())
        else {
            return false;
        };
        let Some(relationship) = reference_target(relationship) else {
            return false;
        };
        let Ok(from) = quire_canonical::to_vec(from, quire_canonical::Limits::new(bytes)) else {
            return false;
        };
        let Ok(to) = quire_canonical::to_vec(to, quire_canonical::Limits::new(bytes)) else {
            return false;
        };
        let key = (relationship.digest.to_string(), from, to);
        if previous.as_ref().is_some_and(|previous| previous >= &key) {
            return false;
        }
        previous = Some(key);
    }
    true
}

fn argument(arguments: &[CheckedTerm], position: usize) -> Option<&CheckedTerm> {
    match arguments.get(position)? {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}

fn group_fields_fit(term: &CheckedTerm, fields: &[(&str, Shape)]) -> bool {
    members(term).is_some_and(|values| {
        values.len() == fields.len()
            && values.iter().zip(fields).all(|(value, (name, shape))| {
                matches!(value, CheckedTerm::Binding { name: actual, value }
                    if actual.as_ref() == *name && shape_fits(value, *shape))
            })
    })
}

#[string_edge]
fn local_details_fit(form: ProtocolForm, operator: &str, arguments: &[CheckedTerm]) -> bool {
    match (form, operator) {
        (ProtocolForm::Control, "choice") => {
            argument(arguments, 2)
                .and_then(members)
                .is_some_and(|cases| {
                    cases.len() >= 2
                        && cases.iter().all(|case| {
                            group_fields_fit(
                                case,
                                &[
                                    ("label", Shape::Text),
                                    ("guard", Shape::BooleanCandidate),
                                    ("body", Shape::Reference),
                                ],
                            )
                        })
                })
        }
        (ProtocolForm::Control, "parallel") => {
            let branches_fit = argument(arguments, 1)
                .and_then(members)
                .is_some_and(|branches| {
                    branches.len() >= 2
                        && branches.iter().all(|branch| {
                            group_fields_fit(
                                branch,
                                &[("label", Shape::Text), ("body", Shape::Reference)],
                            )
                        })
                });
            let join = argument(arguments, 2);
            let join_fit = join.is_some_and(|join| match argument_text(join, 0) {
                Some("all" | "any") => group_fields_fit(join, &[("policy", Shape::Text)]),
                Some("quorum") => {
                    group_fields_fit(join, &[("policy", Shape::Text), ("count", Shape::Integer)])
                }
                Some("predicate") => group_fields_fit(
                    join,
                    &[("policy", Shape::Text), ("predicate", Shape::Reference)],
                ),
                _ => false,
            });
            branches_fit && join_fit
        }
        (ProtocolForm::Control, "repeat") => {
            let maximum = argument(arguments, 2).and_then(members);
            let exhausted = argument(arguments, 6).and_then(members);
            let proof = argument(arguments, 3);
            matches!(
                (maximum, exhausted),
                (Some([]), Some([])) | (Some([_]), Some([_]))
            ) && proof.is_some_and(|proof| {
                members(proof).is_some_and(|fields| {
                    fields.is_empty()
                        || group_fields_fit(
                            proof,
                            &[
                                ("invariant", Shape::BooleanCandidate),
                                ("variant", Shape::Term),
                            ],
                        )
                })
            })
        }
        (ProtocolForm::Control, "await") => {
            argument(arguments, 0).is_some_and(|after| {
                group_fields_fit(
                    after,
                    &[("kind", Shape::Text), ("target", Shape::Reference)],
                ) && matches!(argument_text(after, 0), Some("event" | "compensation"))
            }) && argument(arguments, 2).is_some_and(|within| {
                group_fields_fit(
                    within,
                    &[("lower", Shape::Integer), ("upper", Shape::Integer)],
                )
            })
        }
        (ProtocolForm::Control, "send" | "receive") => {
            let related = if operator == "receive" { 3 } else { 2 };
            argument(arguments, related)
                .and_then(members)
                .is_some_and(|records| {
                    records.iter().all(|record| {
                        group_fields_fit(
                            record,
                            &[
                                ("relationship", Shape::Reference),
                                ("from", Shape::Term),
                                ("to", Shape::Term),
                            ],
                        )
                    })
                })
        }
        (ProtocolForm::Control, "attempt" | "effect" | "event") => {
            let related = match operator {
                "attempt" => 5,
                "effect" => 2,
                "event" => 3,
                _ => unreachable!("validated protocol operator"),
            };
            argument(arguments, related)
                .and_then(members)
                .is_some_and(|records| {
                    records.iter().all(|record| {
                        group_fields_fit(
                            record,
                            &[
                                ("relationship", Shape::Reference),
                                ("from", Shape::Term),
                                ("to", Shape::Term),
                            ],
                        )
                    })
                })
        }
        (ProtocolForm::Compensation, "compensate") => {
            let captures_fit = [5, 7].into_iter().all(|ordinal| {
                argument(arguments, ordinal)
                    .and_then(members)
                    .is_some_and(|records| {
                        records.iter().all(|record| {
                            group_fields_fit(
                                record,
                                &[("binder", Shape::Reference), ("initializer", Shape::Term)],
                            )
                        })
                    })
            });
            captures_fit
                && argument(arguments, 6).is_some_and(|trigger| {
                    group_fields_fit(
                        trigger,
                        &[
                            ("binder", Shape::Reference),
                            ("guard", Shape::BooleanCandidate),
                        ],
                    )
                })
                && argument(arguments, 8).is_some_and(|within| {
                    group_fields_fit(
                        within,
                        &[("lower", Shape::Integer), ("upper", Shape::Integer)],
                    )
                })
                && argument(arguments, 9).is_some_and(|attempts| {
                    group_fields_fit(
                        attempts,
                        &[
                            ("maximum", Shape::Integer),
                            ("record_type", Shape::Reference),
                        ],
                    )
                })
                && argument(arguments, 10).is_some_and(|retry| {
                    group_fields_fit(
                        retry,
                        &[
                            ("earlier", Shape::Reference),
                            ("later", Shape::Reference),
                            ("relation", Shape::BooleanCandidate),
                        ],
                    )
                })
                && argument(arguments, 12).is_some_and(|recovery| {
                    group_fields_fit(
                        recovery,
                        &[
                            ("binder", Shape::Reference),
                            ("predicate", Shape::BooleanCandidate),
                        ],
                    )
                })
        }
        (ProtocolForm::ProtocolClause, "protocol") => clause_details_fit(arguments),
        (ProtocolForm::Role, "role") => {
            let Some(replication) = argument(arguments, 1) else {
                return false;
            };
            members(replication).is_some_and(|fields| fields.is_empty())
                || group_fields_fit(
                    replication,
                    &[
                        ("population", Shape::Text),
                        ("max", Shape::Integer),
                        ("binder", Shape::Reference),
                        ("lifetime", Shape::Text),
                    ],
                ) && matches!(argument_text(replication, 3), Some("workflow" | "scope"))
                || group_fields_fit(
                    replication,
                    &[
                        ("population", Shape::Text),
                        ("max", Shape::Integer),
                        ("binder", Shape::Reference),
                        ("lifetime", Shape::Text),
                        ("until", Shape::BooleanCandidate),
                    ],
                ) && argument_text(replication, 3) == Some("until")
        }
        (ProtocolForm::Channel, "channel") => {
            let delivery = argument(arguments, 4).and_then(text_value);
            matches!(
                delivery,
                Some("unknown" | "at-most-once" | "at-least-once" | "exactly-once-premise")
            ) && argument(arguments, 3).is_some_and(|ordering| {
                group_fields_fit(ordering, &[("kind", Shape::Text)])
                    && argument_text(ordering, 0) == Some("unordered")
                    || group_fields_fit(
                        ordering,
                        &[
                            ("kind", Shape::Text),
                            ("binder", Shape::Reference),
                            ("key", Shape::Term),
                        ],
                    ) && argument_text(ordering, 0) == Some("fifo")
            })
        }
        (ProtocolForm::Queue, "queue") => {
            let Some(capacity) = argument(arguments, 1) else {
                return false;
            };
            let capacity_fits = group_fields_fit(
                capacity,
                &[("kind", Shape::Text), ("value", Shape::Integer)],
            ) && argument_text(capacity, 0) == Some("literal")
                || group_fields_fit(capacity, &[("kind", Shape::Text), ("name", Shape::Text)])
                    && argument_text(capacity, 0) == Some("symbolic");
            let overflow = argument(arguments, 2).and_then(text_value);
            let loss = argument(arguments, 3).and_then(members);
            capacity_fits
                && match (overflow, loss) {
                    (Some("reject" | "block"), Some([])) => true,
                    (Some("loss"), Some([selector])) => {
                        matches!(text_value(selector), Some("head" | "tail" | "new"))
                    }
                    _ => false,
                }
        }
        (ProtocolForm::Obligation, "terminal") => {
            let mode = argument(arguments, 0).and_then(text_value);
            let condition = argument(arguments, 1).and_then(members);
            matches!(
                (mode, condition),
                (Some("any"), Some([])) | (Some("when"), Some([_]))
            )
        }
        _ => true,
    }
}

fn argument_text(term: &CheckedTerm, position: usize) -> Option<&str> {
    members(term)?
        .get(position)
        .and_then(|member| match member {
            CheckedTerm::Binding { value, .. } => text_value(value),
            _ => None,
        })
}

fn validate_envelope<'a>(
    form: ProtocolForm,
    body: &'a CheckedTerm,
    semantic_type: &CheckedNodeId,
    selected_profile: Option<&CheckedArtifactRef>,
) -> Result<(&'a str, &'a [CheckedTerm]), ProtocolBodyDefect> {
    use ProtocolBodyDefect as D;
    let CheckedTerm::Application {
        operator: ApplicationOperator::ProtocolControl,
        operation,
        result_type,
        arguments,
    } = body
    else {
        return Err(D::OperatorIneligible);
    };
    if result_type != semantic_type
        || operation.pointer("/identity").and_then(Value::as_str)
            != Some("quire.op.protocol.control")
    {
        return Err(D::OperatorIneligible);
    }
    let laws = operation
        .get("laws")
        .and_then(Value::as_array)
        .ok_or(D::LawMissing)?;
    let law = laws.first().ok_or(D::LawMissing)?;
    if laws.len() != 1 {
        return Err(D::MalformedLayout);
    }
    if law.get("role").and_then(Value::as_str) != Some("protocol_profile") {
        return Err(D::LawMismatch);
    }
    if !selected_profile.is_some_and(|selected| {
        law.get("definition")
            .and_then(Value::as_object)
            .is_some_and(|definition| {
                definition.len() == 2
                    && definition.get("authority").and_then(Value::as_str)
                        == Some(selected.authority.as_ref())
                    && definition.get("identity").and_then(Value::as_str)
                        == Some(selected.identity.as_ref())
            })
    }) {
        return Err(D::LawUnselected);
    }
    if !operation.get("mode").is_some_and(Value::is_null)
        || !operation
            .get("leaves")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    {
        return Err(D::MalformedLayout);
    }
    let member = operation.get("member").ok_or(D::MemberMismatch)?;
    let operator = member
        .get("operator")
        .and_then(Value::as_str)
        .ok_or(D::MemberMismatch)?;
    if member.as_object().is_none_or(|object| object.len() != 2)
        || member.get("kind").and_then(Value::as_str) != Some("profile_operator")
    {
        return Err(D::MemberMismatch);
    }
    layout(form, operator).ok_or_else(|| {
        if ProtocolForm::ALL
            .iter()
            .any(|&candidate| layout(candidate, operator).is_some())
        {
            D::OperatorIneligible
        } else {
            D::MemberMismatch
        }
    })?;
    Ok((operator, arguments))
}

#[string_edge]
fn validate_layout(
    form: ProtocolForm,
    operator: &str,
    arguments: &[CheckedTerm],
) -> Result<(), ProtocolBodyDefect> {
    use ProtocolBodyDefect as D;
    let Some(expected) = layout(form, operator) else {
        return Err(D::OperatorIneligible);
    };
    if arguments.len() != expected.len()
        || !arguments.iter().zip(expected).all(|(argument, binding)| {
            matches!(argument, CheckedTerm::Binding { name, value }
                if name.as_ref() == binding.name && shape_fits(value, binding.shape))
        })
    {
        return Err(D::MalformedLayout);
    }
    if !local_details_fit(form, operator, arguments) {
        return Err(D::MalformedLayout);
    }
    if form == ProtocolForm::ProtocolClause
        && (3..=8).any(|position| {
            argument(arguments, position).is_none_or(|set| !reference_set_is_ordered(set))
        })
        || form == ProtocolForm::Control
            && matches!(operator, "choice" | "repeat")
            && argument(arguments, 1).is_none_or(|set| !reference_set_is_ordered(set))
        || form == ProtocolForm::Control
            && operator == "attempt"
            && argument(arguments, 3).is_none_or(|set| !reference_set_is_ordered(set))
    {
        return Err(D::NoncanonicalSet);
    }
    Ok(())
}

#[string_edge]
fn validate_related_order(
    form: ProtocolForm,
    operator: &str,
    arguments: &[CheckedTerm],
    bytes: u64,
) -> Result<(), ProtocolBodyDefect> {
    let related = match (form, operator) {
        (ProtocolForm::Control, "send" | "effect") => Some(2),
        (ProtocolForm::Control, "receive" | "event") => Some(3),
        (ProtocolForm::Control, "attempt") => Some(5),
        _ => None,
    };
    if related.is_some_and(|ordinal| {
        argument(arguments, ordinal)
            .is_none_or(|records| !related_records_are_ordered(records, bytes))
    }) {
        return Err(ProtocolBodyDefect::NoncanonicalSet);
    }
    Ok(())
}

/// Strict-wire subset: known operators' fixed shapes, before any graph step.
/// Unknown or misplaced operators retain their protocol-step refusal.
pub(super) fn wire_layout_defect(
    form: ProtocolForm,
    body: &CheckedTerm,
) -> Option<ProtocolLayoutWireDefect> {
    let CheckedTerm::Application {
        operator: ApplicationOperator::ProtocolControl,
        operation,
        arguments,
        ..
    } = body
    else {
        return None;
    };
    if operation
        .get("laws")
        .and_then(Value::as_array)
        .is_some_and(|laws| laws.len() > 1)
        || !operation.get("mode").is_some_and(Value::is_null)
        || !operation
            .get("leaves")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
    {
        return Some(ProtocolLayoutWireDefect::Body);
    }
    let operator = operation
        .pointer("/member/operator")
        .and_then(Value::as_str)?;
    layout(form, operator)?;
    if let Some(defect) = enum_value_defect(form, operator, arguments) {
        return Some(defect);
    }
    if matches!(
        validate_layout(form, operator, arguments),
        Err(ProtocolBodyDefect::MalformedLayout)
    ) {
        return Some(ProtocolLayoutWireDefect::Body);
    }
    if (form, operator) == (ProtocolForm::Queue, "queue") {
        let capacity = argument(arguments, 1);
        if capacity
            .and_then(|capacity| group_value(capacity, "value"))
            .and_then(integer_value)
            .is_some_and(|value| value < BigInt::from(0))
        {
            return Some(ProtocolLayoutWireDefect::NegativeQueueCapacity);
        }
    }
    if (form, operator) == (ProtocolForm::Control, "repeat")
        && argument(arguments, 2)
            .and_then(members)
            .and_then(|items| items.first())
            .and_then(integer_value)
            .is_some_and(|value| value < BigInt::from(0))
    {
        return Some(ProtocolLayoutWireDefect::NegativeBound {
            argument: 2,
            member: 0,
            binding: false,
        });
    }
    if (form, operator) == (ProtocolForm::Control, "await") {
        for member in 0..2 {
            if argument(arguments, 2)
                .and_then(members)
                .and_then(|items| items.get(member))
                .and_then(|item| match item {
                    CheckedTerm::Binding { value, .. } => integer_value(value),
                    _ => None,
                })
                .is_some_and(|value| value < BigInt::from(0))
            {
                return Some(ProtocolLayoutWireDefect::NegativeBound {
                    argument: 2,
                    member,
                    binding: true,
                });
            }
        }
    }
    if (form, operator) == (ProtocolForm::Compensation, "compensate") {
        for member in 0..2 {
            if argument(arguments, 8)
                .and_then(members)
                .and_then(|items| items.get(member))
                .and_then(|item| match item {
                    CheckedTerm::Binding { value, .. } => integer_value(value),
                    _ => None,
                })
                .is_some_and(|value| value < BigInt::from(0))
            {
                return Some(ProtocolLayoutWireDefect::NegativeBound {
                    argument: 8,
                    member,
                    binding: true,
                });
            }
        }
    }
    None
}

fn invalid_enum_at(
    arguments: &[CheckedTerm],
    argument_position: usize,
    member: Option<(usize, bool)>,
    allowed: &[&str],
) -> Option<ProtocolLayoutWireDefect> {
    let mut term = argument(arguments, argument_position)?;
    if let Some((position, binding)) = member {
        term = members(term)?.get(position)?;
        if binding {
            let CheckedTerm::Binding { value, .. } = term else {
                return None;
            };
            term = value;
        }
    }
    let spelling = text_value(term)?;
    (!allowed.contains(&spelling)).then_some(ProtocolLayoutWireDefect::EnumValue {
        argument: argument_position,
        member,
    })
}

#[string_edge]
fn enum_value_defect(
    form: ProtocolForm,
    operator: &str,
    arguments: &[CheckedTerm],
) -> Option<ProtocolLayoutWireDefect> {
    match (form, operator) {
        (ProtocolForm::ProtocolClause, "protocol") => {
            invalid_enum_at(arguments, 1, Some((0, true)), &["origin", "each"])
                .or_else(|| invalid_enum_at(arguments, 10, Some((0, false)), &["adversarial"]))
        }
        (ProtocolForm::Role, "role") => invalid_enum_at(
            arguments,
            1,
            Some((3, true)),
            &["workflow", "scope", "until"],
        ),
        (ProtocolForm::Channel, "channel") => {
            invalid_enum_at(arguments, 3, Some((0, true)), &["unordered", "fifo"]).or_else(|| {
                invalid_enum_at(
                    arguments,
                    4,
                    None,
                    &[
                        "unknown",
                        "at-most-once",
                        "at-least-once",
                        "exactly-once-premise",
                    ],
                )
            })
        }
        (ProtocolForm::Queue, "queue") => {
            invalid_enum_at(arguments, 1, Some((0, true)), &["literal", "symbolic"])
                .or_else(|| invalid_enum_at(arguments, 2, None, &["reject", "block", "loss"]))
                .or_else(|| {
                    invalid_enum_at(arguments, 3, Some((0, false)), &["head", "tail", "new"])
                })
        }
        (ProtocolForm::Obligation, "terminal") => {
            invalid_enum_at(arguments, 0, None, &["any", "when"])
        }
        (ProtocolForm::Control, "parallel") => {
            let optional = |ordinal, allowed: &[&str]| {
                argument(arguments, ordinal)
                    .and_then(members)
                    .and_then(|values| values.first())
                    .and_then(text_value)
                    .filter(|value| !allowed.contains(value))
                    .map(|_| ProtocolLayoutWireDefect::EnumValue {
                        argument: ordinal,
                        member: Some((0, false)),
                    })
            };
            optional(0, &["sc", "tso", "ra"]).or_else(|| optional(4, &["continue", "cancel"]))
        }
        (ProtocolForm::Control, "await") => {
            invalid_enum_at(arguments, 0, Some((0, true)), &["event", "compensation"])
        }
        (ProtocolForm::Control, "attempt") => invalid_enum_at(
            arguments,
            2,
            Some((0, false)),
            &["relaxed", "acquire", "release", "acq_rel", "seq_cst"],
        ),
        (ProtocolForm::Control, "fence") => invalid_enum_at(
            arguments,
            0,
            None,
            &["acquire", "release", "acq_rel", "seq_cst"],
        ),
        _ => None,
    }
}

/// Checks the shared protocol application envelope and fixed local layout.
/// `selected_profile` is the lock's selected `protocol_profile` definition.
///
/// This has no graph context; target forms, binder scope and membership are
/// checked by the protocol reader step.
#[allow(dead_code)] // The reader will call the two ordered phases separately.
pub(super) fn validate_protocol_body(
    form: ProtocolForm,
    body: &CheckedTerm,
    semantic_type: &CheckedNodeId,
    selected_profile: Option<&CheckedArtifactRef>,
    bytes: u64,
) -> Result<(), ProtocolBodyDefect> {
    let (operator, arguments) = validate_envelope(form, body, semantic_type, selected_profile)?;
    validate_layout(form, operator, arguments)?;
    validate_related_order(form, operator, arguments, bytes)
}

fn body_defect(
    position: usize,
    node: &RawSemanticNodeV2,
    defect: ProtocolBodyDefect,
) -> ValidationFailure {
    let body = node_pointer(position).key("body");
    let (code, cause, path) = match defect {
        ProtocolBodyDefect::OperatorIneligible => {
            (Code::IllTyped, Some(Cause::OperatorIneligible), body)
        }
        ProtocolBodyDefect::MemberMismatch => (
            Code::InvalidPackage,
            Some(Cause::OperationMemberMismatch),
            body.key("operation").key("member"),
        ),
        ProtocolBodyDefect::LawMissing => (
            Code::InvalidPackage,
            Some(Cause::OperationLawMissing),
            body.key("operation").key("laws"),
        ),
        ProtocolBodyDefect::LawMismatch => (
            Code::InvalidPackage,
            Some(Cause::OperationLawMismatch),
            body.key("operation").key("laws").index(0).key("role"),
        ),
        ProtocolBodyDefect::LawUnselected => (
            Code::InvalidPackage,
            Some(Cause::OperationLawUnselected),
            body.key("operation").key("laws").index(0).key("definition"),
        ),
        ProtocolBodyDefect::MalformedLayout => (Code::MalformedWire, None, body.key("arguments")),
        ProtocolBodyDefect::NoncanonicalSet => {
            (Code::InvalidSemanticGraph, None, node_pointer(position))
        }
    };
    ValidationFailure::refused_at(code, path, cause, node.node_id.clone())
}

/// Checks name collisions involving a protocol declaration at reader step 3.
/// The earlier generic sweep has already refused collisions entirely among
/// non-protocol declarations.
fn validate_protocol_declaration_names(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    let mut holders: BTreeMap<&[Box<str>], bool> = BTreeMap::new();
    for &position in index.values() {
        if matches!(
            kinds.get(position),
            Some(CheckedNodeKind::ScalarType(
                ScalarTypeForm::Dimension | ScalarTypeForm::Unit
            ))
        ) {
            continue;
        }
        let Some(declaration) = nodes[position].declaration.as_ref() else {
            continue;
        };
        let name = declaration.qualified_name.as_slice();
        let protocol = matches!(kinds.get(position), Some(CheckedNodeKind::Protocol(_)));
        if let Some(&previous_protocol) = holders.get(name) {
            if protocol || previous_protocol {
                return Err(ValidationFailure::refused_at(
                    Code::AmbiguousDeclaration,
                    node_pointer(position)
                        .key("declaration")
                        .key("qualified_name"),
                    Some(Cause::AmbiguousName),
                    nodes[position].node_id.clone(),
                ));
            }
        } else {
            holders.insert(name, protocol);
        }
    }
    Ok(())
}

fn reference_target(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn binding_value<'a>(arguments: &'a [CheckedTerm], name: &str) -> Option<&'a CheckedTerm> {
    arguments.iter().find_map(|argument| match argument {
        CheckedTerm::Binding {
            name: actual,
            value,
        } if actual.as_ref() == name => Some(value.as_ref()),
        _ => None,
    })
}

fn listed_targets(term: &CheckedTerm) -> impl Iterator<Item = &CheckedNodeId> {
    members(term)
        .into_iter()
        .flatten()
        .filter_map(reference_target)
}

#[string_edge]
fn control_children<'a>(operator: &str, arguments: &'a [CheckedTerm]) -> Vec<&'a CheckedNodeId> {
    let mut children = Vec::new();
    match operator {
        "sequence" => {
            if let Some(value) = binding_value(arguments, "children") {
                children.extend(listed_targets(value));
            }
        }
        "choice" | "parallel" => {
            let field = if operator == "choice" {
                "cases"
            } else {
                "branches"
            };
            if let Some(records) = binding_value(arguments, field).and_then(members) {
                for record in records {
                    if let Some(target) = members(record)
                        .and_then(|fields| binding_value(fields, "body"))
                        .and_then(reference_target)
                    {
                        children.push(target);
                    }
                }
            }
        }
        "repeat" => {
            if let Some(target) = binding_value(arguments, "body").and_then(reference_target) {
                children.push(target);
            }
            if let Some(value) = binding_value(arguments, "exhausted") {
                children.extend(listed_targets(value));
            }
        }
        "await" => {
            for field in ["event", "then", "timeout"] {
                if let Some(target) = binding_value(arguments, field).and_then(reference_target) {
                    children.push(target);
                }
            }
        }
        _ => {}
    }
    children
}

fn declaration_defect(position: usize, node: &RawSemanticNodeV2) -> ValidationFailure {
    ValidationFailure::refused_at(
        Code::InvalidSemanticGraph,
        node_pointer(position),
        None,
        node.node_id.clone(),
    )
}

struct MemberCheck<'a> {
    position: usize,
    nodes: &'a [RawSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a BTreeMap<&'a CheckedNodeId, usize>,
    profile: &'a CheckedArtifactRef,
}

impl MemberCheck<'_> {
    fn refuse(&self, code: Code, cause: Option<Cause>, path: JsonPointer) -> ValidationFailure {
        ValidationFailure::refused_at(code, path, cause, self.nodes[self.position].node_id.clone())
    }

    fn refuse_with_detail(
        &self,
        code: Code,
        cause: Cause,
        path: JsonPointer,
        detail: CheckedProtocolRefusalDetail,
    ) -> ValidationFailure {
        let mut refusal = self.refuse(code, Some(cause), path);
        if let ValidationFailure::Refused(value) = &mut refusal {
            value.protocol_detail = Some(Box::new(detail));
        }
        refusal
    }

    fn argument_path(&self, ordinal: usize) -> JsonPointer {
        node_pointer(self.position)
            .key("body")
            .key("arguments")
            .index(ordinal)
            .key("value")
    }

    fn check_target(
        &self,
        term: &CheckedTerm,
        path: JsonPointer,
        expected: impl Fn(usize) -> bool,
    ) -> Result<usize, ValidationFailure> {
        let Some(target) = reference_target(term) else {
            return Err(self.refuse(Code::MalformedWire, None, path));
        };
        let target_path = path.key("target");
        if !self.nodes[self.position].dependencies.contains(target) {
            return Err(self.refuse(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                target_path,
            ));
        }
        let Some(&position) = self.index.get(target) else {
            return Err(self.refuse(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                target_path,
            ));
        };
        if !expected(position) {
            return Err(self.refuse(
                Code::InvalidModelBinding,
                Some(Cause::MalformedDeclaration),
                target_path,
            ));
        }
        Ok(position)
    }

    fn at<'a>(&self, arguments: &'a [CheckedTerm], ordinal: usize) -> &'a CheckedTerm {
        // The strict wire layout and validate_layout establish this binding.
        argument(arguments, ordinal).expect("validated protocol layout")
    }

    fn target_at(
        &self,
        arguments: &[CheckedTerm],
        ordinal: usize,
        expected: impl Fn(usize) -> bool,
    ) -> Result<usize, ValidationFailure> {
        self.check_target(
            self.at(arguments, ordinal),
            self.argument_path(ordinal),
            expected,
        )
    }

    fn is(&self, position: usize, kind: CheckedNodeKind) -> bool {
        self.kinds.get(position) == Some(&kind)
    }

    fn boolean(&self, term: &CheckedTerm, path: JsonPointer) -> Result<(), ValidationFailure> {
        let valid = match term {
            CheckedTerm::Literal {
                type_id,
                value_kind: LiteralKind::Boolean,
                ..
            } => self.index.get(type_id).is_some_and(|&position| {
                self.is(
                    position,
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean),
                )
            }),
            CheckedTerm::Reference { target } => self.index.get(target).is_some_and(|&position| {
                self.index
                    .get(&self.nodes[position].semantic_type)
                    .is_some_and(|&type_position| {
                        self.is(
                            type_position,
                            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean),
                        )
                    })
            }),
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(self.refuse(Code::IllTyped, Some(Cause::NonBooleanRoot), path))
        }
    }

    fn scalar_literal(
        &self,
        term: &CheckedTerm,
        form: ScalarTypeForm,
        path: JsonPointer,
    ) -> Result<(), ValidationFailure> {
        let CheckedTerm::Literal { type_id, .. } = term else {
            return Err(self.refuse(Code::MalformedWire, None, path));
        };
        if self
            .index
            .get(type_id)
            .is_some_and(|&position| self.is(position, CheckedNodeKind::ScalarType(form)))
        {
            Ok(())
        } else {
            Err(self.refuse(Code::IllTyped, Some(Cause::TypeMismatch), path))
        }
    }

    fn same_type(&self, binder: usize, expected: &CheckedNodeId) -> Result<(), ValidationFailure> {
        if &self.nodes[binder].semantic_type == expected {
            Ok(())
        } else {
            Err(self.refuse(
                Code::IllTyped,
                Some(Cause::TypeMismatch),
                node_pointer(self.position),
            ))
        }
    }

    fn term_type<'a>(&'a self, term: &'a CheckedTerm) -> Option<&'a CheckedNodeId> {
        match term {
            CheckedTerm::Literal { type_id, .. } => Some(type_id),
            CheckedTerm::Reference { target } => self
                .index
                .get(target)
                .map(|&position| &self.nodes[position].semantic_type),
            _ => None,
        }
    }

    fn equality_stable(&self, type_id: &CheckedNodeId) -> bool {
        let mut pending = vec![type_id];
        let mut visited = BTreeSet::new();
        while let Some(type_id) = pending.pop() {
            let Some(&position) = self.index.get(type_id) else {
                return false;
            };
            if !visited.insert(position) {
                continue;
            }
            match self.kinds[position] {
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Boolean
                    | ScalarTypeForm::Integer
                    | ScalarTypeForm::Rational
                    | ScalarTypeForm::Decimal
                    | ScalarTypeForm::Text
                    | ScalarTypeForm::String
                    | ScalarTypeForm::Uuid
                    | ScalarTypeForm::Timestamp
                    | ScalarTypeForm::Enum,
                ) => {}
                CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) => {}
                CheckedNodeKind::CompositeType(_) => {
                    let Some(term) = self.terms[position].as_ref() else {
                        return false;
                    };
                    if !collect_type_children(term, &mut pending) {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }

    fn same_clause(
        &self,
        target: usize,
        segment: &str,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        let Some(own) = self.nodes[self.position].declaration.as_ref() else {
            return Ok(false);
        };
        let Some(other) = self.nodes[target].declaration.as_ref() else {
            return Ok(false);
        };
        let own = own.qualified_name.as_slice();
        let other = other.qualified_name.as_slice();
        let own_segment = match self.kinds[self.position] {
            CheckedNodeKind::Protocol(ProtocolForm::Channel) => {
                return Ok(own.len() >= 2
                    && other.len() >= 2
                    && own[..own.len() - 2] == other[..other.len() - 2]
                    && other[other.len() - 2].as_ref() == segment);
            }
            CheckedNodeKind::Protocol(ProtocolForm::Control) => "run",
            CheckedNodeKind::Protocol(ProtocolForm::Compensation) => "compensate",
            _ => return Ok(false),
        };
        let mut clause: Option<&[Box<str>]> = None;
        for &position in self.index.values() {
            meter.charge(1, || node_pointer(self.position))?;
            if !self.is(
                position,
                CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause),
            ) {
                continue;
            }
            let Some(prefix) = self.nodes[position]
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
            else {
                continue;
            };
            if own.len() >= prefix.len() + 2
                && own.starts_with(prefix)
                && own[prefix.len()].as_ref() == own_segment
                && clause.is_none_or(|previous| prefix.len() > previous.len())
            {
                clause = Some(prefix);
            }
        }
        Ok(clause.is_some_and(|prefix| {
            other.len() == prefix.len() + 2
                && other.starts_with(prefix)
                && other[prefix.len()].as_ref() == segment
        }))
    }

    #[string_edge]
    fn clause_lists_relationship(
        &self,
        relationship: &CheckedNodeId,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        let Some(path) = self.nodes[self.position]
            .declaration
            .as_ref()
            .map(|declaration| declaration.qualified_name.as_slice())
        else {
            return Ok(false);
        };
        for &clause in self.index.values() {
            meter.charge(1, || node_pointer(self.position))?;
            if !self.is(
                clause,
                CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause),
            ) {
                continue;
            }
            let Some(prefix) = self.nodes[clause]
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
            else {
                continue;
            };
            if path.len() <= prefix.len()
                || !path.starts_with(prefix)
                || path[prefix.len()].as_ref() != "run"
            {
                continue;
            }
            if self.terms[clause]
                .as_ref()
                .and_then(|term| match term {
                    CheckedTerm::Application { arguments, .. } => {
                        binding_value(arguments, "relationships")
                    }
                    _ => None,
                })
                .is_some_and(|items| listed_targets(items).any(|target| target == relationship))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn nonnegative(
        &self,
        term: &CheckedTerm,
        path: JsonPointer,
        positive: bool,
    ) -> Result<(), ValidationFailure> {
        let Some(value) = integer_value(term) else {
            return Ok(());
        };
        if (positive && value == BigInt::from(0)) || value < BigInt::from(0) {
            Err(self.refuse(Code::IllTyped, Some(Cause::TypeMismatch), path))
        } else {
            Ok(())
        }
    }
}

fn integer_value(term: &CheckedTerm) -> Option<BigInt> {
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = term
    else {
        return None;
    };
    value.as_str().and_then(|number| number.parse().ok())
}

fn collect_type_children<'a>(term: &'a CheckedTerm, pending: &mut Vec<&'a CheckedNodeId>) -> bool {
    match term {
        CheckedTerm::Reference { target } => {
            pending.push(target);
            true
        }
        CheckedTerm::Aggregate { members } => members
            .iter()
            .all(|member| collect_type_children(member, pending)),
        CheckedTerm::Binding { value, .. } => collect_type_children(value, pending),
        CheckedTerm::Literal { .. } => true,
        _ => false,
    }
}

fn group_value<'a>(term: &'a CheckedTerm, name: &str) -> Option<&'a CheckedTerm> {
    binding_value(members(term)?, name)
}

fn check_member_dependencies(
    check: &MemberCheck<'_>,
    term: &CheckedTerm,
    path: JsonPointer,
) -> Result<(), ValidationFailure> {
    match term {
        CheckedTerm::Reference { target } => {
            if !check.nodes[check.position].dependencies.contains(target) {
                return Err(check.refuse(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    path.key("target"),
                ));
            }
        }
        CheckedTerm::Binding { value, .. } => {
            check_member_dependencies(check, value, path.key("value"))?
        }
        CheckedTerm::Aggregate { members } => {
            for (position, member) in members.iter().enumerate() {
                check_member_dependencies(
                    check,
                    member,
                    path.clone().key("members").index(position),
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn check_related_references(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    ordinal: usize,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let records = members(check.at(arguments, ordinal)).expect("validated related records");
    for (index, record) in records.iter().enumerate() {
        let relationship = group_value(record, "relationship").expect("validated relationship");
        let path = check
            .argument_path(ordinal)
            .key("members")
            .index(index)
            .key("members")
            .index(0)
            .key("value");
        let position = check.check_target(relationship, path.clone(), |_| true)?;
        if !check.is(
            position,
            CheckedNodeKind::Relation(RelationForm::Relationship),
        ) || owners
            .recover(&check.nodes[position])
            .ok()
            .is_none_or(|owner| !owner.package.relationships.contains_key(owner.node))
        {
            return Err(check.refuse(
                Code::InvalidModelBinding,
                Some(Cause::WrongExport),
                path.key("target"),
            ));
        }
        if !check.clause_lists_relationship(&check.nodes[position].node_id, meter)? {
            return Err(check.refuse(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                path.key("target"),
            ));
        }
    }
    Ok(())
}

fn check_related_types(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    ordinal: usize,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let records = members(check.at(arguments, ordinal)).expect("validated related records");
    for (index, record) in records.iter().enumerate() {
        let relationship = group_value(record, "relationship")
            .and_then(reference_target)
            .expect("validated relationship reference");
        let relation_position = *check
            .index
            .get(relationship)
            .expect("validated relationship target");
        let owner = owners
            .recover(&check.nodes[relation_position])
            .expect("validated relationship owner");
        let declaration = owner
            .package
            .relationships
            .get(owner.node)
            .expect("validated selected relationship");
        for (end_index, (end_name, declared)) in [
            ("from", &declaration.source.type_ref),
            ("to", &declaration.target.type_ref),
        ]
        .into_iter()
        .enumerate()
        {
            let term = group_value(record, end_name).expect("validated related end");
            let actual = check.term_type(term);
            let Some(Ok(expected_key)) = owner.package.end_declaration_key(declared) else {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::WrongExport),
                    check
                        .argument_path(ordinal)
                        .key("members")
                        .index(index)
                        .key("members")
                        .index(0)
                        .key("value")
                        .key("target"),
                ));
            };
            let expected = CheckedNodeId {
                domain: crate::checked_package::common::NODE_DOMAIN.into(),
                digest: expected_key.into(),
            };
            let actual_owner = actual
                .and_then(|id| check.index.get(id))
                .filter(|&&position| {
                    check.is(
                        position,
                        CheckedNodeKind::CompositeType(CompositeTypeForm::Reference),
                    )
                })
                .and_then(|&position| check.terms[position].as_ref())
                .and_then(members)
                .and_then(|members| match members {
                    [term] => reference_target(term),
                    _ => None,
                })
                .and_then(|id| check.index.get(id))
                .and_then(|&position| owners.recover(&check.nodes[position]).ok());
            let conforms = if let Some(actual_owner) = actual_owner {
                if actual_owner.object_type().is_some()
                    && std::ptr::eq(owner.package, actual_owner.package)
                {
                    let mut budget = Budget::new(meter, owner.selection, owner.package.bytes);
                    owner
                        .package
                        .conforms_to(actual_owner.node, declared, &mut budget)?
                } else {
                    false
                }
            } else {
                false
            };
            if !conforms {
                return Err(check.refuse_with_detail(
                    Code::IllTyped,
                    Cause::TypeMismatch,
                    check
                        .argument_path(ordinal)
                        .key("members")
                        .index(index)
                        .key("members")
                        .index(end_index + 1)
                        .key("value"),
                    CheckedProtocolRefusalDetail::RelatedEndType {
                        relationship: relationship.clone(),
                        end: end_name.into(),
                        expected,
                        actual: actual.cloned(),
                    },
                ));
            }
        }
    }
    Ok(())
}

#[string_edge]
fn check_control_prefix(
    check: &MemberCheck<'_>,
    operator: &str,
    arguments: &[CheckedTerm],
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let control = |position| check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Control));
    let check_list = |ordinal, expected: &dyn Fn(usize) -> bool| {
        for (position, target) in members(check.at(arguments, ordinal))
            .expect("validated list")
            .iter()
            .enumerate()
        {
            check.check_target(
                target,
                check.argument_path(ordinal).key("members").index(position),
                expected,
            )?;
        }
        Ok::<(), ValidationFailure>(())
    };
    let control_operator =
        |position: usize| match check.terms.get(position).and_then(Option::as_ref) {
            Some(CheckedTerm::Application {
                operator: ApplicationOperator::ProtocolControl,
                operation,
                ..
            }) if control(position) => operation
                .pointer("/member/operator")
                .and_then(Value::as_str),
            _ => None,
        };
    match operator {
        "check" => check.boolean(check.at(arguments, 0), check.argument_path(0))?,
        "fence" => {}
        "send" | "receive" => {
            let channel = check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Channel))
            })?;
            let binder_ordinal = if operator == "receive" { 2 } else { 1 };
            let send = if operator == "receive" {
                Some(check.target_at(arguments, 1, |position| {
                    control_operator(position) == Some("send")
                })?)
            } else {
                None
            };
            let binder = check.target_at(arguments, binder_ordinal, |position| {
                check.is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            })?;
            let related_ordinal = if operator == "receive" { 3 } else { 2 };
            check_related_references(check, arguments, related_ordinal, owners, meter)?;
            let carries = check.terms[channel]
                .as_ref()
                .and_then(|term| match term {
                    CheckedTerm::Application { arguments, .. } => {
                        binding_value(arguments, "carries")
                    }
                    _ => None,
                })
                .and_then(reference_target)
                .expect("validated channel carries");
            if &check.nodes[binder].semantic_type != carries {
                return Err(check.refuse_with_detail(
                    Code::IllTyped,
                    Cause::TypeMismatch,
                    node_pointer(check.position),
                    CheckedProtocolRefusalDetail::PayloadType {
                        expected: carries.clone(),
                        actual: check.nodes[binder].semantic_type.clone(),
                    },
                ));
            }
            if let Some(send) = send {
                let same_channel = check.terms[send]
                    .as_ref()
                    .and_then(|term| match term {
                        CheckedTerm::Application { arguments, .. } => {
                            binding_value(arguments, "channel")
                        }
                        _ => None,
                    })
                    .and_then(reference_target)
                    == Some(&check.nodes[channel].node_id);
                if !same_channel {
                    return Err(check.refuse(
                        Code::IllTyped,
                        Some(Cause::OperatorIneligible),
                        node_pointer(check.position),
                    ));
                }
            }
            let constraint_ordinal = if operator == "receive" { 4 } else { 3 };
            check.boolean(
                check.at(arguments, constraint_ordinal),
                check.argument_path(constraint_ordinal),
            )?;
            check_related_types(check, arguments, related_ordinal, owners, meter)?;
        }
        "attempt" => {
            let role = check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role))
            })?;
            if !check.same_clause(role, "role", meter)? {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    check.argument_path(0).key("target"),
                ));
            }
            let operation = check.target_at(arguments, 1, |position| {
                check.is(position, CheckedNodeKind::State(StateForm::OperationAnchor))
            })?;
            let mut contracts = Vec::new();
            for (ordinal, contract) in members(check.at(arguments, 3))
                .expect("validated contracts")
                .iter()
                .enumerate()
            {
                let path = check.argument_path(3).key("members").index(ordinal);
                let clause = check.check_target(contract, path.clone(), |position| {
                    check.is(position, CheckedNodeKind::State(StateForm::StateClause))
                })?;
                contracts.push((clause, path));
            }
            check.target_at(arguments, 4, |position| {
                check.is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            })?;
            check_related_references(check, arguments, 5, owners, meter)?;
            for (clause, path) in contracts {
                let valid = check.terms[clause]
                    .as_ref()
                    .and_then(super::structural::typed_clause_body)
                    .is_some_and(|body| {
                        matches!(
                            body.clause,
                            StateClauseKind::Precondition | StateClauseKind::Postcondition
                        ) && body.anchor == check.nodes[operation].node_id
                    });
                if !valid {
                    return Err(check.refuse(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        path.key("target"),
                    ));
                }
            }
            check.boolean(check.at(arguments, 6), check.argument_path(6))?;
            check_related_types(check, arguments, 5, owners, meter)?;
        }
        "effect" => {
            check.target_at(arguments, 0, |position| {
                control_operator(position) == Some("attempt")
            })?;
            check.target_at(arguments, 1, |position| {
                check.is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            })?;
            check_related_references(check, arguments, 2, owners, meter)?;
            check.boolean(check.at(arguments, 3), check.argument_path(3))?;
            check_related_types(check, arguments, 2, owners, meter)?;
        }
        "event" => {
            let role = check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role))
            })?;
            if !check.same_clause(role, "role", meter)? {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    check.argument_path(0).key("target"),
                ));
            }
            if let Some(compensation) =
                members(check.at(arguments, 1)).and_then(|items| items.first())
            {
                let path = check.argument_path(1).key("members").index(0);
                let template = check.check_target(compensation, path.clone(), |position| {
                    check.is(
                        position,
                        CheckedNodeKind::Protocol(ProtocolForm::Compensation),
                    )
                })?;
                if !check.same_clause(template, "compensate", meter)? {
                    return Err(check.refuse(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        path.key("target"),
                    ));
                }
            }
            check.target_at(arguments, 2, |position| {
                check.is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            })?;
            check_related_references(check, arguments, 3, owners, meter)?;
            check.boolean(check.at(arguments, 4), check.argument_path(4))?;
            check_related_types(check, arguments, 3, owners, meter)?;
        }
        "commit" => {
            let role = check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role))
            })?;
            if !check.same_clause(role, "role", meter)? {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    check.argument_path(0).key("target"),
                ));
            }
            check.target_at(arguments, 1, |position| {
                check.is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            })?;
            check.boolean(check.at(arguments, 2), check.argument_path(2))?;
        }
        "sequence" => check_list(0, &control)?,
        "choice" | "parallel" => {
            let (records_ordinal, body_ordinal) = if operator == "choice" {
                check.target_at(arguments, 0, |position| {
                    check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role))
                })?;
                check_list(1, &|position| {
                    matches!(check.kinds[position], CheckedNodeKind::Expression(_))
                })?;
                (2, 2)
            } else {
                (1, 1)
            };
            let records = members(check.at(arguments, records_ordinal)).expect("validated records");
            for (position, record) in records.iter().enumerate() {
                let fields = members(record).expect("validated record");
                let path = check
                    .argument_path(records_ordinal)
                    .key("members")
                    .index(position);
                check.check_target(
                    argument(fields, body_ordinal).expect("validated body"),
                    path.clone().key("members").index(body_ordinal).key("value"),
                    control,
                )?;
            }
            if operator == "parallel" {
                let join = check.at(arguments, 2);
                if argument_text(join, 0) == Some("predicate") {
                    check.check_target(
                        group_value(join, "predicate").expect("validated predicate"),
                        check.argument_path(2).key("members").index(1).key("value"),
                        |position| matches!(check.kinds[position], CheckedNodeKind::Function(_)),
                    )?;
                }
            }
            let mut previous: Option<&str> = None;
            let mut labels = BTreeSet::new();
            for record in records {
                let label = argument_text(record, 0).expect("validated label");
                if !labels.insert(label) {
                    return Err(check.refuse(
                        Code::AmbiguousDeclaration,
                        Some(Cause::AmbiguousName),
                        node_pointer(check.position),
                    ));
                }
                if previous.is_some_and(|prior| prior.as_bytes() > label.as_bytes()) {
                    return Err(check.refuse(
                        Code::InvalidSemanticGraph,
                        None,
                        node_pointer(check.position),
                    ));
                }
                previous = Some(label);
            }
            if operator == "choice" {
                for (position, record) in records.iter().enumerate() {
                    check.boolean(
                        group_value(record, "guard").expect("validated guard"),
                        check
                            .argument_path(2)
                            .key("members")
                            .index(position)
                            .key("members")
                            .index(1)
                            .key("value"),
                    )?;
                }
            }
            if operator == "parallel" {
                let join = check.at(arguments, 2);
                let joined = members(check.at(arguments, 3)).expect("validated joined");
                let mut prior: Option<&str> = None;
                for (position, item) in joined.iter().enumerate() {
                    let label = text_value(item).expect("validated joined label");
                    if !labels.contains(label) {
                        return Err(check.refuse(
                            Code::MissingDeclaration,
                            Some(Cause::MissingName),
                            check.argument_path(3).key("members").index(position),
                        ));
                    }
                    if prior.is_some_and(|previous| previous.as_bytes() >= label.as_bytes()) {
                        return Err(check.refuse(
                            Code::InvalidSemanticGraph,
                            None,
                            node_pointer(check.position),
                        ));
                    }
                    prior = Some(label);
                }
                let policy = argument_text(join, 0).expect("validated policy");
                let invalid = match policy {
                    "all" => joined.len() != labels.len(),
                    "any" | "predicate" => joined.is_empty(),
                    "quorum" => group_value(join, "count")
                        .and_then(integer_value)
                        .is_none_or(|count| {
                            count < BigInt::from(1) || count > BigInt::from(joined.len())
                        }),
                    _ => unreachable!("validated join policy"),
                };
                if invalid {
                    let path = if policy == "quorum" {
                        check.argument_path(2).key("members").index(1).key("value")
                    } else {
                        check.argument_path(3)
                    };
                    return Err(check.refuse(
                        Code::InvalidPackage,
                        Some(Cause::InvalidValue),
                        path,
                    ));
                }
            }
        }
        "repeat" => {
            check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role))
            })?;
            check_list(1, &|position| {
                matches!(check.kinds[position], CheckedNodeKind::Expression(_))
            })?;
            check.target_at(arguments, 5, control)?;
            if let Some(exhausted) = members(check.at(arguments, 6)).and_then(|items| items.first())
            {
                check.check_target(
                    exhausted,
                    check.argument_path(6).key("members").index(0),
                    control,
                )?;
            }
            if members(check.at(arguments, 3)).is_some_and(|items| !items.is_empty()) {
                check.boolean(
                    group_value(check.at(arguments, 3), "invariant").expect("validated invariant"),
                    check.argument_path(3).key("members").index(0).key("value"),
                )?;
            }
            check.boolean(check.at(arguments, 4), check.argument_path(4))?;
        }
        "await" => {
            let after = check.at(arguments, 0);
            let anchor = group_value(after, "target").expect("validated anchor");
            let kind = argument_text(after, 0).expect("validated anchor kind");
            let anchor_path = check.argument_path(0).key("members").index(1).key("value");
            if kind == "compensation" {
                check.check_target(anchor, anchor_path, |position| {
                    check.is(
                        position,
                        CheckedNodeKind::Protocol(ProtocolForm::Compensation),
                    )
                })?;
            } else {
                check.check_target(anchor, anchor_path, |position| {
                    matches!(
                        control_operator(position),
                        Some("send" | "receive" | "attempt" | "effect" | "event" | "commit")
                    )
                })?;
            }
            let event = check.target_at(arguments, 3, control)?;
            check.target_at(arguments, 4, control)?;
            check.target_at(arguments, 5, control)?;
            let Some(event_form) = control_operator(event) else {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    check.argument_path(3).key("target"),
                ));
            };
            if !matches!(event_form, "receive" | "effect" | "event") {
                return Err(check.refuse_with_detail(
                    Code::UnsupportedConstruct,
                    Cause::DeclarationForm,
                    node_pointer(check.position),
                    CheckedProtocolRefusalDetail::AwaitedEvent {
                        form: event_form.into(),
                        profile: check.profile.clone(),
                    },
                ));
            }
            let within = check.at(arguments, 2);
            let (Some(lower), Some(upper)) = (
                group_value(within, "lower").and_then(integer_value),
                group_value(within, "upper").and_then(integer_value),
            ) else {
                return Err(check.refuse(Code::MalformedWire, None, check.argument_path(2)));
            };
            if lower > upper {
                return Err(check.refuse_with_detail(
                    Code::IllTyped,
                    Cause::TypeMismatch,
                    check.argument_path(2),
                    CheckedProtocolRefusalDetail::Interval {
                        member: "within".into(),
                        expected: "closed interval with lower <= upper".into(),
                        actual: format!("[{lower}, {upper}]").into(),
                    },
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

#[string_edge]
fn check_compensation(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    validated: &[Option<(&str, &[CheckedTerm])>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let parameter = |position| check.is(position, CheckedNodeKind::Value(ValueForm::Parameter));
    let forward = check.target_at(arguments, 0, |position| {
        check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Control))
            && check.terms[position].as_ref().is_some_and(|term| {
                matches!(term, CheckedTerm::Application { operation, .. }
                    if operation.pointer("/member/operator").and_then(Value::as_str) == Some("effect"))
            })
    })?;
    let forward_binder = check.target_at(arguments, 1, parameter)?;
    let role = check.target_at(arguments, 2, |position| {
        check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role))
    })?;
    if !check.same_clause(role, "role", meter)? {
        return Err(check.refuse(
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            check.argument_path(2).key("target"),
        ));
    }
    check.target_at(arguments, 3, |position| {
        check.is(position, CheckedNodeKind::State(StateForm::OperationAnchor))
    })?;
    for ordinal in [5, 7] {
        let captures = members(check.at(arguments, ordinal)).expect("validated captures");
        for (position, record) in captures.iter().enumerate() {
            meter.charge(1, || node_pointer(check.position))?;
            let binder = group_value(record, "binder").expect("validated capture binder");
            check.check_target(
                binder,
                check
                    .argument_path(ordinal)
                    .key("members")
                    .index(position)
                    .key("members")
                    .index(0)
                    .key("value"),
                parameter,
            )?;
        }
    }
    let trigger = check.at(arguments, 6);
    check.check_target(
        group_value(trigger, "binder").expect("validated trigger binder"),
        check.argument_path(6).key("members").index(0).key("value"),
        parameter,
    )?;
    let attempts = check.at(arguments, 9);
    let record_type = check.check_target(
        group_value(attempts, "record_type").expect("validated record type"),
        check.argument_path(9).key("members").index(1).key("value"),
        |position| {
            matches!(
                check.kinds[position],
                CheckedNodeKind::Model(ModelForm::RecordValueType | ModelForm::EventType)
            )
        },
    )?;
    let retry = check.at(arguments, 10);
    let mut retry_binders = Vec::with_capacity(2);
    for (position, name) in ["earlier", "later"].into_iter().enumerate() {
        let binder = check.check_target(
            group_value(retry, name).expect("validated retry binder"),
            check
                .argument_path(10)
                .key("members")
                .index(position)
                .key("value"),
            parameter,
        )?;
        retry_binders.push(binder);
    }
    if let Some(commit) = members(check.at(arguments, 11)).and_then(|items| items.first()) {
        check.check_target(
            commit,
            check.argument_path(11).key("members").index(0),
            |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Control))
                    && check.terms[position].as_ref().is_some_and(|term| {
                        matches!(term, CheckedTerm::Application { operation, .. }
                            if operation.pointer("/member/operator").and_then(Value::as_str) == Some("commit"))
                    })
            },
        )?;
    }
    let recovery = check.at(arguments, 12);
    check.check_target(
        group_value(recovery, "binder").expect("validated recovery binder"),
        check.argument_path(12).key("members").index(0).key("value"),
        parameter,
    )?;
    check.scalar_literal(
        check.at(arguments, 4),
        ScalarTypeForm::Text,
        check.argument_path(4),
    )?;
    let within = check.at(arguments, 8);
    for (index, name) in ["lower", "upper"].into_iter().enumerate() {
        check.scalar_literal(
            group_value(within, name).expect("validated interval bound"),
            ScalarTypeForm::Integer,
            check
                .argument_path(8)
                .key("members")
                .index(index)
                .key("value"),
        )?;
    }
    check.scalar_literal(
        group_value(attempts, "maximum").expect("validated maximum"),
        ScalarTypeForm::Integer,
        check.argument_path(9).key("members").index(0).key("value"),
    )?;
    let forward_effect_binder = check.terms[forward]
        .as_ref()
        .and_then(|term| match term {
            CheckedTerm::Application { arguments, .. } => argument(arguments, 1),
            _ => None,
        })
        .and_then(reference_target)
        .and_then(|target| check.index.get(target).copied());
    let Some(effect_binder) = forward_effect_binder.filter(|&position| parameter(position)) else {
        return Err(check.refuse(
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            check.argument_path(0).key("target"),
        ));
    };
    let expected = &check.nodes[effect_binder].semantic_type;
    let actual = &check.nodes[forward_binder].semantic_type;
    if actual != expected {
        return Err(check.refuse_with_detail(
            Code::IllTyped,
            Cause::TypeMismatch,
            node_pointer(check.position),
            CheckedProtocolRefusalDetail::PayloadType {
                expected: expected.clone(),
                actual: actual.clone(),
            },
        ));
    }
    for binder in retry_binders {
        let expected = &check.nodes[record_type].node_id;
        let actual = &check.nodes[binder].semantic_type;
        if actual != expected {
            return Err(check.refuse_with_detail(
                Code::IllTyped,
                Cause::TypeMismatch,
                node_pointer(check.position),
                CheckedProtocolRefusalDetail::PayloadType {
                    expected: expected.clone(),
                    actual: actual.clone(),
                },
            ));
        }
    }
    check.boolean(
        group_value(trigger, "guard").expect("validated trigger guard"),
        check.argument_path(6).key("members").index(1).key("value"),
    )?;
    check.boolean(
        group_value(retry, "relation").expect("validated retry relation"),
        check.argument_path(10).key("members").index(2).key("value"),
    )?;
    check.boolean(
        group_value(recovery, "predicate").expect("validated recovery predicate"),
        check.argument_path(12).key("members").index(1).key("value"),
    )?;
    let (Some(lower), Some(upper)) = (
        group_value(within, "lower").and_then(integer_value),
        group_value(within, "upper").and_then(integer_value),
    ) else {
        return Err(check.refuse(Code::MalformedWire, None, check.argument_path(8)));
    };
    if lower > upper {
        return Err(check.refuse_with_detail(
            Code::IllTyped,
            Cause::TypeMismatch,
            check.argument_path(8),
            CheckedProtocolRefusalDetail::Interval {
                member: "within".into(),
                expected: "closed interval with lower <= upper".into(),
                actual: format!("[{lower}, {upper}]").into(),
            },
        ));
    }
    let Some(maximum) = group_value(attempts, "maximum").and_then(integer_value) else {
        return Err(check.refuse(
            Code::MalformedWire,
            None,
            check.argument_path(9).key("members").index(0).key("value"),
        ));
    };
    if maximum < BigInt::from(1) {
        return Err(check.refuse_with_detail(
            Code::IllTyped,
            Cause::TypeMismatch,
            check.argument_path(9).key("members").index(0).key("value"),
            CheckedProtocolRefusalDetail::Interval {
                member: "attempts".into(),
                expected: "count of at least one".into(),
                actual: maximum.to_string().into(),
            },
        ));
    }
    validate_compensation_commit_position(check, arguments, validated, meter)?;
    Ok(())
}

#[string_edge]
fn check_member(
    check: &MemberCheck<'_>,
    form: ProtocolForm,
    operator: &str,
    arguments: &[CheckedTerm],
    validated: &[Option<(&str, &[CheckedTerm])>],
    owners: &ModelOwners<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (ordinal, term) in arguments.iter().enumerate() {
        check_member_dependencies(
            check,
            term,
            node_pointer(check.position)
                .key("body")
                .key("arguments")
                .index(ordinal),
        )?;
    }
    if let Some(expected) = check.terms[check.position]
        .as_ref()
        .and_then(super::structural::application_dependencies)
    {
        if !check.nodes[check.position]
            .dependencies
            .iter()
            .eq(expected.iter())
        {
            return Err(check.refuse(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(check.position).key("dependencies"),
            ));
        }
    }
    let role = |position| check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role));
    let parameter = |position| check.is(position, CheckedNodeKind::Value(ValueForm::Parameter));
    match form {
        ProtocolForm::Role => {
            let replication = check.at(arguments, 1);
            if let Some(identity) = text_value(check.at(arguments, 0)) {
                let mut selected = false;
                for row in &lock.model_selections {
                    meter.charge(1, || node_pointer(check.position))?;
                    if row.identity.as_ref() == identity {
                        selected = true;
                        break;
                    }
                }
                if !selected {
                    return Err(check.refuse(
                        Code::MissingDeclaration,
                        Some(Cause::MissingSelection),
                        node_pointer(check.position),
                    ));
                }
                if !members(replication).is_some_and(|fields| fields.is_empty()) {
                    return Err(check.refuse(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        node_pointer(check.position),
                    ));
                }
            } else {
                let model = check.target_at(arguments, 0, |position| {
                    matches!(
                        check.kinds[position],
                        CheckedNodeKind::Model(
                            ModelForm::ObjectType
                                | ModelForm::Process
                                | ModelForm::StateMachine
                                | ModelForm::SystemsInterface
                                | ModelForm::PersistenceInterface
                        )
                    )
                })?;
                if !members(replication).is_some_and(|fields| fields.is_empty()) {
                    check.nonnegative(
                        group_value(replication, "max").expect("validated replication"),
                        check.argument_path(1).key("members").index(1).key("value"),
                        true,
                    )?;
                    let binder = group_value(replication, "binder").expect("validated replication");
                    let binder = check.check_target(
                        binder,
                        check.argument_path(1).key("members").index(2).key("value"),
                        parameter,
                    )?;
                    let type_position =
                        check.index.get(&check.nodes[binder].semantic_type).copied();
                    if !type_position.is_some_and(|position| {
                        check.is(
                            position,
                            CheckedNodeKind::CompositeType(CompositeTypeForm::Reference),
                        ) && check.terms[position]
                            .as_ref()
                            .and_then(members)
                            .is_some_and(|items| {
                                matches!(items, [CheckedTerm::Reference { target }] if target == &check.nodes[model].node_id)
                            })
                    }) {
                        return Err(check.refuse(
                            Code::IllTyped,
                            Some(Cause::TypeMismatch),
                            node_pointer(check.position),
                        ));
                    }
                    if let Some(until) = group_value(replication, "until") {
                        check.boolean(until, node_pointer(check.position))?;
                    }
                }
            }
        }
        ProtocolForm::Channel => {
            for ordinal in [0, 1] {
                let target = check.target_at(arguments, ordinal, role)?;
                if !check.same_clause(target, "role", meter)? {
                    return Err(check.refuse(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        check.argument_path(ordinal).key("target"),
                    ));
                }
            }
            let carries = check.target_at(arguments, 2, |position| {
                matches!(
                    check.kinds[position],
                    CheckedNodeKind::ScalarType(_)
                        | CheckedNodeKind::CompositeType(_)
                        | CheckedNodeKind::Model(
                            ModelForm::RecordValueType
                                | ModelForm::EventType
                                | ModelForm::VariantType
                        )
                )
            })?;
            let ordering = check.at(arguments, 3);
            if argument_text(ordering, 0) == Some("fifo") {
                let binder = group_value(ordering, "binder").expect("validated fifo");
                let position = check.check_target(
                    binder,
                    check.argument_path(3).key("members").index(1).key("value"),
                    parameter,
                )?;
                check.same_type(position, &check.nodes[carries].node_id)?;
                let key = group_value(ordering, "key").expect("validated fifo");
                let key_type = check.term_type(key);
                if !key_type.is_some_and(|type_id| check.equality_stable(type_id)) {
                    return Err(check.refuse(
                        Code::IllTyped,
                        Some(Cause::OperatorIneligible),
                        check.argument_path(3).key("members").index(2).key("value"),
                    ));
                }
            }
        }
        ProtocolForm::Queue => {
            check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Channel))
            })?;
        }
        ProtocolForm::Obligation => match operator {
            "finish" => {
                check.target_at(arguments, 0, parameter)?;
                check.boolean(check.at(arguments, 1), node_pointer(check.position))?;
            }
            "terminal" => {
                if let Some(condition) =
                    members(check.at(arguments, 1)).and_then(|items| items.first())
                {
                    check.boolean(condition, node_pointer(check.position))?;
                }
            }
            _ => unreachable!("validated protocol operator"),
        },
        ProtocolForm::Control => check_control_prefix(check, operator, arguments, owners, meter)?,
        ProtocolForm::Compensation => check_compensation(check, arguments, validated, meter)?,
        ProtocolForm::ProtocolClause | ProtocolForm::ProtocolReference => {}
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum PathHolder {
    ClauseMember {
        clause: usize,
        member: &'static str,
        own: bool,
    },
    Control {
        holder: usize,
    },
}

fn record_declared_path(
    expected: &mut BTreeMap<usize, (PathHolder, bool)>,
    position: usize,
    holder: PathHolder,
) {
    if let Some((_, repeated)) = expected.get_mut(&position) {
        *repeated = true;
    } else {
        expected.insert(position, (holder, false));
    }
}

#[string_edge]
fn validate_protocol_declaration_paths(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &BTreeMap<&CheckedNodeId, usize>,
) -> Result<(), ValidationFailure> {
    let mut expected = BTreeMap::new();
    let mut pending = Vec::new();
    for &clause in index.values() {
        if kinds.get(clause) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        if nodes[clause].declaration.is_none() {
            continue;
        }
        let Some((_, args)) = validated[clause] else {
            continue;
        };
        for (field, segment, expected_form) in [
            ("roles", "role", ProtocolForm::Role),
            ("channels", "channel", ProtocolForm::Channel),
            ("compensations", "compensate", ProtocolForm::Compensation),
        ] {
            if let Some(value) = binding_value(args, field) {
                for target in listed_targets(value) {
                    if let Some(&position) = index.get(target) {
                        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(expected_form)) {
                            continue;
                        }
                        record_declared_path(
                            &mut expected,
                            position,
                            PathHolder::ClauseMember {
                                clause,
                                member: segment,
                                own: true,
                            },
                        );
                    }
                }
            }
        }
        for (field, segment) in [("finish", "finish"), ("terminal", "terminal")] {
            if let Some(value) = binding_value(args, field) {
                let target = reference_target(value).or_else(|| {
                    members(value)
                        .and_then(|items| items.first())
                        .and_then(reference_target)
                });
                if let Some(&position) = target.and_then(|key| index.get(key)) {
                    if kinds.get(position)
                        != Some(&CheckedNodeKind::Protocol(ProtocolForm::Obligation))
                        || validated[position].is_none_or(|(operator, _)| operator != segment)
                    {
                        continue;
                    }
                    record_declared_path(
                        &mut expected,
                        position,
                        PathHolder::ClauseMember {
                            clause,
                            member: segment,
                            own: segment == "finish",
                        },
                    );
                }
            }
        }
        if let Some(&position) = binding_value(args, "run")
            .and_then(reference_target)
            .and_then(|key| index.get(key))
        {
            if kinds.get(position) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                pending.push((
                    position,
                    PathHolder::ClauseMember {
                        clause,
                        member: "run",
                        own: true,
                    },
                ));
            }
        }
    }

    // The visited set bounds malformed containment cycles. Membership and
    // repeated holders are checked in the later graph pass.
    let mut visited = BTreeSet::new();
    while let Some((position, holder)) = pending.pop() {
        record_declared_path(&mut expected, position, holder);
        if !visited.insert(position) {
            continue;
        }
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
            continue;
        }
        let Some((operator, args)) = validated[position] else {
            continue;
        };
        for target in control_children(operator, args) {
            if let Some(&child) = index.get(target) {
                if kinds.get(child) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                    pending.push((child, PathHolder::Control { holder: position }));
                }
            }
        }
    }

    for &position in index.values() {
        let Some(CheckedNodeKind::Protocol(form)) = kinds.get(position).copied() else {
            continue;
        };
        if form == ProtocolForm::ProtocolReference {
            continue;
        }
        let node = &nodes[position];
        if let Some((holder, false)) = expected.get(&position) {
            let Some(path) = node
                .declaration
                .as_ref()
                .map(|d| d.qualified_name.as_slice())
            else {
                return Err(declaration_defect(position, node));
            };
            let matches = match holder {
                PathHolder::ClauseMember {
                    clause,
                    member,
                    own,
                } => {
                    let prefix = nodes[*clause]
                        .declaration
                        .as_ref()
                        .map(|d| d.qualified_name.as_slice());
                    prefix.is_some_and(|prefix| {
                        path.len() == prefix.len() + 1 + usize::from(*own)
                            && path.starts_with(prefix)
                            && path[prefix.len()].as_ref() == *member
                    })
                }
                PathHolder::Control { holder } => {
                    let prefix = nodes[*holder]
                        .declaration
                        .as_ref()
                        .map(|d| d.qualified_name.as_slice());
                    prefix.is_some_and(|prefix| {
                        path.len() == prefix.len() + 1 && path.starts_with(prefix)
                    })
                }
            };
            if !matches {
                return Err(declaration_defect(position, node));
            }
        }
    }
    Ok(())
}

fn record_member_owner(
    owners: &mut [Option<usize>],
    repeated: &mut [bool],
    node: usize,
    clause: usize,
) {
    if owners[node].replace(clause).is_some() {
        repeated[node] = true;
    }
}

#[string_edge]
fn validate_protocol_membership(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut owners = vec![None; nodes.len()];
    let mut repeated = vec![false; nodes.len()];
    let mut pending = Vec::new();
    for &clause in index.values() {
        if kinds.get(clause) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        meter.charge(1, || node_pointer(clause))?;
        let Some((_, arguments)) = validated[clause] else {
            continue;
        };
        for (field, form) in [
            ("roles", ProtocolForm::Role),
            ("channels", ProtocolForm::Channel),
            ("queues", ProtocolForm::Queue),
            ("compensations", ProtocolForm::Compensation),
        ] {
            if let Some(value) = binding_value(arguments, field) {
                for target in listed_targets(value) {
                    meter.charge(1, || node_pointer(clause))?;
                    if let Some(&position) = index.get(target) {
                        if kinds.get(position) == Some(&CheckedNodeKind::Protocol(form)) {
                            record_member_owner(&mut owners, &mut repeated, position, clause);
                        }
                    }
                }
            }
        }
        for field in ["finish", "terminal"] {
            if let Some(value) = binding_value(arguments, field) {
                for target in listed_targets(value).chain(reference_target(value)) {
                    meter.charge(1, || node_pointer(clause))?;
                    if let Some(&position) = index.get(target) {
                        if kinds.get(position)
                            == Some(&CheckedNodeKind::Protocol(ProtocolForm::Obligation))
                        {
                            record_member_owner(&mut owners, &mut repeated, position, clause);
                        }
                    }
                }
            }
        }
        if let Some(&position) = binding_value(arguments, "run")
            .and_then(reference_target)
            .and_then(|target| index.get(target))
        {
            if kinds.get(position) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                pending.push((position, clause));
            }
        }
    }

    let mut visited = BTreeSet::new();
    while let Some((position, clause)) = pending.pop() {
        meter.charge(1, || node_pointer(position))?;
        record_member_owner(&mut owners, &mut repeated, position, clause);
        if !visited.insert(position) {
            continue;
        }
        let Some((operator, arguments)) = validated[position] else {
            continue;
        };
        for target in control_children(operator, arguments) {
            meter.charge(1, || node_pointer(position))?;
            if let Some(&child) = index.get(target) {
                if kinds.get(child) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                    pending.push((child, clause));
                }
            }
        }
    }

    let mut queue_count = vec![0u8; nodes.len()];
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::Queue)) {
            continue;
        }
        meter.charge(1, || node_pointer(position))?;
        let Some((_, arguments)) = validated[position] else {
            continue;
        };
        if let Some(&channel) = binding_value(arguments, "channel")
            .and_then(reference_target)
            .and_then(|target| index.get(target))
        {
            if kinds.get(channel) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Channel)) {
                queue_count[channel] = queue_count[channel].saturating_add(1);
                if owners[position].is_some() && owners[position] != owners[channel] {
                    repeated[position] = true;
                }
            }
        }
    }

    for &position in index.values() {
        let Some(CheckedNodeKind::Protocol(form)) = kinds.get(position) else {
            continue;
        };
        if !matches!(
            form,
            ProtocolForm::Role
                | ProtocolForm::Channel
                | ProtocolForm::Queue
                | ProtocolForm::Obligation
                | ProtocolForm::Control
                | ProtocolForm::Compensation
        ) {
            continue;
        }
        meter.charge(1, || node_pointer(position))?;
        if owners[position].is_none()
            || repeated[position]
            || (*form == ProtocolForm::Channel && queue_count[position] != 1)
        {
            return Err(declaration_defect(position, &nodes[position]));
        }
    }
    Ok(())
}

#[string_edge]
fn commit_is_taken(
    root: usize,
    commit: usize,
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let mut taken = BTreeMap::new();
    let mut pending = vec![(root, false)];
    let mut seen = BTreeSet::new();
    while let Some((position, expanded)) = pending.pop() {
        meter.charge(1, || node_pointer(position))?;
        if !expanded && !seen.insert(position) {
            continue;
        }
        let Some((operator, arguments)) = validated[position] else {
            continue;
        };
        if !expanded && !matches!(operator, "repeat" | "commit") {
            pending.push((position, true));
            for target in control_children(operator, arguments) {
                if let Some(&child) = index.get(target) {
                    pending.push((child, false));
                }
            }
            continue;
        }
        let children = control_children(operator, arguments)
            .into_iter()
            .filter_map(|target| index.get(target).copied())
            .collect::<Vec<_>>();
        let holds = match operator {
            "commit" => position == commit,
            "repeat" => false,
            "sequence" => children.iter().any(|child| taken.get(child) == Some(&true)),
            "parallel"
                if binding_value(arguments, "join")
                    .is_some_and(|join| argument_text(join, 0) == Some("all")) =>
            {
                children.iter().any(|child| taken.get(child) == Some(&true))
            }
            "parallel" | "choice" => {
                !children.is_empty() && children.iter().all(|child| taken.get(child) == Some(&true))
            }
            "await" => ["then", "timeout"].into_iter().all(|field| {
                binding_value(arguments, field)
                    .and_then(reference_target)
                    .and_then(|target| index.get(target))
                    .is_some_and(|child| taken.get(child) == Some(&true))
            }),
            _ => false,
        };
        taken.insert(position, holds);
    }
    Ok(taken.get(&root) == Some(&true))
}

#[string_edge]
fn validate_compensation_commit_position(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    validated: &[Option<(&str, &[CheckedTerm])>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(effect) = argument(arguments, 0)
        .and_then(reference_target)
        .and_then(|target| check.index.get(target).copied())
    else {
        return Ok(());
    };
    let Some(commit) = argument(arguments, 11)
        .and_then(members)
        .and_then(|items| items.first())
        .and_then(reference_target)
        .and_then(|target| check.index.get(target).copied())
    else {
        return Ok(());
    };
    let mut parents = BTreeMap::new();
    for &position in check.index.values() {
        if check.kinds[position] != CheckedNodeKind::Protocol(ProtocolForm::Control) {
            continue;
        }
        meter.charge(1, || node_pointer(position))?;
        let Some((operator, arguments)) = validated[position] else {
            continue;
        };
        for target in control_children(operator, arguments) {
            meter.charge(1, || node_pointer(position))?;
            if let Some(&child) = check.index.get(target) {
                parents.insert(child, position);
            }
        }
    }
    let mut child = effect;
    let mut seen = BTreeSet::new();
    while let Some(&parent) = parents.get(&child) {
        if !seen.insert(parent) {
            break;
        }
        meter.charge(1, || node_pointer(parent))?;
        let Some(("sequence", parent_arguments)) = validated[parent] else {
            child = parent;
            continue;
        };
        for target in binding_value(parent_arguments, "children")
            .into_iter()
            .flat_map(listed_targets)
        {
            let Some(&earlier) = check.index.get(target) else {
                continue;
            };
            if earlier == child {
                break;
            }
            if commit_is_taken(earlier, commit, validated, check.index, meter)? {
                let mut failure = ValidationFailure::refused_at(
                    Code::UnsupportedConstruct,
                    node_pointer(commit),
                    Some(Cause::DeclarationForm),
                    check.nodes[commit].node_id.clone(),
                );
                if let ValidationFailure::Refused(ref mut refusal) = failure {
                    refusal.protocol_detail =
                        Some(Box::new(CheckedProtocolRefusalDetail::CommitPosition {
                            profile: check.profile.clone(),
                            member: "commit".into(),
                            commit: check.nodes[commit].node_id.clone(),
                            effect: check.nodes[effect].node_id.clone(),
                            reason: "commit-precedes-effect".into(),
                        }));
                }
                return Err(failure);
            }
        }
        child = parent;
    }
    Ok(())
}

#[string_edge]
fn check_clause(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    validated: &[Option<(&str, &[CheckedTerm])>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let parameter = |position| check.is(position, CheckedNodeKind::Value(ValueForm::Parameter));
    let over = check.target_at(arguments, 0, parameter)?;
    let activation = check.at(arguments, 1);
    let each = group_value(activation, "kind").and_then(text_value) == Some("each");
    let trigger_position = if each {
        let trigger = group_value(activation, "trigger").expect("validated each trigger");
        Some(check.check_target(
            trigger,
            check.argument_path(1).key("members").index(1).key("value"),
            parameter,
        )?)
    } else {
        None
    };
    let captures = members(check.at(arguments, 2)).expect("validated captures");
    let mut capture_positions = Vec::with_capacity(captures.len());
    for (entry, record) in captures.iter().enumerate() {
        meter.charge(1, || node_pointer(check.position))?;
        let binder = group_value(record, "binder").expect("validated capture binder");
        capture_positions.push(
            check.check_target(
                binder,
                check
                    .argument_path(2)
                    .key("members")
                    .index(entry)
                    .key("members")
                    .index(0)
                    .key("value"),
                parameter,
            )?,
        );
    }
    let mut readable_names = BTreeSet::<Box<str>>::new();
    let mut introduce = |position: usize, expected: usize| {
        let actual = check.terms[position]
            .as_ref()
            .and_then(|body| group_value(body, "level"))
            .and_then(integer_value);
        if actual != Some(BigInt::from(expected)) {
            return Err(ValidationFailure::refused_at(
                Code::InvalidSemanticGraph,
                node_pointer(position),
                None,
                check.nodes[position].node_id.clone(),
            ));
        }
        let Some(name) = check.terms[position]
            .as_ref()
            .and_then(|body| group_value(body, "name"))
            .and_then(text_value)
        else {
            return Err(ValidationFailure::refused_at(
                Code::MalformedWire,
                node_pointer(position).key("body"),
                None,
                check.nodes[position].node_id.clone(),
            ));
        };
        if !readable_names.insert(name.into()) {
            return Err(ValidationFailure::refused_at(
                Code::AmbiguousDeclaration,
                node_pointer(position),
                Some(Cause::AmbiguousName),
                check.nodes[position].node_id.clone(),
            ));
        }
        Ok(())
    };
    for (ordinal, expected) in [
        (3, CheckedNodeKind::Protocol(ProtocolForm::Role)),
        (4, CheckedNodeKind::Relation(RelationForm::Relationship)),
        (5, CheckedNodeKind::Protocol(ProtocolForm::Channel)),
        (6, CheckedNodeKind::Protocol(ProtocolForm::Queue)),
        (7, CheckedNodeKind::Temporal(TemporalForm::TemporalClause)),
        (8, CheckedNodeKind::Protocol(ProtocolForm::Compensation)),
    ] {
        for (entry, target) in members(check.at(arguments, ordinal))
            .expect("validated clause set")
            .iter()
            .enumerate()
        {
            meter.charge(1, || node_pointer(check.position))?;
            let path = check.argument_path(ordinal).key("members").index(entry);
            let position = check.check_target(target, path.clone(), |_| true)?;
            if !check.is(position, expected) {
                let cause = if ordinal == 4 {
                    Cause::WrongExport
                } else {
                    Cause::MalformedDeclaration
                };
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(cause),
                    path.key("target"),
                ));
            }
        }
    }
    if let Some(terminal) = members(check.at(arguments, 9)).and_then(|items| items.first()) {
        check.check_target(
            terminal,
            check.argument_path(9).key("members").index(0),
            |position| {
                check.is(
                    position,
                    CheckedNodeKind::Protocol(ProtocolForm::Obligation),
                ) && validated[position].is_some_and(|(operator, _)| operator == "terminal")
            },
        )?;
    }
    for ordinal in [11, 12] {
        let expected = if ordinal == 11 {
            ProtocolForm::Control
        } else {
            ProtocolForm::Obligation
        };
        check.target_at(arguments, ordinal, |position| {
            check.is(position, CheckedNodeKind::Protocol(expected))
                && (ordinal == 11
                    || validated[position].is_some_and(|(operator, _)| operator == "finish"))
        })?;
    }

    introduce(over, 0)?;
    if let Some(trigger_position) = trigger_position {
        introduce(trigger_position, 1)?;
    }

    for (entry, &binder_position) in capture_positions.iter().enumerate() {
        meter.charge(1, || node_pointer(check.position))?;
        introduce(binder_position, 1 + usize::from(each) + entry)?;
    }
    Ok(())
}

#[string_edge]
fn check_clause_slots(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    validated: &[Option<(&str, &[CheckedTerm])>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if let Some(expected) = check.terms[check.position]
        .as_ref()
        .and_then(super::structural::application_dependencies)
    {
        if !check.nodes[check.position]
            .dependencies
            .iter()
            .eq(expected.iter())
        {
            return Err(check.refuse(
                Code::InvalidSemanticGraph,
                None,
                node_pointer(check.position).key("dependencies"),
            ));
        }
    }
    let scope = ProtocolScope::new(check, arguments, validated, meter)?;
    let activation = check.at(arguments, 1);
    if let Some(guard) = group_value(activation, "guard") {
        let path = check.argument_path(1).key("members").index(2).key("value");
        check_member_dependencies(check, guard, path.clone())?;
        let mut allowed = BTreeSet::new();
        if let Some(trigger) = scope.activation {
            allowed.insert(trigger);
        }
        scope.slot(
            check.position,
            guard,
            &allowed,
            "protocol-activation",
            Some("activation.guard"),
            None,
            meter,
        )?;
        check.boolean(guard, path)?;
    }
    let captures = members(check.at(arguments, 2)).expect("validated captures");
    for (entry, record) in captures.iter().enumerate() {
        meter.charge(1, || node_pointer(check.position))?;
        let initializer = group_value(record, "initializer").expect("validated initializer");
        let path = check
            .argument_path(2)
            .key("members")
            .index(entry)
            .key("members")
            .index(1)
            .key("value");
        check_member_dependencies(check, initializer, path)?;
        let mut allowed = scope.c0[..1 + usize::from(scope.activation.is_some())]
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        allowed.extend(scope.captures[..entry].iter().copied());
        scope.slot(
            check.position,
            initializer,
            &allowed,
            "protocol-capture",
            None,
            Some(entry),
            meter,
        )?;
    }
    scope.check_declaration_slots(meter)?;
    scope.check_control_slots(meter)?;
    scope.check_finish_terminal(meter)?;
    scope.check_compensation_slots(meter)?;
    Ok(())
}

#[derive(Clone, Copy)]
struct ProtocolBinderOrigin {
    owner: usize,
    capture: Option<(&'static str, usize)>,
}

#[derive(Clone, Copy)]
enum DecisionFactUse {
    VisibleEntry,
    Guard,
}

struct ProtocolScope<'a, 'b> {
    check: &'a MemberCheck<'b>,
    arguments: &'a [CheckedTerm],
    validated: &'a [Option<(&'a str, &'a [CheckedTerm])>],
    c0: Vec<usize>,
    activation: Option<usize>,
    captures: Vec<usize>,
    known: BTreeMap<usize, Vec<ProtocolBinderOrigin>>,
    fact_roles: BTreeMap<usize, usize>,
}

impl<'a, 'b> ProtocolScope<'a, 'b> {
    fn target(&self, term: &CheckedTerm) -> usize {
        reference_target(term)
            .and_then(|target| self.check.index.get(target).copied())
            .expect("validated protocol reference")
    }

    fn node_arguments(&self, position: usize) -> (&'a str, &'a [CheckedTerm]) {
        self.validated[position].expect("validated protocol member")
    }

    #[string_edge]
    fn new(
        check: &'a MemberCheck<'b>,
        arguments: &'a [CheckedTerm],
        validated: &'a [Option<(&'a str, &'a [CheckedTerm])>],
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let over = check.target_at(arguments, 0, |_| true)?;
        let activation = group_value(check.at(arguments, 1), "trigger")
            .and_then(reference_target)
            .and_then(|id| check.index.get(id).copied());
        let captures = members(check.at(arguments, 2))
            .expect("validated captures")
            .iter()
            .map(|record| {
                group_value(record, "binder")
                    .and_then(reference_target)
                    .and_then(|id| check.index.get(id).copied())
                    .expect("validated capture binder")
            })
            .collect::<Vec<_>>();
        let mut c0 = vec![over];
        if let Some(trigger) = activation {
            c0.push(trigger);
        }
        c0.extend(captures.iter().copied());
        let mut known: BTreeMap<usize, Vec<ProtocolBinderOrigin>> = BTreeMap::new();
        for (index, &position) in c0.iter().enumerate() {
            meter.charge(1, || node_pointer(check.position))?;
            let capture = if index > usize::from(activation.is_some()) {
                Some((
                    "protocol-capture",
                    index - 1 - usize::from(activation.is_some()),
                ))
            } else {
                None
            };
            known
                .entry(position)
                .or_default()
                .push(ProtocolBinderOrigin {
                    owner: check.position,
                    capture,
                });
        }
        let mut scope = Self {
            check,
            arguments,
            validated,
            c0,
            activation,
            captures,
            known,
            fact_roles: BTreeMap::new(),
        };
        for field in ["roles", "channels", "compensations"] {
            for target in listed_targets(binding_value(arguments, field).expect("validated list")) {
                meter.charge(1, || node_pointer(check.position))?;
                let position = *check.index.get(target).expect("validated member");
                let (_, args) = scope.node_arguments(position);
                match field {
                    "roles" => {
                        if let Some(binder) =
                            group_value(argument(args, 1).expect("role replication"), "binder")
                        {
                            let binder = scope.target(binder);
                            scope
                                .known
                                .entry(binder)
                                .or_default()
                                .push(ProtocolBinderOrigin {
                                    owner: check.position,
                                    capture: None,
                                });
                        }
                    }
                    "channels" => {
                        if let Some(binder) =
                            group_value(argument(args, 3).expect("channel ordering"), "binder")
                        {
                            let binder = scope.target(binder);
                            scope
                                .known
                                .entry(binder)
                                .or_default()
                                .push(ProtocolBinderOrigin {
                                    owner: check.position,
                                    capture: None,
                                });
                        }
                    }
                    "compensations" => {
                        let registration_count =
                            members(argument(args, 5).expect("registration captures"))
                                .expect("registration captures")
                                .len();
                        let activation_count =
                            members(argument(args, 7).expect("activation captures"))
                                .expect("activation captures")
                                .len();
                        for (ordinal, binder) in compensation_binders(args).into_iter().enumerate()
                        {
                            meter.charge(1, || node_pointer(position))?;
                            let capture = if ordinal > 0 && ordinal <= registration_count {
                                Some(("registration-capture", ordinal - 1))
                            } else if ordinal > registration_count + 1
                                && ordinal <= registration_count + 1 + activation_count
                            {
                                Some(("activation-capture", ordinal - registration_count - 2))
                            } else {
                                None
                            };
                            let binder = scope.target(binder);
                            scope
                                .known
                                .entry(binder)
                                .or_default()
                                .push(ProtocolBinderOrigin {
                                    owner: position,
                                    capture,
                                });
                        }
                    }
                    _ => unreachable!("fixed protocol lists"),
                }
            }
        }
        for &position in check.index.values() {
            meter.charge(1, || node_pointer(check.position))?;
            if !check.is(
                position,
                CheckedNodeKind::Protocol(ProtocolForm::Compensation),
            ) {
                continue;
            }
            let (_, args) = scope.node_arguments(position);
            for binder in compensation_binders(args) {
                meter.charge(1, || node_pointer(position))?;
                let binder = scope.target(binder);
                let origins = scope.known.entry(binder).or_default();
                if !origins.iter().any(|origin| origin.owner == position) {
                    origins.push(ProtocolBinderOrigin {
                        owner: position,
                        capture: None,
                    });
                }
            }
        }
        let finish = scope.target(binding_value(arguments, "finish").expect("validated finish"));
        let (_, args) = scope.node_arguments(finish);
        let finish_binder = scope.target(argument(args, 0).expect("validated finish binder"));
        scope
            .known
            .entry(finish_binder)
            .or_default()
            .push(ProtocolBinderOrigin {
                owner: check.position,
                capture: None,
            });
        let root = scope.target(binding_value(arguments, "run").expect("validated root"));
        let mut pending = vec![root];
        let mut visited = BTreeSet::new();
        while let Some(position) = pending.pop() {
            meter.charge(1, || node_pointer(position))?;
            if !visited.insert(position) {
                continue;
            }
            let (operator, args) = scope.node_arguments(position);
            if let Some(binder) = control_binder(operator, args) {
                let binder = scope.target(binder);
                let fact_role = match operator {
                    "receive" => {
                        let channel = scope.target(argument(args, 0).expect("receive channel"));
                        let (_, channel_args) = scope.node_arguments(channel);
                        Some(scope.target(argument(channel_args, 1).expect("channel to role")))
                    }
                    "attempt" | "event" => {
                        Some(scope.target(argument(args, 0).expect("control role")))
                    }
                    _ => None,
                };
                if let Some(role) = fact_role {
                    scope.fact_roles.insert(binder, role);
                }
                scope
                    .known
                    .entry(binder)
                    .or_default()
                    .push(ProtocolBinderOrigin {
                        owner: check.position,
                        capture: None,
                    });
            }
            for child in control_children(operator, args) {
                meter.charge(1, || node_pointer(position))?;
                pending.push(*check.index.get(child).expect("validated child"));
            }
        }
        Ok(scope)
    }

    fn slot(
        &self,
        owner: usize,
        term: &CheckedTerm,
        allowed: &BTreeSet<usize>,
        stage: &'static str,
        barred_slot: Option<&'static str>,
        capture: Option<usize>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let mut pending = Vec::new();
        if let CheckedTerm::Reference { target } = term {
            if let Some(&position) = self.check.index.get(target) {
                pending.push(position);
            }
        }
        let mut visited = BTreeSet::new();
        let mut reads = BTreeSet::new();
        let mut state_read = None;
        while let Some(position) = pending.pop() {
            meter.charge(1, || node_pointer(owner))?;
            if !visited.insert(position) {
                continue;
            }
            if self
                .check
                .is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            {
                reads.insert(position);
                continue;
            }
            if matches!(self.check.kinds[position], CheckedNodeKind::Function(_)) {
                continue;
            }
            if let Some(CheckedTerm::Application { operation, .. }) = &self.check.terms[position] {
                if operation
                    .get("identity")
                    .and_then(Value::as_str)
                    .is_some_and(model_state_operation)
                {
                    state_read = Some(position);
                }
            }
            for target in &self.check.nodes[position].dependencies {
                meter.charge(1, || node_pointer(owner))?;
                if let Some(&next) = self.check.index.get(target) {
                    pending.push(next);
                }
            }
        }
        if let Some(slot) = barred_slot {
            let barred = reads
                .iter()
                .copied()
                .find(|&position| position == self.c0[0])
                .or(state_read);
            if let Some(position) = barred {
                let mut failure = ValidationFailure::refused_at(
                    Code::IllTyped,
                    node_pointer(position),
                    Some(Cause::OperatorIneligible),
                    self.check.nodes[position].node_id.clone(),
                );
                if let ValidationFailure::Refused(ref mut refusal) = failure {
                    refusal.protocol_detail =
                        Some(Box::new(CheckedProtocolRefusalDetail::ModelStateRead {
                            slot: slot.into(),
                        }));
                }
                return Err(failure);
            }
        }
        for position in reads {
            if allowed.contains(&position) {
                continue;
            }
            let origins = self.known.get(&position).map(Vec::as_slice).unwrap_or(&[]);
            let name = self.check.terms[position]
                .as_ref()
                .and_then(|body| group_value(body, "name"))
                .and_then(text_value)
                .expect("validated binder name");
            let read = if origins.iter().any(|origin| {
                origin.owner == owner
                    && origin.capture.is_some_and(|(group, index)| {
                        group == stage && capture.is_some_and(|current| index >= current)
                    })
            }) || (owner == self.check.position
                && origins.iter().any(|origin| {
                    origin.capture.is_some_and(|(group, index)| {
                        group == stage && capture.is_some_and(|current| index >= current)
                    })
                })) {
                "forward"
            } else if !origins.is_empty()
                && origins
                    .iter()
                    .all(|origin| origin.owner != self.check.position && origin.owner != owner)
            {
                "foreign"
            } else {
                "stage"
            };
            let mut failure = ValidationFailure::refused_at(
                Code::MissingDeclaration,
                node_pointer(owner),
                Some(Cause::MissingName),
                self.check.nodes[owner].node_id.clone(),
            );
            if let ValidationFailure::Refused(ref mut refusal) = failure {
                let scope_owner = if self
                    .check
                    .is(owner, CheckedNodeKind::Protocol(ProtocolForm::Compensation))
                {
                    owner
                } else {
                    self.check.position
                };
                refusal.protocol_detail =
                    Some(Box::new(CheckedProtocolRefusalDetail::BinderScope {
                        name: name.into(),
                        stage: stage.into(),
                        owner: self.check.nodes[scope_owner].node_id.clone(),
                        read: read.into(),
                    }));
            }
            return Err(failure);
        }
        Ok(())
    }

    #[string_edge]
    fn decision_facts(
        &self,
        term: &CheckedTerm,
        owner: usize,
        listed: &BTreeSet<usize>,
        use_at: DecisionFactUse,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let CheckedTerm::Reference { target } = term else {
            return Ok(());
        };
        let Some(&root) = self.check.index.get(target) else {
            return Ok(());
        };
        let mut pending = vec![root];
        let mut visited = BTreeSet::new();
        while let Some(position) = pending.pop() {
            meter.charge(1, || node_pointer(owner))?;
            if !visited.insert(position)
                || (matches!(use_at, DecisionFactUse::Guard) && listed.contains(&position))
            {
                continue;
            }
            if matches!(self.check.kinds[position], CheckedNodeKind::Function(_)) {
                continue;
            }
            if let Some(CheckedTerm::Application {
                operation,
                arguments,
                ..
            }) = &self.check.terms[position]
            {
                if operation.get("identity").and_then(Value::as_str)
                    == Some("quire.op.record.project")
                    && operation.pointer("/member/kind").and_then(Value::as_str) == Some("field")
                {
                    if let (Some(CheckedTerm::Reference { target }), Some(field)) = (
                        arguments.first(),
                        operation.pointer("/member/name").and_then(Value::as_str),
                    ) {
                        if let Some(&binder) = self.check.index.get(target) {
                            if self
                                .check
                                .is(binder, CheckedNodeKind::Value(ValueForm::Parameter))
                                && !self.captures.contains(&binder)
                            {
                                let reason = match self.fact_roles.get(&binder) {
                                    Some(&role) if role != owner => Some("other-role"),
                                    Some(_)
                                        if matches!(use_at, DecisionFactUse::Guard)
                                            && !listed.contains(&position) =>
                                    {
                                        Some("unlisted")
                                    }
                                    Some(_) => None,
                                    None => Some("ineligible-kind"),
                                };
                                if let Some(reason) = reason {
                                    let node = self.check.nodes[position].node_id.clone();
                                    let mut failure = ValidationFailure::refused_at(
                                        Code::WrongSnapshot,
                                        node_pointer(position),
                                        Some(Cause::WrongObservation),
                                        node.clone(),
                                    );
                                    if let ValidationFailure::Refused(ref mut refusal) = failure {
                                        refusal.protocol_detail = Some(Box::new(
                                            CheckedProtocolRefusalDetail::DecisionFact {
                                                node,
                                                owner: self.check.nodes[owner].node_id.clone(),
                                                read: CheckedProtocolDecisionRead {
                                                    binder: self.check.nodes[binder]
                                                        .node_id
                                                        .clone(),
                                                    field: field.into(),
                                                },
                                                reason: reason.into(),
                                            },
                                        ));
                                    }
                                    return Err(failure);
                                }
                            }
                        }
                    }
                    continue;
                }
            }
            for target in self.check.nodes[position].dependencies.iter().rev() {
                meter.charge(1, || node_pointer(owner))?;
                if let Some(&next) = self.check.index.get(target) {
                    pending.push(next);
                }
            }
        }
        Ok(())
    }

    fn introduce(
        &self,
        position: usize,
        input: &BTreeSet<usize>,
        meter: &mut WorkMeter,
    ) -> Result<BTreeSet<usize>, ValidationFailure> {
        meter.charge(1, || node_pointer(position))?;
        let actual = self.check.terms[position]
            .as_ref()
            .and_then(|body| group_value(body, "level"))
            .and_then(integer_value);
        if actual != Some(BigInt::from(input.len())) {
            return Err(ValidationFailure::refused_at(
                Code::InvalidSemanticGraph,
                node_pointer(position),
                None,
                self.check.nodes[position].node_id.clone(),
            ));
        }
        let name = self.check.terms[position]
            .as_ref()
            .and_then(|body| group_value(body, "name"))
            .and_then(text_value)
            .expect("validated binder name");
        for &other in input {
            meter.charge(1, || node_pointer(position))?;
            let other_name = self.check.terms[other]
                .as_ref()
                .and_then(|body| group_value(body, "name"))
                .and_then(text_value)
                .expect("validated binder name");
            if name == other_name {
                return Err(ValidationFailure::refused_at(
                    Code::AmbiguousDeclaration,
                    node_pointer(position),
                    Some(Cause::AmbiguousName),
                    self.check.nodes[position].node_id.clone(),
                ));
            }
        }
        let mut output = input.clone();
        output.insert(position);
        Ok(output)
    }

    fn check_declaration_slots(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let base = self.c0.iter().copied().collect::<BTreeSet<_>>();
        for target in listed_targets(binding_value(self.arguments, "roles").expect("roles")) {
            meter.charge(1, || node_pointer(self.check.position))?;
            let role = *self.check.index.get(target).expect("validated role");
            let (_, args) = self.node_arguments(role);
            let replication = argument(args, 1).expect("replication");
            if let Some(binder) = group_value(replication, "binder") {
                let with_binder = self.introduce(self.target(binder), &base, meter)?;
                if let Some(until) = group_value(replication, "until") {
                    self.slot(
                        role,
                        until,
                        &with_binder,
                        "role-lifetime",
                        None,
                        None,
                        meter,
                    )?;
                }
            }
        }
        for target in listed_targets(binding_value(self.arguments, "channels").expect("channels")) {
            meter.charge(1, || node_pointer(self.check.position))?;
            let channel = *self.check.index.get(target).expect("validated channel");
            let (_, args) = self.node_arguments(channel);
            let ordering = argument(args, 3).expect("ordering");
            if let Some(binder) = group_value(ordering, "binder") {
                self.introduce(self.target(binder), &base, meter)?;
                if let Some(key) = group_value(ordering, "key") {
                    let allowed = BTreeSet::from([self.target(binder)]);
                    self.slot(
                        channel,
                        key,
                        &allowed,
                        "channel-key",
                        Some("channel.key"),
                        None,
                        meter,
                    )?;
                }
            }
        }
        Ok(())
    }

    #[string_edge]
    fn control_entry(
        &self,
        position: usize,
        input: BTreeSet<usize>,
        meter: &mut WorkMeter,
    ) -> Result<ControlScopeFrame, ValidationFailure> {
        meter.charge(1, || node_pointer(position))?;
        let (operator, args) = self.node_arguments(position);
        for _ in &input {
            meter.charge(1, || node_pointer(position))?;
        }
        let output = if let Some(binder) = control_binder(operator, args) {
            self.introduce(self.target(binder), &input, meter)?
        } else {
            input.clone()
        };
        match operator {
            "choice" | "repeat" => {
                let visible = argument(args, 1).expect("visible");
                let listed = members(visible)
                    .expect("visible entries")
                    .iter()
                    .map(|entry| self.target(entry))
                    .collect::<BTreeSet<_>>();
                let owner = self.target(argument(args, 0).expect("decision owner"));
                for entry in members(visible).expect("visible entries") {
                    self.slot(position, entry, &input, "run", Some("visible"), None, meter)?;
                    self.decision_facts(
                        entry,
                        owner,
                        &listed,
                        DecisionFactUse::VisibleEntry,
                        meter,
                    )?;
                }
                if operator == "choice" {
                    for case in members(argument(args, 2).expect("cases")).expect("cases") {
                        let guard = group_value(case, "guard").expect("guard");
                        self.slot(
                            position,
                            guard,
                            &input,
                            "run",
                            Some("choice.guard"),
                            None,
                            meter,
                        )?;
                        self.decision_facts(guard, owner, &listed, DecisionFactUse::Guard, meter)?;
                    }
                } else {
                    let guard = argument(args, 4).expect("repeat guard");
                    self.slot(
                        position,
                        guard,
                        &input,
                        "run",
                        Some("repeat.guard"),
                        None,
                        meter,
                    )?;
                    self.decision_facts(guard, owner, &listed, DecisionFactUse::Guard, meter)?;
                    let proof = argument(args, 3).expect("repeat proof");
                    for item in members(proof).expect("proof") {
                        if let CheckedTerm::Binding { value, .. } = item {
                            self.slot(position, value, &input, "run", None, None, meter)?;
                        }
                    }
                }
            }
            "check" => {
                self.slot(
                    position,
                    argument(args, 0).expect("condition"),
                    &input,
                    "run",
                    Some("check.condition"),
                    None,
                    meter,
                )?;
            }
            "send" | "receive" | "attempt" | "effect" | "event" | "commit" => {
                let constraint = match operator {
                    "send" => 3,
                    "receive" => 4,
                    "attempt" => 6,
                    "effect" => 3,
                    "event" => 4,
                    "commit" => 2,
                    _ => unreachable!("step control"),
                };
                self.slot(
                    position,
                    argument(args, constraint).expect("constraint"),
                    &output,
                    "run",
                    None,
                    None,
                    meter,
                )?;
                if matches!(
                    operator,
                    "send" | "receive" | "attempt" | "effect" | "event"
                ) {
                    let related = constraint - 1;
                    for record in
                        members(argument(args, related).expect("related")).expect("related list")
                    {
                        for end in ["from", "to"] {
                            self.slot(
                                position,
                                group_value(record, end).expect("related end"),
                                &output,
                                "run",
                                None,
                                None,
                                meter,
                            )?;
                        }
                    }
                }
            }
            _ => {}
        }
        let children = control_children(operator, args)
            .into_iter()
            .map(|target| *self.check.index.get(target).expect("validated child"))
            .collect();
        Ok(ControlScopeFrame {
            position,
            input,
            output,
            children,
            next: 0,
        })
    }

    #[string_edge]
    fn check_control_slots(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let root = self.target(binding_value(self.arguments, "run").expect("run"));
        let base = self.c0.iter().copied().collect::<BTreeSet<_>>();
        let mut frames = vec![self.control_entry(root, base, meter)?];
        while let Some(frame) = frames.last_mut() {
            let (operator, args) = self.node_arguments(frame.position);
            if let Some(&child) = frame.children.get(frame.next) {
                let child_number = frame.next;
                frame.next += 1;
                let child_input = match operator {
                    "sequence" => frame.output.clone(),
                    "await" if child_number == 1 => frame.output.clone(),
                    _ => frame.input.clone(),
                };
                frames.push(self.control_entry(child, child_input, meter)?);
                continue;
            }
            let all_join = operator == "parallel"
                && group_value(argument(args, 2).expect("parallel join"), "policy")
                    .and_then(text_value)
                    == Some("all");
            let out = if operator == "sequence" || all_join {
                frame.output.clone()
            } else if matches!(operator, "choice" | "repeat" | "parallel" | "await") {
                frame.input.clone()
            } else {
                frame.output.clone()
            };
            frames.pop();
            if let Some(parent) = frames.last_mut() {
                let (parent_operator, _) = self.node_arguments(parent.position);
                if parent_operator == "sequence" || parent_operator == "await" {
                    parent.output = out;
                } else if parent_operator == "parallel" {
                    let (_, args) = self.node_arguments(parent.position);
                    let all = group_value(argument(args, 2).expect("parallel join"), "policy")
                        .and_then(text_value)
                        == Some("all");
                    if !all {
                        continue;
                    }
                    for binder in out.difference(&parent.input) {
                        let name = self.check.terms[*binder]
                            .as_ref()
                            .and_then(|body| group_value(body, "name"))
                            .and_then(text_value)
                            .expect("validated binder name");
                        let mut duplicate = parent.output.contains(binder);
                        for previous in parent.output.difference(&parent.input) {
                            meter.charge(1, || node_pointer(*binder))?;
                            let previous_name = self.check.terms[*previous]
                                .as_ref()
                                .and_then(|body| group_value(body, "name"))
                                .and_then(text_value)
                                .expect("validated binder name");
                            duplicate |= name == previous_name;
                        }
                        if duplicate {
                            return Err(ValidationFailure::refused_at(
                                Code::AmbiguousDeclaration,
                                node_pointer(*binder),
                                Some(Cause::AmbiguousName),
                                self.check.nodes[*binder].node_id.clone(),
                            ));
                        }
                        parent.output.insert(*binder);
                    }
                }
            } else {
                self.check_finish_slot(&out, meter)?;
            }
        }
        Ok(())
    }

    fn check_finish_slot(
        &self,
        input: &BTreeSet<usize>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let finish = self.target(binding_value(self.arguments, "finish").expect("finish"));
        let (_, args) = self.node_arguments(finish);
        let binder = self.target(argument(args, 0).expect("finish binder"));
        let allowed = self.introduce(binder, input, meter)?;
        self.slot(
            finish,
            argument(args, 1).expect("finish constraint"),
            &allowed,
            "finish",
            None,
            None,
            meter,
        )
    }

    fn check_finish_terminal(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let Some(term) = binding_value(self.arguments, "terminal") else {
            return Ok(());
        };
        let Some(target) = listed_targets(term).next() else {
            return Ok(());
        };
        let terminal = *self.check.index.get(target).expect("validated terminal");
        let (_, args) = self.node_arguments(terminal);
        let Some(condition) =
            members(argument(args, 1).expect("terminal condition")).and_then(|items| items.first())
        else {
            return Ok(());
        };
        let mut allowed = BTreeSet::new();
        if self.activation.is_none() {
            allowed.insert(self.c0[0]);
            allowed.extend(self.captures.iter().copied());
        }
        self.slot(terminal, condition, &allowed, "terminal", None, None, meter)
    }

    fn check_compensation_slots(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let base = self.c0.iter().copied().collect::<BTreeSet<_>>();
        for target in
            listed_targets(binding_value(self.arguments, "compensations").expect("compensations"))
        {
            meter.charge(1, || node_pointer(self.check.position))?;
            let template = *self.check.index.get(target).expect("validated template");
            let (_, args) = self.node_arguments(template);
            let forward = self.target(argument(args, 1).expect("forward binder"));
            let mut level_set = self.introduce(forward, &base, meter)?;
            let mut registration = Vec::new();
            for (ordinal, capture) in members(argument(args, 5).expect("registration captures"))
                .expect("registration captures")
                .iter()
                .enumerate()
            {
                let initializer = group_value(capture, "initializer").expect("initializer");
                self.slot(
                    template,
                    initializer,
                    &level_set,
                    "registration-capture",
                    None,
                    Some(ordinal),
                    meter,
                )?;
                let binder = self.target(group_value(capture, "binder").expect("capture binder"));
                level_set = self.introduce(binder, &level_set, meter)?;
                registration.push(binder);
            }
            let trigger = argument(args, 6).expect("trigger");
            let trigger_binder =
                self.target(group_value(trigger, "binder").expect("trigger binder"));
            level_set = self.introduce(trigger_binder, &level_set, meter)?;
            let mut guard_allowed = self.captures.iter().copied().collect::<BTreeSet<_>>();
            guard_allowed.extend(registration.iter().copied());
            guard_allowed.insert(trigger_binder);
            self.slot(
                template,
                group_value(trigger, "guard").expect("trigger guard"),
                &guard_allowed,
                "trigger-guard",
                Some("trigger.guard"),
                None,
                meter,
            )?;
            let mut activation = Vec::new();
            for (ordinal, capture) in members(argument(args, 7).expect("activation captures"))
                .expect("activation captures")
                .iter()
                .enumerate()
            {
                let initializer = group_value(capture, "initializer").expect("initializer");
                let mut allowed = guard_allowed.clone();
                allowed.insert(self.c0[0]);
                allowed.extend(activation.iter().copied());
                self.slot(
                    template,
                    initializer,
                    &allowed,
                    "activation-capture",
                    None,
                    Some(ordinal),
                    meter,
                )?;
                let binder = self.target(group_value(capture, "binder").expect("capture binder"));
                level_set = self.introduce(binder, &level_set, meter)?;
                activation.push(binder);
            }
            let retry = argument(args, 10).expect("retry");
            let earlier = self.target(group_value(retry, "earlier").expect("earlier"));
            level_set = self.introduce(earlier, &level_set, meter)?;
            let later = self.target(group_value(retry, "later").expect("later"));
            level_set = self.introduce(later, &level_set, meter)?;
            let mut captured = self.captures.iter().copied().collect::<BTreeSet<_>>();
            captured.extend(registration.iter().copied());
            captured.extend(activation.iter().copied());
            let mut retry_allowed = captured.clone();
            retry_allowed.extend([earlier, later]);
            self.slot(
                template,
                group_value(retry, "relation").expect("retry relation"),
                &retry_allowed,
                "retry",
                Some("retry.relation"),
                None,
                meter,
            )?;
            let recovery = argument(args, 12).expect("recovery");
            let recovery_binder =
                self.target(group_value(recovery, "binder").expect("recovery binder"));
            self.introduce(recovery_binder, &level_set, meter)?;
            captured.insert(recovery_binder);
            self.slot(
                template,
                group_value(recovery, "predicate").expect("recovery predicate"),
                &captured,
                "recovery",
                Some("recovery.predicate"),
                None,
                meter,
            )?;
        }
        Ok(())
    }
}

struct ControlScopeFrame {
    position: usize,
    input: BTreeSet<usize>,
    output: BTreeSet<usize>,
    children: Vec<usize>,
    next: usize,
}

#[string_edge]
fn model_state_operation(identity: &str) -> bool {
    matches!(
        identity,
        "quire.op.state.pre"
            | "quire.op.model.deref"
            | "quire.op.model.navigate"
            | "quire.op.model.reaches"
            | "quire.op.model.reaches_field"
            | "quire.op.model.lookup"
            | "quire.op.model.all_instances"
            | "quire.op.model.dispatch_call"
    )
}

#[string_edge]
fn control_binder<'a>(operator: &str, args: &'a [CheckedTerm]) -> Option<&'a CheckedTerm> {
    let ordinal = match operator {
        "send" | "commit" => 1,
        "receive" => 2,
        "attempt" => 4,
        "effect" => 1,
        "event" => 2,
        _ => return None,
    };
    argument(args, ordinal)
}

fn compensation_binders(args: &[CheckedTerm]) -> Vec<&CheckedTerm> {
    let mut binders = vec![argument(args, 1).expect("forward binder")];
    for capture in members(argument(args, 5).expect("registration captures")).expect("capture list")
    {
        binders.push(group_value(capture, "binder").expect("capture binder"));
    }
    binders
        .push(group_value(argument(args, 6).expect("trigger"), "binder").expect("trigger binder"));
    for capture in members(argument(args, 7).expect("activation captures")).expect("capture list") {
        binders.push(group_value(capture, "binder").expect("capture binder"));
    }
    let retry = argument(args, 10).expect("retry");
    for name in ["earlier", "later"] {
        binders.push(group_value(retry, name).expect("retry binder"));
    }
    binders.push(
        group_value(argument(args, 12).expect("recovery"), "binder").expect("recovery binder"),
    );
    binders
}

fn causal_edge(
    edges: &mut [Vec<usize>],
    from: usize,
    to: usize,
    position: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    meter.charge(1, || node_pointer(position))?;
    edges[from].push(to);
    Ok(())
}

#[string_edge]
fn check_clause_causality(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    validated: &[Option<(&str, &[CheckedTerm])>],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let root = binding_value(arguments, "run")
        .and_then(reference_target)
        .and_then(|target| check.index.get(target).copied())
        .expect("validated clause root");
    let mut preorder = Vec::new();
    let mut order = vec![None; check.nodes.len()];
    let mut pending = vec![root];
    while let Some(position) = pending.pop() {
        meter.charge(1, || node_pointer(position))?;
        if order[position].is_some() {
            continue;
        }
        order[position] = Some(preorder.len());
        preorder.push(position);
        let (operator, args) = validated[position].expect("validated control");
        for target in control_children(operator, args).into_iter().rev() {
            if let Some(&child) = check.index.get(target) {
                pending.push(child);
            }
        }
    }
    let mut edges = vec![Vec::new(); preorder.len() * 2];
    let port =
        |position: usize, exit: bool| order[position].map(|index| index * 2 + usize::from(exit));
    let cross_port = |target: usize, exit: bool, owner: usize, path: JsonPointer| {
        port(target, exit).ok_or_else(|| {
            ValidationFailure::refused_at(
                Code::InvalidModelBinding,
                path,
                Some(Cause::MalformedDeclaration),
                check.nodes[owner].node_id.clone(),
            )
        })
    };
    for &position in &preorder {
        meter.charge(1, || node_pointer(position))?;
        let (operator, args) = validated[position].expect("validated control");
        let enter = port(position, false).expect("tree member");
        let exit = enter + 1;
        let children: Vec<usize> = control_children(operator, args)
            .into_iter()
            .filter_map(|target| check.index.get(target).copied())
            .collect();
        match operator {
            "sequence" => {
                let mut previous = enter;
                for &child in &children {
                    causal_edge(
                        &mut edges,
                        previous,
                        port(child, false).expect("tree child"),
                        position,
                        meter,
                    )?;
                    previous = port(child, true).expect("tree child");
                }
                causal_edge(&mut edges, previous, exit, position, meter)?;
            }
            "choice" => {
                for &child in &children {
                    causal_edge(
                        &mut edges,
                        enter,
                        port(child, false).expect("tree child"),
                        position,
                        meter,
                    )?;
                    causal_edge(
                        &mut edges,
                        port(child, true).expect("tree child"),
                        exit,
                        position,
                        meter,
                    )?;
                }
            }
            "parallel" => {
                let branches = binding_value(args, "branches")
                    .and_then(members)
                    .expect("validated branches");
                let joined: BTreeSet<&str> = binding_value(args, "joined")
                    .into_iter()
                    .flat_map(|term| members(term).into_iter().flatten())
                    .filter_map(text_value)
                    .collect();
                for (branch, &child) in branches.iter().zip(&children) {
                    causal_edge(
                        &mut edges,
                        enter,
                        port(child, false).expect("tree child"),
                        position,
                        meter,
                    )?;
                    let label = group_value(branch, "label")
                        .and_then(text_value)
                        .expect("validated label");
                    if joined.contains(label) {
                        causal_edge(
                            &mut edges,
                            port(child, true).expect("tree child"),
                            exit,
                            position,
                            meter,
                        )?;
                    }
                }
            }
            "repeat" => {
                let body = children[0];
                causal_edge(
                    &mut edges,
                    enter,
                    port(body, false).expect("tree child"),
                    position,
                    meter,
                )?;
                causal_edge(
                    &mut edges,
                    port(body, true).expect("tree child"),
                    exit,
                    position,
                    meter,
                )?;
                if let Some(&exhausted) = children.get(1) {
                    causal_edge(
                        &mut edges,
                        enter,
                        port(exhausted, false).expect("tree child"),
                        position,
                        meter,
                    )?;
                    causal_edge(
                        &mut edges,
                        port(exhausted, true).expect("tree child"),
                        exit,
                        position,
                        meter,
                    )?;
                }
            }
            "await" => {
                let [event, then, timeout] = children.as_slice() else {
                    unreachable!("validated await")
                };
                for (from, to) in [
                    (enter, port(*event, false)),
                    (port(*event, true).expect("tree child"), port(*then, false)),
                    (enter, port(*timeout, false)),
                    (port(*then, true).expect("tree child"), Some(exit)),
                    (port(*timeout, true).expect("tree child"), Some(exit)),
                ] {
                    causal_edge(&mut edges, from, to.expect("tree child"), position, meter)?;
                }
                let after = binding_value(args, "after").expect("validated after");
                let target = group_value(after, "target")
                    .and_then(reference_target)
                    .and_then(|id| check.index.get(id).copied())
                    .expect("validated anchor");
                let anchor =
                    if group_value(after, "kind").and_then(text_value) == Some("compensation") {
                        let (_, template_args) = validated[target].expect("validated template");
                        let effect = binding_value(template_args, "forward")
                            .and_then(reference_target)
                            .and_then(|id| check.index.get(id).copied())
                            .expect("validated forward effect");
                        cross_port(
                            effect,
                            false,
                            position,
                            node_pointer(position)
                                .key("body")
                                .key("arguments")
                                .index(0)
                                .key("value")
                                .key("members")
                                .index(1)
                                .key("value")
                                .key("target"),
                        )?
                    } else {
                        cross_port(
                            target,
                            true,
                            position,
                            node_pointer(position)
                                .key("body")
                                .key("arguments")
                                .index(0)
                                .key("value")
                                .key("members")
                                .index(1)
                                .key("value")
                                .key("target"),
                        )?
                    };
                causal_edge(&mut edges, anchor, enter, position, meter)?;
            }
            "receive" | "effect" => {
                causal_edge(&mut edges, enter, exit, position, meter)?;
                let source = binding_value(args, "of")
                    .and_then(reference_target)
                    .and_then(|id| check.index.get(id).copied())
                    .expect("validated source");
                let source_port = cross_port(
                    source,
                    true,
                    position,
                    node_pointer(position)
                        .key("body")
                        .key("arguments")
                        .index(usize::from(operator == "receive"))
                        .key("value")
                        .key("target"),
                )?;
                causal_edge(&mut edges, source_port, enter, position, meter)?;
            }
            _ => causal_edge(&mut edges, enter, exit, position, meter)?,
        }
    }

    let mut colors = vec![0_u8; edges.len()];
    for start in 0..edges.len() {
        if colors[start] != 0 {
            continue;
        }
        colors[start] = 1;
        let mut stack = vec![(start, 0_usize)];
        while let Some((vertex, next)) = stack.last_mut() {
            meter.charge(1, || node_pointer(preorder[*vertex / 2]))?;
            if let Some(&successor) = edges[*vertex].get(*next) {
                *next += 1;
                if colors[successor] == 1 {
                    let first = stack
                        .iter()
                        .position(|(node, _)| *node == successor)
                        .expect("gray node in stack");
                    let controls = stack[first..]
                        .iter()
                        .map(|(node, _)| *node / 2)
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .map(|ordinal| check.nodes[preorder[ordinal]].node_id.clone())
                        .collect();
                    let position = preorder[successor / 2];
                    let mut failure = ValidationFailure::refused_at(
                        Code::InvalidPackage,
                        node_pointer(position),
                        Some(Cause::DefinitionCycle),
                        check.nodes[position].node_id.clone(),
                    );
                    if let ValidationFailure::Refused(ref mut refusal) = failure {
                        refusal.protocol_detail =
                            Some(Box::new(CheckedProtocolRefusalDetail::CausalCycle {
                                controls,
                            }));
                    }
                    return Err(failure);
                }
                if colors[successor] == 0 {
                    colors[successor] = 1;
                    stack.push((successor, 0));
                }
            } else {
                colors[*vertex] = 2;
                stack.pop();
            }
        }
    }

    let mut steps = vec![false; check.nodes.len()];
    for &position in preorder.iter().rev() {
        meter.charge(1, || node_pointer(position))?;
        let (operator, args) = validated[position].expect("validated control");
        let children = control_children(operator, args)
            .into_iter()
            .filter_map(|id| check.index.get(id).copied())
            .collect::<Vec<_>>();
        steps[position] = match operator {
            "send" | "receive" | "attempt" | "effect" | "event" | "commit" | "fence" | "await"
            | "parallel" => true,
            "sequence" => children.iter().any(|&child| steps[child]),
            "choice" => children.iter().all(|&child| steps[child]),
            "repeat" | "check" => false,
            _ => unreachable!("validated control operator"),
        };
    }
    for &position in &preorder {
        let Some(("repeat", args)) = validated[position] else {
            continue;
        };
        let body = binding_value(args, "body")
            .and_then(reference_target)
            .and_then(|id| check.index.get(id).copied())
            .expect("validated repeat body");
        if !steps[body] {
            let mut failure = ValidationFailure::refused_at(
                Code::UnsupportedConstruct,
                node_pointer(position),
                Some(Cause::DeclarationForm),
                check.nodes[position].node_id.clone(),
            );
            if let ValidationFailure::Refused(ref mut refusal) = failure {
                refusal.protocol_detail =
                    Some(Box::new(CheckedProtocolRefusalDetail::RepeatProgress {
                        control: check.nodes[position].node_id.clone(),
                        profile: check.profile.clone(),
                    }));
            }
            return Err(failure);
        }
    }
    Ok(())
}

/// Validate protocol placement, local members, membership and clause layout.
pub(super) fn validate_protocol_nodes(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    lock: &CheckedPackageLockV2,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<(), ValidationFailure> {
    let selected_profile = lock
        .profile_selections
        .iter()
        .find(|selection| selection.role == CheckedSelectionRole::ProtocolProfile)
        .map(|selection| &selection.definition);
    let has_temporal_profile = lock
        .profile_selections
        .iter()
        .any(|selection| selection.role == CheckedSelectionRole::TemporalProfile);

    // Placement is one complete pass, so it precedes every operator defect.
    let mut validated = vec![None; nodes.len()];
    for &position in index.values() {
        let Some(CheckedTerm::Application {
            operator: ApplicationOperator::ProtocolControl,
            ..
        }) = terms.get(position).and_then(Option::as_ref)
        else {
            continue;
        };
        if !matches!(
            kinds.get(position),
            Some(CheckedNodeKind::Protocol(form)) if *form != ProtocolForm::ProtocolReference
        ) {
            return Err(body_defect(
                position,
                &nodes[position],
                ProtocolBodyDefect::OperatorIneligible,
            ));
        }
    }

    for &position in index.values() {
        let Some(CheckedNodeKind::Protocol(form)) = kinds.get(position).copied() else {
            continue;
        };
        if form == ProtocolForm::ProtocolReference {
            continue;
        }
        let node = &nodes[position];
        meter.charge(1, || node_pointer(position).key("body"))?;
        let Some(body) = terms.get(position).and_then(Option::as_ref) else {
            return Err(body_defect(
                position,
                node,
                ProtocolBodyDefect::OperatorIneligible,
            ));
        };
        let (operator, arguments) =
            validate_envelope(form, body, &node.semantic_type, selected_profile)
                .map_err(|defect| body_defect(position, node, defect))?;
        validated[position] = Some((operator, arguments));

        if !has_temporal_profile && matches!(operator, "await" | "compensate") {
            return Err(ValidationFailure::refused_at(
                Code::MissingDeclaration,
                node_pointer(position),
                Some(Cause::MissingSelection),
                node.node_id.clone(),
            ));
        }
    }
    validate_protocol_declaration_names(nodes, kinds, index)?;
    validate_protocol_declaration_paths(nodes, kinds, &validated, index)?;
    // Local member shapes precede membership, which precedes clause checks.
    for form in [
        ProtocolForm::Role,
        ProtocolForm::Channel,
        ProtocolForm::Queue,
        ProtocolForm::Obligation,
        ProtocolForm::Control,
        ProtocolForm::Compensation,
    ] {
        for &position in index.values() {
            if kinds.get(position) != Some(&CheckedNodeKind::Protocol(form)) {
                continue;
            }
            let Some((operator, arguments)) = validated[position] else {
                return Err(body_defect(
                    position,
                    &nodes[position],
                    ProtocolBodyDefect::OperatorIneligible,
                ));
            };
            validate_layout(form, operator, arguments)
                .map_err(|defect| body_defect(position, &nodes[position], defect))?;
            validate_related_order(form, operator, arguments, bytes)
                .map_err(|defect| body_defect(position, &nodes[position], defect))?;
            let check = MemberCheck {
                position,
                nodes,
                kinds,
                terms,
                index,
                profile: selected_profile.expect("validated protocol profile law"),
            };
            check_member(
                &check, form, operator, arguments, &validated, owners, lock, meter,
            )?;
        }
    }
    validate_protocol_membership(nodes, kinds, &validated, index, meter)?;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        let Some((operator, arguments)) = validated[position] else {
            return Err(body_defect(
                position,
                &nodes[position],
                ProtocolBodyDefect::OperatorIneligible,
            ));
        };
        validate_layout(ProtocolForm::ProtocolClause, operator, arguments)
            .map_err(|defect| body_defect(position, &nodes[position], defect))?;
        let check = MemberCheck {
            position,
            nodes,
            kinds,
            terms,
            index,
            profile: selected_profile.expect("validated protocol profile law"),
        };
        check_clause(&check, arguments, &validated, meter)?;
        check_clause_causality(&check, arguments, &validated, meter)?;
        check_clause_slots(&check, arguments, &validated, meter)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::shared::CheckedNodeId;
    use crate::checked_package::v2::CheckedTerm;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn validate_protocol_body(
        form: ProtocolForm,
        body: &Value,
        semantic_type: &Value,
        selected_profile: Option<&Value>,
    ) -> Result<(), ProtocolBodyDefect> {
        let term = CheckedTerm::decode_admitted(body).expect("valid admitted term fixture");
        let type_id: CheckedNodeId =
            serde_json::from_value(semantic_type.clone()).expect("valid semantic type fixture");
        let selected = selected_profile.map(|value| {
            serde_json::from_value::<CheckedArtifactRef>(value.clone())
                .expect("valid selected profile fixture")
        });
        super::validate_protocol_body(
            form,
            &term,
            &type_id,
            selected.as_ref(),
            crate::checked_package::shared::CheckedPackageReadLimits::bounded().bytes,
        )
    }

    type QSpecFrame = (ProtocolForm, &'static str, &'static [(&'static str, Shape)]);

    fn example(shape: Shape) -> Value {
        let type_id = json!({"domain": "quire.checked-semantic-node/v1", "digest": "b".repeat(64)});
        match shape {
            Shape::Reference | Shape::ReferenceOrText | Shape::Term => {
                json!({"term": "reference", "target": {"domain": "quire.checked-semantic-node/v1", "digest": "a".repeat(64)}})
            }
            Shape::Text => {
                json!({"term": "literal", "value_kind": "text", "value": "name", "type": type_id})
            }
            Shape::Integer => {
                json!({"term": "literal", "value_kind": "integer", "value": 1, "type": type_id})
            }
            Shape::Boolean | Shape::BooleanCandidate => {
                json!({"term": "literal", "value_kind": "boolean", "value": true, "type": type_id})
            }
            Shape::References
            | Shape::Texts
            | Shape::OptionalReference
            | Shape::OptionalText
            | Shape::OptionalInteger
            | Shape::OptionalBooleanCandidate
            | Shape::Group
            | Shape::Records => json!({"term": "aggregate", "members": []}),
        }
    }

    fn body(form: ProtocolForm, operator: &str) -> Value {
        let arguments: Vec<Value> = layout(form, operator)
            .expect("catalogued operator")
            .iter()
            .map(|binding| {
                let value = example_binding(form, operator, binding);
                json!({"term": "binding", "name": binding.name, "value": value})
            })
            .collect();
        json!({
            "term": "application",
            "operator": "protocol_control",
            "operation": {
                "identity": "quire.op.protocol.control",
                "laws": [{"role": "protocol_profile", "definition": {"authority": "quire", "identity": "quire.protocol.finite-global/v1"}}],
                "mode": null,
                "member": {"kind": "profile_operator", "operator": operator},
                "leaves": []
            },
            "result_type": {"domain": "quire.checked-semantic-node/v1", "digest": "b".repeat(64)},
            "arguments": arguments
        })
    }

    fn selected() -> Value {
        json!({"authority": "quire", "identity": "quire.protocol.finite-global/v1"})
    }

    fn text(value: &str) -> Value {
        json!({
            "term": "literal", "value_kind": "text", "value": value,
            "type": {"domain": "quire.checked-semantic-node/v1", "digest": "b".repeat(64)}
        })
    }

    fn example_binding(form: ProtocolForm, operator: &str, binding: &Binding) -> Value {
        match (form, operator, binding.name) {
            (ProtocolForm::ProtocolClause, "protocol", "activation") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "kind", "value": text("origin")}
                ]
            }),
            (ProtocolForm::Channel, "channel", "ordering") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "kind", "value": text("unordered")}
                ]
            }),
            (ProtocolForm::Queue, "queue", "capacity") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "kind", "value": text("literal")},
                    {"term": "binding", "name": "value", "value": example(Shape::Integer)}
                ]
            }),
            (ProtocolForm::Queue, "queue", "overflow") => text("reject"),
            (ProtocolForm::Channel, "channel", "delivery") => text("unknown"),
            (ProtocolForm::Obligation, "terminal", "mode") => text("any"),
            (ProtocolForm::Control, "choice", "cases") => json!({
                "term": "aggregate", "members": [
                    {"term": "aggregate", "members": [
                        {"term": "binding", "name": "label", "value": text("Left")},
                        {"term": "binding", "name": "guard", "value": example(Shape::Boolean)},
                        {"term": "binding", "name": "body", "value": example(Shape::Reference)}
                    ]},
                    {"term": "aggregate", "members": [
                        {"term": "binding", "name": "label", "value": text("Right")},
                        {"term": "binding", "name": "guard", "value": example(Shape::Boolean)},
                        {"term": "binding", "name": "body", "value": example(Shape::Reference)}
                    ]}
                ]
            }),
            (ProtocolForm::Control, "parallel", "branches") => json!({
                "term": "aggregate", "members": [
                    {"term": "aggregate", "members": [
                        {"term": "binding", "name": "label", "value": text("Left")},
                        {"term": "binding", "name": "body", "value": example(Shape::Reference)}
                    ]},
                    {"term": "aggregate", "members": [
                        {"term": "binding", "name": "label", "value": text("Right")},
                        {"term": "binding", "name": "body", "value": example(Shape::Reference)}
                    ]}
                ]
            }),
            (ProtocolForm::Control, "parallel", "join") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "policy", "value": text("all")}
                ]
            }),
            (ProtocolForm::Control, "parallel", "joined") => json!({
                "term": "aggregate", "members": [text("Left"), text("Right")]
            }),
            (ProtocolForm::Control, "await", "after") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "kind", "value": text("event")},
                    {"term": "binding", "name": "target", "value": example(Shape::Reference)}
                ]
            }),
            (ProtocolForm::Control, "await", "within") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "lower", "value": example(Shape::Integer)},
                    {"term": "binding", "name": "upper", "value": example(Shape::Integer)}
                ]
            }),
            (ProtocolForm::Compensation, "compensate", "trigger") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "binder", "value": example(Shape::Reference)},
                    {"term": "binding", "name": "guard", "value": example(Shape::Boolean)}
                ]
            }),
            (ProtocolForm::Compensation, "compensate", "within") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "lower", "value": example(Shape::Integer)},
                    {"term": "binding", "name": "upper", "value": example(Shape::Integer)}
                ]
            }),
            (ProtocolForm::Compensation, "compensate", "attempts") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "maximum", "value": example(Shape::Integer)},
                    {"term": "binding", "name": "record_type", "value": example(Shape::Reference)}
                ]
            }),
            (ProtocolForm::Compensation, "compensate", "retry") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "earlier", "value": example(Shape::Reference)},
                    {"term": "binding", "name": "later", "value": example(Shape::Reference)},
                    {"term": "binding", "name": "relation", "value": example(Shape::Boolean)}
                ]
            }),
            (ProtocolForm::Compensation, "compensate", "recovery") => json!({
                "term": "aggregate", "members": [
                    {"term": "binding", "name": "binder", "value": example(Shape::Reference)},
                    {"term": "binding", "name": "predicate", "value": example(Shape::Boolean)}
                ]
            }),
            _ => example(binding.shape),
        }
    }

    fn boolean() -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": "b".repeat(64)})
    }

    /// QSpec TC-946 covers the twenty names and seven forms in FR-464.
    #[trace("TC-048")]
    #[test]
    fn protocol_outer_catalog_recognizes_each_operator_only_for_its_form() {
        let cases = [
            (ProtocolForm::ProtocolClause, "protocol"),
            (ProtocolForm::Role, "role"),
            (ProtocolForm::Channel, "channel"),
            (ProtocolForm::Queue, "queue"),
            (ProtocolForm::Obligation, "finish"),
            (ProtocolForm::Obligation, "terminal"),
            (ProtocolForm::Control, "sequence"),
            (ProtocolForm::Control, "choice"),
            (ProtocolForm::Control, "parallel"),
            (ProtocolForm::Control, "repeat"),
            (ProtocolForm::Control, "await"),
            (ProtocolForm::Control, "check"),
            (ProtocolForm::Control, "send"),
            (ProtocolForm::Control, "receive"),
            (ProtocolForm::Control, "attempt"),
            (ProtocolForm::Control, "effect"),
            (ProtocolForm::Control, "event"),
            (ProtocolForm::Control, "commit"),
            (ProtocolForm::Control, "fence"),
            (ProtocolForm::Compensation, "compensate"),
        ];
        for (form, operator) in cases {
            let value = body(form, operator);
            assert_eq!(
                validate_protocol_body(form, &value, &boolean(), Some(&selected())),
                Ok(()),
                "{operator}"
            );
            let other_form = if form == ProtocolForm::ProtocolClause {
                ProtocolForm::Role
            } else {
                ProtocolForm::ProtocolClause
            };
            assert_eq!(
                validate_protocol_body(other_form, &value, &boolean(), Some(&selected())),
                Err(ProtocolBodyDefect::OperatorIneligible),
                "{operator}"
            );
        }
        let mut unknown = body(ProtocolForm::Control, "check");
        unknown["operation"]["member"]["operator"] = json!("jump");
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Control,
                &unknown,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MemberMismatch)
        );
        unknown["operation"]["member"] = Value::Null;
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Control,
                &unknown,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MemberMismatch)
        );
    }

    /// QSpec TC-946, TC-947 and TC-948 supply these independent representative frames.
    #[trace("TC-048")]
    #[test]
    fn protocol_outer_layout_accepts_independent_qspec_frames() {
        let cases: &[QSpecFrame] = &[
            (
                ProtocolForm::ProtocolClause,
                "protocol",
                &[
                    ("over", Shape::Reference),
                    ("activation", Shape::Group),
                    ("captures", Shape::Records),
                    ("roles", Shape::References),
                    ("relationships", Shape::References),
                    ("channels", Shape::References),
                    ("queues", Shape::References),
                    ("requirements", Shape::References),
                    ("compensations", Shape::References),
                    ("terminal", Shape::OptionalReference),
                    ("scheduling", Shape::OptionalText),
                    ("run", Shape::Reference),
                    ("finish", Shape::Reference),
                ],
            ),
            (
                ProtocolForm::Role,
                "role",
                &[
                    ("model", Shape::ReferenceOrText),
                    ("replication", Shape::Group),
                ],
            ),
            (
                ProtocolForm::Channel,
                "channel",
                &[
                    ("from", Shape::Reference),
                    ("to", Shape::Reference),
                    ("carries", Shape::Reference),
                    ("ordering", Shape::Group),
                    ("delivery", Shape::Text),
                ],
            ),
            (
                ProtocolForm::Queue,
                "queue",
                &[
                    ("channel", Shape::Reference),
                    ("capacity", Shape::Group),
                    ("overflow", Shape::Text),
                    ("loss", Shape::OptionalText),
                ],
            ),
            (
                ProtocolForm::Obligation,
                "finish",
                &[
                    ("binder", Shape::Reference),
                    ("constraint", Shape::BooleanCandidate),
                ],
            ),
            (
                ProtocolForm::Obligation,
                "terminal",
                &[
                    ("mode", Shape::Text),
                    ("condition", Shape::OptionalBooleanCandidate),
                ],
            ),
            (
                ProtocolForm::Control,
                "choice",
                &[
                    ("owner", Shape::Reference),
                    ("visible", Shape::References),
                    ("cases", Shape::Records),
                ],
            ),
            (
                ProtocolForm::Compensation,
                "compensate",
                &[
                    ("forward", Shape::Reference),
                    ("forward_binder", Shape::Reference),
                    ("role", Shape::Reference),
                    ("operation", Shape::Reference),
                    ("clock", Shape::Text),
                    ("registration_captures", Shape::Records),
                    ("trigger", Shape::Group),
                    ("activation_captures", Shape::Records),
                    ("within", Shape::Group),
                    ("attempts", Shape::Group),
                    ("retry", Shape::Group),
                    ("commit", Shape::OptionalReference),
                    ("recovery", Shape::Group),
                ],
            ),
        ];
        for &(form, operator, bindings) in cases {
            let mut value = body(form, operator);
            value["arguments"] = Value::Array(
                bindings
                    .iter()
                    .map(|&(name, shape)| {
                        let term = example_binding(form, operator, &Binding { name, shape });
                        json!({"term": "binding", "name": name, "value": term})
                    })
                    .collect(),
            );
            assert_eq!(
                validate_protocol_body(form, &value, &boolean(), Some(&selected())),
                Ok(()),
                "QSpec frame for {operator}"
            );
            value["arguments"][0]["name"] = json!("not_a_protocol_binding");
            assert_eq!(
                validate_protocol_body(form, &value, &boolean(), Some(&selected())),
                Err(ProtocolBodyDefect::MalformedLayout),
                "mutated QSpec frame for {operator}"
            );
        }
    }

    /// The role, channel and queue cases exercise QSpec TC-947 outer shapes.
    #[trace("TC-048")]
    #[test]
    fn protocol_declaration_layout_refuses_missing_swapped_and_wrong_shaped_bindings() {
        let role = body(ProtocolForm::Role, "role");
        assert_eq!(
            validate_protocol_body(ProtocolForm::Role, &role, &boolean(), Some(&selected())),
            Ok(())
        );
        let mut swapped = body(ProtocolForm::Channel, "channel");
        swapped["arguments"]
            .as_array_mut()
            .expect("arguments")
            .swap(0, 1);
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Channel,
                &swapped,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        let mut missing = body(ProtocolForm::Queue, "queue");
        missing["arguments"]
            .as_array_mut()
            .expect("arguments")
            .pop();
        assert_eq!(
            validate_protocol_body(ProtocolForm::Queue, &missing, &boolean(), Some(&selected())),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        let mut literal_target = body(ProtocolForm::Role, "role");
        literal_target["arguments"][0]["value"] = example(Shape::Integer);
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Role,
                &literal_target,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        let mut repeated_optional = body(ProtocolForm::Queue, "queue");
        repeated_optional["arguments"][3]["value"]["members"] =
            json!([example(Shape::Text), example(Shape::Text)]);
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Queue,
                &repeated_optional,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
    }

    /// QSpec TC-948 supplies the fixed control and compensation argument frames.
    #[trace("TC-048")]
    #[test]
    fn control_and_compensation_layouts_keep_their_order_and_term_families() {
        let mut compensate = body(ProtocolForm::Compensation, "compensate");
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Compensation,
                &compensate,
                &boolean(),
                Some(&selected())
            ),
            Ok(())
        );
        compensate["arguments"][12]["name"] = json!("commit");
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Compensation,
                &compensate,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        let mut check = body(ProtocolForm::Control, "check");
        check["arguments"][0]["value"] = example(Shape::References);
        assert_eq!(
            validate_protocol_body(ProtocolForm::Control, &check, &boolean(), Some(&selected())),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        let mut sequence = body(ProtocolForm::Control, "sequence");
        sequence["arguments"][0]["value"]["members"] = json!([example(Shape::Text)]);
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Control,
                &sequence,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        let mut choice = body(ProtocolForm::Control, "choice");
        choice["arguments"][2]["value"]["members"] =
            json!([{"term": "aggregate", "members": [example(Shape::Reference)]}]);
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Control,
                &choice,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
    }

    /// Trace: FR-038-AC-282, FR-038-AC-289
    #[trace("TC-048")]
    #[test]
    fn compensation_nested_groups_require_their_fixed_bindings() {
        let valid = body(ProtocolForm::Compensation, "compensate");
        for ordinal in [6, 8, 9, 10, 12] {
            let mut missing = valid.clone();
            missing["arguments"][ordinal]["value"]["members"] = json!([]);
            assert_eq!(
                validate_protocol_body(
                    ProtocolForm::Compensation,
                    &missing,
                    &boolean(),
                    Some(&selected()),
                ),
                Err(ProtocolBodyDefect::MalformedLayout),
                "nested group at argument {ordinal}"
            );
        }
        let mut captures = valid;
        captures["arguments"][5]["value"]["members"] = json!([{
            "term": "aggregate", "members": [
                {"term": "binding", "name": "initializer", "value": example(Shape::Term)},
                {"term": "binding", "name": "binder", "value": example(Shape::Reference)}
            ]
        }]);
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Compensation,
                &captures,
                &boolean(),
                Some(&selected()),
            ),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
    }

    /// QSpec TC-946 also requires the selected profile law to be exact.
    #[trace("TC-048")]
    #[test]
    fn protocol_body_distinguishes_profile_identity_and_law_defects() {
        let mut value = body(ProtocolForm::Control, "check");
        let complete = json!({"authority": "quire", "identity": "quire.protocol.complete/v1"});
        assert_eq!(
            validate_protocol_body(ProtocolForm::Control, &value, &boolean(), Some(&complete)),
            Err(ProtocolBodyDefect::LawUnselected)
        );
        assert_eq!(
            validate_protocol_body(ProtocolForm::Control, &value, &boolean(), None),
            Err(ProtocolBodyDefect::LawUnselected)
        );
        value["operation"]["laws"] = json!([]);
        assert_eq!(
            validate_protocol_body(ProtocolForm::Control, &value, &boolean(), Some(&selected())),
            Err(ProtocolBodyDefect::LawMissing)
        );
        value["operation"]["laws"] =
            json!([{"role": "temporal_profile", "definition": selected()}]);
        assert_eq!(
            validate_protocol_body(ProtocolForm::Control, &value, &boolean(), Some(&selected())),
            Err(ProtocolBodyDefect::LawMismatch)
        );
        value["operation"]["laws"] = json!([
            {"role": "protocol_profile", "definition": selected()},
            {"role": "protocol_profile", "definition": selected()}
        ]);
        assert_eq!(
            validate_protocol_body(ProtocolForm::Control, &value, &boolean(), Some(&selected())),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
    }

    /// QSpec TC-946 requires the shared application class and result type.
    #[trace("TC-048")]
    #[test]
    fn protocol_application_requires_its_operator_class_and_result_type() {
        let mut wrong_class = body(ProtocolForm::Control, "send");
        wrong_class["operator"] = json!("state_clause");
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Control,
                &wrong_class,
                &boolean(),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::OperatorIneligible)
        );
        let wrong_type = body(ProtocolForm::Control, "send");
        assert_eq!(
            validate_protocol_body(
                ProtocolForm::Control,
                &wrong_type,
                &json!({"domain": "quire.checked-semantic-node/v1", "digest": "c".repeat(64)}),
                Some(&selected())
            ),
            Err(ProtocolBodyDefect::OperatorIneligible)
        );
    }

    /// QSpec TC-946 C2 and C8-C9 fix the activation group's exact alternatives.
    #[trace("TC-048")]
    #[test]
    fn protocol_clause_activation_requires_origin_or_each_shape() {
        let mut clause = body(ProtocolForm::ProtocolClause, "protocol");
        let admitted = |body: &Value| {
            validate_protocol_body(
                ProtocolForm::ProtocolClause,
                body,
                &boolean(),
                Some(&selected()),
            )
        };
        assert_eq!(admitted(&clause), Ok(()));

        clause["arguments"][1]["value"]["members"] = json!([
            {"term": "binding", "name": "kind", "value": text("each")},
            {"term": "binding", "name": "trigger", "value": example(Shape::Reference)},
            {"term": "binding", "name": "guard", "value": example(Shape::Boolean)}
        ]);
        assert_eq!(admitted(&clause), Ok(()));

        clause["arguments"][1]["value"]["members"][0]["value"] = text("always");
        assert_eq!(admitted(&clause), Err(ProtocolBodyDefect::MalformedLayout));
        clause["arguments"][1]["value"]["members"][0]["value"] = text("origin");
        assert_eq!(admitted(&clause), Err(ProtocolBodyDefect::MalformedLayout));
        clause["arguments"][1]["value"]["members"] = json!([
            {"term": "binding", "name": "kind", "value": text("each")}
        ]);
        assert_eq!(admitted(&clause), Err(ProtocolBodyDefect::MalformedLayout));
    }

    /// QSpec TC-946 C1-C6 and K3 separate ordered captures from canonical sets.
    #[trace("TC-048")]
    #[test]
    fn protocol_clause_checks_capture_records_scheduling_and_set_order() {
        let mut clause = body(ProtocolForm::ProtocolClause, "protocol");
        let check = |body: &Value| {
            validate_protocol_body(
                ProtocolForm::ProtocolClause,
                body,
                &boolean(),
                Some(&selected()),
            )
        };
        clause["arguments"][2]["value"]["members"] = json!([
            {"term": "aggregate", "members": [
                {"term": "binding", "name": "binder", "value": example(Shape::Reference)},
                {"term": "binding", "name": "initializer", "value": example(Shape::Term)}
            ]}
        ]);
        clause["arguments"][10]["value"]["members"] = json!([text("adversarial")]);
        assert_eq!(check(&clause), Ok(()));
        clause["arguments"][2]["value"]["members"][0]["members"][0]["name"] = json!("initializer");
        assert_eq!(check(&clause), Err(ProtocolBodyDefect::MalformedLayout));
        clause["arguments"][2]["value"]["members"] = json!([]);
        clause["arguments"][10]["value"]["members"] = json!([text("fair")]);
        assert_eq!(check(&clause), Err(ProtocolBodyDefect::MalformedLayout));

        clause["arguments"][10]["value"]["members"] = json!([]);
        let mut first = example(Shape::Reference);
        let mut second = example(Shape::Reference);
        first["target"]["digest"] = json!("a".repeat(64));
        second["target"]["digest"] = json!("b".repeat(64));
        clause["arguments"][3]["value"]["members"] = json!([first, second]);
        assert_eq!(check(&clause), Ok(()));
        clause["arguments"][3]["value"]["members"]
            .as_array_mut()
            .expect("roles set")
            .swap(0, 1);
        assert_eq!(check(&clause), Err(ProtocolBodyDefect::NoncanonicalSet));
    }

    /// QSpec TC-947 fixes the nested declaration alternatives and their enums.
    #[trace("TC-048")]
    #[test]
    fn protocol_declaration_nested_shapes_refuse_wrong_alternatives() {
        let check =
            |form, body: &Value| validate_protocol_body(form, body, &boolean(), Some(&selected()));

        let mut role = body(ProtocolForm::Role, "role");
        assert_eq!(check(ProtocolForm::Role, &role), Ok(()));
        role["arguments"][1]["value"]["members"] = json!([
            {"term": "binding", "name": "population", "value": text("orders")},
            {"term": "binding", "name": "max", "value": example(Shape::Integer)},
            {"term": "binding", "name": "binder", "value": example(Shape::Reference)},
            {"term": "binding", "name": "lifetime", "value": text("until")},
            {"term": "binding", "name": "until", "value": example(Shape::Boolean)}
        ]);
        assert_eq!(check(ProtocolForm::Role, &role), Ok(()));
        role["arguments"][1]["value"]["members"]
            .as_array_mut()
            .expect("replication fields")
            .pop();
        assert_eq!(
            check(ProtocolForm::Role, &role),
            Err(ProtocolBodyDefect::MalformedLayout)
        );

        let mut channel = body(ProtocolForm::Channel, "channel");
        assert_eq!(check(ProtocolForm::Channel, &channel), Ok(()));
        channel["arguments"][3]["value"]["members"] = json!([
            {"term": "binding", "name": "kind", "value": text("fifo")},
            {"term": "binding", "name": "binder", "value": example(Shape::Reference)},
            {"term": "binding", "name": "key", "value": example(Shape::Term)}
        ]);
        assert_eq!(check(ProtocolForm::Channel, &channel), Ok(()));
        channel["arguments"][4]["value"] = text("best-effort");
        assert_eq!(
            check(ProtocolForm::Channel, &channel),
            Err(ProtocolBodyDefect::MalformedLayout)
        );

        let mut queue = body(ProtocolForm::Queue, "queue");
        assert_eq!(check(ProtocolForm::Queue, &queue), Ok(()));
        queue["arguments"][2]["value"] = text("loss");
        assert_eq!(
            check(ProtocolForm::Queue, &queue),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        queue["arguments"][3]["value"]["members"] = json!([text("head")]);
        assert_eq!(check(ProtocolForm::Queue, &queue), Ok(()));

        let mut terminal = body(ProtocolForm::Obligation, "terminal");
        assert_eq!(check(ProtocolForm::Obligation, &terminal), Ok(()));
        terminal["arguments"][0]["value"] = text("when");
        assert_eq!(
            check(ProtocolForm::Obligation, &terminal),
            Err(ProtocolBodyDefect::MalformedLayout)
        );
        terminal["arguments"][1]["value"]["members"] = json!([example(Shape::Boolean)]);
        assert_eq!(check(ProtocolForm::Obligation, &terminal), Ok(()));
    }

    /// Trace: FR-038-AC-288
    #[trace("TC-048")]
    #[test]
    fn repeat_and_await_nested_layouts_enforce_pair_and_field_order() {
        let check = |body: &Value| {
            validate_protocol_body(ProtocolForm::Control, body, &boolean(), Some(&selected()))
        };
        let mut repeat = body(ProtocolForm::Control, "repeat");
        assert_eq!(check(&repeat), Ok(()));
        repeat["arguments"][2]["value"]["members"] = json!([example(Shape::Integer)]);
        assert_eq!(check(&repeat), Err(ProtocolBodyDefect::MalformedLayout));
        repeat["arguments"][6]["value"]["members"] = json!([example(Shape::Reference)]);
        assert_eq!(check(&repeat), Ok(()));
        repeat["arguments"][3]["value"]["members"] = json!([
            {"term": "binding", "name": "variant", "value": example(Shape::Term)},
            {"term": "binding", "name": "invariant", "value": example(Shape::Boolean)}
        ]);
        assert_eq!(check(&repeat), Err(ProtocolBodyDefect::MalformedLayout));

        let mut awaited = body(ProtocolForm::Control, "await");
        assert_eq!(check(&awaited), Ok(()));
        awaited["arguments"][0]["value"]["members"]
            .as_array_mut()
            .expect("after")
            .swap(0, 1);
        assert_eq!(check(&awaited), Err(ProtocolBodyDefect::MalformedLayout));
        awaited = body(ProtocolForm::Control, "await");
        awaited["arguments"][2]["value"]["members"]
            .as_array_mut()
            .expect("within")
            .swap(0, 1);
        assert_eq!(check(&awaited), Err(ProtocolBodyDefect::MalformedLayout));
    }
}
