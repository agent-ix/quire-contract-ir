//! The fixed outer body grammar of the twenty finite global protocol operators.
//!
//! The protocol step runs after refinement and timed models, before operations.

use super::{
    ApplicationOperator, CheckedNodeKind, CheckedPackageLockV2, CheckedSelectionRole, CheckedTerm,
    LiteralKind, ProtocolForm, RawSemanticNodeV2, ScalarTypeForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{CheckedArtifactRef, CheckedNodeId};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use ix_trace_rs::string_edge;
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
    References,
    Texts,
    OptionalReference,
    OptionalText,
    OptionalInteger,
    OptionalBoolean,
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
    field("constraint", Shape::Boolean),
];
const TERMINAL: &[Binding] = &[
    field("mode", Shape::Text),
    field("condition", Shape::OptionalBoolean),
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
    field("guard", Shape::Boolean),
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
const CHECK: &[Binding] = &[field("condition", Shape::Boolean)];
const SEND: &[Binding] = &[
    field("channel", Shape::Reference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::Boolean),
];
const RECEIVE: &[Binding] = &[
    field("channel", Shape::Reference),
    field("of", Shape::Reference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::Boolean),
];
const ATTEMPT: &[Binding] = &[
    field("role", Shape::Reference),
    field("operation", Shape::Reference),
    field("ordering", Shape::OptionalText),
    field("contracts", Shape::References),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::Boolean),
];
const EFFECT: &[Binding] = &[
    field("of", Shape::Reference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::Boolean),
];
const EVENT: &[Binding] = &[
    field("role", Shape::Reference),
    field("compensation", Shape::OptionalReference),
    field("binder", Shape::Reference),
    field("related", Shape::Records),
    field("constraint", Shape::Boolean),
];
const COMMIT: &[Binding] = &[
    field("role", Shape::Reference),
    field("binder", Shape::Reference),
    field("constraint", Shape::Boolean),
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
                || matches!(
                    value,
                    CheckedTerm::Literal {
                        value_kind: LiteralKind::Boolean,
                        value,
                        ..
                    } if value.as_bool().is_some()
                )
        }
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
        Shape::OptionalBoolean => members(value).is_some_and(|items| {
            items.len() <= 1 && items.iter().all(|item| shape_fits(item, Shape::Boolean))
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

fn local_details_fit(form: ProtocolForm, operator: &str, arguments: &[CheckedTerm]) -> bool {
    match (form, operator) {
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
                        ("until", Shape::Boolean),
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
    {
        return Err(D::NoncanonicalSet);
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
    matches!(
        validate_layout(form, operator, arguments),
        Err(ProtocolBodyDefect::MalformedLayout)
    )
    .then_some(ProtocolLayoutWireDefect::Body)
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
) -> Result<(), ProtocolBodyDefect> {
    let (operator, arguments) = validate_envelope(form, body, semantic_type, selected_profile)?;
    validate_layout(form, operator, arguments)
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

fn record_declared_path(
    expected: &mut BTreeMap<usize, (Vec<Box<str>>, bool)>,
    position: usize,
    path: Vec<Box<str>>,
) {
    if let Some((_, repeated)) = expected.get_mut(&position) {
        *repeated = true;
    } else {
        expected.insert(position, (path, false));
    }
}

fn charge_path_copy(
    meter: &mut WorkMeter,
    position: usize,
    path: &[Box<str>],
) -> Result<(), ValidationFailure> {
    let bytes = path.iter().fold(0_u64, |total, segment| {
        total.saturating_add(u64::try_from(segment.len()).unwrap_or(u64::MAX))
    });
    meter.charge(bytes, || node_pointer(position).key("declaration"))
}

#[string_edge]
fn validate_protocol_declaration_paths(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &BTreeMap<&CheckedNodeId, usize>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut expected = BTreeMap::new();
    let mut pending = Vec::new();
    for &clause in index.values() {
        if kinds.get(clause) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        let Some(prefix) = nodes[clause]
            .declaration
            .as_ref()
            .map(|d| &d.qualified_name)
        else {
            continue;
        };
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
                        charge_path_copy(meter, position, prefix)?;
                        let mut path = prefix.clone();
                        path.push(segment.into());
                        if let Some(name) = nodes[position]
                            .declaration
                            .as_ref()
                            .and_then(|d| d.qualified_name.last())
                        {
                            path.push(name.clone());
                        }
                        record_declared_path(&mut expected, position, path);
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
                    charge_path_copy(meter, position, prefix)?;
                    let mut path = prefix.clone();
                    path.push(segment.into());
                    if segment == "finish" {
                        if let Some(name) = nodes[position]
                            .declaration
                            .as_ref()
                            .and_then(|d| d.qualified_name.last())
                        {
                            path.push(name.clone());
                        }
                    }
                    record_declared_path(&mut expected, position, path);
                }
            }
        }
        if let Some(&position) = binding_value(args, "run")
            .and_then(reference_target)
            .and_then(|key| index.get(key))
        {
            if kinds.get(position) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                charge_path_copy(meter, position, prefix)?;
                let mut path = prefix.clone();
                path.push("run".into());
                pending.push((position, path));
            }
        }
    }

    // The visited set bounds malformed containment cycles. Membership and
    // repeated holders are checked in the later graph pass.
    let mut visited = BTreeSet::new();
    while let Some((position, mut parent_path)) = pending.pop() {
        let Some(name) = nodes[position]
            .declaration
            .as_ref()
            .and_then(|d| d.qualified_name.last())
        else {
            continue;
        };
        parent_path.push(name.clone());
        record_declared_path(&mut expected, position, parent_path.clone());
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
                    charge_path_copy(meter, child, &parent_path)?;
                    pending.push((child, parent_path.clone()));
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
        if let Some((path, false)) = expected.get(&position) {
            if node
                .declaration
                .as_ref()
                .is_none_or(|d| &d.qualified_name != path)
            {
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
    meter: &mut WorkMeter,
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
    validate_protocol_declaration_paths(nodes, kinds, &validated, index, meter)?;
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
        super::validate_protocol_body(form, &term, &type_id, selected.as_ref())
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
            Shape::Boolean => {
                json!({"term": "literal", "value_kind": "boolean", "value": true, "type": type_id})
            }
            Shape::References
            | Shape::Texts
            | Shape::OptionalReference
            | Shape::OptionalText
            | Shape::OptionalInteger
            | Shape::OptionalBoolean
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
                &[("binder", Shape::Reference), ("constraint", Shape::Boolean)],
            ),
            (
                ProtocolForm::Obligation,
                "terminal",
                &[("mode", Shape::Text), ("condition", Shape::OptionalBoolean)],
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
        check["arguments"][0]["value"] = example(Shape::Integer);
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
}
