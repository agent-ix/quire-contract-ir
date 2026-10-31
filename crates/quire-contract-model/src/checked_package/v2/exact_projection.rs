// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only projection of admitted checked types into Exact value types.

use super::extension::integer_range_bounds;
use super::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CompositeTypeForm,
    ModelForm, ScalarTypeForm,
};
use quire_exact::{Integer, IntegerInterval, ValueType};

/// Selected-model identities supplied by the checking stage alongside a package.
///
/// An empty selection supports scalar reads. Identity-bearing reads refuse
/// until the selected-model stage supplies a complete identity witness.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectedExactIdentities {
    _private: (),
}

impl SelectedExactIdentities {
    /// An empty selection for reads of types without declaration identities.
    pub fn empty() -> Self {
        Self::default()
    }
}

/// One complete Exact value type read from an admitted package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactTypeProjection {
    node: CheckedNodeId,
    type_node: CheckedNodeId,
    value_type: ValueType,
}

impl ExactTypeProjection {
    /// The requested structural node.
    pub fn node(&self) -> &CheckedNodeId {
        &self.node
    }

    /// The semantic type node resolved from the admitted graph.
    pub fn type_node(&self) -> &CheckedNodeId {
        &self.type_node
    }

    /// The complete Exact type.
    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }
}

/// Stable reason a checked type cannot be projected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactTypeProjectionErrorCode {
    /// The request does not resolve in this admitted package.
    UnknownNode,
    /// The node belongs to a different package.
    ForeignNode,
    /// A selected declaration has no authoritative identity row.
    UnavailableIdentityWitness,
    /// The requested type has no faithful Exact representation yet.
    UnsupportedExactType,
    /// The caller's finite projection work limit was consumed.
    ExhaustedWork,
}

impl ExactTypeProjectionErrorCode {
    /// Stable machine-readable spelling of the refusal.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownNode => "unknown_node",
            Self::ForeignNode => "foreign_node",
            Self::UnavailableIdentityWitness => "unavailable_identity_witness",
            Self::UnsupportedExactType => "unsupported_exact_type",
            Self::ExhaustedWork => "exhausted_work",
        }
    }

    /// Every refusal code exposed by this projection.
    pub const fn all() -> &'static [Self] {
        &[
            Self::UnknownNode,
            Self::ForeignNode,
            Self::UnavailableIdentityWitness,
            Self::UnsupportedExactType,
            Self::ExhaustedWork,
        ]
    }

    /// Decodes an exact stable refusal spelling.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

/// A typed refusal or incomplete read; no partial value type is returned.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("Exact type projection refused: {code:?} at {form}")]
pub struct ExactTypeProjectionError {
    /// Stable refusal category.
    pub code: ExactTypeProjectionErrorCode,
    /// The structural node the caller requested.
    pub requested: Box<CheckedNodeId>,
    /// First type node whose projection failed, if it resolved.
    pub at: Option<Box<CheckedNodeId>>,
    /// Admitted form at the failure locus, or `unknown_node`.
    pub form: Box<str>,
    /// Work successfully charged before the refusal.
    pub work: u64,
}

fn charge_work(
    requested: &CheckedNodeId,
    at: &CheckedNodeId,
    form: &str,
    work: &mut u64,
    limit: u64,
) -> Result<(), ExactTypeProjectionError> {
    charge_work_amount(requested, at, form, work, limit, 1)
}

fn charge_work_amount(
    requested: &CheckedNodeId,
    at: &CheckedNodeId,
    form: &str,
    work: &mut u64,
    limit: u64,
    amount: u64,
) -> Result<(), ExactTypeProjectionError> {
    if amount > limit.saturating_sub(*work) {
        return Err(ExactTypeProjectionError {
            code: ExactTypeProjectionErrorCode::ExhaustedWork,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(at.clone())),
            form: form.into(),
            work: *work,
        });
    }
    *work += amount;
    Ok(())
}

