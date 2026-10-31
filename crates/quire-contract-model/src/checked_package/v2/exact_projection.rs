// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only projection of admitted checked types into Exact value types.

use super::extension::integer_range_bounds;
use super::model_members::DeclarationForm;
use super::structural::reference_type_target_typed;
use super::{
    BoundedDomainForm, CheckedNodeId, CheckedNodeKind, CheckedPackageV2, CompositeTypeForm,
    ModelForm, ScalarTypeForm,
};
use quire_exact::{
    EffectiveId, Integer, IntegerInterval, MemberId, MemberOwner, NodeKey, UnitId, ValueType,
};
use std::collections::{BTreeMap, BTreeSet};

/// The Exact identity produced for one selected declaration form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedExactIdentity {
    /// A selected record declaration's structural key.
    Record(NodeKey),
    /// A selected object declaration's effective identity.
    Object(EffectiveId),
    /// A selected union declaration's structural key.
    Union(NodeKey),
    /// A selected declared unit's identity.
    DeclaredUnit(UnitId),
}

/// The selected declaration form paired with its Exact identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SelectedExactDeclarationForm {
    /// `model/record_value_type`.
    RecordValueType,
    /// `model/object_type`.
    ObjectType,
    /// `composite_type/union`.
    Union,
    /// `scalar_type/unit`.
    DeclaredUnit,
}

impl SelectedExactDeclarationForm {
    fn matches(self, identity: &SelectedExactIdentity) -> bool {
        matches!(
            (self, identity),
            (Self::RecordValueType, SelectedExactIdentity::Record(_))
                | (Self::ObjectType, SelectedExactIdentity::Object(_))
                | (Self::Union, SelectedExactIdentity::Union(_))
                | (
                    Self::DeclaredUnit,
                    SelectedExactIdentity::DeclaredUnit(UnitId::Declared(_))
                )
        )
    }

    fn matches_admitted(self, form: DeclarationForm) -> bool {
        matches!(
            (self, form),
            (Self::RecordValueType, DeclarationForm::RecordValueType)
                | (Self::ObjectType, DeclarationForm::ObjectType)
        )
    }
}

/// One QSL-stage declaration row, keyed by selected original IR identity.
///
/// IR validates this row's structural join. The opaque Exact identity's
/// provenance remains a producer fact; constructing a row in another caller
/// does not make that identity authoritative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedExactDeclarationRow {
    /// The full structural node identity in the admitted package.
    pub structural_node: CheckedNodeId,
    /// The selected domain document identity.
    pub selected_document: Box<str>,
    /// The selected declaration form.
    pub form: SelectedExactDeclarationForm,
    /// The original IR declaration identity in that document.
    pub original_identity: Box<str>,
    /// The identity issued by the QSL checking/model stage.
    pub exact_identity: SelectedExactIdentity,
}

impl SelectedExactDeclarationRow {
    /// Carry a complete producer row; no bare Exact identity is accepted.
    pub fn new(
        structural_node: CheckedNodeId,
        selected_document: impl Into<Box<str>>,
        form: SelectedExactDeclarationForm,
        original_identity: impl Into<Box<str>>,
        exact_identity: SelectedExactIdentity,
    ) -> Self {
        Self {
            structural_node,
            selected_document: selected_document.into(),
            form,
            original_identity: original_identity.into(),
            exact_identity,
        }
    }
}

/// One QSL-stage record field row, keyed by its selected original IR identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedExactFieldRow {
    /// The selected record's structural node identity.
    pub record_node: CheckedNodeId,
    /// The selected domain document identity.
    pub selected_document: Box<str>,
    /// The original IR record declaration identity.
    pub record_original_identity: Box<str>,
    /// The selected IR field identity, including inherited fields.
    pub field_original_identity: Box<str>,
    /// The field identity issued by the QSL checking/model stage.
    pub member_id: MemberId,
}

impl SelectedExactFieldRow {
    /// Carry a complete producer field row; no bare member ID is accepted.
    pub fn new(
        record_node: CheckedNodeId,
        selected_document: impl Into<Box<str>>,
        record_original_identity: impl Into<Box<str>>,
        field_original_identity: impl Into<Box<str>>,
        member_id: MemberId,
    ) -> Self {
        Self {
            record_node,
            selected_document: selected_document.into(),
            record_original_identity: record_original_identity.into(),
            field_original_identity: field_original_identity.into(),
            member_id,
        }
    }
}

