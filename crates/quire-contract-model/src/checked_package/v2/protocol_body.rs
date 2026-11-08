//! The fixed outer body grammar of the twenty finite global protocol operators.
//!
//! The protocol step runs after refinement and timed models, before operations.

use super::graph_storage::GraphIndex;
use super::model_members::{Budget, ModelOwners};
use super::{
    ApplicationOperator, CheckedFairnessTarget, CheckedNodeKind, CheckedPackageLockV2,
    CheckedPackageV2, CheckedSelectionRole, CheckedTerm, CompositeTypeForm, ExpressionForm,
    LiteralKind, ModelForm, ProtocolForm, RawSemanticNodeV2, RelationForm, ScalarTypeForm,
    StateClauseKind, StateForm, TemporalForm, ValueForm, WorkMeter,
};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedArtifactRef, CheckedNodeId, CheckedProtocolDecisionRead, CheckedProtocolRefusalDetail,
};
use crate::checked_package::shared::{
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code, JsonPointer,
};
use ix_trace_rs::string_edge;
#[cfg(test)]
use serde_json::Value;

fn node_work(meter: &mut WorkMeter, position: usize) -> Result<(), ValidationFailure> {
    let path = super::stage_storage::Path::root()
        .key("semantic_graph")
        .key("nodes")
        .index(position);
    let site = super::natural::site(meter, path);
    super::natural::work_at(meter, site, 1)
}

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
    /// The borrowed selected package and original Port declaration of an admitted role.
    ///
    /// A legacy role without a member, an unknown node, or a non-role returns `None`.
    pub fn protocol_role_model_member(&self, role: &CheckedNodeId) -> Option<(&str, &str)> {
        let position = self.position_of(role)?;
        if self.kinds[position] != CheckedNodeKind::Protocol(ProtocolForm::Role) {
            return None;
        }
        let CheckedTerm::Application { arguments, .. } =
            &self.wire.semantic_graph.nodes[position].body
        else {
            return None;
        };
        role_member_texts(binding_value(arguments, "model_member")?)
    }

    /// Derives every local clause's activation requirements from the admitted graph.
    ///
    /// The rows are ordered by clause and then required temporal node identity.
    /// This read-only view does not charge reader work or change package identity.
    pub fn protocol_activation_relation(&self) -> Vec<CheckedProtocolActivationRow> {
        let mut rows = Vec::new();
        for (position, node) in self.wire.semantic_graph.nodes.iter().enumerate() {
            if self.kinds[position] != CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause) {
                continue;
            }
            let CheckedTerm::Application { arguments, .. } =
                &self.wire.semantic_graph.nodes[position].body
            else {
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
            left.clause
                .cmp(&right.clause)
                .then_with(|| left.required_temporal.cmp(&right.required_temporal))
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
    EnumValue,
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

pub(super) fn text_value(term: &CheckedTerm) -> Option<&str> {
    match term {
        CheckedTerm::Literal {
            value_kind: LiteralKind::Text,
            value,
            ..
        } => value.as_str(),
        _ => None,
    }
}

/// A channel's `delivery` premise as QSpec FR-465 spells it: one of the four
/// policy words, or a per-send delivery count interval `[lo,hi]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChannelDelivery {
    /// `unknown`: duplication and loss are both possible.
    Unknown,
    /// `at-most-once`: loss is possible, duplication is not.
    AtMostOnce,
    /// `at-least-once`: duplication is possible, loss is not.
    AtLeastOnce,
    /// `exactly-once-premise`: an authored premise of one delivery per send.
    ExactlyOncePremise,
    /// `[lo,hi]`: one send produces `d` deliveries with `lo <= d <= hi`.
    Between { lo: u64, hi: u64 },
}

impl ChannelDelivery {
    /// Parses a delivery text. An interval is canonical: no space, decimal
    /// naturals with no sign and no leading zero, at most `u64::MAX`, and
    /// `lo <= hi`. Any other text is `None`.
    #[string_edge]
    pub(super) fn parse(text: &str) -> Option<Self> {
        match text {
            "unknown" => Some(Self::Unknown),
            "at-most-once" => Some(Self::AtMostOnce),
            "at-least-once" => Some(Self::AtLeastOnce),
            "exactly-once-premise" => Some(Self::ExactlyOncePremise),
            _ => {
                let (lo, hi) = text.strip_prefix('[')?.strip_suffix(']')?.split_once(',')?;
                let (lo, hi) = (canonical_natural(lo)?, canonical_natural(hi)?);
                (lo <= hi).then_some(Self::Between { lo, hi })
            }
        }
    }
}

/// A non-empty run of ASCII digits with no leading zero (`0` itself is
/// admitted) that fits `u64`.
fn canonical_natural(numeral: &str) -> Option<u64> {
    let digits_only = !numeral.is_empty() && numeral.bytes().all(|byte| byte.is_ascii_digit());
    let leading_zero = numeral.len() > 1 && numeral.starts_with('0');
    if !digits_only || leading_zero {
        return None;
    }
    numeral.parse().ok()
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
                left < right
            }
            _ => false,
        })
    })
}

/// The canonical bytes of `term` within `bytes`, in a block of exactly their
/// length charged before it is requested; `None` when the term has no
/// canonical encoding within the bound. The length is counted by encoding
/// into nothing first.
fn canonical_term_bytes(
    term: &CheckedTerm,
    bytes: u64,
    meter: &mut WorkMeter,
) -> Result<Option<Vec<u8>>, ValidationFailure> {
    struct Count(u64);
    impl quire_canonical::Sink for Count {
        fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), quire_canonical::Error> {
            self.0 = self.0.saturating_add(bytes.len() as u64);
            Ok(())
        }
    }
    let limits = quire_canonical::Limits::new(bytes);
    let mut count = Count(0);
    if quire_canonical::encode(&mut count, term, limits).is_err() {
        return Ok(None);
    }
    let Ok(length) = usize::try_from(count.0) else {
        return Ok(None);
    };
    meter.reserve(length)?;
    let mut encoded = Vec::with_capacity(length);
    Ok(quire_canonical::encode(&mut encoded, term, limits)
        .is_ok()
        .then_some(encoded))
}

fn related_records_are_ordered(
    term: &CheckedTerm,
    bytes: u64,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some(records) = members(term) else {
        return Ok(false);
    };
    let mut previous: Option<(&CheckedNodeId, Vec<u8>, Vec<u8>)> = None;
    for record in records {
        let Some([relationship, from, to]) = named_fields(record, &["relationship", "from", "to"])
            .and_then(|fields| <[_; 3]>::try_from(fields).ok())
        else {
            return Ok(false);
        };
        let Some(relationship) = reference_target(relationship) else {
            return Ok(false);
        };
        let Some(from) = canonical_term_bytes(from, bytes, meter)? else {
            return Ok(false);
        };
        let Some(to) = canonical_term_bytes(to, bytes, meter)? else {
            return Ok(false);
        };
        if let Some((held, held_from, held_to)) = &previous {
            let order = meter
                .compare_ids(held, relationship)?
                .then_with(|| held_from.cmp(&from))
                .then_with(|| held_to.cmp(&to));
            if order.is_ge() {
                return Ok(false);
            }
        }
        previous = Some((relationship, from, to));
    }
    Ok(true)
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
            delivery.and_then(ChannelDelivery::parse).is_some()
                && argument(arguments, 3).is_some_and(|ordering| {
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

#[ix_trace_rs::string_edge]
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
    if result_type != semantic_type || operation.identity() != Some("quire.op.protocol.control") {
        return Err(D::OperatorIneligible);
    }
    let laws = &operation.record().laws;
    let law = laws.first().ok_or(D::LawMissing)?;
    if laws.len() != 1 {
        return Err(D::MalformedLayout);
    }
    if law.role.as_ref() != "protocol_profile" {
        return Err(D::LawMismatch);
    }
    if !selected_profile.is_some_and(|selected| selected == &law.definition) {
        return Err(D::LawUnselected);
    }
    if operation.record().mode.is_some() || !operation.record().leaves.is_empty() {
        return Err(D::MalformedLayout);
    }
    let member = operation.member_node().ok_or(D::MemberMismatch)?;
    let operator = member
        .get("operator")
        .and_then(super::TermNode::as_str)
        .ok_or(D::MemberMismatch)?;
    if !member.exact_members(&["kind", "operator"])
        || member.get("kind").and_then(super::TermNode::as_str) != Some("profile_operator")
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
    let optional_role_member = form == ProtocolForm::Role
        && arguments.len() == expected.len() + 1
        && matches!(arguments.last(), Some(CheckedTerm::Binding { name, value })
            if name.as_ref() == "model_member" && shape_fits(value, Shape::Group));
    if (arguments.len() != expected.len() && !optional_role_member)
        || !arguments
            .iter()
            .take(expected.len())
            .zip(expected)
            .all(|(argument, binding)| {
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
    meter: &mut WorkMeter,
) -> Result<Option<ProtocolBodyDefect>, ValidationFailure> {
    let related = match (form, operator) {
        (ProtocolForm::Control, "send" | "effect") => Some(2),
        (ProtocolForm::Control, "receive" | "event") => Some(3),
        (ProtocolForm::Control, "attempt") => Some(5),
        _ => None,
    };
    let Some(ordinal) = related else {
        return Ok(None);
    };
    let ordered = match argument(arguments, ordinal) {
        Some(records) => related_records_are_ordered(records, bytes, meter)?,
        None => false,
    };
    Ok((!ordered).then_some(ProtocolBodyDefect::NoncanonicalSet))
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
    if operation.record().laws.len() > 1
        || operation.record().mode.is_some()
        || !operation.record().leaves.is_empty()
    {
        return Some(ProtocolLayoutWireDefect::Body);
    }
    let operator = operation.member_operator()?;
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
            .is_some_and(|value| value < Numeral::exact(0))
        {
            return Some(ProtocolLayoutWireDefect::NegativeQueueCapacity);
        }
    }
    if (form, operator) == (ProtocolForm::Control, "repeat")
        && argument(arguments, 2)
            .and_then(members)
            .and_then(|items| items.first())
            .and_then(integer_value)
            .is_some_and(|value| value < Numeral::exact(0))
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
                .is_some_and(|value| value < Numeral::exact(0))
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
                .is_some_and(|value| value < Numeral::exact(0))
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
    (!allowed.contains(&spelling)).then_some(ProtocolLayoutWireDefect::EnumValue)
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
                argument(arguments, 4)
                    .and_then(text_value)
                    .filter(|text| ChannelDelivery::parse(text).is_none())
                    .map(|_| ProtocolLayoutWireDefect::EnumValue)
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
                    .map(|_| ProtocolLayoutWireDefect::EnumValue)
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
    // This view is not on the read path: its own unmetered meter.
    let mut meter = WorkMeter::new(u64::MAX);
    match validate_related_order(form, operator, arguments, bytes, &mut meter) {
        Ok(None) => Ok(()),
        Ok(Some(defect)) => Err(defect),
        Err(_) => Err(ProtocolBodyDefect::NoncanonicalSet),
    }
}

fn body_defect(
    position: usize,
    node: &RawSemanticNodeV2,
    defect: ProtocolBodyDefect,
) -> ValidationFailure {
    let node_at = node_pointer(position);
    let body = node_pointer(position).key("body");
    let (code, cause, path, document_pointer) = match defect {
        ProtocolBodyDefect::OperatorIneligible => (
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            node_at.clone(),
            None,
        ),
        ProtocolBodyDefect::MemberMismatch => (
            Code::InvalidPackage,
            Some(Cause::OperationMemberMismatch),
            node_at.clone(),
            Some(body.key("operation").key("member")),
        ),
        ProtocolBodyDefect::LawMissing => (
            Code::InvalidPackage,
            Some(Cause::OperationLawMissing),
            node_at.clone(),
            Some(body.key("operation").key("laws")),
        ),
        ProtocolBodyDefect::LawMismatch => (
            Code::InvalidPackage,
            Some(Cause::OperationLawMismatch),
            node_at.clone(),
            Some(body.key("operation").key("laws").index(0)),
        ),
        ProtocolBodyDefect::LawUnselected => (
            Code::InvalidPackage,
            Some(Cause::OperationLawUnselected),
            node_at,
            Some(body.key("operation").key("laws").index(0).key("definition")),
        ),
        ProtocolBodyDefect::MalformedLayout => {
            (Code::MalformedWire, None, body.key("arguments"), None)
        }
        ProtocolBodyDefect::NoncanonicalSet => (
            Code::InvalidSemanticGraph,
            None,
            node_pointer(position),
            None,
        ),
    };
    let mut failure = ValidationFailure::refused_at(code, path, cause, node.node_id.clone());
    if let ValidationFailure::Refused(ref mut refusal) = failure {
        refusal.document_pointer = document_pointer;
    }
    failure
}

/// Checks name collisions involving a protocol declaration at reader step 3.
/// The earlier generic sweep has already refused collisions entirely among
/// non-protocol declarations.
fn validate_protocol_declaration_names(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    // Declared names in order, each with whether a protocol holds it.
    meter.at_node(0, "protocol");
    let mut holders: Vec<(&[Box<str>], bool)> = meter.vec(
        nodes
            .iter()
            .filter(|node| node.declaration.is_some())
            .count(),
    )?;
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
        let held = holders.binary_search_by(|(held, _)| (*held).cmp(name));
        if let Ok(previous) = held {
            let previous_protocol = holders[previous].1;
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
        } else if let Err(at) = held {
            holders.insert(at, (name, protocol));
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

pub(super) fn binding_value<'a>(
    arguments: &'a [CheckedTerm],
    name: &str,
) -> Option<&'a CheckedTerm> {
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
fn visit_control_children<'a>(
    operator: &str,
    arguments: &'a [CheckedTerm],
    children: &mut impl FnMut(&'a CheckedNodeId),
) {
    match operator {
        "sequence" => {
            if let Some(value) = binding_value(arguments, "children") {
                listed_targets(value).for_each(&mut *children);
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
                        children(target);
                    }
                }
            }
        }
        "repeat" => {
            if let Some(target) = binding_value(arguments, "body").and_then(reference_target) {
                children(target);
            }
            if let Some(value) = binding_value(arguments, "exhausted") {
                listed_targets(value).for_each(&mut *children);
            }
        }
        "await" => {
            for field in ["event", "then", "timeout"] {
                if let Some(target) = binding_value(arguments, field).and_then(reference_target) {
                    children(target);
                }
            }
        }
        _ => {}
    }
}

/// The child control nodes of one control node as graph positions, in
/// operand order, in a block sized by counting them first and charged before
/// it is requested; a child outside the index is left out.
fn control_positions(
    operator: &str,
    arguments: &[CheckedTerm],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<Vec<usize>, ValidationFailure> {
    let mut count = 0_usize;
    visit_control_children(operator, arguments, &mut |child| {
        count += usize::from(index.get(child).is_some());
    });
    let mut children = meter.vec(count)?;
    visit_control_children(operator, arguments, &mut |child| {
        if let Some(&position) = index.get(child) {
            children.push(position);
        }
    });
    Ok(children)
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
    catalog: &'a super::operation_catalog::OperationCatalog,
    position: usize,
    nodes: &'a [RawSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a GraphIndex<'a>,
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
        path: impl FnOnce() -> JsonPointer,
        expected: impl Fn(usize) -> bool,
    ) -> Result<usize, ValidationFailure> {
        // The pointer is built only to name a refusal.
        let Some(target) = reference_target(term) else {
            return Err(self.refuse(Code::MalformedWire, None, path()));
        };
        let Some(&position) = self.index.get(target) else {
            return Err(self.refuse(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                path().key("target"),
            ));
        };
        if !expected(position) {
            return Err(self.refuse(
                Code::InvalidModelBinding,
                Some(Cause::MalformedDeclaration),
                path().key("target"),
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
            || self.argument_path(ordinal),
            expected,
        )
    }

    fn is(&self, position: usize, kind: CheckedNodeKind) -> bool {
        self.kinds.get(position) == Some(&kind)
    }

    fn boolean(
        &self,
        term: &CheckedTerm,
        path: impl FnOnce() -> JsonPointer,
    ) -> Result<(), ValidationFailure> {
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
            Err(self.refuse(Code::IllTyped, Some(Cause::NonBooleanRoot), path()))
        }
    }

    fn boolean_at_node(
        &self,
        term: &CheckedTerm,
        document_pointer: JsonPointer,
    ) -> Result<(), ValidationFailure> {
        self.boolean(term, || document_pointer.clone())
            .map_err(|mut failure| {
                if let ValidationFailure::Refused(ref mut refusal) = failure {
                    refusal.set_primary_path(Some(node_pointer(self.position)));
                    refusal.document_pointer = Some(document_pointer);
                }
                failure
            })
    }

    fn scalar_literal(
        &self,
        term: &CheckedTerm,
        form: ScalarTypeForm,
        path: impl FnOnce() -> JsonPointer,
    ) -> Result<(), ValidationFailure> {
        let CheckedTerm::Literal { type_id, .. } = term else {
            return Err(self.refuse(Code::MalformedWire, None, path()));
        };
        if self
            .index
            .get(type_id)
            .is_some_and(|&position| self.is(position, CheckedNodeKind::ScalarType(form)))
        {
            Ok(())
        } else {
            Err(self.refuse(Code::IllTyped, Some(Cause::TypeMismatch), path()))
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

    fn equality_stable(
        &self,
        type_id: &CheckedNodeId,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        let mut pending = meter.vec(1)?;
        pending.push(type_id);
        let mut visited: Vec<usize> = Vec::new();
        while let Some(type_id) = pending.pop() {
            node_work(meter, self.position)?;
            let Some(&position) = self.index.get(type_id) else {
                return Ok(false);
            };
            if !meter.insert_sorted(&mut visited, position)? {
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
                        return Ok(false);
                    };
                    if !collect_type_children(term, &mut pending, meter)? {
                        return Ok(false);
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
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
            node_work(meter, self.position)?;
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
            node_work(meter, self.position)?;
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
        if (positive && value == Numeral::exact(0)) || value < Numeral::exact(0) {
            Err(self.refuse(Code::IllTyped, Some(Cause::TypeMismatch), path))
        } else {
            Ok(())
        }
    }
}

fn integer_value(term: &CheckedTerm) -> Option<Numeral<'_>> {
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = term
    else {
        return None;
    };
    value.as_str().and_then(Numeral::parse)
}

/// An integer literal read in place: its sign and the digits as the wire
/// spells them, compared and printed without a heap or a bounded width.
#[derive(Clone, Copy, Debug)]
struct Numeral<'a> {
    negative: bool,
    digits: Digits<'a>,
    /// The literal's own spelling, when it is one.
    text: Option<&'a str>,
}

#[derive(Clone, Copy, Debug)]
enum Digits<'a> {
    Text(&'a str),
    Inline([u8; 20], usize),
}

impl<'a> Numeral<'a> {
    /// A small constant. `u64::MAX` has twenty digits.
    fn exact(value: impl TryInto<u64>) -> Self {
        let mut rest = value.try_into().unwrap_or(u64::MAX);
        let mut buffer = [0_u8; 20];
        let mut at = buffer.len();
        loop {
            at -= 1;
            buffer[at] = b'0' + (rest % 10) as u8;
            rest /= 10;
            if rest == 0 {
                break;
            }
        }
        Self {
            negative: false,
            digits: Digits::Inline(buffer, at),
            text: None,
        }
    }

    /// `[+-]?` then decimal digits, a digit separator `_` allowed after the
    /// first digit, as the integer text of the wire reads.
    fn parse(text: &'a str) -> Option<Self> {
        let (negative, digits) = match text.as_bytes().first()? {
            b'-' => (true, &text[1..]),
            b'+' => (false, &text[1..]),
            _ => (false, text),
        };
        let valid = !digits.is_empty()
            && !digits.starts_with('_')
            && digits
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_digit());
        valid.then_some(Self {
            negative,
            digits: Digits::Text(digits),
            text: Some(text),
        })
    }

    /// Whether `self > other`. Two canonical spellings (an optional `-`, no
    /// leading zero, no `-0`) are compared as quire-exact integers built on
    /// the read's budget; any other spelling is ordered by its digits.
    fn exceeds(&self, other: &Self, meter: &mut WorkMeter) -> Result<bool, ValidationFailure> {
        fn canonical(text: &str) -> bool {
            let digits = text.strip_prefix('-').unwrap_or(text);
            match digits.as_bytes() {
                [b'0'] => digits.len() == text.len(),
                [b'1'..=b'9', rest @ ..] => rest.iter().all(u8::is_ascii_digit),
                _ => false,
            }
        }
        if let (Some(left), Some(right)) = (self.text, other.text) {
            if canonical(left) && canonical(right) {
                if let (Some(left), Some(right)) =
                    (meter.exact_integer(left)?, meter.exact_integer(right)?)
                {
                    return Ok(left > right);
                }
            }
        }
        Ok(self > other)
    }

    fn significant(&self) -> impl Iterator<Item = u8> + '_ {
        let raw = match &self.digits {
            Digits::Text(text) => text.as_bytes(),
            Digits::Inline(buffer, at) => &buffer[*at..],
        };
        raw.iter()
            .copied()
            .filter(|byte| *byte != b'_')
            .skip_while(|byte| *byte == b'0')
    }

    fn is_negative(&self) -> bool {
        self.negative && self.significant().next().is_some()
    }

    fn magnitude_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.significant()
            .count()
            .cmp(&other.significant().count())
            .then_with(|| self.significant().cmp(other.significant()))
    }
}

impl PartialEq for Numeral<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Numeral<'_> {}

impl PartialOrd for Numeral<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Numeral<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.is_negative(), other.is_negative()) {
            (false, true) => std::cmp::Ordering::Greater,
            (true, false) => std::cmp::Ordering::Less,
            (false, false) => self.magnitude_cmp(other),
            (true, true) => other.magnitude_cmp(self),
        }
    }
}

