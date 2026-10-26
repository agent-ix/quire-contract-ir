//! The fixed outer body grammar of the twenty finite global protocol operators.
//!
//! The protocol step runs after refinement and timed models, before operations.

use super::model_members::{Budget, ModelOwners};
use super::{
    ApplicationOperator, CheckedNodeKind, CheckedPackageLockV2, CheckedSelectionRole, CheckedTerm,
    CompositeTypeForm, LiteralKind, ModelForm, ProtocolForm, RawSemanticNodeV2, RelationForm,
    ScalarTypeForm, StateClauseKind, StateForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedProtocolRefusalDetail,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use ix_trace_rs::string_edge;
use num_bigint::BigInt;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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
    field("model", Shape::Reference),
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

    fn same_clause(&self, target: usize, segment: &str) -> bool {
        let Some(own) = self.nodes[self.position].declaration.as_ref() else {
            return false;
        };
        let Some(other) = self.nodes[target].declaration.as_ref() else {
            return false;
        };
        let own = own.qualified_name.as_slice();
        let other = other.qualified_name.as_slice();
        own.len() >= 2
            && other.len() >= 2
            && own[..own.len() - 2] == other[..other.len() - 2]
            && other[other.len() - 2].as_ref() == segment
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

    #[string_edge]
    fn queue_channel_same_clause(&self, channel: usize) -> bool {
        let Some(channel_path) = self.nodes[channel]
            .declaration
            .as_ref()
            .map(|declaration| declaration.qualified_name.as_slice())
        else {
            return false;
        };
        for &clause in self.index.values() {
            if !self.is(
                clause,
                CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause),
            ) {
                continue;
            }
            let Some(CheckedTerm::Application { arguments, .. }) = self.terms[clause].as_ref()
            else {
                continue;
            };
            let listed = binding_value(arguments, "queues").is_some_and(|queues| {
                listed_targets(queues).any(|target| target == &self.nodes[self.position].node_id)
            });
            if !listed {
                continue;
            }
            let Some(prefix) = self.nodes[clause]
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
            else {
                return false;
            };
            if channel_path.len() != prefix.len() + 2
                || !channel_path.starts_with(prefix)
                || channel_path[prefix.len()].as_ref() != "channel"
            {
                return false;
            }
        }
        // An unlisted queue is diagnosed by the later membership pass.
        true
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
            if !check.same_clause(role, "role") {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    check.argument_path(0).key("target"),
                ));
            }
            let operation = check.target_at(arguments, 1, |position| {
                check.is(position, CheckedNodeKind::State(StateForm::OperationAnchor))
            })?;
            for (ordinal, contract) in members(check.at(arguments, 3))
                .expect("validated contracts")
                .iter()
                .enumerate()
            {
                let path = check.argument_path(3).key("members").index(ordinal);
                let clause = check.check_target(contract, path.clone(), |position| {
                    check.is(position, CheckedNodeKind::State(StateForm::StateClause))
                })?;
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
            check.target_at(arguments, 4, |position| {
                check.is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            })?;
            check_related_references(check, arguments, 5, owners, meter)?;
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
            if !check.same_clause(role, "role") {
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
                if !check.same_clause(template, "compensate") {
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
            if !check.same_clause(role, "role") {
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
fn check_member(
    check: &MemberCheck<'_>,
    form: ProtocolForm,
    operator: &str,
    arguments: &[CheckedTerm],
    owners: &ModelOwners<'_>,
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
            let replication = check.at(arguments, 1);
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
                let type_position = check.index.get(&check.nodes[binder].semantic_type).copied();
                if !type_position.is_some_and(|position| {
                    check.is(position, CheckedNodeKind::CompositeType(CompositeTypeForm::Reference))
                        && check.terms[position].as_ref().and_then(members).is_some_and(|items| {
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
        ProtocolForm::Channel => {
            for ordinal in [0, 1] {
                let target = check.target_at(arguments, ordinal, role)?;
                if !check.same_clause(target, "role") {
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
            let channel = check.target_at(arguments, 0, |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Channel))
            })?;
            if !check.queue_channel_same_clause(channel) {
                return Err(check.refuse(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    check.argument_path(0).key("target"),
                ));
            }
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
        ProtocolForm::Compensation => {}
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

/// First protocol reader passes: placement, operator/law and local layout.
/// Graph member joins, membership and clause scope follow in later passes.
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
    // Local member shapes are visited by form, then by ascending key. The
    // clause follows the members, as FR-464's reader order requires.
    for form in [
        ProtocolForm::Role,
        ProtocolForm::Channel,
        ProtocolForm::Queue,
        ProtocolForm::Obligation,
        ProtocolForm::Control,
        ProtocolForm::Compensation,
        ProtocolForm::ProtocolClause,
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
            if matches!(
                form,
                ProtocolForm::Role
                    | ProtocolForm::Channel
                    | ProtocolForm::Queue
                    | ProtocolForm::Obligation
                    | ProtocolForm::Control
            ) {
                let check = MemberCheck {
                    position,
                    nodes,
                    kinds,
                    terms,
                    index,
                    profile: selected_profile.expect("validated protocol profile law"),
                };
                check_member(&check, form, operator, arguments, owners, meter)?;
            }
        }
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
            Shape::Reference | Shape::Term => {
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
                &[("model", Shape::Reference), ("replication", Shape::Group)],
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
        literal_target["arguments"][0]["value"] = example(Shape::Text);
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
