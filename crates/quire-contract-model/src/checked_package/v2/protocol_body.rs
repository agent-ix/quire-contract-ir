//! The fixed outer body grammar of the twenty finite global protocol operators.
//!
//! This validator is kept separate from the reader until the protocol step is
//! inserted between abstraction and operations. The later step supplies the
//! selected profile and maps each defect to a located reader refusal.

use super::ProtocolForm;
use serde_json::Value;

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
}

#[derive(Clone, Copy)]
enum Shape {
    Reference,
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

fn members(value: &Value) -> Option<&[Value]> {
    (value.get("term")?.as_str()? == "aggregate")
        .then_some(value.get("members")?.as_array()?.as_slice())
}

fn shape_fits(value: &Value, shape: Shape) -> bool {
    let term = value.get("term").and_then(Value::as_str);
    match shape {
        Shape::Reference => term == Some("reference"),
        Shape::Text => {
            term == Some("literal")
                && value.get("value_kind").and_then(Value::as_str) == Some("text")
                && value.get("value").and_then(Value::as_str).is_some()
        }
        Shape::Integer => {
            term == Some("literal")
                && value.get("value_kind").and_then(Value::as_str) == Some("integer")
        }
        Shape::Boolean => {
            term == Some("reference")
                || term == Some("literal")
                    && value.get("value_kind").and_then(Value::as_str) == Some("boolean")
                    && value.get("value").and_then(Value::as_bool).is_some()
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
                .all(|item| item.get("term").and_then(Value::as_str) == Some("binding"))
        }),
        Shape::Records => members(value).is_some_and(|items| {
            items.iter().all(|item| {
                members(item).is_some_and(|record| {
                    record
                        .iter()
                        .all(|entry| entry.get("term").and_then(Value::as_str) == Some("binding"))
                })
            })
        }),
    }
}

/// Checks the shared protocol application envelope and the fixed outer layout.
/// `selected_profile` is the lock's selected `protocol_profile` definition.
///
/// This has no graph context; target forms, nested record layouts, canonical
/// ordering and binder scope belong to the later protocol reader step.
#[allow(dead_code)] // The reader step will call this once QSpec fixtures are published.
pub(super) fn validate_protocol_body(
    form: ProtocolForm,
    body: &Value,
    semantic_type: &Value,
    selected_profile: Option<&Value>,
) -> Result<(), ProtocolBodyDefect> {
    use ProtocolBodyDefect as D;
    if body.get("term").and_then(Value::as_str) != Some("application")
        || body.get("operator").and_then(Value::as_str) != Some("protocol_control")
        || body.get("result_type") != Some(semantic_type)
        || body.pointer("/operation/identity").and_then(Value::as_str)
            != Some("quire.op.protocol.control")
    {
        return Err(D::OperatorIneligible);
    }
    let operation = body.get("operation").ok_or(D::OperatorIneligible)?;
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
    if selected_profile.is_none() || selected_profile != law.get("definition") {
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
    let expected = layout(form, operator).ok_or_else(|| {
        if ProtocolForm::ALL
            .iter()
            .any(|&candidate| layout(candidate, operator).is_some())
        {
            D::OperatorIneligible
        } else {
            D::MemberMismatch
        }
    })?;
    let arguments = body
        .get("arguments")
        .and_then(Value::as_array)
        .ok_or(D::MalformedLayout)?;
    if arguments.len() != expected.len()
        || !arguments.iter().zip(expected).all(|(argument, binding)| {
            argument.get("term").and_then(Value::as_str) == Some("binding")
                && argument.get("name").and_then(Value::as_str) == Some(binding.name)
                && argument
                    .get("value")
                    .is_some_and(|value| shape_fits(value, binding.shape))
        })
    {
        return Err(D::MalformedLayout);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn example(shape: Shape) -> Value {
        match shape {
            Shape::Reference => {
                json!({"term": "reference", "target": {"domain": "quire.checked-semantic-node/v1", "digest": "a".repeat(64)}})
            }
            Shape::Text => {
                json!({"term": "literal", "value_kind": "text", "value": "name", "type": Value::Null})
            }
            Shape::Integer => {
                json!({"term": "literal", "value_kind": "integer", "value": 1, "type": Value::Null})
            }
            Shape::Boolean => {
                json!({"term": "literal", "value_kind": "boolean", "value": true, "type": Value::Null})
            }
            Shape::Leaf => example(Shape::Reference),
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
            .map(|binding| json!({"term": "binding", "name": binding.name, "value": example(binding.shape)}))
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

    fn boolean() -> Value {
        json!({"domain": "quire.checked-semantic-node/v1", "digest": "b".repeat(64)})
    }

    /// The twenty names and their seven forms are the QSpec FR-464 catalog.
    #[trace("TC-946")]
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
    }

    /// The role, channel and queue cases exercise QSpec TC-947 outer shapes.
    #[trace("TC-947")]
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

    /// TC-948 supplies the fixed control and compensation argument frames.
    #[trace("TC-948")]
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

    #[trace("TC-946")]
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

    #[trace("TC-946")]
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
}