impl std::fmt::Display for Numeral<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;
        if self.is_negative() {
            formatter.write_char('-')?;
        }
        let mut any = false;
        for digit in self.significant() {
            formatter.write_char(char::from(digit))?;
            any = true;
        }
        if !any {
            formatter.write_char('0')?;
        }
        Ok(())
    }
}

fn collect_type_children<'a>(
    term: &'a CheckedTerm,
    pending: &mut Vec<&'a CheckedNodeId>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    match term {
        CheckedTerm::Reference { target } => {
            meter.push(pending, target)?;
            Ok(true)
        }
        CheckedTerm::Aggregate { members } => {
            for member in members {
                if !collect_type_children(member, pending, meter)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        CheckedTerm::Binding { value, .. } => collect_type_children(value, pending, meter),
        CheckedTerm::Literal { .. } => Ok(true),
        _ => Ok(false),
    }
}

pub(super) fn group_value<'a>(term: &'a CheckedTerm, name: &str) -> Option<&'a CheckedTerm> {
    binding_value(members(term)?, name)
}

fn check_member_targets(
    check: &MemberCheck<'_>,
    term: &CheckedTerm,
    path: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    match term {
        CheckedTerm::Reference { target } => {
            if !check.index.contains_key(target) {
                return Err(check.refuse(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    path().key("target"),
                ));
            }
        }
        CheckedTerm::Binding { value, .. } => {
            check_member_targets(check, value, &|| path().key("value"))?
        }
        CheckedTerm::Aggregate { members } => {
            for (position, member) in members.iter().enumerate() {
                check_member_targets(check, member, &|| path().key("members").index(position))?;
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
        let position = check.check_target(relationship, || path.clone(), |_| true)?;
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
            let Some(expected) = owner
                .package
                .end_declaration_form(declared)
                .and_then(|form| owners.declaration_id(&owner.package.identity, declared, form))
            else {
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
                    let mut budget = Budget::new(meter, owner.package.root, owner.package.bytes);
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
                        expected: expected.clone(),
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
                || check.argument_path(ordinal).key("members").index(position),
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
            }) if control(position) => operation.member_text("operator"),
            _ => None,
        };
    match operator {
        "check" => check.boolean(check.at(arguments, 0), || check.argument_path(0))?,
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
            check.boolean(check.at(arguments, constraint_ordinal), || {
                check.argument_path(constraint_ordinal)
            })?;
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
            let listed_contracts = members(check.at(arguments, 3)).expect("validated contracts");
            let mut contracts = meter.vec(listed_contracts.len())?;
            for (ordinal, contract) in listed_contracts.iter().enumerate() {
                let path = check.argument_path(3).key("members").index(ordinal);
                let clause = check.check_target(
                    contract,
                    || path.clone(),
                    |position| check.is(position, CheckedNodeKind::State(StateForm::StateClause)),
                )?;
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
                        ) && *body.anchor == check.nodes[operation].node_id
                    });
                if !valid {
                    return Err(check.refuse(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        path.key("target"),
                    ));
                }
            }
            check.boolean(check.at(arguments, 6), || check.argument_path(6))?;
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
            check.boolean(check.at(arguments, 3), || check.argument_path(3))?;
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
                let template = check.check_target(
                    compensation,
                    || path.clone(),
                    |position| {
                        check.is(
                            position,
                            CheckedNodeKind::Protocol(ProtocolForm::Compensation),
                        )
                    },
                )?;
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
            check.boolean(check.at(arguments, 4), || check.argument_path(4))?;
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
            check.boolean(check.at(arguments, 2), || check.argument_path(2))?;
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
                    || path.clone().key("members").index(body_ordinal).key("value"),
                    control,
                )?;
            }
            if operator == "parallel" {
                let join = check.at(arguments, 2);
                if argument_text(join, 0) == Some("predicate") {
                    check.check_target(
                        group_value(join, "predicate").expect("validated predicate"),
                        || check.argument_path(2).key("members").index(1).key("value"),
                        |position| matches!(check.kinds[position], CheckedNodeKind::Function(_)),
                    )?;
                }
            }
            let mut previous: Option<&str> = None;
            let mut labels: Vec<&str> = meter.vec(records.len())?;
            for record in records {
                let label = argument_text(record, 0).expect("validated label");
                if !meter.insert_sorted(&mut labels, label)? {
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
                        || {
                            check
                                .argument_path(2)
                                .key("members")
                                .index(position)
                                .key("members")
                                .index(1)
                                .key("value")
                        },
                    )?;
                }
            }
            if operator == "parallel" {
                let join = check.at(arguments, 2);
                let joined = members(check.at(arguments, 3)).expect("validated joined");
                let mut prior: Option<&str> = None;
                for (position, item) in joined.iter().enumerate() {
                    let label = text_value(item).expect("validated joined label");
                    if labels.binary_search(&label).is_err() {
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
                            count < Numeral::exact(1) || count > Numeral::exact(joined.len())
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
                    || check.argument_path(6).key("members").index(0),
                    control,
                )?;
            }
            if members(check.at(arguments, 3)).is_some_and(|items| !items.is_empty()) {
                check.boolean(
                    group_value(check.at(arguments, 3), "invariant").expect("validated invariant"),
                    || check.argument_path(3).key("members").index(0).key("value"),
                )?;
            }
            check.boolean_at_node(check.at(arguments, 4), check.argument_path(4))?;
        }
        "await" => {
            let after = check.at(arguments, 0);
            let anchor = group_value(after, "target").expect("validated anchor");
            let kind = argument_text(after, 0).expect("validated anchor kind");
            let anchor_path = check.argument_path(0).key("members").index(1).key("value");
            if kind == "compensation" {
                check.check_target(
                    anchor,
                    || anchor_path,
                    |position| {
                        check.is(
                            position,
                            CheckedNodeKind::Protocol(ProtocolForm::Compensation),
                        )
                    },
                )?;
            } else {
                check.check_target(
                    anchor,
                    || anchor_path,
                    |position| {
                        matches!(
                            control_operator(position),
                            Some("send" | "receive" | "attempt" | "effect" | "event" | "commit")
                        )
                    },
                )?;
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
            if lower.exceeds(&upper, meter)? {
                return Err(check.refuse_with_detail(
                    Code::IllTyped,
                    Cause::TypeMismatch,
                    check.argument_path(2),
                    CheckedProtocolRefusalDetail::Interval {
                        member: "within".into(),
                        expected: "closed interval with lower <= upper".into(),
                        actual: meter.formatted(format_args!("[{lower}, {upper}]"))?,
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
                    if operation.member_text("operator") == Some("effect"))
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
            node_work(meter, check.position)?;
            let binder = group_value(record, "binder").expect("validated capture binder");
            check.check_target(
                binder,
                || {
                    check
                        .argument_path(ordinal)
                        .key("members")
                        .index(position)
                        .key("members")
                        .index(0)
                        .key("value")
                },
                parameter,
            )?;
        }
    }
    let trigger = check.at(arguments, 6);
    check.check_target(
        group_value(trigger, "binder").expect("validated trigger binder"),
        || check.argument_path(6).key("members").index(0).key("value"),
        parameter,
    )?;
    let attempts = check.at(arguments, 9);
    let record_type = check.check_target(
        group_value(attempts, "record_type").expect("validated record type"),
        || check.argument_path(9).key("members").index(1).key("value"),
        |position| {
            matches!(
                check.kinds[position],
                CheckedNodeKind::Model(ModelForm::RecordValueType | ModelForm::EventType)
            )
        },
    )?;
    let retry = check.at(arguments, 10);
    let mut retry_binders = [0_usize; 2];
    for (position, name) in ["earlier", "later"].into_iter().enumerate() {
        let binder = check.check_target(
            group_value(retry, name).expect("validated retry binder"),
            || {
                check
                    .argument_path(10)
                    .key("members")
                    .index(position)
                    .key("value")
            },
            parameter,
        )?;
        retry_binders[position] = binder;
    }
    if let Some(commit) = members(check.at(arguments, 11)).and_then(|items| items.first()) {
        check.check_target(
            commit,
            || check.argument_path(11).key("members").index(0),
            |position| {
                check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Control))
                    && check.terms[position].as_ref().is_some_and(|term| {
                        matches!(term, CheckedTerm::Application { operation, .. }
                            if operation.member_text("operator") == Some("commit"))
                    })
            },
        )?;
    }
    let recovery = check.at(arguments, 12);
    check.check_target(
        group_value(recovery, "binder").expect("validated recovery binder"),
        || check.argument_path(12).key("members").index(0).key("value"),
        parameter,
    )?;
    check.scalar_literal(check.at(arguments, 4), ScalarTypeForm::Text, || {
        check.argument_path(4)
    })?;
    let within = check.at(arguments, 8);
    for (index, name) in ["lower", "upper"].into_iter().enumerate() {
        check.scalar_literal(
            group_value(within, name).expect("validated interval bound"),
            ScalarTypeForm::Integer,
            || {
                check
                    .argument_path(8)
                    .key("members")
                    .index(index)
                    .key("value")
            },
        )?;
    }
    check.scalar_literal(
        group_value(attempts, "maximum").expect("validated maximum"),
        ScalarTypeForm::Integer,
        || check.argument_path(9).key("members").index(0).key("value"),
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
        || check.argument_path(6).key("members").index(1).key("value"),
    )?;
    check.boolean(
        group_value(retry, "relation").expect("validated retry relation"),
        || check.argument_path(10).key("members").index(2).key("value"),
    )?;
    check.boolean(
        group_value(recovery, "predicate").expect("validated recovery predicate"),
        || check.argument_path(12).key("members").index(1).key("value"),
    )?;
    let (Some(lower), Some(upper)) = (
        group_value(within, "lower").and_then(integer_value),
        group_value(within, "upper").and_then(integer_value),
    ) else {
        return Err(check.refuse(Code::MalformedWire, None, check.argument_path(8)));
    };
    if lower.exceeds(&upper, meter)? {
        return Err(check.refuse_with_detail(
            Code::IllTyped,
            Cause::TypeMismatch,
            check.argument_path(8),
            CheckedProtocolRefusalDetail::Interval {
                member: "within".into(),
                expected: "closed interval with lower <= upper".into(),
                actual: meter.formatted(format_args!("[{lower}, {upper}]"))?,
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
    if maximum < Numeral::exact(1) {
        return Err(check.refuse_with_detail(
            Code::IllTyped,
            Cause::TypeMismatch,
            check.argument_path(9).key("members").index(0).key("value"),
            CheckedProtocolRefusalDetail::Interval {
                member: "attempts".into(),
                expected: "count of at least one".into(),
                actual: meter.formatted(format_args!("{maximum}"))?,
            },
        ));
    }
    validate_compensation_commit_position(check, arguments, validated, meter)?;
    Ok(())
}

#[string_edge]
fn role_member_texts(term: &CheckedTerm) -> Option<(&str, &str)> {
    if !group_fields_fit(term, &[("package", Shape::Text), ("node", Shape::Text)]) {
        return None;
    }
    let package = group_value(term, "package").and_then(text_value)?;
    let node = group_value(term, "node").and_then(text_value)?;
    (!package.is_empty() && !node.is_empty()).then_some((package, node))
}

fn check_role_member(
    check: &MemberCheck<'_>,
    arguments: &[CheckedTerm],
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let Some(member) = argument(arguments, 2) else {
        return Ok(());
    };
    node_work(meter, check.position)?;
    let texts = role_member_texts(member);
    let invalid = || {
        let mut failure = check.refuse(
            Code::InvalidModelBinding,
            Some(Cause::MalformedDeclaration),
            node_pointer(check.position),
        );
        if let (ValidationFailure::Refused(ref mut refusal), Some((package, node))) =
            (&mut failure, texts)
        {
            refusal.protocol_detail = Some(Box::new(CheckedProtocolRefusalDetail::ModelMember {
                package: package.into(),
                node: node.into(),
            }));
        }
        failure
    };
    // The model/selection check has already run. A selected whole-model text
    // still cannot carry a member, before the member's contents are considered.
    let Some(model) = argument(arguments, 0).and_then(reference_target) else {
        return Err(invalid());
    };
    let Some((package, node)) = texts else {
        return Err(check.refuse(Code::MalformedWire, None, node_pointer(check.position)));
    };
    let Some(owner) = owners.owner_by_key(model) else {
        return Err(invalid());
    };
    if owner.package.identity.as_ref() != package || !owner.package.is_systems_port(node) {
        return Err(invalid());
    }
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
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    for (ordinal, term) in arguments.iter().enumerate() {
        check_member_targets(check, term, &|| {
            node_pointer(check.position)
                .key("body")
                .key("arguments")
                .index(ordinal)
        })?;
    }
    let role = |position| check.is(position, CheckedNodeKind::Protocol(ProtocolForm::Role));
    let parameter = |position| check.is(position, CheckedNodeKind::Value(ValueForm::Parameter));
    match form {
        ProtocolForm::Role => {
            let replication = check.at(arguments, 1);
            if let Some(identity) = text_value(check.at(arguments, 0)) {
                node_work(meter, check.position)?;
                if !owners.selects(identity) {
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
                        || check.argument_path(1).key("members").index(2).key("value"),
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
                        check.boolean(until, || node_pointer(check.position))?;
                    }
                }
            }
            check_role_member(check, arguments, owners, meter)?;
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
                    || check.argument_path(3).key("members").index(1).key("value"),
                    parameter,
                )?;
                check.same_type(position, &check.nodes[carries].node_id)?;
                let key = group_value(ordering, "key").expect("validated fifo");
                let key_type = check.term_type(key);
                let stable = match key_type {
                    Some(type_id) => check.equality_stable(type_id, meter)?,
                    None => false,
                };
                if !stable {
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
                check.boolean(check.at(arguments, 1), || node_pointer(check.position))?;
            }
            "terminal" => {
                if let Some(condition) =
                    members(check.at(arguments, 1)).and_then(|items| items.first())
                {
                    check.boolean(condition, || node_pointer(check.position))?;
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
    expected: &mut [Option<(PathHolder, bool)>],
    position: usize,
    holder: PathHolder,
) {
    if let Some((_, repeated)) = &mut expected[position] {
        *repeated = true;
    } else {
        expected[position] = Some((holder, false));
    }
}

#[string_edge]
fn validate_protocol_declaration_paths(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    meter.at_node(0, "protocol");
    let mut expected = meter.filled(nodes.len(), None)?;
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
                meter.push(
                    &mut pending,
                    (
                        position,
                        PathHolder::ClauseMember {
                            clause,
                            member: "run",
                            own: true,
                        },
                    ),
                )?;
            }
        }
    }

    // The visited set bounds malformed containment cycles. Membership and
    // repeated holders are checked in the later graph pass.
    let mut visited = meter.filled(nodes.len(), false)?;
    while let Some((position, holder)) = pending.pop() {
        record_declared_path(&mut expected, position, holder);
        if std::mem::replace(&mut visited[position], true) {
            continue;
        }
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
            continue;
        }
        let Some((operator, args)) = validated[position] else {
            continue;
        };
        for child in control_positions(operator, args, index, meter)? {
            if kinds.get(child) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                meter.push(
                    &mut pending,
                    (child, PathHolder::Control { holder: position }),
                )?;
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
        if let Some((holder, false)) = &expected[position] {
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
fn collect_protocol_membership(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(Vec<Option<usize>>, Vec<bool>), ValidationFailure> {
    meter.at_node(0, "protocol");
    let mut owners = meter.filled(nodes.len(), None)?;
    let mut repeated = meter.filled(nodes.len(), false)?;
    let mut pending = Vec::new();
    for &clause in index.values() {
        if kinds.get(clause) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        node_work(meter, clause)?;
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
                    node_work(meter, clause)?;
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
                    node_work(meter, clause)?;
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
                meter.push(&mut pending, (position, clause))?;
            }
        }
    }

    let mut visited = meter.filled(nodes.len(), false)?;
    while let Some((position, clause)) = pending.pop() {
        node_work(meter, position)?;
        record_member_owner(&mut owners, &mut repeated, position, clause);
        if std::mem::replace(&mut visited[position], true) {
            continue;
        }
        let Some((operator, arguments)) = validated[position] else {
            continue;
        };
        for child in control_positions(operator, arguments, index, meter)? {
            node_work(meter, position)?;
            if kinds.get(child) == Some(&CheckedNodeKind::Protocol(ProtocolForm::Control)) {
                meter.push(&mut pending, (child, clause))?;
            }
        }
    }

    Ok((owners, repeated))
}

/// Resolve a protocol fairness target at its temporal reader-order position.
/// Full protocol admission still reports its own shape and membership defects.
pub(super) fn validate_temporal_fairness(
    requirement: usize,
    fair: usize,
    target: &CheckedFairnessTarget,
    graph: &super::frame::StepGraph<'_, '_>,
    terms: &[Option<CheckedTerm>],
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let selected_profile = lock
        .profile_selections
        .iter()
        .find(|selection| selection.role == CheckedSelectionRole::ProtocolProfile)
        .map(|selection| &selection.definition);
    meter.at_node(0, "protocol");
    let mut validated = meter.filled(graph.nodes.len(), None)?;
    for &position in graph.index.values() {
        let Some(CheckedNodeKind::Protocol(form)) = graph.kinds.get(position).copied() else {
            continue;
        };
        node_work(meter, position)?;
        let Some(term) = terms.get(position).and_then(Option::as_ref) else {
            continue;
        };
        if let Ok((operator, arguments)) = validate_envelope(
            form,
            term,
            &graph.nodes[position].semantic_type,
            selected_profile,
        ) {
            if validate_layout(form, operator, arguments).is_ok() {
                validated[position] = Some((operator, arguments));
            }
        }
    }
    let (owners, _) =
        collect_protocol_membership(graph.nodes, graph.kinds, &validated, graph.index, meter)?;
    let context = ProtocolFairnessContext {
        terms,
        nodes: graph.nodes,
        kinds: graph.kinds,
        validated: &validated,
        index: graph.index,
        owners: &owners,
    };
    let mut matched = false;
    for &clause in graph.index.values() {
        if graph.kinds[clause] != CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause) {
            continue;
        }
        node_work(meter, clause)?;
        let Some((_, arguments)) = validated[clause] else {
            continue;
        };
        let Some(requirements) = binding_value(arguments, "requirements") else {
            continue;
        };
        for key in listed_targets(requirements) {
            node_work(meter, clause)?;
            if key == &graph.nodes[requirement].node_id {
                matched = true;
                validate_protocol_fairness_target(&context, target, fair, clause, meter)?;
            }
        }
    }
    if !matched {
        if let Some(clause) =
            external_compensation_owner_clause(&context, target).filter(|&clause| {
                graph.nodes[clause].node_id.package == graph.nodes[requirement].node_id.package
            })
        {
            validate_protocol_fairness_target(&context, target, fair, clause, meter)?;
        } else {
            return Err(fairness_defect(
                fair,
                graph.nodes,
                target_identity(target).1,
                false,
            ));
        }
    }
    Ok(())
}

#[string_edge]
fn validate_protocol_membership(
    terms: &[Option<CheckedTerm>],
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let (owners, mut repeated) =
        collect_protocol_membership(nodes, kinds, validated, index, meter)?;

    validate_protocol_fairness(
        &ProtocolFairnessContext {
            terms,
            nodes,
            kinds,
            validated,
            index,
            owners: &owners,
        },
        meter,
    )?;

    meter.at_node(0, "protocol");
    let mut queue_count = meter.filled(nodes.len(), 0u8)?;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::Queue)) {
            continue;
        }
        node_work(meter, position)?;
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
        node_work(meter, position)?;
        if owners[position].is_none()
            || repeated[position]
            || (*form == ProtocolForm::Channel && queue_count[position] != 1)
        {
            return Err(declaration_defect(position, &nodes[position]));
        }
    }
    Ok(())
}

struct ProtocolFairnessContext<'a> {
    terms: &'a [Option<CheckedTerm>],
    nodes: &'a [RawSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    validated: &'a [Option<(&'a str, &'a [CheckedTerm])>],
    index: &'a GraphIndex<'a>,
    owners: &'a [Option<usize>],
}