impl CheckedPackageV2 {
    /// Read one admitted node's semantic type without reparsing its source.
    ///
    /// The finite `work_limit` charges the requested node and every type node
    /// inspected. The selected identities are required for declaration-bearing
    /// types. A refusal never exposes a partial Exact type.
    pub fn project_exact_type(
        &self,
        requested: &CheckedNodeId,
        _identities: &SelectedExactIdentities,
        work_limit: u64,
    ) -> Result<ExactTypeProjection, ExactTypeProjectionError> {
        let mut work = 0;
        charge_work(requested, requested, "node", &mut work, work_limit)?;
        let node = self
            .node(requested)
            .ok_or_else(|| ExactTypeProjectionError {
                code: if requested.package == self.wire.name {
                    ExactTypeProjectionErrorCode::UnknownNode
                } else {
                    ExactTypeProjectionErrorCode::ForeignNode
                },
                requested: Box::new(requested.clone()),
                at: None,
                form: "unknown_node".into(),
                work,
            })?;
        let type_id = &node.semantic_type;
        charge_work(requested, type_id, "semantic_type", &mut work, work_limit)?;
        let type_node = self.node(type_id).ok_or_else(|| ExactTypeProjectionError {
            code: ExactTypeProjectionErrorCode::UnknownNode,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(type_id.clone())),
            form: "unknown_node".into(),
            work,
        })?;
        let kind = self
            .node_kind(type_id)
            .ok_or_else(|| ExactTypeProjectionError {
                code: ExactTypeProjectionErrorCode::UnknownNode,
                requested: Box::new(requested.clone()),
                at: Some(Box::new(type_id.clone())),
                form: "unknown_node".into(),
                work,
            })?;
        let range_bounds = (kind
            == CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange))
        .then(|| self.term(type_id).and_then(integer_range_bounds))
        .flatten();
        if let Some((lower, upper)) = range_bounds {
            let bytes = lower.len().saturating_add(upper.len());
            charge_work_amount(
                requested,
                type_id,
                "bounded_domain/integer_range",
                &mut work,
                work_limit,
                u64::try_from(bytes).unwrap_or(u64::MAX),
            )?;
        }
        let refuse = |code| ExactTypeProjectionError {
            code,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(type_id.clone())),
            form: format!("{}/{}", type_node.node_tag, type_node.semantic_form).into(),
            work,
        };
        let value_type = match kind {
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean) => ValueType::Boolean,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer) => ValueType::Integer,
            CheckedNodeKind::ScalarType(ScalarTypeForm::String) => ValueType::String,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Uuid) => ValueType::Uuid,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Timestamp) => ValueType::Timestamp,
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => {
                let (lower, upper) = range_bounds
                    .ok_or_else(|| refuse(ExactTypeProjectionErrorCode::UnsupportedExactType))?;
                let lower = lower
                    .parse::<Integer>()
                    .map_err(|_| refuse(ExactTypeProjectionErrorCode::UnsupportedExactType))?;
                let upper = upper
                    .parse::<Integer>()
                    .map_err(|_| refuse(ExactTypeProjectionErrorCode::UnsupportedExactType))?;
                let interval = IntegerInterval::new(lower, upper)
                    .map_err(|_| refuse(ExactTypeProjectionErrorCode::UnsupportedExactType))?;
                ValueType::Int(interval)
            }
            CheckedNodeKind::Model(ModelForm::RecordValueType)
            | CheckedNodeKind::CompositeType(CompositeTypeForm::Reference)
            | CheckedNodeKind::CompositeType(CompositeTypeForm::Union) => {
                return Err(refuse(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                ));
            }
            _ => return Err(refuse(ExactTypeProjectionErrorCode::UnsupportedExactType)),
        };
        Ok(ExactTypeProjection {
            node: requested.clone(),
            type_node: type_id.clone(),
            value_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{CheckedPackageWireV2, RetainedModels};
    use ix_trace_rs::trace;
    use serde_json::{json, Value};

    fn id(path: &str) -> Value {
        json!({"package":"exact-test","path":[path],"ordinal":0})
    }

    fn node(path: &str, tag: &str, form: &str, type_path: &str, body: Value) -> Value {
        json!({
            "node_id":id(path),
            "schema_version":"quire.checked-semantic-graph/v2",
            "node_tag":tag,"semantic_form":form,"semantic_type":id(type_path),
            "dependencies":[],"occurrences":[],"body":body,
        })
    }

    fn fixture() -> CheckedPackageV2 {
        let empty = json!({"term":"aggregate","members":[]});
        let range = |path: &str, lower: &str, upper: &str| {
            node(
                path,
                "bounded_domain",
                "integer_range",
                "integer",
                json!({
                    "term":"aggregate","members":[
                        {"term":"binding","name":"min","value":{"term":"literal","type":id("integer"),"value_kind":"integer","value":lower}},
                        {"term":"binding","name":"max","value":{"term":"literal","type":id("integer"),"value_kind":"integer","value":upper}},
                    ]
                }),
            )
        };
        let wire: CheckedPackageWireV2 = super::super::test_support::private_wire(json!({
            "contract_version":"quire.checked-package/v2","name":"exact-test",
            "lock":{"sources":[],"edition":{"role":"edition","definition":{"authority":"test","identity":"edition"}},
                "profile_selections":[],"definition_selections":[],"required_features":[],"dependency_selections":[]},
            "semantic_graph":{"graph_version":"quire.checked-semantic-graph/v2","nodes":[
                node("boolean","scalar_type","boolean","boolean",empty.clone()),
                node("integer","scalar_type","integer","integer",empty.clone()),
                node("string","scalar_type","string","string",empty.clone()),
                node("uuid","scalar_type","uuid","uuid",empty.clone()),
                node("timestamp","scalar_type","timestamp","timestamp",empty.clone()),
                range("range1000","0","1000"),
                range("range3","0","3"),
                range("range9","0","9"),
                node("total","value","parameter","range1000",empty.clone()),
                node("tally","value","parameter","range3",empty.clone()),
                node("count","value","parameter","range9",empty.clone()),
                node("receipt","model","record_value_type","receipt",empty.clone()),
                node("alias","composite_type","alias","alias",empty),
            ]},
            "source_map":[],"capability_report":[],
            "diagnostics":{"catalog":{"authority":"test","identity":"diagnostics"},"entries":[]},
        }));
        let terms = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| node.body.clone())
            .collect();
        let kinds = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| {
                CheckedNodeKind::decode(
                    super::super::CheckedNodeTag::from_wire(&node.node_tag).expect("known tag"),
                    &node.semantic_form,
                )
                .expect("known form")
            })
            .collect();
        CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            structural_keys: Vec::new(),
            source_index: super::super::retained_source_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds,
            bytes: 1 << 20,
            models: RetainedModels::default(),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
        }
    }

    fn checked_id(path: &str) -> CheckedNodeId {
        serde_json::from_value(id(path)).expect("fixture identity")
    }

    /// Trace: FR-352-AC-1, FR-352-AC-5
    #[trace("FR-352-AC-1", "FR-352-AC-5")]
    #[test]
    fn scalar_and_inclusive_range_projection() {
        let package = fixture();
        let identities = SelectedExactIdentities::empty();
        for (path, expected) in [
            ("boolean", ValueType::Boolean),
            ("integer", ValueType::Integer),
            ("string", ValueType::String),
            ("uuid", ValueType::Uuid),
            ("timestamp", ValueType::Timestamp),
        ] {
            let projected = package
                .project_exact_type(&checked_id(path), &identities, 2)
                .expect("scalar");
            assert_eq!(projected.value_type(), &expected, "{path}");
        }
        for (path, upper) in [("total", 1000_i64), ("tally", 3), ("count", 9)] {
            let projected = package
                .project_exact_type(&checked_id(path), &identities, 12)
                .expect("range");
            let expected = ValueType::Int(
                IntegerInterval::new(Integer::from(0_i64), Integer::from(upper)).expect("ordered"),
            );
            assert_eq!(projected.value_type(), &expected, "{path}");
        }
    }

    /// Trace: FR-352-AC-4, FR-352-AC-6, FR-352-AC-7
    #[trace("FR-352-AC-4", "FR-352-AC-6", "FR-352-AC-7")]
    #[test]
    fn unknown_foreign_unsupported_and_exhausted_are_distinct() {
        let package = fixture();
        let identities = SelectedExactIdentities::empty();
        let refusal = |id: CheckedNodeId, limit| {
            package
                .project_exact_type(&id, &identities, limit)
                .expect_err("refused")
        };
        let mut changed = checked_id("boolean");
        changed.ordinal = 1;
        assert_eq!(
            refusal(changed, 2).code,
            ExactTypeProjectionErrorCode::UnknownNode
        );
        assert_eq!(
            refusal(checked_id("different-path"), 2).code,
            ExactTypeProjectionErrorCode::UnknownNode
        );
        let mut foreign = checked_id("boolean");
        foreign.package = "another-package".into();
        assert_eq!(
            refusal(foreign, 2).code,
            ExactTypeProjectionErrorCode::ForeignNode
        );
        assert_eq!(
            refusal(checked_id("alias"), 2).code,
            ExactTypeProjectionErrorCode::UnsupportedExactType
        );
        assert_eq!(
            refusal(checked_id("receipt"), 2).code,
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness
        );
        let exhausted = refusal(checked_id("total"), 1);
        assert_eq!(exhausted.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(exhausted.work, 1);
        assert_eq!(exhausted.at, Some(Box::new(checked_id("range1000"))));
        assert_eq!(
            package.project_exact_type(&checked_id("total"), &identities, 12),
            package
                .clone()
                .project_exact_type(&checked_id("total"), &identities, 12)
        );
    }
}