/// Selected-model identities supplied by the checking stage alongside a package.
///
/// An empty selection supports scalar reads. Identity-bearing reads refuse
/// until the selected-model stage supplies a complete identity witness.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectedExactIdentities {
    declarations: BTreeMap<CheckedNodeId, SelectedExactDeclarationRow>,
    fields: Vec<SelectedExactFieldRow>,
}

impl SelectedExactIdentities {
    /// An empty selection for reads of types without declaration identities.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Validate the stage's selected rows against one admitted package.
    ///
    /// This checks structural, selected-document, form and original-identity
    /// joins. IR cannot authenticate the opaque Exact payload. Production
    /// callers obtain these rows from the QSL checking/model identity stage;
    /// caller-created rows alone grant no authority.
    pub fn from_stage_rows(
        package: &CheckedPackageV2,
        declarations: impl IntoIterator<Item = SelectedExactDeclarationRow>,
        fields: impl IntoIterator<Item = SelectedExactFieldRow>,
    ) -> Result<Self, ExactTypeProjectionError> {
        let mut selected = Self::empty();
        let mut owners = BTreeSet::new();
        let mut node_identities = BTreeSet::new();
        let mut object_identities = BTreeSet::new();
        let mut unit_identities = BTreeSet::new();
        for row in declarations {
            let node = &row.structural_node;
            let Some((document, original, form)) = package.models.selected_declaration(node) else {
                return Err(selected_declaration_error(package, node));
            };
            if document != row.selected_document.as_ref()
                || original != row.original_identity.as_ref()
                || !row.form.matches_admitted(form)
                || !row.form.matches(&row.exact_identity)
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            let unique_exact = match &row.exact_identity {
                SelectedExactIdentity::Record(key) | SelectedExactIdentity::Union(key) => {
                    node_identities.insert(key.clone())
                }
                SelectedExactIdentity::Object(effective) => {
                    object_identities.insert(effective.clone())
                }
                SelectedExactIdentity::DeclaredUnit(UnitId::Declared(key)) => {
                    unit_identities.insert(key.clone())
                }
                SelectedExactIdentity::DeclaredUnit(UnitId::Compound(_)) => false,
            };
            if !unique_exact
                || !owners.insert((
                    row.selected_document.clone(),
                    row.form,
                    row.original_identity.clone(),
                ))
                || selected.declarations.contains_key(node)
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
            selected.declarations.insert(node.clone(), row);
        }
        let mut seen_fields = BTreeSet::new();
        for row in fields {
            let node = &row.record_node;
            let Some((document, original, form)) = package.models.selected_declaration(node) else {
                return Err(selected_declaration_error(package, node));
            };
            if document != row.selected_document.as_ref()
                || original != row.record_original_identity.as_ref()
                || form != DeclarationForm::RecordValueType
                || !package
                    .models
                    .selected_record_field(node, &row.field_original_identity)
            {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            let Some(declaration) = selected.declarations.get(node) else {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                ));
            };
            let SelectedExactIdentity::Record(record_key) = &declaration.exact_identity else {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            };
            if row.member_id.owner() != &MemberOwner::Node(record_key.clone()) {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                ));
            }
            if !seen_fields.insert((node.clone(), row.field_original_identity.clone())) {
                return Err(witness_error(
                    node,
                    ExactTypeProjectionErrorCode::ConflictingIdentityWitness,
                ));
            }
            selected.fields.push(row);
        }
        Ok(selected)
    }

    /// The validated declaration rows, keyed by structural node identity.
    pub fn declaration_rows(&self) -> impl Iterator<Item = &SelectedExactDeclarationRow> {
        self.declarations.values()
    }

    /// The validated effective-field rows in producer order.
    pub fn field_rows(&self) -> &[SelectedExactFieldRow] {
        &self.fields
    }

    fn declaration_identity(
        &self,
        package: &CheckedPackageV2,
        key: &CheckedNodeId,
    ) -> Result<&SelectedExactIdentity, ExactTypeProjectionErrorCode> {
        let (document, original, form) = package
            .models
            .selected_declaration(key)
            .ok_or(ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration)?;
        let row = self
            .declarations
            .get(key)
            .ok_or(ExactTypeProjectionErrorCode::UnavailableIdentityWitness)?;
        if document != row.selected_document.as_ref()
            || original != row.original_identity.as_ref()
            || !row.form.matches_admitted(form)
            || !row.form.matches(&row.exact_identity)
        {
            return Err(ExactTypeProjectionErrorCode::MismatchedIdentityWitness);
        }
        Ok(&row.exact_identity)
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
    /// A selected declaration row disagrees with admission-retained facts.
    MismatchedIdentityWitness,
    /// Competing rows claim one selected declaration or structural node.
    ConflictingIdentityWitness,
    /// The selected declaration is not resolved in the admitted package.
    UnresolvedSelectedDeclaration,
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
            Self::MismatchedIdentityWitness => "mismatched_identity_witness",
            Self::ConflictingIdentityWitness => "conflicting_identity_witness",
            Self::UnresolvedSelectedDeclaration => "unresolved_selected_declaration",
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
            Self::MismatchedIdentityWitness,
            Self::ConflictingIdentityWitness,
            Self::UnresolvedSelectedDeclaration,
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

fn witness_error(
    node: &CheckedNodeId,
    code: ExactTypeProjectionErrorCode,
) -> ExactTypeProjectionError {
    ExactTypeProjectionError {
        code,
        requested: Box::new(node.clone()),
        at: Some(Box::new(node.clone())),
        form: "selected_identity".into(),
        work: 0,
    }
}

fn selected_declaration_error(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
) -> ExactTypeProjectionError {
    let code = if package.node(node).is_some() {
        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
    } else if node.package == package.wire.name {
        ExactTypeProjectionErrorCode::UnknownNode
    } else {
        ExactTypeProjectionErrorCode::ForeignNode
    };
    witness_error(node, code)
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
        identities: &SelectedExactIdentities,
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
        let refuse = |code, charged| ExactTypeProjectionError {
            code,
            requested: Box::new(requested.clone()),
            at: Some(Box::new(type_id.clone())),
            form: format!("{}/{}", type_node.node_tag, type_node.semantic_form).into(),
            work: charged,
        };
        let value_type = match kind {
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean) => ValueType::Boolean,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer) => ValueType::Integer,
            CheckedNodeKind::ScalarType(ScalarTypeForm::String) => ValueType::String,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Uuid) => ValueType::Uuid,
            CheckedNodeKind::ScalarType(ScalarTypeForm::Timestamp) => ValueType::Timestamp,
            CheckedNodeKind::BoundedDomain(BoundedDomainForm::IntegerRange) => {
                let (lower, upper) = range_bounds.ok_or_else(|| {
                    refuse(ExactTypeProjectionErrorCode::UnsupportedExactType, work)
                })?;
                let lower = lower.parse::<Integer>().map_err(|_| {
                    refuse(ExactTypeProjectionErrorCode::UnsupportedExactType, work)
                })?;
                let upper = upper.parse::<Integer>().map_err(|_| {
                    refuse(ExactTypeProjectionErrorCode::UnsupportedExactType, work)
                })?;
                let interval = IntegerInterval::new(lower, upper).map_err(|_| {
                    refuse(ExactTypeProjectionErrorCode::UnsupportedExactType, work)
                })?;
                ValueType::Int(interval)
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Reference) => {
                let target = reference_type_target_typed(type_node).ok_or_else(|| {
                    refuse(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        work,
                    )
                })?;
                charge_work(
                    requested,
                    &target,
                    "reference_target",
                    &mut work,
                    work_limit,
                )?;
                let refuse_target = |code, charged| ExactTypeProjectionError {
                    code,
                    requested: Box::new(requested.clone()),
                    at: Some(Box::new(target.clone())),
                    form: self.node(&target).map_or_else(
                        || Box::<str>::from("unresolved_selected_declaration"),
                        |node| format!("{}/{}", node.node_tag, node.semantic_form).into(),
                    ),
                    work: charged,
                };
                if self.node_kind(&target) != Some(CheckedNodeKind::Model(ModelForm::ObjectType)) {
                    return Err(refuse_target(
                        ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration,
                        work,
                    ));
                }
                match identities
                    .declaration_identity(self, &target)
                    .map_err(|code| refuse_target(code, work))?
                {
                    SelectedExactIdentity::Object(effective) => {
                        ValueType::Reference(effective.clone())
                    }
                    _ => {
                        return Err(refuse_target(
                            ExactTypeProjectionErrorCode::MismatchedIdentityWitness,
                            work,
                        ))
                    }
                }
            }
            CheckedNodeKind::Model(ModelForm::RecordValueType) => {
                identities
                    .declaration_identity(self, type_id)
                    .map_err(|code| refuse(code, work))?;
                return Err(refuse(
                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                    work,
                ));
            }
            CheckedNodeKind::CompositeType(CompositeTypeForm::Union) => {
                return Err(refuse(
                    ExactTypeProjectionErrorCode::UnavailableIdentityWitness,
                    work,
                ));
            }
            _ => {
                return Err(refuse(
                    ExactTypeProjectionErrorCode::UnsupportedExactType,
                    work,
                ))
            }
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
            ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
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