/// Join fairness through the same retained clause membership used by controls.
fn validate_protocol_fairness(
    context: &ProtocolFairnessContext<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let &ProtocolFairnessContext {
        terms,
        nodes,
        kinds,
        validated,
        index,
        ..
    } = context;
    meter.at_node(0, "protocol");
    let mut checked = meter.filled(nodes.len(), false)?;
    for &clause in index.values() {
        if kinds.get(clause) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        let Some((_, arguments)) = validated[clause] else {
            continue;
        };
        let Some(requirements) = binding_value(arguments, "requirements") else {
            continue;
        };
        for requirement in listed_targets(requirements) {
            node_work(meter, clause)?;
            let Some(&requirement) = index.get(requirement) else {
                continue;
            };
            let Some(CheckedTerm::Application { arguments, .. }) = terms[requirement].as_ref()
            else {
                continue;
            };
            let Some(fairness) = arguments.get(4) else {
                continue;
            };
            for fair in listed_targets(fairness) {
                node_work(meter, requirement)?;
                let Some(&fair) = index.get(fair) else {
                    continue;
                };
                let Some(target) = fairness_target(terms[fair].as_ref(), meter)? else {
                    continue;
                };
                if matches!(
                    target,
                    CheckedFairnessTarget::Operation { .. }
                        | CheckedFairnessTarget::HybridTransition { .. }
                ) {
                    continue;
                }
                checked[fair] = true;
                validate_protocol_fairness_target(context, &target, fair, clause, meter)?;
            }
        }
    }
    let mut external_references = None;
    for &fair in index.values() {
        let Some(target) = fairness_target(terms[fair].as_ref(), meter)? else {
            continue;
        };
        if checked[fair]
            || matches!(
                target,
                CheckedFairnessTarget::Operation { .. }
                    | CheckedFairnessTarget::HybridTransition { .. }
            )
        {
            continue;
        }
        if matches!(target, CheckedFairnessTarget::CompensationTemplate { .. }) {
            if external_references.is_none() {
                external_references = Some(external_temporal_fairness_references(context, meter)?);
            }
            if external_references
                .as_ref()
                .is_some_and(|references| references[fair])
            {
                continue;
            }
        }
        node_work(meter, fair)?;
        return Err(fairness_defect(
            fair,
            nodes,
            target_identity(&target).1,
            false,
        ));
    }
    Ok(())
}

fn external_compensation_owner_clause(
    context: &ProtocolFairnessContext<'_>,
    target: &CheckedFairnessTarget,
) -> Option<usize> {
    let CheckedFairnessTarget::CompensationTemplate { node } = target else {
        return None;
    };
    let &position = context.index.get(node)?;
    if context.kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause) {
        return Some(position);
    }
    context.owners[position]
}

fn external_temporal_fairness_references(
    context: &ProtocolFairnessContext<'_>,
    meter: &mut WorkMeter,
) -> Result<Vec<bool>, ValidationFailure> {
    meter.at_node(0, "protocol");
    let mut references = meter.filled(context.nodes.len(), false)?;
    for &clause in context.index.values() {
        if context.kinds.get(clause)
            != Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause))
        {
            continue;
        }
        let Some(CheckedTerm::Application { arguments, .. }) =
            context.terms.get(clause).and_then(Option::as_ref)
        else {
            continue;
        };
        let Some(fairness) = arguments.get(4) else {
            continue;
        };
        node_work(meter, clause)?;
        for key in listed_targets(fairness) {
            node_work(meter, clause)?;
            if let Some(&fair) = context.index.get(key) {
                references[fair] = true;
            }
        }
    }
    Ok(references)
}

fn fairness_target(
    term: Option<&CheckedTerm>,
    meter: &mut WorkMeter,
) -> Result<Option<CheckedFairnessTarget>, ValidationFailure> {
    let Some(CheckedTerm::Application { operation, .. }) = term else {
        return Ok(None);
    };
    operation.fairness_target_syntax(meter)
}

fn target_identity(target: &CheckedFairnessTarget) -> (&CheckedNodeId, &'static str) {
    match target {
        CheckedFairnessTarget::Operation { declaration, .. }
        | CheckedFairnessTarget::HybridTransition { declaration, .. } => {
            (declaration, "declaration")
        }
        CheckedFairnessTarget::ParallelBranch { parallel, .. } => (parallel, "parallel"),
        CheckedFairnessTarget::ProtocolRoot { node }
        | CheckedFairnessTarget::AttemptNode { node }
        | CheckedFairnessTarget::CompensationTemplate { node } => (node, "node"),
    }
}

fn fairness_defect(
    fair: usize,
    nodes: &[RawSemanticNodeV2],
    field: &str,
    malformed: bool,
) -> ValidationFailure {
    ValidationFailure::refused_at(
        if malformed {
            Code::InvalidModelBinding
        } else {
            Code::MissingDeclaration
        },
        node_pointer(fair)
            .key("body")
            .key("operation")
            .key("member")
            .key("target")
            .key(field),
        Some(if malformed {
            Cause::MalformedDeclaration
        } else {
            Cause::MissingName
        }),
        nodes[fair].node_id.clone(),
    )
}

#[string_edge]
fn validate_protocol_fairness_target(
    context: &ProtocolFairnessContext<'_>,
    target: &CheckedFairnessTarget,
    fair: usize,
    clause: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let &ProtocolFairnessContext {
        nodes,
        kinds,
        validated,
        index,
        owners,
        ..
    } = context;
    let (identity, field) = target_identity(target);
    node_work(meter, fair)?;
    if identity.package != nodes[clause].node_id.package {
        return Err(fairness_defect(fair, nodes, field, false));
    }
    let Some(&position) = index.get(identity) else {
        return Err(fairness_defect(fair, nodes, field, false));
    };
    if position != clause && owners[position] != Some(clause) {
        return Err(fairness_defect(fair, nodes, field, false));
    }
    let kind_matches = match target {
        CheckedFairnessTarget::ProtocolRoot { .. } => position == clause,
        CheckedFairnessTarget::AttemptNode { .. } => {
            kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Control)
                && validated[position].is_some_and(|(op, _)| op == "attempt")
        }
        CheckedFairnessTarget::ParallelBranch { .. } => {
            kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Control)
                && validated[position].is_some_and(|(op, _)| op == "parallel")
        }
        CheckedFairnessTarget::CompensationTemplate { .. } => {
            kinds[position] == CheckedNodeKind::Protocol(ProtocolForm::Compensation)
        }
        CheckedFairnessTarget::Operation { .. }
        | CheckedFairnessTarget::HybridTransition { .. } => return Ok(()),
    };
    if !kind_matches {
        return Err(fairness_defect(fair, nodes, field, true));
    }
    if let CheckedFairnessTarget::ParallelBranch { branch, .. } = target {
        let mut found = false;
        if let Some((_, arguments)) = validated[position] {
            if let Some(branches) = binding_value(arguments, "branches").and_then(members) {
                for record in branches {
                    node_work(meter, fair)?;
                    found |=
                        group_value(record, "label").and_then(text_value) == Some(branch.as_ref());
                }
            }
        }
        if !found {
            return Err(fairness_defect(fair, nodes, "branch", false));
        }
    }
    Ok(())
}

/// Generation-stamped scratch for the commit search: one block per check,
/// reused by every search of it, so a search costs only what it visits.
struct CommitScratch {
    generation: u32,
    seen: Vec<u32>,
    /// The generation a verdict was recorded in, and the verdict.
    taken: Vec<(u32, bool)>,
}

impl CommitScratch {
    fn new(nodes: usize, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        Ok(Self {
            generation: 0,
            seen: meter.filled(nodes, 0)?,
            taken: meter.filled(nodes, (0, false))?,
        })
    }

    fn is_taken(&self, position: usize) -> bool {
        self.taken[position] == (self.generation, true)
    }
}

#[string_edge]
fn commit_is_taken(
    root: usize,
    commit: usize,
    validated: &[Option<(&str, &[CheckedTerm])>],
    index: &GraphIndex<'_>,
    scratch: &mut CommitScratch,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    scratch.generation += 1;
    let generation = scratch.generation;
    let mut pending = meter.vec(1)?;
    pending.push((root, false));
    while let Some((position, expanded)) = pending.pop() {
        node_work(meter, position)?;
        if !expanded && std::mem::replace(&mut scratch.seen[position], generation) == generation {
            continue;
        }
        let Some((operator, arguments)) = validated[position] else {
            continue;
        };
        if !expanded && !matches!(operator, "repeat" | "commit") {
            meter.push(&mut pending, (position, true))?;
            for child in control_positions(operator, arguments, index, meter)? {
                meter.push(&mut pending, (child, false))?;
            }
            continue;
        }
        let children = control_positions(operator, arguments, index, meter)?;
        let holds = match operator {
            "commit" => position == commit,
            "repeat" => false,
            "sequence" => children.iter().any(|&child| scratch.is_taken(child)),
            "parallel"
                if binding_value(arguments, "join")
                    .is_some_and(|join| argument_text(join, 0) == Some("all")) =>
            {
                children.iter().any(|&child| scratch.is_taken(child))
            }
            "parallel" | "choice" => {
                !children.is_empty() && children.iter().all(|&child| scratch.is_taken(child))
            }
            "await" => ["then", "timeout"].into_iter().all(|field| {
                binding_value(arguments, field)
                    .and_then(reference_target)
                    .and_then(|target| index.get(target))
                    .is_some_and(|&child| scratch.is_taken(child))
            }),
            _ => false,
        };
        scratch.taken[position] = (generation, holds);
    }
    Ok(scratch.is_taken(root))
}

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
    meter.at_node(0, "protocol");
    let mut parents: Vec<Option<usize>> = meter.filled(check.nodes.len(), None)?;
    for &position in check.index.values() {
        if check.kinds[position] != CheckedNodeKind::Protocol(ProtocolForm::Control) {
            continue;
        }
        node_work(meter, position)?;
        let Some((operator, arguments)) = validated[position] else {
            continue;
        };
        for child in control_positions(operator, arguments, check.index, meter)? {
            node_work(meter, position)?;
            parents[child] = Some(position);
        }
    }
    let mut child = effect;
    let mut seen = meter.filled(check.nodes.len(), false)?;
    let mut scratch = CommitScratch::new(check.nodes.len(), meter)?;
    while let Some(parent) = parents[child] {
        if std::mem::replace(&mut seen[parent], true) {
            break;
        }
        node_work(meter, parent)?;
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
            if commit_is_taken(earlier, commit, validated, check.index, &mut scratch, meter)? {
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
            || check.argument_path(1).key("members").index(1).key("value"),
            parameter,
        )?)
    } else {
        None
    };
    let captures = members(check.at(arguments, 2)).expect("validated captures");
    let mut capture_positions = meter.vec(captures.len())?;
    for (entry, record) in captures.iter().enumerate() {
        node_work(meter, check.position)?;
        let binder = group_value(record, "binder").expect("validated capture binder");
        capture_positions.push(check.check_target(
            binder,
            || {
                check
                    .argument_path(2)
                    .key("members")
                    .index(entry)
                    .key("members")
                    .index(0)
                    .key("value")
            },
            parameter,
        )?);
    }
    // Binders introduced: the subject, the trigger when present, each capture.
    let mut readable_names: Vec<&str> = meter.vec(2 + captures.len())?;
    let mut introduce = |position: usize, expected: usize, meter: &mut WorkMeter| {
        let actual = check.terms[position]
            .as_ref()
            .and_then(|body| group_value(body, "level"))
            .and_then(integer_value);
        if actual != Some(Numeral::exact(expected)) {
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
        if !meter.insert_sorted(&mut readable_names, name)? {
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
            node_work(meter, check.position)?;
            let path = || check.argument_path(ordinal).key("members").index(entry);
            let position = check.check_target(target, path, |_| true)?;
            if !check.is(position, expected) {
                let cause = if ordinal == 4 {
                    Cause::WrongExport
                } else {
                    Cause::MalformedDeclaration
                };
                return Err(ValidationFailure::refused_at(
                    Code::InvalidModelBinding,
                    path().key("target"),
                    Some(cause),
                    check.nodes[position].node_id.clone(),
                ));
            }
        }
    }
    if let Some(terminal) = members(check.at(arguments, 9)).and_then(|items| items.first()) {
        check.check_target(
            terminal,
            || check.argument_path(9).key("members").index(0),
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

    introduce(over, 0, meter)?;
    if let Some(trigger_position) = trigger_position {
        introduce(trigger_position, 1, meter)?;
    }

    for (entry, &binder_position) in capture_positions.iter().enumerate() {
        node_work(meter, check.position)?;
        introduce(binder_position, 1 + usize::from(each) + entry, meter)?;
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
    let scope = ProtocolScope::new(check, arguments, validated, meter)?;
    let activation = check.at(arguments, 1);
    if let Some(guard) = group_value(activation, "guard") {
        let path = || check.argument_path(1).key("members").index(2).key("value");
        check_member_targets(check, guard, &path)?;
        let triggers: &[usize] = match &scope.activation {
            Some(trigger) => std::slice::from_ref(trigger),
            None => &[],
        };
        let allowed = PositionSet::of(&[triggers], meter)?;
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
        node_work(meter, check.position)?;
        let initializer = group_value(record, "initializer").expect("validated initializer");
        let path = || {
            check
                .argument_path(2)
                .key("members")
                .index(entry)
                .key("members")
                .index(1)
                .key("value")
        };
        check_member_targets(check, initializer, &path)?;
        let allowed = PositionSet::of(
            &[
                &scope.c0[..1 + usize::from(scope.activation.is_some())],
                &scope.captures[..entry],
            ],
            meter,
        )?;
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
    /// Where each node was introduced as a binder, by node position.
    known: Vec<Vec<ProtocolBinderOrigin>>,
    fact_roles: Vec<Option<usize>>,
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
        let capture_records = members(check.at(arguments, 2)).expect("validated captures");
        let mut captures = meter.vec(capture_records.len())?;
        captures.extend(capture_records.iter().map(|record| {
            group_value(record, "binder")
                .and_then(reference_target)
                .and_then(|id| check.index.get(id).copied())
                .expect("validated capture binder")
        }));
        let mut c0 = meter.vec(2 + captures.len())?;
        c0.push(over);
        if let Some(trigger) = activation {
            c0.push(trigger);
        }
        c0.extend(captures.iter().copied());
        let mut known: Vec<Vec<ProtocolBinderOrigin>> =
            meter.filled(check.nodes.len(), Vec::new())?;
        for (index, &position) in c0.iter().enumerate() {
            node_work(meter, check.position)?;
            let capture = if index > usize::from(activation.is_some()) {
                Some((
                    "protocol-capture",
                    index - 1 - usize::from(activation.is_some()),
                ))
            } else {
                None
            };
            meter.push(
                &mut known[position],
                ProtocolBinderOrigin {
                    owner: check.position,
                    capture,
                },
            )?;
        }
        let mut scope = Self {
            check,
            arguments,
            validated,
            c0,
            activation,
            captures,
            known,
            fact_roles: meter.filled(check.nodes.len(), None)?,
        };
        for field in ["roles", "channels", "compensations"] {
            for target in listed_targets(binding_value(arguments, field).expect("validated list")) {
                node_work(meter, check.position)?;
                let position = *check.index.get(target).expect("validated member");
                let (_, args) = scope.node_arguments(position);
                match field {
                    "roles" => {
                        if let Some(binder) =
                            group_value(argument(args, 1).expect("role replication"), "binder")
                        {
                            let binder = scope.target(binder);
                            meter.push(
                                &mut scope.known[binder],
                                ProtocolBinderOrigin {
                                    owner: check.position,
                                    capture: None,
                                },
                            )?;
                        }
                    }
                    "channels" => {
                        if let Some(binder) =
                            group_value(argument(args, 3).expect("channel ordering"), "binder")
                        {
                            let binder = scope.target(binder);
                            meter.push(
                                &mut scope.known[binder],
                                ProtocolBinderOrigin {
                                    owner: check.position,
                                    capture: None,
                                },
                            )?;
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
                        for (ordinal, binder) in
                            compensation_binders(args, meter)?.into_iter().enumerate()
                        {
                            node_work(meter, position)?;
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
                            meter.push(
                                &mut scope.known[binder],
                                ProtocolBinderOrigin {
                                    owner: position,
                                    capture,
                                },
                            )?;
                        }
                    }
                    _ => unreachable!("fixed protocol lists"),
                }
            }
        }
        for &position in check.index.values() {
            node_work(meter, check.position)?;
            if !check.is(
                position,
                CheckedNodeKind::Protocol(ProtocolForm::Compensation),
            ) {
                continue;
            }
            let (_, args) = scope.node_arguments(position);
            for binder in compensation_binders(args, meter)? {
                node_work(meter, position)?;
                let binder = scope.target(binder);
                if !scope.known[binder]
                    .iter()
                    .any(|origin| origin.owner == position)
                {
                    meter.push(
                        &mut scope.known[binder],
                        ProtocolBinderOrigin {
                            owner: position,
                            capture: None,
                        },
                    )?;
                }
            }
        }
        let finish = scope.target(binding_value(arguments, "finish").expect("validated finish"));
        let (_, args) = scope.node_arguments(finish);
        let finish_binder = scope.target(argument(args, 0).expect("validated finish binder"));
        meter.push(
            &mut scope.known[finish_binder],
            ProtocolBinderOrigin {
                owner: check.position,
                capture: None,
            },
        )?;
        let root = scope.target(binding_value(arguments, "run").expect("validated root"));
        let mut pending = meter.vec(1)?;
        pending.push(root);
        let mut visited = meter.filled(check.nodes.len(), false)?;
        while let Some(position) = pending.pop() {
            node_work(meter, position)?;
            if std::mem::replace(&mut visited[position], true) {
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
                    scope.fact_roles[binder] = Some(role);
                }
                meter.push(
                    &mut scope.known[binder],
                    ProtocolBinderOrigin {
                        owner: check.position,
                        capture: None,
                    },
                )?;
            }
            for child in control_positions(operator, args, check.index, meter)? {
                node_work(meter, position)?;
                meter.push(&mut pending, child)?;
            }
        }
        Ok(scope)
    }

    #[string_edge]
    fn slot(
        &self,
        owner: usize,
        term: &CheckedTerm,
        allowed: &PositionSet,
        stage: &'static str,
        barred_slot: Option<&'static str>,
        capture: Option<usize>,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let mut pending = meter.vec(1)?;
        pending.push(SlotWalk::Term(term, None));
        // Nodes entered, with the local scope each was entered in.
        let mut visited: Vec<(usize, Option<usize>)> = Vec::new();
        let mut locals = SlotLocals::default();
        let mut arm_binders: Vec<usize> = Vec::new();
        let mut reads: Vec<usize> = Vec::new();
        let mut state_read = None;
        while let Some(item) = pending.pop() {
            node_work(meter, owner)?;
            match item {
                SlotWalk::Node(position, scope) => {
                    if !meter.insert_sorted(&mut visited, (position, scope))? {
                        continue;
                    }
                    if self
                        .check
                        .is(position, CheckedNodeKind::Value(ValueForm::Parameter))
                    {
                        let name = self.check.terms[position]
                            .as_ref()
                            .and_then(|body| group_value(body, "name"))
                            .and_then(text_value)
                            .expect("validated parameter name");
                        // A same-name lexical binding cannot launder an actual
                        // protocol-stage parameter into a local one.
                        if !self.known[position].is_empty()
                            || !locals.contains(
                                scope,
                                position,
                                name,
                                &arm_binders,
                                meter,
                                owner,
                            )?
                        {
                            meter.insert_sorted(&mut reads, position)?;
                        }
                        continue;
                    }
                    if matches!(self.check.kinds[position], CheckedNodeKind::Function(_)) {
                        continue;
                    }
                    if let Some(CheckedTerm::Application { operation, .. }) =
                        &self.check.terms[position]
                    {
                        if operation.identity().is_some_and(model_state_operation) {
                            state_read = Some(position);
                        }
                    }
                    if self
                        .check
                        .is(position, CheckedNodeKind::Expression(ExpressionForm::Case))
                    {
                        if let Some(CheckedTerm::Application {
                            operator: ApplicationOperator::Case,
                            operation,
                            arguments,
                            ..
                        }) = &self.check.terms[position]
                        {
                            if operation.identity() == Some("quire.op.control.case") {
                                if let Some((scrutinee, arms)) = arguments.split_first() {
                                    let mut scoped_arms = meter.vec(arms.len())?;
                                    let mut valid = true;
                                    'arms: for (ordinal, arm) in arms.iter().enumerate() {
                                        node_work(meter, owner)?;
                                        let CheckedTerm::Binding { value, .. } = arm else {
                                            valid = false;
                                            break;
                                        };
                                        let CheckedTerm::Aggregate { members } = value.as_ref()
                                        else {
                                            valid = false;
                                            break;
                                        };
                                        let [CheckedTerm::Aggregate { members: binders }, body] =
                                            members.as_slice()
                                        else {
                                            valid = false;
                                            break;
                                        };
                                        let mut binder_positions = meter.vec(binders.len())?;
                                        for binder in binders {
                                            node_work(meter, owner)?;
                                            let CheckedTerm::Reference { target } = binder else {
                                                valid = false;
                                                break 'arms;
                                            };
                                            let Some(&binder) = self.check.index.get(target) else {
                                                valid = false;
                                                break 'arms;
                                            };
                                            binder_positions.push(binder);
                                        }
                                        scoped_arms.push((ordinal, binder_positions, body));
                                    }
                                    if valid {
                                        for (ordinal, binders, body) in
                                            scoped_arms.into_iter().rev()
                                        {
                                            let mut arm_scope =
                                                Some(locals.arm(scope, position, ordinal, meter)?);
                                            for binder in binders {
                                                meter.insert_sorted(&mut arm_binders, binder)?;
                                                arm_scope = Some(
                                                    locals.bind_node(arm_scope, binder, meter)?,
                                                );
                                            }
                                            meter.push(
                                                &mut pending,
                                                SlotWalk::Term(body, arm_scope),
                                            )?;
                                        }
                                        meter
                                            .push(&mut pending, SlotWalk::Term(scrutinee, scope))?;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    if let Some(body) = &self.check.terms[position] {
                        meter.push(&mut pending, SlotWalk::Term(body, scope))?;
                    }
                }
                SlotWalk::Bind(CheckedTerm::Binding { name, value }, scope) => {
                    let scope = Some(locals.bind(scope, name.as_ref(), meter)?);
                    if matches!(value.as_ref(), CheckedTerm::Binding { .. }) {
                        meter.push(&mut pending, SlotWalk::Bind(value, scope))?;
                    } else {
                        meter.push(&mut pending, SlotWalk::Term(value, scope))?;
                    }
                }
                SlotWalk::Bind(term, scope) => {
                    meter.push(&mut pending, SlotWalk::Term(term, scope))?
                }
                SlotWalk::Term(term, scope) => match term {
                    CheckedTerm::Reference { target } => {
                        if let Some(&position) = self.check.index.get(target) {
                            meter.push(&mut pending, SlotWalk::Node(position, scope))?;
                        }
                    }
                    CheckedTerm::Application {
                        operator,
                        operation,
                        arguments,
                        ..
                    } => {
                        let entry = operation
                            .identity()
                            .and_then(|id| self.check.catalog.entry(id))
                            .filter(|entry| entry.operator == *operator);
                        if entry.is_some() && *operator == ApplicationOperator::Let {
                            if let [CheckedTerm::Binding { name, value }, body] =
                                arguments.as_slice()
                            {
                                let inner = Some(locals.bind(scope, name.as_ref(), meter)?);
                                meter.push(&mut pending, SlotWalk::Term(body, inner))?;
                                meter.push(&mut pending, SlotWalk::Term(value, scope))?;
                                continue;
                            }
                        }
                        meter.room(&mut pending, arguments.len())?;
                        for (ordinal, argument) in arguments.iter().enumerate().rev() {
                            if entry
                                .and_then(|entry| entry.operands.get(ordinal))
                                .is_some_and(|family| &**family == "binder")
                            {
                                pending.push(SlotWalk::Bind(argument, scope));
                            } else {
                                pending.push(SlotWalk::Term(argument, scope));
                            }
                        }
                    }
                    CheckedTerm::Aggregate { members } => {
                        meter.room(&mut pending, members.len())?;
                        pending.extend(members.iter().rev().map(|term| SlotWalk::Term(term, scope)))
                    }
                    CheckedTerm::Binding { value, .. } => {
                        meter.push(&mut pending, SlotWalk::Term(value, scope))?
                    }
                    CheckedTerm::Literal { .. }
                    | CheckedTerm::DependencyReference { .. }
                    | CheckedTerm::Frame { .. }
                    | CheckedTerm::AbstractionRelation { .. }
                    | CheckedTerm::ImplementationGuarantee { .. }
                    | CheckedTerm::ComponentComposition { .. }
                    | CheckedTerm::Refinement { .. }
                    | CheckedTerm::ExtensionForm { .. }
                    | CheckedTerm::NominalBody(_)
                    | CheckedTerm::OperationAnchorBody(_) => {}
                },
            }
        }
        if let Some(slot) = barred_slot {
            let barred = reads
                .binary_search(&self.c0[0])
                .ok()
                .map(|_| self.c0[0])
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
        for position in reads.iter().copied() {
            if allowed.contains(position) {
                continue;
            }
            let origins = self.known[position].as_slice();
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
        listed: &PositionSet,
        use_at: DecisionFactUse,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let CheckedTerm::Reference { target } = term else {
            return Ok(());
        };
        let Some(&root) = self.check.index.get(target) else {
            return Ok(());
        };
        let mut pending = meter.vec(1)?;
        pending.push(root);
        let mut visited: Vec<usize> = Vec::new();
        while let Some(position) = pending.pop() {
            node_work(meter, owner)?;
            if !meter.insert_sorted(&mut visited, position)?
                || (matches!(use_at, DecisionFactUse::Guard) && listed.contains(position))
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
                if operation.identity() == Some("quire.op.record.project")
                    && operation.member_text("kind") == Some("field")
                {
                    if let (Some(CheckedTerm::Reference { target }), Some(field)) =
                        (arguments.first(), operation.member_text("name"))
                    {
                        if let Some(&binder) = self.check.index.get(target) {
                            if self
                                .check
                                .is(binder, CheckedNodeKind::Value(ValueForm::Parameter))
                            {
                                if !self.captures.contains(&binder) {
                                    let reason = match self.fact_roles[binder] {
                                        Some(role) if role != owner => Some("other-role"),
                                        Some(_)
                                            if matches!(use_at, DecisionFactUse::Guard)
                                                && !listed.contains(position) =>
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
                                        if let ValidationFailure::Refused(ref mut refusal) = failure
                                        {
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
                                continue;
                            }
                        }
                    }
                }
            }
            for target in self.check.nodes[position].dependencies.iter().rev() {
                node_work(meter, owner)?;
                if let Some(&next) = self.check.index.get(target) {
                    meter.push(&mut pending, next)?;
                }
            }
        }
        Ok(())
    }

    /// A decision may project a record field, but cannot read the record binder itself.
    fn bare_decision_record_read(
        &self,
        term: &CheckedTerm,
        owner: usize,
        listed: &PositionSet,
        use_at: DecisionFactUse,
        slot: &'static str,
        meter: &mut WorkMeter,
    ) -> Result<(), ValidationFailure> {
        let CheckedTerm::Reference { target } = term else {
            return Ok(());
        };
        let Some(&root) = self.check.index.get(target) else {
            return Ok(());
        };
        let mut pending = meter.vec(1)?;
        pending.push(root);
        let mut visited: Vec<usize> = Vec::new();
        while let Some(position) = pending.pop() {
            node_work(meter, owner)?;
            if !meter.insert_sorted(&mut visited, position)?
                || (matches!(use_at, DecisionFactUse::Guard) && listed.contains(position))
                || matches!(self.check.kinds[position], CheckedNodeKind::Function(_))
            {
                continue;
            }
            let body = self.check.terms[position].as_ref();
            let projection = matches!(body,
                Some(CheckedTerm::Application { operation, .. })
                if operation.identity()
                    == Some("quire.op.record.project")
                    && operation.member_text("kind") == Some("field"));
            let bare = match body {
                Some(CheckedTerm::Reference { target }) => self
                    .check
                    .index
                    .get(target)
                    .copied()
                    .filter(|&binder| self.record_parameter(binder))
                    .map(|_| position),
                Some(CheckedTerm::Application { arguments, .. }) if !projection => arguments
                    .iter()
                    .filter_map(reference_target)
                    .filter_map(|target| self.check.index.get(target).copied())
                    .find(|&binder| self.record_parameter(binder))
                    .map(|_| position),
                _ => None,
            };
            if let Some(read) = bare {
                let mut failure = ValidationFailure::refused_at(
                    Code::IllTyped,
                    node_pointer(read),
                    Some(Cause::OperatorIneligible),
                    self.check.nodes[read].node_id.clone(),
                );
                if let ValidationFailure::Refused(ref mut refusal) = failure {
                    refusal.protocol_detail =
                        Some(Box::new(CheckedProtocolRefusalDetail::ModelStateRead {
                            slot: slot.into(),
                        }));
                }
                return Err(failure);
            }
            if projection {
                continue;
            }
            for target in self.check.nodes[position].dependencies.iter().rev() {
                node_work(meter, owner)?;
                if let Some(&next) = self.check.index.get(target) {
                    meter.push(&mut pending, next)?;
                }
            }
        }
        Ok(())
    }

    fn record_parameter(&self, position: usize) -> bool {
        self.check
            .is(position, CheckedNodeKind::Value(ValueForm::Parameter))
            && self
                .check
                .index
                .get(&self.check.nodes[position].semantic_type)
                .is_some_and(|&ty| {
                    matches!(
                        self.check.kinds[ty],
                        CheckedNodeKind::CompositeType(CompositeTypeForm::Record)
                            | CheckedNodeKind::Model(ModelForm::RecordValueType)
                    )
                })
    }

    fn introduce(
        &self,
        position: usize,
        input: &PositionSet,
        meter: &mut WorkMeter,
    ) -> Result<PositionSet, ValidationFailure> {
        node_work(meter, position)?;
        let actual = self.check.terms[position]
            .as_ref()
            .and_then(|body| group_value(body, "level"))
            .and_then(integer_value);
        if actual != Some(Numeral::exact(input.len())) {
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
        for other in input.iter() {
            node_work(meter, position)?;
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
        input.with(&[position], meter)
    }

    fn check_declaration_slots(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let base = PositionSet::of(&[&self.c0], meter)?;
        for target in listed_targets(binding_value(self.arguments, "roles").expect("roles")) {
            node_work(meter, self.check.position)?;
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
            node_work(meter, self.check.position)?;
            let channel = *self.check.index.get(target).expect("validated channel");
            let (_, args) = self.node_arguments(channel);
            let ordering = argument(args, 3).expect("ordering");
            if let Some(binder) = group_value(ordering, "binder") {
                self.introduce(self.target(binder), &base, meter)?;
                if let Some(key) = group_value(ordering, "key") {
                    let allowed = PositionSet::of(&[&[self.target(binder)]], meter)?;
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
        input: PositionSet,
        meter: &mut WorkMeter,
    ) -> Result<ControlScopeFrame, ValidationFailure> {
        node_work(meter, position)?;
        let (operator, args) = self.node_arguments(position);
        for _ in input.iter() {
            node_work(meter, position)?;
        }
        let output = if let Some(binder) = control_binder(operator, args) {
            self.introduce(self.target(binder), &input, meter)?
        } else {
            input.duplicate(meter)?
        };
        match operator {
            "choice" | "repeat" => {
                let visible = argument(args, 1).expect("visible");
                let entries = members(visible).expect("visible entries");
                let mut targets = meter.vec(entries.len())?;
                targets.extend(entries.iter().map(|entry| self.target(entry)));
                let listed = PositionSet::of(&[&targets], meter)?;
                let owner = self.target(argument(args, 0).expect("decision owner"));
                for entry in members(visible).expect("visible entries") {
                    self.slot(position, entry, &input, "run", Some("visible"), None, meter)?;
                    self.bare_decision_record_read(
                        entry,
                        owner,
                        &listed,
                        DecisionFactUse::VisibleEntry,
                        "visible",
                        meter,
                    )?;
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
                        self.bare_decision_record_read(
                            guard,
                            owner,
                            &listed,
                            DecisionFactUse::Guard,
                            "choice.guard",
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
                    self.bare_decision_record_read(
                        guard,
                        owner,
                        &listed,
                        DecisionFactUse::Guard,
                        "repeat.guard",
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
        let children = control_positions(operator, args, self.check.index, meter)?;
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
        let base = PositionSet::of(&[&self.c0], meter)?;
        let mut frames = meter.vec(1)?;
        frames.push(self.control_entry(root, base, meter)?);
        while let Some(frame) = frames.last_mut() {
            let (operator, args) = self.node_arguments(frame.position);
            if let Some(&child) = frame.children.get(frame.next) {
                let child_number = frame.next;
                frame.next += 1;
                let child_input = match operator {
                    "sequence" => frame.output.duplicate(meter)?,
                    "await" if child_number == 1 => frame.output.duplicate(meter)?,
                    _ => frame.input.duplicate(meter)?,
                };
                let entered = self.control_entry(child, child_input, meter)?;
                meter.push(&mut frames, entered)?;
                continue;
            }
            let all_join = operator == "parallel"
                && group_value(argument(args, 2).expect("parallel join"), "policy")
                    .and_then(text_value)
                    == Some("all");
            let out = if operator == "sequence" || all_join {
                frame.output.duplicate(meter)?
            } else if matches!(operator, "choice" | "repeat" | "parallel" | "await") {
                frame.input.duplicate(meter)?
            } else {
                frame.output.duplicate(meter)?
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
                        let binder = &binder;
                        let name = self.check.terms[*binder]
                            .as_ref()
                            .and_then(|body| group_value(body, "name"))
                            .and_then(text_value)
                            .expect("validated binder name");
                        let mut duplicate = parent.output.contains(*binder);
                        for previous in parent.output.difference(&parent.input) {
                            node_work(meter, *binder)?;
                            let previous_name = self.check.terms[previous]
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
                        parent.output.insert(*binder, meter)?;
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
        input: &PositionSet,
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
        let allowed = PositionSet::of(
            &[
                &self.c0[..1],
                if self.activation.is_none() {
                    &self.captures
                } else {
                    &[]
                },
            ],
            meter,
        )?;
        self.slot(terminal, condition, &allowed, "terminal", None, None, meter)
    }

    fn check_compensation_slots(&self, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        let base = PositionSet::of(&[&self.c0], meter)?;
        for target in
            listed_targets(binding_value(self.arguments, "compensations").expect("compensations"))
        {
            node_work(meter, self.check.position)?;
            let template = *self.check.index.get(target).expect("validated template");
            let (_, args) = self.node_arguments(template);
            let forward = self.target(argument(args, 1).expect("forward binder"));
            let mut level_set = self.introduce(forward, &base, meter)?;
            let registration_captures = members(argument(args, 5).expect("registration captures"))
                .expect("registration captures");
            let mut registration = meter.vec(registration_captures.len())?;
            for (ordinal, capture) in registration_captures.iter().enumerate() {
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
            let guard_allowed =
                PositionSet::of(&[&self.captures, &registration, &[trigger_binder]], meter)?;
            self.slot(
                template,
                group_value(trigger, "guard").expect("trigger guard"),
                &guard_allowed,
                "trigger-guard",
                Some("trigger.guard"),
                None,
                meter,
            )?;
            let activation_captures = members(argument(args, 7).expect("activation captures"))
                .expect("activation captures");
            let mut activation = meter.vec(activation_captures.len())?;
            for (ordinal, capture) in activation_captures.iter().enumerate() {
                let initializer = group_value(capture, "initializer").expect("initializer");
                let allowed =
                    PositionSet::of(&[&guard_allowed.0, &self.c0[..1], &activation], meter)?;
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
            let mut captured =
                PositionSet::of(&[&self.captures, &registration, &activation], meter)?;
            let retry_allowed = captured.with(&[earlier, later], meter)?;
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
            captured.insert(recovery_binder, meter)?;
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

enum SlotWalk<'a> {
    Node(usize, Option<usize>),
    Term(&'a CheckedTerm, Option<usize>),
    Bind(&'a CheckedTerm, Option<usize>),
}

/// A set of node positions as one sorted, duplicate-free block of exactly
/// the requested capacity, charged before it is requested. Positions are
/// dense node indices, so the order is the numeric order a position map
/// would iterate in.
#[derive(Default)]
struct PositionSet(Vec<usize>);

impl PositionSet {
    /// The union of `parts`, in a block sized for their total length.
    fn of(parts: &[&[usize]], meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let total = parts.iter().map(|part| part.len()).sum();
        let mut items = meter.vec(total)?;
        for part in parts {
            items.extend_from_slice(part);
        }
        items.sort_unstable();
        items.dedup();
        Ok(Self(items))
    }

    /// This set with `extra` added, in a new block.
    fn with(&self, extra: &[usize], meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        Self::of(&[&self.0, extra], meter)
    }

    fn duplicate(&self, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        Ok(Self(meter.copy_slice(&self.0)?))
    }

    fn insert(&mut self, position: usize, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
        meter.insert_sorted(&mut self.0, position).map(drop)
    }

    fn contains(&self, position: usize) -> bool {
        self.0.binary_search(&position).is_ok()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().copied()
    }

    /// Members of this set that `other` lacks, in order.
    fn difference<'s>(&'s self, other: &'s Self) -> impl Iterator<Item = usize> + 's {
        self.iter().filter(|position| !other.contains(*position))
    }
}

/// How one local scope is entered from its parent.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum SlotKey<'a> {
    Name(Option<usize>, &'a str),
    Arm(Option<usize>, usize, usize),
    Binder(Option<usize>, usize),
}

/// Local scopes of one slot walk. Entering the same scope from the same
/// parent gives the same id, so a revisited term is recognised; the lookup
/// is one sorted exact block grown only after the growth is charged.
#[derive(Default)]
struct SlotLocals<'a> {
    nodes: Vec<(Option<usize>, SlotLocal<'a>)>,
    entered: Vec<(SlotKey<'a>, usize)>,
}

enum SlotLocal<'a> {
    Name(&'a str),
    Binder(usize),
    Arm,
}

impl<'a> SlotLocals<'a> {
    fn enter(
        &mut self,
        parent: Option<usize>,
        key: SlotKey<'a>,
        local: SlotLocal<'a>,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        match self.entered.binary_search_by(|(held, _)| held.cmp(&key)) {
            Ok(at) => Ok(self.entered[at].1),
            Err(at) => {
                let id = self.nodes.len();
                meter.push(&mut self.nodes, (parent, local))?;
                meter.room(&mut self.entered, 1)?;
                self.entered.insert(at, (key, id));
                Ok(id)
            }
        }
    }

    fn bind(
        &mut self,
        parent: Option<usize>,
        name: &'a str,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        self.enter(
            parent,
            SlotKey::Name(parent, name),
            SlotLocal::Name(name),
            meter,
        )
    }

    fn arm(
        &mut self,
        parent: Option<usize>,
        case: usize,
        ordinal: usize,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        self.enter(
            parent,
            SlotKey::Arm(parent, case, ordinal),
            SlotLocal::Arm,
            meter,
        )
    }

    fn bind_node(
        &mut self,
        parent: Option<usize>,
        binder: usize,
        meter: &mut WorkMeter,
    ) -> Result<usize, ValidationFailure> {
        self.enter(
            parent,
            SlotKey::Binder(parent, binder),
            SlotLocal::Binder(binder),
            meter,
        )
    }

    fn contains(
        &self,
        mut scope: Option<usize>,
        position: usize,
        name: &str,
        arm_binders: &[usize],
        meter: &mut WorkMeter,
        owner: usize,
    ) -> Result<bool, ValidationFailure> {
        while let Some(id) = scope {
            node_work(meter, owner)?;
            let (parent, local) = &self.nodes[id];
            let found = match local {
                SlotLocal::Name(local) => {
                    arm_binders.binary_search(&position).is_err() && *local == name
                }
                SlotLocal::Binder(local) => *local == position,
                SlotLocal::Arm => false,
            };
            if found {
                return Ok(true);
            }
            scope = *parent;
        }
        Ok(false)
    }
}

struct ControlScopeFrame {
    position: usize,
    input: PositionSet,
    output: PositionSet,
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

/// The binders a compensation template declares, in declaration order, in
/// a block sized for them and charged first.
fn compensation_binders<'a>(
    args: &'a [CheckedTerm],
    meter: &mut WorkMeter,
) -> Result<Vec<&'a CheckedTerm>, ValidationFailure> {
    let registration =
        members(argument(args, 5).expect("registration captures")).expect("capture list");
    let activation =
        members(argument(args, 7).expect("activation captures")).expect("capture list");
    let mut binders = meter.vec(6 + registration.len() + activation.len())?;
    binders.push(argument(args, 1).expect("forward binder"));
    for capture in registration {
        binders.push(group_value(capture, "binder").expect("capture binder"));
    }
    binders
        .push(group_value(argument(args, 6).expect("trigger"), "binder").expect("trigger binder"));
    for capture in activation {
        binders.push(group_value(capture, "binder").expect("capture binder"));
    }
    let retry = argument(args, 10).expect("retry");
    for name in ["earlier", "later"] {
        binders.push(group_value(retry, name).expect("retry binder"));
    }
    binders.push(
        group_value(argument(args, 12).expect("recovery"), "binder").expect("recovery binder"),
    );
    Ok(binders)
}

fn causal_edge(
    edges: &mut [Vec<usize>],
    from: usize,
    to: usize,
    position: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    node_work(meter, position)?;
    meter.push(&mut edges[from], to)
}

#[derive(Clone, Copy)]
enum GuardOp {
    Constant(bool),
    Atom(usize),
    Load(usize),
    Store(usize),
    Not,
    And,
    Or,
    Implies,
}

enum GuardAtom<'a> {
    Node(&'a CheckedNodeId),
    Inline,
}

impl GuardAtom<'_> {
    fn same_node(&self, id: &CheckedNodeId) -> bool {
        matches!(self, Self::Node(candidate) if *candidate == id)
    }
}

fn guard_connective(identity: Option<&str>, arity: usize) -> Option<GuardOp> {
    match (identity, arity) {
        (Some("quire.op.boolean.not"), 1) => Some(GuardOp::Not),
        (Some("quire.op.boolean.and"), 2) => Some(GuardOp::And),
        (Some("quire.op.boolean.or"), 2) => Some(GuardOp::Or),
        (Some("quire.op.boolean.implies"), 2) => Some(GuardOp::Implies),
        _ => None,
    }
}

fn guard_never_exits(
    check: &MemberCheck<'_>,
    args: &[CheckedTerm],
    position: usize,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let visible = binding_value(args, "visible").expect("validated visible");
    let mut atoms: Vec<GuardAtom<'_>> = meter.vec(listed_targets(visible).count())?;
    atoms.extend(listed_targets(visible).map(GuardAtom::Node));
    let mut program = Vec::new();
    let mut compiled = Vec::new();
    let guard = binding_value(args, "guard").expect("validated guard");
    let mut pending = meter.vec(1)?;
    pending.push((guard, None, false));
    while let Some((term, key, complete)) = pending.pop() {
        if complete {
            let CheckedTerm::Application {
                operation,
                arguments,
                ..
            } = term
            else {
                unreachable!("only connective applications complete")
            };
            meter.push(
                &mut program,
                guard_connective(operation.identity(), arguments.len())
                    .expect("validated connective arity"),
            )?;
            if let Some(id) = key {
                let index = compiled
                    .iter()
                    .position(|candidate| candidate == &id)
                    .expect("registered connective");
                meter.push(&mut program, GuardOp::Store(index))?;
            }
            continue;
        }
        if let Some(id) = key {
            if let Some(index) = atoms.iter().position(|candidate| candidate.same_node(id)) {
                meter.push(&mut program, GuardOp::Atom(index))?;
                continue;
            }
        }
        let (term, key) = match term {
            CheckedTerm::Reference { target } => {
                if let Some(index) = atoms
                    .iter()
                    .position(|candidate| candidate.same_node(target))
                {
                    meter.push(&mut program, GuardOp::Atom(index))?;
                    continue;
                }
                if let Some(index) = compiled.iter().position(|candidate| *candidate == target) {
                    meter.push(&mut program, GuardOp::Load(index))?;
                    continue;
                }
                let target_position = *check.index.get(target).expect("validated guard reference");
                if !matches!(
                    check.kinds[target_position],
                    CheckedNodeKind::Value(ValueForm::Literal) | CheckedNodeKind::Expression(_)
                ) {
                    let index = atoms.len();
                    meter.push(&mut atoms, GuardAtom::Node(target))?;
                    meter.push(&mut program, GuardOp::Atom(index))?;
                    continue;
                }
                let Some(referenced) = check.terms[target_position].as_ref() else {
                    let index = atoms.len();
                    meter.push(&mut atoms, GuardAtom::Node(target))?;
                    meter.push(&mut program, GuardOp::Atom(index))?;
                    continue;
                };
                (referenced, Some(target))
            }
            _ => (term, key),
        };
        match term {
            CheckedTerm::Literal {
                value_kind: LiteralKind::Boolean,
                value,
                ..
            } => {
                meter.push(
                    &mut program,
                    GuardOp::Constant(value.as_bool().expect("validated Boolean literal")),
                )?;
            }
            CheckedTerm::Application {
                operation,
                arguments,
                ..
            } if guard_connective(operation.identity(), arguments.len()).is_some() => {
                if let Some(id) = key {
                    meter.push(&mut compiled, id)?;
                }
                meter.push(&mut pending, (term, key, true))?;
                meter.room(&mut pending, arguments.len())?;
                for argument in arguments {
                    pending.push((argument, None, false));
                }
            }
            _ => {
                let index = atoms.len();
                meter.push(
                    &mut atoms,
                    match key {
                        Some(id) => GuardAtom::Node(id),
                        None => GuardAtom::Inline,
                    },
                )?;
                meter.push(&mut program, GuardOp::Atom(index))?;
            }
        }
    }
    let mut valuation = meter.filled(atoms.len(), false)?;
    let mut values = meter.vec(program.len())?;
    let mut cache = meter.filled(compiled.len(), false)?;
    loop {
        node_work(meter, position)?;
        values.clear();
        for op in &program {
            match op {
                GuardOp::Constant(value) => values.push(*value),
                GuardOp::Atom(index) => values.push(valuation[*index]),
                GuardOp::Load(index) => values.push(cache[*index]),
                GuardOp::Store(index) => {
                    cache[*index] = *values.last().expect("validated connective result");
                }
                GuardOp::Not => {
                    let value = values.pop().expect("validated not operand");
                    values.push(!value);
                }
                GuardOp::And | GuardOp::Or | GuardOp::Implies => {
                    let left = values.pop().expect("validated binary left operand");
                    let right = values.pop().expect("validated binary right operand");
                    values.push(match op {
                        GuardOp::And => left && right,
                        GuardOp::Or => left || right,
                        GuardOp::Implies => !left || right,
                        _ => unreachable!(),
                    });
                }
            }
        }
        if !values.pop().expect("validated guard value") {
            return Ok(false);
        }
        let Some(bit) = valuation.iter().position(|value| !value) else {
            return Ok(true);
        };
        valuation[..bit].fill(false);
        valuation[bit] = true;
    }
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
    meter.at_node(root, "protocol");
    let mut preorder = meter.vec(check.nodes.len())?;
    let mut order: Vec<Option<usize>> = meter.filled(check.nodes.len(), None)?;
    let mut pending = meter.vec(1)?;
    pending.push(root);
    while let Some(position) = pending.pop() {
        node_work(meter, position)?;
        if order[position].is_some() {
            continue;
        }
        order[position] = Some(preorder.len());
        preorder.push(position);
        let (operator, args) = validated[position].expect("validated control");
        for child in control_positions(operator, args, check.index, meter)?
            .into_iter()
            .rev()
        {
            meter.push(&mut pending, child)?;
        }
    }
    let mut edges: Vec<Vec<usize>> = meter.filled(preorder.len() * 2, Vec::new())?;
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
        node_work(meter, position)?;
        let (operator, args) = validated[position].expect("validated control");
        let enter = port(position, false).expect("tree member");
        let exit = enter + 1;
        let children = control_positions(operator, args, check.index, meter)?;
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
                let joined_labels = || {
                    binding_value(args, "joined")
                        .into_iter()
                        .flat_map(|term| members(term).into_iter().flatten())
                        .filter_map(text_value)
                };
                let mut joined: Vec<&str> = meter.vec(joined_labels().count())?;
                joined.extend(joined_labels());
                joined.sort_unstable();
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
                    if joined.binary_search(&label).is_ok() {
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

    let mut colors = meter.filled(edges.len(), 0_u8)?;
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for start in 0..edges.len() {
        if colors[start] != 0 {
            continue;
        }
        colors[start] = 1;
        meter.push(&mut stack, (start, 0_usize))?;
        while let Some((vertex, next)) = stack.last_mut() {
            node_work(meter, preorder[*vertex / 2])?;
            if let Some(&successor) = edges[*vertex].get(*next) {
                *next += 1;
                if colors[successor] == 1 {
                    let first = stack
                        .iter()
                        .position(|(node, _)| *node == successor)
                        .expect("gray node in stack");
                    let mut ordinals = meter.vec(stack.len() - first)?;
                    ordinals.extend(stack[first..].iter().map(|(node, _)| *node / 2));
                    ordinals.sort_unstable();
                    ordinals.dedup();
                    let controls = ordinals
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
                    meter.push(&mut stack, (successor, 0))?;
                }
            } else {
                colors[*vertex] = 2;
                stack.pop();
            }
        }
    }

    let mut nested = meter.filled(check.nodes.len(), false)?;
    let mut pending = meter.vec(1)?;
    pending.push((root, false, false));
    while let Some((position, inside_body, in_exhausted)) = pending.pop() {
        nested[position] = inside_body && !in_exhausted;
        let (operator, args) = validated[position].expect("validated control");
        let children = control_positions(operator, args, check.index, meter)?;
        for (ordinal, child) in children.into_iter().enumerate().rev() {
            meter.push(
                &mut pending,
                (
                    child,
                    inside_body || (operator == "repeat" && ordinal == 0),
                    in_exhausted || (operator == "repeat" && ordinal == 1),
                ),
            )?;
        }
    }
    let mut guard_never_exits_by_control = meter.filled(check.nodes.len(), false)?;
    for &position in &preorder {
        let Some(("repeat", args)) = validated[position] else {
            continue;
        };
        if nested[position] {
            guard_never_exits_by_control[position] =
                guard_never_exits(check, args, position, meter)?;
        }
    }
    let mut steps = meter.filled(check.nodes.len(), false)?;
    for &position in preorder.iter().rev() {
        node_work(meter, position)?;
        let (operator, args) = validated[position].expect("validated control");
        let children = control_positions(operator, args, check.index, meter)?;
        steps[position] = match operator {
            "send" | "receive" | "attempt" | "effect" | "event" | "commit" | "fence" | "await"
            | "parallel" => true,
            "sequence" => children.iter().any(|&child| steps[child]),
            "choice" => children.iter().all(|&child| steps[child]),
            "repeat" => {
                let maximum_positive = binding_value(args, "maximum")
                    .and_then(members)
                    .expect("validated maximum")
                    .first()
                    .is_none_or(|maximum| {
                        integer_value(maximum).expect("validated maximum") > Numeral::exact(0)
                    });
                guard_never_exits_by_control[position] && maximum_positive && steps[children[0]]
            }
            "check" => false,
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

fn uses_temporal_protocol_operator(body: &CheckedTerm) -> bool {
    matches!(
        body,
        CheckedTerm::Application {
            operator: ApplicationOperator::ProtocolControl,
            operation,
            ..
        } if matches!(
            operation.member_text("operator"),
            Some("await" | "compensate")
        )
    )
}

/// Whether protocol clock validation depends on a missing selection. This
/// only defers the dependent temporal-binding clock-kind refusal; protocol
/// step 2 reports the selection refusal in node order.
pub(super) fn has_missing_temporal_selection(
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    lock: &CheckedPackageLockV2,
) -> bool {
    if lock
        .profile_selections
        .iter()
        .any(|selection| selection.role == CheckedSelectionRole::TemporalProfile)
    {
        return false;
    }
    for &position in index.values() {
        if !matches!(kinds.get(position), Some(CheckedNodeKind::Protocol(_))) {
            continue;
        }
        if terms
            .get(position)
            .and_then(Option::as_ref)
            .is_some_and(uses_temporal_protocol_operator)
        {
            return true;
        }
    }
    false
}

/// Validate protocol placement, local members, membership and clause layout.
/// What the protocol step reads beside the graph: the operation catalog, the
/// package lock and the byte ceiling.
pub(super) struct ProtocolPass<'a> {
    pub(super) catalog: &'a super::operation_catalog::OperationCatalog,
    pub(super) lock: &'a CheckedPackageLockV2,
    pub(super) bytes: u64,
}

pub(super) fn validate_protocol_nodes(
    nodes: &[RawSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    pass: ProtocolPass<'_>,
    owners: &ModelOwners<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let ProtocolPass {
        catalog,
        lock,
        bytes,
    } = pass;
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
    meter.at_node(0, "protocol");
    let mut validated = meter.filled(nodes.len(), None)?;
    for &position in index.values() {
        meter.at_node(
            position,
            super::graph_storage::node_form(&nodes[position].node_tag),
        );
        if matches!(
            terms.get(position).and_then(Option::as_ref),
            Some(CheckedTerm::Application {
                operator: ApplicationOperator::ProtocolControl,
                ..
            })
        ) && !matches!(
            kinds.get(position),
            Some(CheckedNodeKind::Protocol(form)) if *form != ProtocolForm::ProtocolReference
        ) {
            return Err(body_defect(
                position,
                &nodes[position],
                ProtocolBodyDefect::OperatorIneligible,
            ));
        }
        if kinds.get(position) == Some(&CheckedNodeKind::State(StateForm::StateClause)) {
            let mut has_protocol_member_reference = false;
            if let Some(body) = terms.get(position).and_then(Option::as_ref) {
                body.visit_reference_targets(&mut |target| {
                    has_protocol_member_reference |= index
                        .get(target)
                        .and_then(|&target_position| kinds.get(target_position))
                        .is_some_and(|kind| {
                            matches!(
                                kind,
                                CheckedNodeKind::Protocol(form)
                                    if !matches!(
                                        form,
                                        ProtocolForm::ProtocolClause | ProtocolForm::ProtocolReference
                                    )
                            )
                        });
                });
            }
            if has_protocol_member_reference {
                return Err(ValidationFailure::refused_at(
                    Code::IllTyped,
                    node_pointer(position),
                    Some(Cause::OperatorIneligible),
                    nodes[position].node_id.clone(),
                ));
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
        meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
        let work_path = super::stage_storage::Path::root()
            .key("semantic_graph")
            .key("nodes")
            .index(position)
            .key("body");
        let work_site = super::natural::site(meter, work_path);
        super::natural::work_at(meter, work_site, 1)?;
        let Some(body) = terms.get(position).and_then(Option::as_ref) else {
            return Err(body_defect(
                position,
                node,
                ProtocolBodyDefect::OperatorIneligible,
            ));
        };
        if !has_temporal_profile && uses_temporal_protocol_operator(body) {
            return Err(ValidationFailure::refused_at(
                Code::MissingDeclaration,
                node_pointer(position),
                Some(Cause::MissingSelection),
                node.node_id.clone(),
            ));
        }
        let (operator, arguments) =
            validate_envelope(form, body, &node.semantic_type, selected_profile)
                .map_err(|defect| body_defect(position, node, defect))?;
        if index
            .get(&node.semantic_type)
            .and_then(|&type_position| kinds.get(type_position))
            != Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean))
        {
            return Err(ValidationFailure::refused_at(
                Code::IllTyped,
                node_pointer(position),
                Some(Cause::OperatorIneligible),
                node.node_id.clone(),
            ));
        }
        validated[position] = Some((operator, arguments));
    }
    validate_protocol_declaration_names(nodes, kinds, index, meter)?;
    validate_protocol_declaration_paths(nodes, kinds, &validated, index, meter)?;
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
            meter.at_node(
                position,
                super::graph_storage::node_form(&nodes[position].node_tag),
            );
            if let Some(defect) = validate_related_order(form, operator, arguments, bytes, meter)? {
                return Err(body_defect(position, &nodes[position], defect));
            }
            let check = MemberCheck {
                catalog,
                position,
                nodes,
                kinds,
                terms,
                index,
                profile: selected_profile.expect("validated protocol profile law"),
            };
            check_member(&check, form, operator, arguments, &validated, owners, meter)?;
        }
    }
    validate_protocol_membership(terms, nodes, kinds, &validated, index, meter)?;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause)) {
            continue;
        }
        let node = &nodes[position];
        let Some(owner_model) = node.owner_model.as_ref() else {
            return Err(ValidationFailure::refused(
                Code::MalformedWire,
                node_pointer(position),
            ));
        };
        let owner_at = || node_pointer(position).key("owner_model");
        let owner_position = index.get(owner_model).copied().ok_or_else(|| {
            ValidationFailure::refused_at(
                Code::MissingDeclaration,
                owner_at(),
                Some(Cause::MissingName),
                node.node_id.clone(),
            )
        })?;
        let target = &nodes[owner_position];
        if kinds[owner_position] != CheckedNodeKind::Model(ModelForm::ObjectType) {
            return Err(ValidationFailure::refused_at(
                Code::InvalidModelBinding,
                owner_at(),
                Some(Cause::MalformedDeclaration),
                node.node_id.clone(),
            ));
        }
        let owner = owners.recover(target).map_err(|failure| {
            ValidationFailure::refused_at(
                failure.code,
                owner_at(),
                Some(failure.cause),
                node.node_id.clone(),
            )
        })?;
        if owner.form != super::model_members::DeclarationForm::ObjectType {
            return Err(ValidationFailure::refused_at(
                Code::InvalidModelBinding,
                owner_at(),
                Some(Cause::MalformedDeclaration),
                node.node_id.clone(),
            ));
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
            catalog,
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

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn healthy_clause_members_guards_and_captures_build_no_diagnostic_paths() {
        let reference = |name| json!({"term":"reference", "target":id(name)});
        let truth = || {
            json!({"term":"literal", "type":id('b'),
            "value_kind":"boolean", "value":true})
        };
        let parameter = |name: &str, level: usize| {
            json!({"term":"aggregate", "members":[
                {"term":"binding", "name":"name", "value":{
                    "term":"literal", "type":id('r'), "value_kind":"text", "value":name}},
                {"term":"binding", "name":"level", "value":{
                    "term":"literal", "type":id('q'), "value_kind":"integer", "value":level.to_string()}}
            ]})
        };
        let catalog = super::super::operation_catalog::owned_operation_catalog();
        let profile = serde_json::from_value(selected()).expect("protocol profile");
        for (slot_checks, listed, captures) in [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, false, true),
        ] {
            let mut clause = body(ProtocolForm::ProtocolClause, "protocol");
            clause["arguments"][0]["value"] = reference('a');
            clause["arguments"][11]["value"] = reference('e');
            clause["arguments"][12]["value"] = reference('f');
            if listed {
                for (ordinal, name) in [(3, 'h'), (4, 'i'), (5, 'j'), (6, 'k'), (7, 'l'), (8, 'm')]
                {
                    clause["arguments"][ordinal]["value"]["members"] = json!([reference(name)]);
                }
                clause["arguments"][3]["value"]["members"] =
                    json!([reference('h'), reference('n')]);
            }
            if captures {
                clause["arguments"][1]["value"]["members"] = json!([
                    {"term":"binding", "name":"kind", "value":text("each")},
                    {"term":"binding", "name":"trigger", "value":reference('o')},
                    {"term":"binding", "name":"guard", "value":truth()}
                ]);
                clause["arguments"][2]["value"]["members"] = json!([
                    {"term":"aggregate", "members":[
                        {"term":"binding", "name":"binder", "value":reference('d')},
                        {"term":"binding", "name":"initializer", "value":truth()}
                    ]}
                ]);
            }
            let mut finish = body(ProtocolForm::Obligation, "finish");
            finish["arguments"][0]["value"] = reference('c');
            finish["arguments"][1]["value"] = truth();
            let mut definitions = vec![
                (
                    'b',
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean),
                    "scalar_type",
                    "boolean",
                    json!({"term":"aggregate", "members":[]}),
                ),
                (
                    'q',
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Integer),
                    "scalar_type",
                    "integer",
                    json!({"term":"aggregate", "members":[]}),
                ),
                (
                    'r',
                    CheckedNodeKind::ScalarType(ScalarTypeForm::Text),
                    "scalar_type",
                    "text",
                    json!({"term":"aggregate", "members":[]}),
                ),
                (
                    'a',
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    "value",
                    "parameter",
                    parameter("subject", 0),
                ),
                (
                    'c',
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    "value",
                    "parameter",
                    parameter("finished", 1 + 2 * usize::from(captures)),
                ),
                (
                    'd',
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    "value",
                    "parameter",
                    parameter("captured", 1 + usize::from(captures)),
                ),
                (
                    'o',
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    "value",
                    "parameter",
                    parameter("trigger", 1),
                ),
                (
                    'e',
                    CheckedNodeKind::Protocol(ProtocolForm::Control),
                    "protocol",
                    "control",
                    body(ProtocolForm::Control, "sequence"),
                ),
                (
                    'f',
                    CheckedNodeKind::Protocol(ProtocolForm::Obligation),
                    "protocol",
                    "obligation",
                    finish,
                ),
                (
                    'g',
                    CheckedNodeKind::Protocol(ProtocolForm::ProtocolClause),
                    "protocol",
                    "protocol_clause",
                    clause,
                ),
            ];
            if listed {
                for (name, kind, tag, form) in [
                    (
                        'h',
                        CheckedNodeKind::Protocol(ProtocolForm::Role),
                        "protocol",
                        "role",
                    ),
                    (
                        'i',
                        CheckedNodeKind::Relation(RelationForm::Relationship),
                        "relation",
                        "relationship",
                    ),
                    (
                        'j',
                        CheckedNodeKind::Protocol(ProtocolForm::Channel),
                        "protocol",
                        "channel",
                    ),
                    (
                        'k',
                        CheckedNodeKind::Protocol(ProtocolForm::Queue),
                        "protocol",
                        "queue",
                    ),
                    (
                        'l',
                        CheckedNodeKind::Temporal(TemporalForm::TemporalClause),
                        "temporal",
                        "temporal_clause",
                    ),
                    (
                        'm',
                        CheckedNodeKind::Protocol(ProtocolForm::Compensation),
                        "protocol",
                        "compensation",
                    ),
                    (
                        'n',
                        CheckedNodeKind::Protocol(ProtocolForm::Role),
                        "protocol",
                        "role",
                    ),
                ] {
                    definitions.push((
                        name,
                        kind,
                        tag,
                        form,
                        json!({"term":"aggregate", "members":[]}),
                    ));
                }
            }
            let nodes: Vec<RawSemanticNodeV2> = definitions
                .iter()
                .map(|(name, _, tag, form, body)| {
                    serde_json::from_value(json!({
                        "node_id":id(*name), "schema_version":"quire.checked-semantic-graph/v2",
                        "node_tag":tag, "semantic_form":form, "semantic_type":id('b'),
                        "dependencies":[], "occurrences":[], "body":body
                    }))
                    .expect("typed protocol fixture node")
                })
                .collect();
            let kinds: Vec<_> = definitions.iter().map(|(_, kind, _, _, _)| *kind).collect();
            let terms: Vec<_> = definitions
                .iter()
                .map(|(_, _, _, _, term)| {
                    Some(CheckedTerm::decode_admitted(term).expect("typed protocol fixture term"))
                })
                .collect();
            let index: GraphIndex<'_> = nodes
                .iter()
                .enumerate()
                .map(|(position, node)| (&node.node_id, position))
                .collect();
            let position = 9;
            let validated: Vec<_> = terms
                .iter()
                .enumerate()
                .map(|(position, term)| match (kinds[position], term) {
                    (
                        CheckedNodeKind::Protocol(form),
                        Some(term @ CheckedTerm::Application { .. }),
                    ) => {
                        let (operator, arguments) = validate_envelope(
                            form,
                            term,
                            &nodes[position].semantic_type,
                            Some(&profile),
                        )
                        .expect("validated protocol fixture envelope");
                        validate_layout(form, operator, arguments)
                            .expect("validated protocol fixture layout");
                        Some((operator, arguments))
                    }
                    _ => None,
                })
                .collect();
            let check = MemberCheck {
                catalog: &catalog,
                position,
                nodes: &nodes,
                kinds: &kinds,
                terms: &terms,
                index: &index,
                profile: &profile,
            };
            let arguments = validated[position].expect("clause arguments").1;
            let run = |budget: &mut super::super::NativeJoinBudget| {
                let before = budget.read_storage_consumed();
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(budget);
                    meter.at_node(position, "protocol");
                    result = Some(if slot_checks {
                        check_clause_slots(&check, arguments, &validated, &mut meter)
                    } else {
                        check_clause(&check, arguments, &validated, &mut meter)
                    });
                })
                .bytes_total;
                assert_eq!(
                    allocated,
                    budget.read_storage_consumed() - before,
                    "only actual charged scratch requests are made on healthy clauses"
                );
                result.expect("measured clause result")
            };
            let mut ample =
                super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
            let ample_result = run(&mut ample);
            assert!(ample_result.is_ok(), "healthy clause must validate (slots={slot_checks}, listed={listed}, captures={captures}): {ample_result:?}");
            let required = ample.read_storage_consumed();
            assert!(required > 0, "real clause scratch blocks are exercised");
            let work = ample.consumed().work;
            let mut exact =
                super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
            assert!(run(&mut exact).is_ok());
            assert_eq!(exact.consumed().work, work);
            for limit in [0, required - 1] {
                let mut denied =
                    super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
                let Err(ValidationFailure::Budget(stop)) = run(&mut denied) else {
                    panic!("scratch allocation must be denied")
                };
                assert_eq!(
                    stop.resource,
                    super::super::PackageReadResource::ReadStorage
                );
                assert!(denied.first_unpaid_read_storage().is_some());
            }
            let mut repeated = super::super::NativeJoinBudget::new(u64::MAX, 0)
                .with_read_storage_limit(2 * required);
            assert!(run(&mut repeated).is_ok());
            assert!(run(&mut repeated).is_ok());
            assert_eq!(repeated.read_storage_consumed(), 2 * required);
            assert_eq!(repeated.consumed().work, 2 * work);
        }
    }

    /// The position set agrees with the ordered set it replaces on every
    /// operation the binder scopes use, and each block is charged first
    /// (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn position_sets_agree_with_ordered_sets_and_charge_each_block() {
        use crate::NativeJoinBudget;
        use std::collections::BTreeSet;
        let run = |limit: u64| -> Option<u64> {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut meter = WorkMeter::over(&mut budget);
            let base_parts: [&[usize]; 3] = [&[9, 2, 7], &[2, 11], &[]];
            let mut reference: BTreeSet<usize> = base_parts
                .iter()
                .flat_map(|part| part.iter().copied())
                .collect();
            let mut set = PositionSet::of(&base_parts, &mut meter).ok()?;
            let check = |set: &PositionSet, reference: &BTreeSet<usize>| {
                assert_eq!(
                    set.iter().collect::<Vec<_>>(),
                    reference.iter().copied().collect::<Vec<_>>()
                );
                assert_eq!(set.len(), reference.len());
                for probe in 0..14 {
                    assert_eq!(set.contains(probe), reference.contains(&probe), "{probe}");
                }
            };
            check(&set, &reference);
            let wider = set.with(&[4, 7, 13], &mut meter).ok()?;
            let wider_reference: BTreeSet<usize> =
                reference.iter().copied().chain([4, 7, 13]).collect();
            check(&wider, &wider_reference);
            let copy = set.duplicate(&mut meter).ok()?;
            check(&copy, &reference);
            for position in [5, 9, 0] {
                set.insert(position, &mut meter).ok()?;
                reference.insert(position);
                check(&set, &reference);
            }
            assert_eq!(
                wider.difference(&set).collect::<Vec<_>>(),
                wider_reference
                    .difference(&reference)
                    .copied()
                    .collect::<Vec<_>>()
            );
            Some(budget.read_storage_consumed())
        };
        let spent = run(u64::MAX).expect("unbounded run completes");
        assert!(spent > 0, "the blocks are charged");
        assert_eq!(run(spent), Some(spent));
        assert_eq!(run(spent - 1), None, "one byte short is denied");
    }

    /// Protocol integers order exactly beyond any machine width, and read in
    /// place: no request is made for them (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn numeral_orders_and_prints_integers_beyond_machine_width_in_place() {
        fn parse(text: &str) -> Numeral<'_> {
            Numeral::parse(text).expect("integer text")
        }
        let huge = "1000000000000000000000000000000000000000000000000";
        let larger = "1000000000000000000000000000000000000000000000001";
        assert!(parse(huge) < parse(larger));
        assert!(parse(&format!("-{larger}")) < parse(&format!("-{huge}")));
        assert!(parse(&format!("-{huge}")) < Numeral::exact(0));
        assert!(parse(huge) > Numeral::exact(usize::MAX));
        assert_eq!(parse("-0"), Numeral::exact(0));
        assert_eq!(parse("+007_0"), Numeral::exact(70_u32));
        assert_eq!(parse("-0070").to_string(), "-70");
        assert_eq!(parse("000").to_string(), "0");
        assert!(Numeral::parse("1_x").is_none() && Numeral::parse("_1").is_none());
        assert!(Numeral::parse("-").is_none() && Numeral::parse("").is_none());
    }

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

    fn id(name: char) -> Value {
        json!({"package": "test/protocol", "path": [name.to_string()], "ordinal": 0})
    }

    fn example(shape: Shape) -> Value {
        let type_id = id('b');
        match shape {
            Shape::Reference | Shape::ReferenceOrText | Shape::Term => {
                json!({"term": "reference", "target": id('a')})
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
            "result_type": id('b'),
            "arguments": arguments
        })
    }

    fn selected() -> Value {
        json!({"authority": "quire", "identity": "quire.protocol.finite-global/v1"})
    }

    fn text(value: &str) -> Value {
        json!({
            "term": "literal", "value_kind": "text", "value": value,
            "type": id('b')
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
        id('b')
    }

    /// QSpec FR-465-AC-10: the delivery text is one of four policy words or a
    /// canonical count interval `[lo,hi]` of `u64` naturals with `lo <= hi`.
    /// Trace: FR-038-AC-286
    #[trace("TC-947")]
    #[test]
    fn channel_delivery_parses_policy_words_and_canonical_intervals() {
        for (text, expected) in [
            ("unknown", ChannelDelivery::Unknown),
            ("at-most-once", ChannelDelivery::AtMostOnce),
            ("at-least-once", ChannelDelivery::AtLeastOnce),
            ("exactly-once-premise", ChannelDelivery::ExactlyOncePremise),
            ("[0,3]", ChannelDelivery::Between { lo: 0, hi: 3 }),
            ("[0,0]", ChannelDelivery::Between { lo: 0, hi: 0 }),
            ("[2,2]", ChannelDelivery::Between { lo: 2, hi: 2 }),
            (
                "[0,18446744073709551615]",
                ChannelDelivery::Between {
                    lo: 0,
                    hi: u64::MAX,
                },
            ),
        ] {
            assert_eq!(ChannelDelivery::parse(text), Some(expected), "{text}");
        }
        assert_ne!(
            ChannelDelivery::parse("[0,3]"),
            ChannelDelivery::parse("[0,4]")
        );
        assert_ne!(
            ChannelDelivery::parse("[1,1]"),
            ChannelDelivery::parse("exactly-once-premise")
        );
        for text in [
            "[4,3]",
            "[0, 3]",
            "[ 0,3]",
            "[00,3]",
            "[0,03]",
            "[+0,3]",
            "[-1,3]",
            "[0.5,3]",
            "[0,18446744073709551616]",
            "[3]",
            "[0,3",
            "0,3]",
            "[0,3]x",
            "[,3]",
            "[0,]",
            "[0,1,2]",
            "",
            "exactly-once",
        ] {
            assert_eq!(ChannelDelivery::parse(text), None, "{text}");
        }
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
                &id('c'),
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
        first["target"] = id('a');
        second["target"] = id('b');
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

#[cfg(test)]
mod work_storage_tests {
    use super::super::stage_storage::Path;
    use super::super::{
        NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
    };
    use super::*;
    use ix_trace_rs::trace;

    #[trace("TC-472", "FR-038-AC-508", "FR-038-AC-511")]
    #[test]
    fn protocol_work_denial_prepays_exact_pointer_and_retains_prior_locus() {
        for (position, body) in [(0, false), (23, false), (1000, false), (23, true)] {
            let path = Path::root()
                .key("semantic_graph")
                .key("nodes")
                .index(position);
            let path = if body { path.key("body") } else { path };
            let expected_path = path.build();
            // at_node must not overwrite the budget's inherited active locus.
            let locus = NativeJoinLocus {
                source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 25),
                form: "declarations",
            };
            let run = |budget: &mut NativeJoinBudget| {
                budget.set_active_locus(Some(locus));
                let mut result = None;
                let allocated = allocation_counter::measure(|| {
                    let mut meter = WorkMeter::over(budget);
                    meter.at_node(position, "protocol");
                    result = Some(if body {
                        let site = super::super::natural::site(&meter, path);
                        super::super::natural::work_at(&mut meter, site, 1)
                    } else {
                        node_work(&mut meter, position)
                    });
                })
                .bytes_total;
                (result.unwrap(), allocated)
            };
            let mut admitted = NativeJoinBudget::new(1, 0);
            assert_eq!(run(&mut admitted), (Ok(()), 0));
            assert_eq!(admitted.consumed().work, 1);
            let mut ample = NativeJoinBudget::new(0, 0).with_read_storage_limit(u64::MAX);
            let (expected, allocated) = run(&mut ample);
            let Err(ValidationFailure::Budget(stop)) = &expected else {
                panic!("Work denial")
            };
            assert_eq!(stop.resource, super::super::PackageReadResource::Work);
            assert_eq!(stop.amount, 1);
            assert_eq!(stop.path.as_ref(), Some(&expected_path));
            let required = ample.read_storage_consumed();
            assert_eq!(required, expected_path.as_str().len() as u64);
            assert_eq!(allocated, required);
            assert_eq!(ample.first_unpaid_locus(), Some(locus));
            let mut exact = NativeJoinBudget::new(0, 0).with_read_storage_limit(required);
            assert_eq!(run(&mut exact), (expected.clone(), required));
            for limit in [0, required - 1] {
                let mut short = NativeJoinBudget::new(0, 0).with_read_storage_limit(limit);
                let (result, allocated) = run(&mut short);
                let Err(ValidationFailure::Budget(stop)) = result else {
                    panic!("Work denial")
                };
                assert_eq!(stop.path, None);
                assert_eq!(allocated, 0);
                assert_eq!(short.read_storage_consumed(), 0);
                assert_eq!(short.first_unpaid_locus(), Some(locus));
                assert_eq!(short.consumed(), ample.consumed());
            }
            let mut repeated = NativeJoinBudget::new(0, 0).with_read_storage_limit(2 * required);
            assert_eq!(run(&mut repeated), (expected.clone(), required));
            assert_eq!(run(&mut repeated), (expected, required));
            assert_eq!(repeated.read_storage_consumed(), 2 * required);
            assert_eq!(repeated.first_unpaid_locus(), Some(locus));
            assert_eq!(repeated.consumed(), ample.consumed());
        }
    }
}
