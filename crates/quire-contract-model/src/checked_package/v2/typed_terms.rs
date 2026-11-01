//! Typed admitted V2 body vocabulary.
//!
//! The wire reader validates the flat QSpec grammar and each body before
//! decoding this representation for admitted graph joins. The flat grammar
//! has bounded term depth, so serde's recursion here is independent of input
//! byte length.

use super::{
    ApplicationOperator, BodyTerm, LawRole, LiteralKind, OperationConstraintKind,
    OperationMemberKind, OperationModeKind, StateClauseKind,
};
use super::{
    RefinementAssume, RefinementEnsure, RefinementHistory, RefinementKey, RefinementObject,
    RefinementPopulation, RefinementStep, RefinementStepArgument, RefinementStepTarget,
};
use crate::checked_package::shared::{CheckedArtifactRef, CheckedNodeId, CheckedNodePathSegment};
use crate::identity::ExecutionPoint;
use num_bigint::BigInt;
use quire_canonical::{Encode, Error, FixedShape, Sink, Writer};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{error::Category, Value};

/// An application operation whose state-clause point has a typed carrier.
/// Equality compares the represented record and point; admission views are
/// private provenance and do not affect represented-body equality.
#[derive(Clone, Debug)]
pub struct CheckedApplicationOperation {
    record: Value,
    execution_point: Option<ExecutionPoint>,
    views: Option<CheckedOperationViews>,
    #[cfg(feature = "test-support")]
    test_fixture: bool,
}

impl PartialEq for CheckedApplicationOperation {
    fn eq(&self, other: &Self) -> bool {
        self.record == other.record && self.execution_point == other.execution_point
    }
}

impl Eq for CheckedApplicationOperation {}

/// An operation view was requested without reader admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedOperationViewError {
    /// The operation was decoded directly or read from unauthenticated V1 bytes.
    #[error("operation views are not admitted")]
    NotAdmitted,
}

/// Typed, validated input for a test-only application operation.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckedApplicationOperationFixture {
    /// Optional operation identity, including intentional absence.
    pub identity: Option<Box<str>>,
    /// Optional closed mode kind and its value.
    pub mode: Option<(OperationModeKind, Box<str>)>,
    /// Ordered definitions with the fixed `integer_division` law role.
    pub law_definitions: Vec<CheckedArtifactRef>,
    /// Optional member declaration key.
    pub member_declaration: Option<CheckedNodeId>,
}

/// A malformed typed test operation input.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CheckedApplicationOperationFixtureError {
    /// A present operation identity was empty.
    #[error("operation identity is empty")]
    EmptyIdentity,
    /// A present mode value was empty.
    #[error("operation mode value is empty")]
    EmptyModeValue,
    /// A law definition had an empty authority.
    #[error("law definition authority is empty")]
    EmptyLawAuthority,
    /// A law definition had an empty identity.
    #[error("law definition identity is empty")]
    EmptyLawIdentity,
    /// The member declaration has an empty package or path, or an ordinal
    /// outside the exact JSON integer range.
    #[error("member declaration node id is invalid")]
    InvalidMemberNodeId,
}

/// The admitted kind and value of an operation's mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedOperationModeView<'a> {
    /// The catalogued mode kind.
    pub kind: OperationModeKind,
    /// The admitted mode value.
    pub value: &'a str,
}

/// One selected operation law, in wire order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationLaw {
    /// Closed law role.
    pub role: LawRole,
    /// Selected definition.
    pub definition: CheckedArtifactRef,
}

/// One catalogued cross-operand constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationConstraint {
    /// Closed constraint choice.
    pub kind: OperationConstraintKind,
    /// Operand positions, in catalog order.
    pub operands: Vec<u64>,
    /// Required family or group when the constraint names one.
    pub family: Option<Box<str>>,
}

/// The closed shape of an admitted operation member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedOperationMember {
    /// A named field of a declaration.
    Field {
        /// Declaring type node.
        declaration: CheckedNodeId,
        /// Admitted field name.
        name: Box<str>,
        /// Source read form for a model-object projection; absent for record fields.
        read_form: Option<CheckedFieldReadForm>,
    },
    /// A declared tuple position.
    Position {
        /// Declaring tuple node.
        declaration: CheckedNodeId,
        /// Exact position, including beyond the JSON safe integer range.
        position: BigInt,
    },
    /// An element type of a collection.
    Element {
        /// Declaring collection node.
        declaration: CheckedNodeId,
    },
    /// A named relationship end.
    RelationshipEnd {
        /// Declaring relationship node.
        declaration: CheckedNodeId,
        /// Named end role.
        name: Box<str>,
        /// Producer supplied destination type node.
        destination_type: CheckedNodeId,
        /// Population node, when the operation links one.
        population: Option<CheckedNodeId>,
    },
    /// A named dispatched operation.
    Operation {
        /// Declaring model node.
        declaration: CheckedNodeId,
        /// Admitted operation name.
        name: Box<str>,
    },
    /// An explicit type argument.
    TypeArgument {
        /// Type argument declaration node.
        declaration: CheckedNodeId,
    },
    /// An operator selected by a profile.
    ProfileOperator {
        /// Explicit operator, when present in the admitted record.
        operator: Option<Box<str>>,
    },
    /// A state clause.
    StateClause {
        /// Closed clause kind.
        clause: StateClauseKind,
    },
    /// An optional temporal interval.
    TemporalInterval {
        /// Interval, or an unbounded interval when absent.
        interval: Option<CheckedOperationInterval>,
    },
    /// A fairness clause.
    Fairness {
        /// Weak or strong fairness.
        fairness_kind: CheckedFairnessKind,
        /// Whole or per participant granularity.
        granularity: CheckedFairnessGranularity,
        /// Exact producer-selected fairness subject.
        target: CheckedFairnessTarget,
    },
}

/// A closed fairness subject carrying its admitted structural identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedFairnessTarget {
    /// An effective model operation.
    Operation {
        /// Declaring model node.
        declaration: CheckedNodeId,
        /// Original operation name.
        name: Box<str>,
    },
    /// The owning protocol clause.
    ProtocolRoot {
        /// Protocol clause node.
        node: CheckedNodeId,
    },
    /// An attempt control in the owning protocol.
    AttemptNode {
        /// Attempt control node.
        node: CheckedNodeId,
    },
    /// One named branch of a parallel control.
    ParallelBranch {
        /// Exact parallel control node.
        parallel: CheckedNodeId,
        /// Authored branch label.
        branch: Box<str>,
    },
    /// An owning protocol's compensation template.
    CompensationTemplate {
        /// Compensation node.
        node: CheckedNodeId,
    },
    /// A transition of a hybrid model or mode.
    HybridTransition {
        /// Declaring hybrid model or mode.
        declaration: CheckedNodeId,
        /// Original transition name.
        name: Box<str>,
    },
}

impl CheckedFairnessTarget {
    pub(super) fn node_mut(&mut self) -> &mut CheckedNodeId {
        match self {
            Self::Operation { declaration, .. } | Self::HybridTransition { declaration, .. } => {
                declaration
            }
            Self::ProtocolRoot { node }
            | Self::AttemptNode { node }
            | Self::CompensationTemplate { node } => node,
            Self::ParallelBranch { parallel, .. } => parallel,
        }
    }
}

/// Source syntax retained for a model-object field read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedFieldReadForm {
    /// A direct source field read, such as `view.total`.
    Bare,
    /// A source field read with explicit dereference, such as `deref(view).total`.
    ExplicitDeref,
}

impl CheckedFieldReadForm {
    #[ix_trace_rs::string_edge]
    pub(super) fn from_wire(value: &str) -> Option<Self> {
        match value {
            "bare" => Some(Self::Bare),
            "explicit_deref" => Some(Self::ExplicitDeref),
            _ => None,
        }
    }
}

/// A closed temporal interval shape.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CheckedOperationInterval {
    /// Integer positions, including an unbounded upper end.
    Integer {
        /// Inclusive lower position.
        #[serde(deserialize_with = "decode_decimal_bigint")]
        lower: BigInt,
        /// Inclusive upper position, or no upper limit.
        #[serde(deserialize_with = "decode_optional_decimal_bigint")]
        upper: Option<BigInt>,
    },
    /// Time magnitudes with explicit open or closed ends.
    Timed {
        /// Lower time magnitude.
        lower: CheckedOperationRational,
        /// Upper time magnitude, when bounded.
        upper: Option<CheckedOperationRational>,
        /// Lower endpoint inclusion.
        lower_end: CheckedIntervalEnd,
        /// Upper endpoint inclusion.
        upper_end: CheckedIntervalEnd,
    },
}

/// An exact nonnegative rational interval bound.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedOperationRational {
    /// Decimal numerator.
    #[serde(deserialize_with = "decode_decimal_bigint")]
    pub numerator: BigInt,
    /// Decimal denominator.
    #[serde(deserialize_with = "decode_decimal_bigint")]
    pub denominator: BigInt,
}

/// Whether an interval includes its endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedIntervalEnd {
    /// Includes the endpoint.
    Closed,
    /// Excludes the endpoint.
    Open,
}

/// The strength of a fairness clause.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedFairnessKind {
    /// Weak fairness.
    Weak,
    /// Strong fairness.
    Strong,
}

/// Whether fairness applies to the whole or each participant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckedFairnessGranularity {
    /// Whole transition.
    Whole,
    /// Each participant.
    Each,
}

/// A leaf's selected mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationMode {
    /// Closed mode kind.
    pub kind: OperationModeKind,
    /// Catalogued mode value.
    pub value: Box<str>,
}

/// One structural leaf, in the checked operation's order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationLeaf {
    /// Structural path segments.
    pub path: Vec<Box<str>>,
    /// Selected laws in order.
    pub laws: Vec<CheckedOperationLaw>,
    /// Mode selected for this leaf.
    pub mode: Option<CheckedOperationMode>,
}

/// Complete owned admission data retained behind the borrowed read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CheckedOperationRecord {
    pub(super) identity: Box<str>,
    pub(super) mode: Option<CheckedOperationMode>,
    pub(super) laws: Vec<CheckedOperationLaw>,
    pub(super) member: Option<CheckedOperationMember>,
    pub(super) constraints: Vec<CheckedOperationConstraint>,
    pub(super) leaves: Vec<CheckedOperationLeaf>,
}

/// One borrowed read of a reader-admitted operation and its catalog rules.
#[derive(Clone, Copy, Debug)]
pub struct CheckedOperationRead<'a> {
    /// Catalogued identity.
    pub identity: &'a str,
    /// Admitted mode, when present.
    pub mode: Option<CheckedOperationModeView<'a>>,
    /// Ordered selected laws.
    pub laws: &'a [CheckedOperationLaw],
    /// Closed typed member.
    pub member: Option<&'a CheckedOperationMember>,
    /// State-clause execution point, when the member carries one.
    pub execution_point: Option<&'a ExecutionPoint>,
    /// Catalogued constraints in order.
    pub constraints: &'a [CheckedOperationConstraint],
    /// Ordered structural leaves.
    pub leaves: &'a [CheckedOperationLeaf],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct CheckedOperationViews {
    pub(super) record: Option<Box<CheckedOperationRecord>>,
    pub(super) state_clause_kind: Option<StateClauseKind>,
    pub(super) projection_field: Option<(CheckedNodeId, Box<str>)>,
    pub(super) member_declaration: Option<CheckedNodeId>,
    pub(super) mode: Option<(OperationModeKind, Box<str>)>,
    pub(super) law_definitions: Vec<CheckedArtifactRef>,
}

impl CheckedOperationViews {
    fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        if let Some(record) = &mut self.record {
            if let Some(member) = &mut record.member {
                member.replace_node_id(old, new);
            }
        }
        if let Some((declaration, _)) = &mut self.projection_field {
            if declaration == old {
                *declaration = new.clone();
            }
        }
        if let Some(declaration) = &mut self.member_declaration {
            if declaration == old {
                *declaration = new.clone();
            }
        }
    }
}

impl CheckedOperationMember {
    pub(super) fn map_node_ids(
        &mut self,
        mut map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>,
    ) -> Option<()> {
        let mut replace = |id: &mut CheckedNodeId| {
            *id = map(id)?;
            Some(())
        };
        match self {
            Self::Field { declaration, .. }
            | Self::Position { declaration, .. }
            | Self::Element { declaration }
            | Self::Operation { declaration, .. }
            | Self::TypeArgument { declaration } => replace(declaration)?,
            Self::Fairness { target, .. } => replace(target.node_mut())?,
            Self::RelationshipEnd {
                declaration,
                destination_type,
                population,
                ..
            } => {
                replace(declaration)?;
                replace(destination_type)?;
                if let Some(population) = population {
                    replace(population)?;
                }
            }
            Self::ProfileOperator { .. }
            | Self::StateClause { .. }
            | Self::TemporalInterval { .. } => {}
        }
        Some(())
    }

    pub(super) fn from_validated_value(value: &Value) -> Option<Self> {
        let kind = OperationMemberKind::from_wire(value.get("kind")?.as_str()?)?;
        let declaration = || CheckedNodeId::from_value(value.get("declaration")?);
        let name = || Some(Box::<str>::from(value.get("name")?.as_str()?));
        let node = |key| CheckedNodeId::from_value(value.get(key)?);
        Some(match kind {
            OperationMemberKind::Field => Self::Field {
                declaration: declaration()?,
                name: name()?,
                read_form: match value.get("read_form") {
                    Some(value) => Some(CheckedFieldReadForm::from_wire(value.as_str()?)?),
                    None => None,
                },
            },
            OperationMemberKind::Position => {
                let position = value.get("position")?;
                let text = match position {
                    Value::Number(number) => number.to_string(),
                    Value::String(text) => text.clone(),
                    _ => return None,
                };
                Self::Position {
                    declaration: declaration()?,
                    position: BigInt::parse_bytes(text.as_bytes(), 10)?,
                }
            }
            OperationMemberKind::Element => Self::Element {
                declaration: declaration()?,
            },
            OperationMemberKind::RelationshipEnd => Self::RelationshipEnd {
                declaration: declaration()?,
                name: name()?,
                destination_type: node("destination_type")?,
                population: match value.get("population") {
                    Some(_) => Some(node("population")?),
                    None => None,
                },
            },
            OperationMemberKind::Operation => Self::Operation {
                declaration: declaration()?,
                name: name()?,
            },
            OperationMemberKind::TypeArgument => Self::TypeArgument {
                declaration: declaration()?,
            },
            OperationMemberKind::ProfileOperator => Self::ProfileOperator {
                operator: value
                    .get("operator")
                    .and_then(Value::as_str)
                    .map(Into::into),
            },
            OperationMemberKind::StateClause => Self::StateClause {
                clause: StateClauseKind::from_wire(value.get("clause")?.as_str()?)?,
            },
            OperationMemberKind::TemporalInterval => Self::TemporalInterval {
                interval: match value.get("interval")? {
                    Value::Null => None,
                    interval => Some(serde_json::from_value(interval.clone()).ok()?),
                },
            },
            OperationMemberKind::Fairness => Self::Fairness {
                fairness_kind: serde_json::from_value(value.get("fairness_kind")?.clone()).ok()?,
                granularity: serde_json::from_value(value.get("granularity")?.clone()).ok()?,
                target: serde_json::from_value(value.get("target")?.clone()).ok()?,
            },
        })
    }

    fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        let replace = |id: &mut CheckedNodeId| {
            if id == old {
                *id = new.clone();
            }
        };
        match self {
            Self::Field { declaration, .. }
            | Self::Position { declaration, .. }
            | Self::Element { declaration }
            | Self::Operation { declaration, .. }
            | Self::TypeArgument { declaration } => replace(declaration),
            Self::Fairness { target, .. } => replace(target.node_mut()),
            Self::RelationshipEnd {
                declaration,
                destination_type,
                population,
                ..
            } => {
                replace(declaration);
                replace(destination_type);
                if let Some(population) = population {
                    replace(population);
                }
            }
            Self::ProfileOperator { .. }
            | Self::StateClause { .. }
            | Self::TemporalInterval { .. } => {}
        }
    }
}

fn decode_decimal_bigint<'de, D: Deserializer<'de>>(deserializer: D) -> Result<BigInt, D::Error> {
    let text = Box::<str>::deserialize(deserializer)?;
    BigInt::parse_bytes(text.as_bytes(), 10)
        .ok_or_else(|| serde::de::Error::custom("expected a decimal integer"))
}

fn decode_optional_decimal_bigint<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<BigInt>, D::Error> {
    let text = Option::<Box<str>>::deserialize(deserializer)?;
    text.map(|text| {
        BigInt::parse_bytes(text.as_bytes(), 10)
            .ok_or_else(|| serde::de::Error::custom("expected a decimal integer"))
    })
    .transpose()
}

impl CheckedApplicationOperation {
    /// Construct the catalogued state-clause operation without a JSON payload.
    pub fn state_clause(clause: StateClauseKind, execution_point: Option<ExecutionPoint>) -> Self {
        Self {
            record: serde_json::json!({
                "identity": "quire.op.state.clause", "laws": [], "mode": null,
                "member": {"kind": "state_clause", "clause": clause}, "leaves": []
            }),
            execution_point,
            views: Some(CheckedOperationViews {
                state_clause_kind: Some(clause),
                ..CheckedOperationViews::default()
            }),
            #[cfg(feature = "test-support")]
            test_fixture: false,
        }
    }

    /// Construct a test operation without admitting a package or accepting a JSON view.
    #[cfg(feature = "test-support")]
    #[ix_trace_rs::string_edge]
    pub fn from_test_fixture(
        fixture: CheckedApplicationOperationFixture,
    ) -> Result<Self, CheckedApplicationOperationFixtureError> {
        use CheckedApplicationOperationFixtureError as FixtureError;

        if fixture.identity.as_deref() == Some("") {
            return Err(FixtureError::EmptyIdentity);
        }
        if fixture
            .mode
            .as_ref()
            .is_some_and(|(_, value)| value.is_empty())
        {
            return Err(FixtureError::EmptyModeValue);
        }
        for definition in &fixture.law_definitions {
            if definition.authority.is_empty() {
                return Err(FixtureError::EmptyLawAuthority);
            }
            if definition.identity.is_empty() {
                return Err(FixtureError::EmptyLawIdentity);
            }
        }
        if fixture
            .member_declaration
            .as_ref()
            .is_some_and(|key| !key.is_well_formed())
        {
            return Err(FixtureError::InvalidMemberNodeId);
        }

        let mut record = serde_json::Map::new();
        if let Some(identity) = &fixture.identity {
            record.insert("identity".into(), Value::String(identity.to_string()));
        }
        record.insert(
            "laws".into(),
            Value::Array(
                fixture
                    .law_definitions
                    .iter()
                    .map(|definition| {
                        serde_json::json!({
                            "role": "integer_division", "definition": definition
                        })
                    })
                    .collect(),
            ),
        );
        record.insert(
            "mode".into(),
            fixture.mode.as_ref().map_or(
                Value::Null,
                |(kind, value)| serde_json::json!({"kind": kind, "value": value}),
            ),
        );
        record.insert(
            "member".into(),
            fixture.member_declaration.as_ref().map_or(Value::Null, |declaration| {
                serde_json::json!({"kind": "type_argument", "declaration": declaration})
            }),
        );
        record.insert("leaves".into(), Value::Array(Vec::new()));

        Ok(Self {
            record: Value::Object(record),
            execution_point: None,
            views: Some(CheckedOperationViews {
                member_declaration: fixture.member_declaration,
                mode: fixture.mode,
                law_definitions: fixture.law_definitions,
                ..CheckedOperationViews::default()
            }),
            test_fixture: true,
        })
    }

    /// Whether this value came from the feature-gated fixture constructor.
    #[cfg(feature = "test-support")]
    pub fn is_test_fixture(&self) -> bool {
        self.test_fixture
    }

    /// The retained point of a state-clause application, if present.
    pub fn execution_point(&self) -> Option<&ExecutionPoint> {
        self.execution_point.as_ref()
    }

    /// The catalogued identity of this application operation.
    pub fn identity(&self) -> Option<&str> {
        self.record.get("identity").and_then(Value::as_str)
    }

    /// The admitted state-clause kind, or `None` for another member shape.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn state_clause_kind(&self) -> Result<Option<StateClauseKind>, CheckedOperationViewError> {
        Ok(self.admitted_views()?.state_clause_kind)
    }

    /// The admitted field declaration and name, or `None` for another member shape.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn projection_field(
        &self,
    ) -> Result<Option<(&CheckedNodeId, &str)>, CheckedOperationViewError> {
        Ok(self
            .admitted_views()?
            .projection_field
            .as_ref()
            .map(|(declaration, name)| (declaration, name.as_ref())))
    }

    /// The admitted member declaration, when the member carries one.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn member_declaration(&self) -> Result<Option<&CheckedNodeId>, CheckedOperationViewError> {
        Ok(self.admitted_views()?.member_declaration.as_ref())
    }

    /// The admitted operation mode kind and value, when present.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn mode(&self) -> Result<Option<CheckedOperationModeView<'_>>, CheckedOperationViewError> {
        Ok(self
            .admitted_views()?
            .mode
            .as_ref()
            .map(|(kind, value)| CheckedOperationModeView { kind: *kind, value }))
    }

    /// The admitted law definitions in operation order.
    /// Returns `NotAdmitted` when validated views were not installed.
    pub fn law_definitions(&self) -> Result<&[CheckedArtifactRef], CheckedOperationViewError> {
        Ok(&self.admitted_views()?.law_definitions)
    }

    /// Borrow the complete reader-admitted operation and catalog decisions.
    /// Direct decoding and test fixtures return `NotAdmitted`.
    pub fn read(&self) -> Result<CheckedOperationRead<'_>, CheckedOperationViewError> {
        let record = self
            .admitted_views()?
            .record
            .as_ref()
            .ok_or(CheckedOperationViewError::NotAdmitted)?;
        Ok(CheckedOperationRead {
            identity: &record.identity,
            mode: record.mode.as_ref().map(|mode| CheckedOperationModeView {
                kind: mode.kind,
                value: &mode.value,
            }),
            laws: &record.laws,
            member: record.member.as_ref(),
            execution_point: self.execution_point.as_ref(),
            constraints: &record.constraints,
            leaves: &record.leaves,
        })
    }

    pub(super) fn admitted_views(
        &self,
    ) -> Result<&CheckedOperationViews, CheckedOperationViewError> {
        self.views
            .as_ref()
            .ok_or(CheckedOperationViewError::NotAdmitted)
    }

    pub(super) fn install_admitted_views(&mut self, views: CheckedOperationViews) {
        self.views = Some(views);
    }

    pub(super) fn record(&self) -> &Value {
        &self.record
    }

    pub(super) fn get(&self, key: &str) -> Option<&Value> {
        self.record.get(key)
    }

    pub(super) fn pointer(&self, pointer: &str) -> Option<&Value> {
        self.record.pointer(pointer)
    }

    #[cfg(test)]
    pub(super) fn record_mut(&mut self) -> &mut Value {
        &mut self.record
    }

    fn from_record(mut record: Value) -> Self {
        let execution_point =
            if record.get("identity").and_then(Value::as_str) == Some("quire.op.state.clause") {
                let member = record.get_mut("member").and_then(Value::as_object_mut);
                member.and_then(|member| {
                    let point = ExecutionPoint::from_wire(member.get("execution_point")?)?;
                    member.remove("execution_point");
                    Some(point)
                })
            } else {
                None
            };
        Self {
            record,
            execution_point,
            views: None,
            #[cfg(feature = "test-support")]
            test_fixture: false,
        }
    }

    fn with_point(&self) -> Value {
        let mut record = self.record.clone();
        if let (Some(point), Some(member)) = (
            &self.execution_point,
            record.get_mut("member").and_then(Value::as_object_mut),
        ) {
            member.insert(
                "execution_point".into(),
                match point {
                    ExecutionPoint::Initialization { name } => {
                        serde_json::json!({"kind": "initialization", "name": name})
                    }
                    ExecutionPoint::Handler { name } => {
                        serde_json::json!({"kind": "handler", "name": name})
                    }
                    ExecutionPoint::Pre { operation } => {
                        serde_json::json!({"kind": "pre", "operation": operation})
                    }
                    ExecutionPoint::Post { operation } => {
                        serde_json::json!({"kind": "post", "operation": operation})
                    }
                },
            );
        }
        record
    }
}

impl Serialize for CheckedApplicationOperation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.execution_point.is_some() {
            self.with_point().serialize(serializer)
        } else {
            self.record.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for CheckedApplicationOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_record(Value::deserialize(deserializer)?))
    }
}

impl Encode for CheckedApplicationOperation {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        if self.execution_point.is_some() {
            self.with_point().encode_into(writer)
        } else {
            self.record.encode_into(writer)
        }
    }
}

/// An admitted operation anchor body with three expression bindings and an
/// optional bare point binding. The point itself is not an expression term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedOperationAnchorBody {
    /// Context, operation and frame bindings, in wire order.
    pub members: [CheckedTerm; 3],
    /// The optional closed fourth binding's bare value.
    pub execution_point: Option<ExecutionPoint>,
}

impl Serialize for CheckedOperationAnchorBody {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Members<'a>(&'a CheckedOperationAnchorBody);
        impl Serialize for Members<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                #[derive(Serialize)]
                struct PointBinding<'a> {
                    term: &'static str,
                    name: &'static str,
                    value: &'a ExecutionPoint,
                }
                let mut sequence = serializer.serialize_seq(Some(
                    self.0.members.len() + usize::from(self.0.execution_point.is_some()),
                ))?;
                for member in &self.0.members {
                    sequence.serialize_element(member)?;
                }
                if let Some(point) = &self.0.execution_point {
                    sequence.serialize_element(&PointBinding {
                        term: "binding",
                        name: "execution_point",
                        value: point,
                    })?;
                }
                sequence.end()
            }
        }
        let mut object = serializer.serialize_map(Some(2))?;
        object.serialize_entry("term", "aggregate")?;
        object.serialize_entry("members", &Members(self))?;
        object.end()
    }
}

/// An admitted binding name with the closed contextual names decoded once.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedBindingName {
    raw: Box<str>,
    kind: super::TermBindingName,
}

/// The contextual meaning of an admitted binding name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedBindingNameKind<'a> {
    /// The `parameters` binding.
    Parameters,
    /// The `body` binding.
    Body,
    /// The `optional` binding.
    Optional,
    /// The `clause` binding.
    Clause,
    /// Any other name, retained verbatim.
    Other(&'a str),
}

impl CheckedBindingName {
    pub(super) fn internal_kind(&self) -> super::TermBindingName {
        self.kind
    }

    /// Read the contextual name without losing an unrecognized exact name.
    pub fn kind(&self) -> CheckedBindingNameKind<'_> {
        match self.kind {
            super::TermBindingName::Parameters => CheckedBindingNameKind::Parameters,
            super::TermBindingName::Body => CheckedBindingNameKind::Body,
            super::TermBindingName::Optional => CheckedBindingNameKind::Optional,
            super::TermBindingName::Clause => CheckedBindingNameKind::Clause,
            super::TermBindingName::Other => CheckedBindingNameKind::Other(&self.raw),
        }
    }
}

impl From<Box<str>> for CheckedBindingName {
    fn from(raw: Box<str>) -> Self {
        let kind = super::TermBindingName::from_wire(&raw);
        Self { raw, kind }
    }
}

impl From<&str> for CheckedBindingName {
    fn from(raw: &str) -> Self {
        Box::<str>::from(raw).into()
    }
}

impl From<String> for CheckedBindingName {
    fn from(raw: String) -> Self {
        raw.into_boxed_str().into()
    }
}

impl AsRef<str> for CheckedBindingName {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

impl std::fmt::Display for CheckedBindingName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.raw.fmt(formatter)
    }
}

impl std::ops::Deref for CheckedBindingName {
    type Target = str;
    fn deref(&self) -> &str {
        &self.raw
    }
}

impl Serialize for CheckedBindingName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CheckedBindingName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Box::<str>::deserialize(deserializer).map(Self::from)
    }
}

/// A literal's decoded value. Exact numbers are retained without floating point conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedLiteralValue<'a> {
    /// Boolean value.
    Boolean(bool),
    /// Arbitrary precision signed integer.
    Integer(BigInt),
    /// Reduced rational or quantity magnitude.
    Rational {
        numerator: BigInt,
        denominator: BigInt,
    },
    /// Decimal coefficient and the exact written scale.
    Decimal { coefficient: BigInt, scale: usize },
    /// IEEE binary32 bits, including signed zero and NaN payloads.
    Float32Bits(u32),
    /// IEEE binary64 bits, including signed zero and NaN payloads.
    Float64Bits(u64),
    /// Text value.
    Text(&'a str),
    /// Native string value.
    String(&'a str),
    /// Canonical UUID spelling.
    Uuid(&'a str),
    /// Signed nanoseconds from the POSIX epoch.
    Timestamp(i128),
    /// Exact enumeration value.
    Enum(&'a str),
    /// Absent option value.
    None,
}

/// A borrowed read of one literal's type, kind, and decoded value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedLiteralRead<'a> {
    /// The literal's declared type node.
    pub type_id: &'a CheckedNodeId,
    /// The admitted literal kind.
    pub kind: LiteralKind,
    /// The decoded value.
    pub value: CheckedLiteralValue<'a>,
}

fn rational_parts(value: &Value) -> Option<(BigInt, BigInt)> {
    let text = value.as_str()?;
    let (numerator, denominator) = text.split_once('/').unwrap_or((text, "1"));
    Some((
        BigInt::parse_bytes(numerator.as_bytes(), 10)?,
        BigInt::parse_bytes(denominator.as_bytes(), 10)?,
    ))
}

fn literal_value(kind: LiteralKind, value: &Value) -> Option<CheckedLiteralValue<'_>> {
    Some(match kind {
        LiteralKind::Boolean => CheckedLiteralValue::Boolean(value.as_bool()?),
        LiteralKind::Integer => {
            let text = match value {
                Value::String(text) => text.clone(),
                Value::Number(number) => number.to_string(),
                _ => return None,
            };
            CheckedLiteralValue::Integer(BigInt::parse_bytes(text.as_bytes(), 10)?)
        }
        LiteralKind::Rational | LiteralKind::Quantity => {
            let (numerator, denominator) = rational_parts(value)?;
            CheckedLiteralValue::Rational {
                numerator,
                denominator,
            }
        }
        LiteralKind::Decimal => {
            let text = value.as_str()?;
            let (whole, fractional) = text.split_once('.').unwrap_or((text, ""));
            let scale = fractional.len();
            let digits = format!("{whole}{fractional}");
            CheckedLiteralValue::Decimal {
                coefficient: BigInt::parse_bytes(digits.as_bytes(), 10)?,
                scale,
            }
        }
        LiteralKind::Float32Bits => {
            CheckedLiteralValue::Float32Bits(u32::from_str_radix(value.as_str()?, 16).ok()?)
        }
        LiteralKind::Float64Bits => {
            CheckedLiteralValue::Float64Bits(u64::from_str_radix(value.as_str()?, 16).ok()?)
        }
        LiteralKind::Text => CheckedLiteralValue::Text(value.as_str()?),
        LiteralKind::String => CheckedLiteralValue::String(value.as_str()?),
        LiteralKind::Uuid => CheckedLiteralValue::Uuid(value.as_str()?),
        LiteralKind::Timestamp => CheckedLiteralValue::Timestamp(value.as_str()?.parse().ok()?),
        LiteralKind::Enum => CheckedLiteralValue::Enum(value.as_str()?),
        LiteralKind::None if value.is_null() => CheckedLiteralValue::None,
        LiteralKind::None => return None,
    })
}

/// A body term in an admitted checked-package V2 node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "term", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedTerm {
    /// A typed scalar literal.
    Literal {
        /// The scalar type node.
        #[serde(rename = "type")]
        type_id: CheckedNodeId,
        /// The closed literal kind.
        value_kind: LiteralKind,
        /// The literal payload; its shape depends on `value_kind`.
        value: Value,
    },
    /// A reference to another graph node.
    Reference {
        /// The referenced node.
        target: CheckedNodeId,
    },
    /// An operation application at a node body root.
    Application {
        /// The closed operator class.
        operator: ApplicationOperator,
        /// The catalogued operation's closed wire record.
        operation: CheckedApplicationOperation,
        /// The declared result type node.
        result_type: CheckedNodeId,
        /// Typed argument terms in source order.
        arguments: Vec<CheckedTerm>,
    },
    /// An ordered aggregate.
    Aggregate {
        /// Typed members in source order.
        members: Vec<CheckedTerm>,
    },
    /// A named aggregate member.
    Binding {
        /// The member name.
        name: CheckedBindingName,
        /// The member value.
        value: Box<CheckedTerm>,
    },
    /// A reference to a selected dependency package node.
    DependencyReference {
        /// Dependency package identity.
        package: Box<str>,
        /// The dependency's node key.
        node: CheckedNodeId,
    },
    /// The closed state frame body.
    Frame {
        /// Write grants.
        modifies: Vec<CheckedFrameModification>,
        /// Created nodes.
        creates: Vec<CheckedNodeId>,
        /// Deleted nodes.
        deletes: Vec<CheckedNodeId>,
        /// Scoped deletion grants.
        deletes_scoped: Vec<CheckedScopedDeletion>,
    },
    /// The closed abstraction relation body.
    AbstractionRelation {
        /// Object mappings.
        objects: Vec<CheckedAbstractionObject>,
        /// Population mappings.
        populations: Vec<CheckedAbstractionPopulation>,
        /// Frame mappings.
        frames: Vec<CheckedAbstractionFrame>,
    },
    /// A closed model or protocol refinement declaration.
    Refinement {
        /// Abstract model declaration.
        #[serde(rename = "abstract")]
        abstract_key: Box<RefinementKey>,
        /// Selected infinite-trace profile.
        profile: CheckedArtifactRef,
        /// Population mappings.
        populations: Vec<RefinementPopulation>,
        /// Object mappings.
        objects: Vec<RefinementObject>,
        /// Step mappings.
        steps: Vec<RefinementStep>,
        /// History fields.
        histories: Vec<RefinementHistory>,
        /// Concrete fairness assumptions.
        assume: Vec<RefinementAssume>,
        /// Abstract fairness guarantees.
        ensure: Vec<RefinementEnsure>,
        /// Abstract protocol clause, when selected.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        protocol: Option<Box<RefinementKey>>,
        /// Nonempty alias-free internal list, when supplied.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        abstract_internal: Option<Vec<Box<str>>>,
        /// Identity-bound checked classes of the selected subjects.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        class_catalog: Option<super::refinement::RefinementClassCatalog>,
    },
    /// The closed extension declaration body.
    ExtensionForm {
        /// Selected definition.
        definition: CheckedArtifactRef,
        /// Definition-owned node kind.
        node_kind: Box<str>,
        /// Ordered clause bindings.
        clauses: Vec<CheckedTerm>,
    },
    /// Closed source-authored enum, dimension, unit or equivalence body.
    #[serde(untagged, skip_deserializing)]
    NominalBody(Box<super::identity::NominalBody>),
    /// A dedicated operation anchor body; its bare point is not a term.
    #[serde(untagged, skip_deserializing)]
    OperationAnchorBody(Box<CheckedOperationAnchorBody>),
}

enum NodeMapWork<'a> {
    Term(&'a mut CheckedTerm),
    Id(&'a mut CheckedNodeId),
    Operation(&'a mut CheckedApplicationOperation),
    Nominal(&'a mut super::identity::NominalBody),
}

fn push_refinement_key<'a>(work: &mut Vec<NodeMapWork<'a>>, key: &'a mut RefinementKey) {
    match key {
        RefinementKey::Local(node) => work.push(NodeMapWork::Id(node)),
        RefinementKey::External(external) => work.push(NodeMapWork::Id(&mut external.node)),
    }
}

#[ix_trace_rs::string_edge]
fn map_operation_ids(
    operation: &mut CheckedApplicationOperation,
    map: &mut impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>,
) {
    let Some(member) = operation
        .record
        .get_mut("member")
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    let kind = member
        .get("kind")
        .and_then(Value::as_str)
        .and_then(OperationMemberKind::from_wire);
    let is_field = kind == Some(OperationMemberKind::Field);
    let fields: &[&str] = match kind {
        Some(OperationMemberKind::RelationshipEnd) => {
            &["declaration", "destination_type", "population"]
        }
        Some(
            OperationMemberKind::Field
            | OperationMemberKind::Position
            | OperationMemberKind::Element
            | OperationMemberKind::Operation
            | OperationMemberKind::TypeArgument
            | OperationMemberKind::Fairness,
        ) => &["declaration"],
        Some(
            OperationMemberKind::ProfileOperator
            | OperationMemberKind::StateClause
            | OperationMemberKind::TemporalInterval,
        )
        | None => &[],
    };
    let mut mapped_declaration = None;
    let mut mapped_destination = None;
    let mut mapped_population = None;
    for field in fields {
        if let Some(value) = member.get_mut(*field) {
            if let Some(original) = CheckedNodeId::from_value(value) {
                let replacement = map(&original);
                if let Some(replacement) = &replacement {
                    *value = serde_json::to_value(replacement)
                        .expect("checked node ID serialization is infallible");
                }
                let mapped = replacement.unwrap_or(original);
                match *field {
                    "declaration" => mapped_declaration = Some(mapped),
                    "destination_type" => mapped_destination = Some(mapped),
                    "population" => mapped_population = Some(mapped),
                    _ => {}
                }
            }
        }
    }

    let Some(views) = &mut operation.views else {
        return;
    };
    if let Some(record) = &mut views.record {
        if let Some(member) = &mut record.member {
            match member {
                CheckedOperationMember::Field { declaration, .. }
                | CheckedOperationMember::Position { declaration, .. }
                | CheckedOperationMember::Element { declaration }
                | CheckedOperationMember::Operation { declaration, .. }
                | CheckedOperationMember::TypeArgument { declaration }
                | CheckedOperationMember::Fairness { declaration, .. } => {
                    if let Some(mapped) = &mapped_declaration {
                        *declaration = mapped.clone();
                    }
                }
                CheckedOperationMember::RelationshipEnd {
                    declaration,
                    destination_type,
                    population,
                    ..
                } => {
                    if let Some(mapped) = &mapped_declaration {
                        *declaration = mapped.clone();
                    }
                    if let Some(mapped) = &mapped_destination {
                        *destination_type = mapped.clone();
                    }
                    if let (Some(population), Some(mapped)) = (population, &mapped_population) {
                        *population = mapped.clone();
                    }
                }
                CheckedOperationMember::ProfileOperator { .. }
                | CheckedOperationMember::StateClause { .. }
                | CheckedOperationMember::TemporalInterval { .. } => {}
            }
        }
    }
    if let (Some((declaration, _)), Some(mapped)) =
        (&mut views.projection_field, &mapped_declaration)
    {
        if is_field {
            *declaration = mapped.clone();
        }
    }
    if let (Some(declaration), Some(mapped)) = (&mut views.member_declaration, &mapped_declaration)
    {
        *declaration = mapped.clone();
    }
}

/// Borrowed, one-level decoding for early family wire checks. Children are
/// checked in their original order, with their own refusal pointers, before
/// the complete admitted tree is constructed.
pub(in crate::checked_package) enum CheckedTermView<'a> {
    Application {
        operator: ApplicationOperator,
        operation: &'a Value,
        result_type: &'a Value,
        arguments: &'a [Value],
    },
    Aggregate {
        members: &'a [Value],
    },
    Binding {
        name: &'a str,
        value: &'a Value,
    },
    Reference {
        target: &'a Value,
    },
    Literal {
        type_id: &'a Value,
        value_kind: LiteralKind,
        value: &'a Value,
    },
}

impl CheckedTerm {
    /// Read a scalar literal without inspecting its JSON wire payload.
    /// Returns `None` for another term or a directly decoded malformed literal.
    pub fn literal_read(&self) -> Option<CheckedLiteralRead<'_>> {
        let Self::Literal {
            type_id,
            value_kind,
            value,
        } = self
        else {
            return None;
        };
        Some(CheckedLiteralRead {
            type_id,
            kind: *value_kind,
            value: literal_value(*value_kind, value)?,
        })
    }

    pub(in crate::checked_package) fn view(value: &Value) -> Option<CheckedTermView<'_>> {
        Self::view_from_wire(crate::checked_package::common::WireTerm::decode(value))
    }

    pub(in crate::checked_package) fn view_from_wire(
        wire: crate::checked_package::common::WireTerm<'_>,
    ) -> Option<CheckedTermView<'_>> {
        use crate::checked_package::common::{exact_members, TermField, TermTag};
        let object = wire.object()?;
        match wire.tag() {
            TermTag::Known(BodyTerm::Application)
                if exact_members(
                    object,
                    &["term", "operator", "operation", "result_type", "arguments"],
                ) =>
            {
                Some(CheckedTermView::Application {
                    operator: wire.operator()?,
                    operation: wire.field(TermField::Operation)?,
                    result_type: wire.field(TermField::ResultType)?,
                    arguments: wire.array(TermField::Arguments)?,
                })
            }
            TermTag::Known(BodyTerm::Aggregate) if exact_members(object, &["term", "members"]) => {
                Some(CheckedTermView::Aggregate {
                    members: wire.array(TermField::Members)?,
                })
            }
            TermTag::Known(BodyTerm::Binding)
                if exact_members(object, &["term", "name", "value"]) =>
            {
                Some(CheckedTermView::Binding {
                    name: wire.field(TermField::Name)?.as_str()?,
                    value: wire.field(TermField::Value)?,
                })
            }
            TermTag::Known(BodyTerm::Reference) if exact_members(object, &["term", "target"]) => {
                Some(CheckedTermView::Reference {
                    target: wire.field(TermField::Target)?,
                })
            }
            TermTag::Known(BodyTerm::Literal)
                if exact_members(object, &["term", "type", "value_kind", "value"]) =>
            {
                Some(CheckedTermView::Literal {
                    type_id: wire.field(TermField::Type)?,
                    value_kind: wire.literal_kind()?,
                    value: wire.field(TermField::Value)?,
                })
            }
            _ => None,
        }
    }
}

/// Why a validated wire body could not become its retained typed term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TermConversionCause {
    MissingTerm,
    UnknownTerm,
    MissingMember {
        member: &'static str,
    },
    MalformedMember {
        member: &'static str,
        category: Category,
    },
}

impl CheckedTerm {
    /// Return nested reference targets in term order, excluding type IDs and
    /// other node identities.
    pub fn reference_targets(&self) -> Vec<&CheckedNodeId> {
        let mut targets = Vec::new();
        self.visit_reference_targets(&mut |target| targets.push(target));
        targets
    }

    pub(super) fn visit_reference_targets<'a>(
        &'a self,
        visitor: &mut impl FnMut(&'a CheckedNodeId),
    ) {
        match self {
            Self::Reference { target } => visitor(target),
            Self::Application { arguments, .. } => {
                for argument in arguments {
                    argument.visit_reference_targets(visitor);
                }
            }
            Self::Aggregate { members } => {
                for member in members {
                    member.visit_reference_targets(visitor);
                }
            }
            Self::OperationAnchorBody(body) => {
                for member in &body.members {
                    member.visit_reference_targets(visitor);
                }
            }
            Self::NominalBody(body) => body.visit_reference_targets(visitor),
            Self::Binding { value, .. } => value.visit_reference_targets(visitor),
            Self::ExtensionForm { clauses, .. } => {
                for clause in clauses {
                    clause.visit_reference_targets(visitor);
                }
            }
            Self::Refinement {
                objects,
                steps,
                histories,
                ..
            } => {
                super::refinement::visit_reference_targets(objects, steps, histories, visitor);
            }
            Self::Literal { .. }
            | Self::DependencyReference { .. }
            | Self::Frame { .. }
            | Self::AbstractionRelation { .. } => {}
        }
    }

    /// Substitute an exact node identity in this term's typed fields and
    /// closed operation payloads without changing ordinary strings.
    pub fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        fn replace(id: &mut CheckedNodeId, old: &CheckedNodeId, new: &CheckedNodeId) {
            if id == old {
                *id = new.clone();
            }
        }
        match self {
            Self::Literal { type_id, value, .. } => {
                replace(type_id, old, new);
                replace_node_ids_in_value(value, old, new);
            }
            Self::Reference { target } => replace(target, old, new),
            Self::Application {
                operation,
                result_type,
                arguments,
                ..
            } => {
                replace_node_ids_in_value(&mut operation.record, old, new);
                if let Some(views) = &mut operation.views {
                    views.replace_node_id(old, new);
                }
                replace(result_type, old, new);
                for argument in arguments {
                    argument.replace_node_id(old, new);
                }
            }
            Self::Aggregate { members } => {
                for member in members {
                    member.replace_node_id(old, new);
                }
            }
            Self::OperationAnchorBody(body) => {
                for member in &mut body.members {
                    member.replace_node_id(old, new);
                }
            }
            Self::NominalBody(body) => body.replace_node_id(old, new),
            Self::Binding { value, .. } => value.replace_node_id(old, new),
            Self::DependencyReference { node, .. } => replace(node, old, new),
            Self::Frame {
                modifies,
                creates,
                deletes,
                ..
            } => {
                for modification in modifies {
                    match modification {
                        CheckedFrameModification::Relationship { declaration }
                        | CheckedFrameModification::Field { declaration, .. } => {
                            replace(declaration, old, new)
                        }
                    }
                }
                for id in creates.iter_mut().chain(deletes) {
                    replace(id, old, new);
                }
            }
            Self::AbstractionRelation {
                objects,
                populations,
                frames,
            } => {
                for object in objects {
                    replace(&mut object.type_id, old, new);
                }
                for population in populations {
                    replace(&mut population.population, old, new);
                }
                for frame in frames {
                    replace(&mut frame.context, old, new);
                }
            }
            Self::ExtensionForm { clauses, .. } => {
                for clause in clauses {
                    clause.replace_node_id(old, new);
                }
            }
            Self::Refinement { .. } => {
                super::refinement::replace_node_ids(self, old, new);
            }
        }
    }

    /// Rewrite each logical typed node identity slot once, in term preorder.
    ///
    /// The walk uses an explicit stack so term nesting does not consume the
    /// call stack. Replacements are applied only at their original slots and
    /// are not passed through the callback again.
    pub fn map_node_ids(&mut self, mut map: impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>) {
        let mut work = vec![NodeMapWork::Term(self)];
        while let Some(item) = work.pop() {
            match item {
                NodeMapWork::Id(id) => {
                    if let Some(replacement) = map(id) {
                        *id = replacement;
                    }
                }
                NodeMapWork::Operation(operation) => map_operation_ids(operation, &mut map),
                NodeMapWork::Nominal(body) => body.map_node_ids(&mut map),
                NodeMapWork::Term(term) => match term {
                    Self::Literal { type_id, .. } => work.push(NodeMapWork::Id(type_id)),
                    Self::Reference { target } => work.push(NodeMapWork::Id(target)),
                    Self::Application {
                        operation,
                        result_type,
                        arguments,
                        ..
                    } => {
                        work.extend(arguments.iter_mut().rev().map(NodeMapWork::Term));
                        work.push(NodeMapWork::Id(result_type));
                        work.push(NodeMapWork::Operation(operation));
                    }
                    Self::Aggregate { members } => {
                        work.extend(members.iter_mut().rev().map(NodeMapWork::Term));
                    }
                    Self::Binding { value, .. } => work.push(NodeMapWork::Term(value)),
                    Self::DependencyReference { node, .. } => work.push(NodeMapWork::Id(node)),
                    Self::Frame {
                        modifies,
                        creates,
                        deletes,
                        ..
                    } => {
                        work.extend(deletes.iter_mut().rev().map(NodeMapWork::Id));
                        work.extend(creates.iter_mut().rev().map(NodeMapWork::Id));
                        for modification in modifies.iter_mut().rev() {
                            let declaration = match modification {
                                CheckedFrameModification::Relationship { declaration }
                                | CheckedFrameModification::Field { declaration, .. } => {
                                    declaration
                                }
                            };
                            work.push(NodeMapWork::Id(declaration));
                        }
                    }
                    Self::AbstractionRelation {
                        objects,
                        populations,
                        frames,
                    } => {
                        for frame in frames.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut frame.context));
                        }
                        for population in populations.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut population.population));
                        }
                        for object in objects.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut object.type_id));
                        }
                    }
                    Self::ExtensionForm { clauses, .. } => {
                        work.extend(clauses.iter_mut().rev().map(NodeMapWork::Term));
                    }
                    Self::OperationAnchorBody(body) => {
                        work.extend(body.members.iter_mut().rev().map(NodeMapWork::Term));
                    }
                    Self::NominalBody(body) => work.push(NodeMapWork::Nominal(body)),
                    Self::Refinement {
                        abstract_key,
                        populations,
                        objects,
                        steps,
                        histories,
                        assume,
                        ensure,
                        protocol,
                        class_catalog,
                        ..
                    } => {
                        if let Some(catalog) = class_catalog {
                            for entry in catalog.r#abstract.iter_mut().rev() {
                                if let super::refinement::RefinementAbstractClass::Declared {
                                    node,
                                    binder_types,
                                    ..
                                } = entry
                                {
                                    for binder in binder_types.iter_mut().rev() {
                                        push_refinement_key(&mut work, binder);
                                    }
                                    push_refinement_key(&mut work, node);
                                }
                            }
                            for entry in catalog.concrete.iter_mut().rev() {
                                if let super::refinement::RefinementConcreteClass::Declared {
                                    node,
                                    binder_types,
                                    ..
                                } = entry
                                {
                                    work.extend(binder_types.iter_mut().rev().map(NodeMapWork::Id));
                                    work.push(NodeMapWork::Id(node));
                                }
                            }
                        }
                        if let Some(key) = protocol {
                            push_refinement_key(&mut work, key);
                        }
                        for row in ensure.iter_mut().rev() {
                            push_refinement_key(&mut work, &mut row.operation);
                        }
                        for row in assume.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut row.operation));
                        }
                        for row in histories.iter_mut().rev() {
                            for update in row.updates.iter_mut().rev() {
                                work.push(NodeMapWork::Term(&mut update.expression));
                                work.push(NodeMapWork::Id(&mut update.operation));
                            }
                            work.push(NodeMapWork::Term(&mut row.initial));
                            work.push(NodeMapWork::Id(&mut row.type_id));
                            work.push(NodeMapWork::Id(&mut row.concrete_type));
                        }
                        for row in steps.iter_mut().rev() {
                            match &mut row.target {
                                RefinementStepTarget::Operation {
                                    operation,
                                    receiver,
                                    arguments,
                                } => {
                                    for argument in arguments.iter_mut().rev() {
                                        if let RefinementStepArgument::Term(term) = argument {
                                            work.push(NodeMapWork::Term(term));
                                        }
                                    }
                                    work.push(NodeMapWork::Term(receiver));
                                    push_refinement_key(&mut work, operation);
                                }
                                RefinementStepTarget::ProtocolStep {
                                    node, arguments, ..
                                } => {
                                    for argument in arguments.iter_mut().rev() {
                                        if let RefinementStepArgument::Term(term) = argument {
                                            work.push(NodeMapWork::Term(term));
                                        }
                                    }
                                    if let Some(key) = node {
                                        push_refinement_key(&mut work, key);
                                    }
                                }
                                RefinementStepTarget::Stutter | RefinementStepTarget::Any => {}
                            }
                            if let Some(node) = &mut row.source_node {
                                work.push(NodeMapWork::Id(node));
                            }
                        }
                        for row in objects.iter_mut().rev() {
                            work.extend(
                                row.fields
                                    .iter_mut()
                                    .rev()
                                    .map(|field| NodeMapWork::Term(&mut field.expression)),
                            );
                            work.push(NodeMapWork::Id(&mut row.concrete_type));
                            push_refinement_key(&mut work, &mut row.abstract_type);
                        }
                        for row in populations.iter_mut().rev() {
                            work.push(NodeMapWork::Id(&mut row.concrete));
                            push_refinement_key(&mut work, &mut row.r#abstract);
                        }
                        push_refinement_key(&mut work, abstract_key);
                    }
                },
            }
        }
    }
    /// Construct a checked reference body without an untyped JSON intermediate.
    pub fn reference(target: CheckedNodeId) -> Self {
        Self::Reference { target }
    }

    /// Construct an ordered checked aggregate body.
    pub fn aggregate(members: Vec<Self>) -> Self {
        Self::Aggregate { members }
    }

    /// Construct one named aggregate member.
    pub fn binding(name: impl Into<Box<str>>, value: Self) -> Self {
        Self::Binding {
            name: name.into().into(),
            value: Box::new(value),
        }
    }

    /// Construct a text literal; the wire payload is a JSON string.
    pub fn literal_text(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Text,
            value: Value::String(value.into().into()),
        }
    }

    /// Construct a rational literal; the wire payload is a JSON string.
    pub fn literal_rational(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Rational,
            value: Value::String(value.into().into()),
        }
    }

    /// Construct a quantity literal; the wire payload is a JSON string.
    pub fn literal_quantity(type_id: CheckedNodeId, value: impl Into<Box<str>>) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Quantity,
            value: Value::String(value.into().into()),
        }
    }

    /// Construct a nonnegative integer literal; the wire payload is a JSON number.
    pub fn literal_integer(type_id: CheckedNodeId, value: u64) -> Self {
        Self::Literal {
            type_id,
            value_kind: LiteralKind::Integer,
            value: Value::Number(value.into()),
        }
    }

    /// Decodes only a body that the reader's flat and closed-shape checks
    /// already accepted. Public serde decoding does not provide that bound.
    pub(super) fn decode_admitted(value: &Value) -> Result<Self, TermConversionCause> {
        if value.get("term").and_then(Value::as_str) == Some("refinement") {
            return super::refinement::decode_body(
                value,
                &crate::checked_package::common::Trail::Base(&[]),
            )
            .map_err(|_| TermConversionCause::MalformedMember {
                member: "refinement",
                category: Category::Data,
            });
        }
        fn field<T: DeserializeOwned>(
            wire: crate::checked_package::common::WireTerm<'_>,
            name: crate::checked_package::common::TermField,
        ) -> Result<T, TermConversionCause> {
            let member = wire.field(name).ok_or(TermConversionCause::MissingMember {
                member: name.as_wire(),
            })?;
            serde_json::from_value(member.clone()).map_err(|error| {
                TermConversionCause::MalformedMember {
                    member: name.as_wire(),
                    category: error.classify(),
                }
            })
        }

        fn children(
            wire: crate::checked_package::common::WireTerm<'_>,
            name: crate::checked_package::common::TermField,
        ) -> Result<Vec<CheckedTerm>, TermConversionCause> {
            wire.field(name)
                .ok_or(TermConversionCause::MissingMember {
                    member: name.as_wire(),
                })?
                .as_array()
                .ok_or(TermConversionCause::MalformedMember {
                    member: name.as_wire(),
                    category: Category::Data,
                })?
                .iter()
                .map(CheckedTerm::decode_admitted)
                .collect()
        }

        use crate::checked_package::common::{TermField, TermTag, WireTerm};
        let wire = WireTerm::decode(value);
        match wire.tag() {
            TermTag::Known(BodyTerm::Literal) => Ok(Self::Literal {
                type_id: field(wire, TermField::Type)?,
                value_kind: field(wire, TermField::ValueKind)?,
                value: field(wire, TermField::Value)?,
            }),
            TermTag::Known(BodyTerm::Reference) => Ok(Self::Reference {
                target: field(wire, TermField::Target)?,
            }),
            TermTag::Known(BodyTerm::Application) => Ok(Self::Application {
                operator: field(wire, TermField::Operator)?,
                operation: field(wire, TermField::Operation)?,
                result_type: field(wire, TermField::ResultType)?,
                arguments: children(wire, TermField::Arguments)?,
            }),
            TermTag::Known(BodyTerm::Aggregate) => Ok(Self::Aggregate {
                members: children(wire, TermField::Members)?,
            }),
            TermTag::Known(BodyTerm::Binding) => Ok(Self::Binding {
                name: field(wire, TermField::Name)?,
                value: Box::new(Self::decode_admitted(
                    wire.field(TermField::Value)
                        .ok_or(TermConversionCause::MissingMember { member: "value" })?,
                )?),
            }),
            TermTag::Known(BodyTerm::DependencyReference) => Ok(Self::DependencyReference {
                package: field(wire, TermField::Package)?,
                node: field(wire, TermField::Node)?,
            }),
            TermTag::Known(BodyTerm::Frame) => Ok(Self::Frame {
                modifies: field(wire, TermField::Modifies)?,
                creates: field(wire, TermField::Creates)?,
                deletes: field(wire, TermField::Deletes)?,
                deletes_scoped: field(wire, TermField::DeletesScoped)?,
            }),
            TermTag::Known(BodyTerm::AbstractionRelation) => Ok(Self::AbstractionRelation {
                objects: field(wire, TermField::Objects)?,
                populations: field(wire, TermField::Populations)?,
                frames: field(wire, TermField::Frames)?,
            }),
            TermTag::ExtensionForm => Ok(Self::ExtensionForm {
                definition: field(wire, TermField::Definition)?,
                node_kind: field(wire, TermField::NodeKind)?,
                clauses: children(wire, TermField::Clauses)?,
            }),
            TermTag::Missing => Err(TermConversionCause::MissingTerm),
            TermTag::NonString => Err(TermConversionCause::MalformedMember {
                member: "term",
                category: Category::Data,
            }),
            TermTag::Unknown => Err(TermConversionCause::UnknownTerm),
        }
    }
}

impl Encode for CheckedTerm {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        fn encode_rational<S: Sink + ?Sized>(
            value: &super::identity::CheckedRational,
            writer: &mut Writer<'_, S>,
        ) -> Result<(), Error> {
            writer.begin_object()?;
            writer.name("numerator")?;
            writer.string(&value.numerator)?;
            writer.name("denominator")?;
            writer.string(&value.denominator)?;
            writer.end_object()
        }
        writer.begin_object()?;
        match self {
            Self::Literal {
                type_id,
                value_kind,
                value,
            } => {
                writer.name("term")?;
                writer.string("literal")?;
                writer.name("type")?;
                type_id.encode_into(writer)?;
                writer.name("value_kind")?;
                writer.serialize(value_kind)?;
                writer.name("value")?;
                value.encode_into(writer)?;
            }
            Self::Reference { target } => {
                writer.name("term")?;
                writer.string("reference")?;
                writer.name("target")?;
                target.encode_into(writer)?;
            }
            Self::Application {
                operator,
                operation,
                result_type,
                arguments,
            } => {
                writer.name("term")?;
                writer.string("application")?;
                writer.name("operator")?;
                writer.serialize(operator)?;
                writer.name("operation")?;
                operation.encode_into(writer)?;
                writer.name("result_type")?;
                result_type.encode_into(writer)?;
                writer.name("arguments")?;
                writer.begin_array()?;
                for argument in arguments {
                    argument.encode_into(writer)?;
                }
                writer.end_array()?;
            }
            Self::Aggregate { members } => {
                writer.name("term")?;
                writer.string("aggregate")?;
                writer.name("members")?;
                writer.begin_array()?;
                for member in members {
                    member.encode_into(writer)?;
                }
                writer.end_array()?;
            }
            Self::Binding { name, value } => {
                writer.name("term")?;
                writer.string("binding")?;
                writer.name("name")?;
                writer.string(name)?;
                writer.name("value")?;
                value.encode_into(writer)?;
            }
            Self::DependencyReference { package, node } => {
                writer.name("term")?;
                writer.string("dependency_reference")?;
                writer.name("package")?;
                writer.serialize(package)?;
                writer.name("node")?;
                node.encode_into(writer)?;
            }
            Self::Frame {
                modifies,
                creates,
                deletes,
                deletes_scoped,
            } => {
                writer.name("term")?;
                writer.string("frame")?;
                writer.name("modifies")?;
                super::refinement::encode_rows(modifies, writer)?;
                writer.name("creates")?;
                super::refinement::encode_rows(creates, writer)?;
                writer.name("deletes")?;
                super::refinement::encode_rows(deletes, writer)?;
                writer.name("deletes_scoped")?;
                writer.serialize(deletes_scoped)?;
            }
            Self::AbstractionRelation {
                objects,
                populations,
                frames,
            } => {
                writer.name("term")?;
                writer.string("abstraction_relation")?;
                writer.name("objects")?;
                super::refinement::encode_rows(objects, writer)?;
                writer.name("populations")?;
                super::refinement::encode_rows(populations, writer)?;
                writer.name("frames")?;
                super::refinement::encode_rows(frames, writer)?;
            }
            Self::Refinement {
                abstract_key,
                profile,
                populations,
                objects,
                steps,
                histories,
                assume,
                ensure,
                protocol,
                abstract_internal,
                class_catalog,
            } => {
                writer.name("term")?;
                writer.string("refinement")?;
                writer.name("abstract")?;
                abstract_key.encode_into(writer)?;
                writer.name("profile")?;
                writer.serialize(profile)?;
                writer.name("populations")?;
                super::refinement::encode_rows(populations, writer)?;
                writer.name("objects")?;
                super::refinement::encode_rows(objects, writer)?;
                writer.name("steps")?;
                super::refinement::encode_rows(steps, writer)?;
                writer.name("histories")?;
                super::refinement::encode_rows(histories, writer)?;
                writer.name("assume")?;
                super::refinement::encode_rows(assume, writer)?;
                writer.name("ensure")?;
                super::refinement::encode_rows(ensure, writer)?;
                if let Some(protocol) = protocol {
                    writer.name("protocol")?;
                    protocol.encode_into(writer)?;
                }
                if let Some(internal) = abstract_internal {
                    writer.name("abstract_internal")?;
                    writer.serialize(internal)?;
                }
                if let Some(catalog) = class_catalog {
                    writer.name("class_catalog")?;
                    catalog.encode_into(writer)?;
                }
            }
            Self::ExtensionForm {
                definition,
                node_kind,
                clauses,
            } => {
                writer.name("term")?;
                writer.string("extension_form")?;
                writer.name("definition")?;
                writer.serialize(definition)?;
                writer.name("node_kind")?;
                writer.string(node_kind)?;
                writer.name("clauses")?;
                writer.begin_array()?;
                for clause in clauses {
                    clause.encode_into(writer)?;
                }
                writer.end_array()?;
            }
            Self::NominalBody(body) => match body.kind() {
                super::identity::NominalBodyKind::Enum { ordered, members } => {
                    writer.name("term")?;
                    writer.string("enum")?;
                    writer.name("ordered")?;
                    writer.serialize(ordered)?;
                    writer.name("members")?;
                    writer.serialize(members)?;
                }
                super::identity::NominalBodyKind::Dimension { terms } => {
                    writer.name("term")?;
                    writer.string("dimension")?;
                    writer.name("terms")?;
                    writer.begin_array()?;
                    for term in terms {
                        writer.begin_object()?;
                        writer.name("dimension_node_id")?;
                        term.dimension_node_id.encode_into(writer)?;
                        writer.name("exponent")?;
                        writer.string(&term.exponent)?;
                        writer.end_object()?;
                    }
                    writer.end_array()?;
                }
                super::identity::NominalBodyKind::Unit {
                    dimension_node_id,
                    target_unit_node_id,
                    scale,
                    offset,
                } => {
                    writer.name("term")?;
                    writer.string("unit")?;
                    writer.name("dimension_node_id")?;
                    dimension_node_id.encode_into(writer)?;
                    writer.name("target_unit_node_id")?;
                    if let Some(target) = target_unit_node_id {
                        target.encode_into(writer)?;
                    } else {
                        writer.serialize(&Option::<u8>::None)?;
                    }
                    writer.name("scale")?;
                    encode_rational(scale, writer)?;
                    writer.name("offset")?;
                    encode_rational(offset, writer)?;
                }
                super::identity::NominalBodyKind::DimensionEquivalence {
                    dimension_node_ids,
                    root_scale,
                } => {
                    writer.name("term")?;
                    writer.string("dimension_equivalence")?;
                    writer.name("dimension_node_ids")?;
                    writer.begin_array()?;
                    for id in dimension_node_ids {
                        id.encode_into(writer)?;
                    }
                    writer.end_array()?;
                    writer.name("root_scale")?;
                    encode_rational(root_scale, writer)?;
                }
            },
            Self::OperationAnchorBody(body) => {
                writer.name("term")?;
                writer.string("aggregate")?;
                writer.name("members")?;
                writer.begin_array()?;
                for member in &body.members {
                    member.encode_into(writer)?;
                }
                if let Some(point) = &body.execution_point {
                    writer.begin_object()?;
                    writer.name("term")?;
                    writer.string("binding")?;
                    writer.name("name")?;
                    writer.string("execution_point")?;
                    writer.name("value")?;
                    writer.serialize(point)?;
                    writer.end_object()?;
                }
                writer.end_array()?;
            }
        }
        writer.end_object()
    }
}

fn replace_node_ids_in_value(value: &mut Value, old: &CheckedNodeId, new: &CheckedNodeId) {
    let mut pending = vec![value];
    while let Some(current) = pending.pop() {
        if value_is_node_id(current, old) {
            *current = node_id_value(new);
            continue;
        }
        match current {
            Value::Array(items) => pending.extend(items.iter_mut()),
            Value::Object(object) => pending.extend(object.values_mut()),
            _ => {}
        }
    }
}

fn value_is_node_id(value: &Value, id: &CheckedNodeId) -> bool {
    let mut pending = vec![(value, id)];
    while let Some((value, id)) = pending.pop() {
        let Some(object) = value.as_object() else {
            return false;
        };
        if object.len() != 3
            || object.get("package").and_then(Value::as_str) != Some(id.package.as_ref())
            || object.get("ordinal").and_then(Value::as_u64) != Some(id.ordinal)
        {
            return false;
        }
        let Some(path) = object.get("path").and_then(Value::as_array) else {
            return false;
        };
        if path.len() != id.path.len() {
            return false;
        }
        for (segment, expected) in path.iter().zip(&id.path) {
            match expected {
                CheckedNodePathSegment::Name(name) if segment.as_str() == Some(name.as_ref()) => {}
                CheckedNodePathSegment::Node(node) => pending.push((segment, node)),
                CheckedNodePathSegment::Index(index)
                    if segment.get("node").and_then(Value::as_u64) == Some(*index) => {}
                _ => return false,
            }
        }
    }
    true
}

fn node_id_value(id: &CheckedNodeId) -> Value {
    enum Task<'a> {
        Node(&'a CheckedNodeId),
        Name(&'a str),
        Index(u64),
        Finish(&'a CheckedNodeId),
    }
    let mut pending = vec![Task::Node(id)];
    let mut values = Vec::new();
    while let Some(task) = pending.pop() {
        match task {
            Task::Node(node) => {
                pending.push(Task::Finish(node));
                for segment in node.path.iter().rev() {
                    pending.push(match segment {
                        CheckedNodePathSegment::Name(name) => Task::Name(name),
                        CheckedNodePathSegment::Node(child) => Task::Node(child),
                        CheckedNodePathSegment::Index(index) => Task::Index(*index),
                    });
                }
            }
            Task::Name(name) => values.push(Value::String(name.to_owned())),
            Task::Index(index) => {
                let mut object = serde_json::Map::new();
                object.insert("node".into(), Value::Number(index.into()));
                values.push(Value::Object(object));
            }
            Task::Finish(node) => {
                let path = values.split_off(values.len() - node.path.len());
                let mut object = serde_json::Map::new();
                object.insert("package".into(), Value::String(node.package.to_string()));
                object.insert("path".into(), Value::Array(path));
                object.insert("ordinal".into(), Value::Number(node.ordinal.into()));
                values.push(Value::Object(object));
            }
        }
    }
    values.pop().expect("one constructed node id")
}

/// One frame write grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedFrameModification {
    /// A relationship declaration.
    Relationship {
        /// The relationship node.
        declaration: CheckedNodeId,
    },
    /// A field declaration and its scope.
    Field {
        /// The field declaration node.
        declaration: CheckedNodeId,
        /// Field name.
        name: Box<str>,
        /// The write scope.
        scope: CheckedFieldScope,
    },
}

/// The scope of one frame field grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(rename_all = "kebab-case")]
pub enum CheckedFieldScope {
    /// Every object in the frame.
    EveryObject,
    /// The receiver only.
    Receiver,
}

/// One scoped deletion grant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckedScopedDeletion {
    /// Delete the receiver.
    Receiver,
    /// Delete a named parameter.
    Parameter {
        /// Parameter name.
        name: Box<str>,
    },
}

/// One abstraction object mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionObject {
    /// Model type node.
    #[serde(rename = "type")]
    pub type_id: CheckedNodeId,
    /// Rust type path.
    #[serde(rename = "rust_type")]
    pub type_path: Vec<Box<str>>,
    /// Field mappings.
    pub fields: Vec<CheckedAbstractionField>,
}

/// One abstraction field mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionField {
    /// Model field.
    pub name: Box<str>,
    /// Rust field.
    #[serde(rename = "rust_field")]
    pub field_ident: Box<str>,
}

/// One abstraction population mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionPopulation {
    /// Population node.
    pub population: CheckedNodeId,
    /// Rust collection path.
    pub collection: Vec<Box<str>>,
}

/// One abstraction frame mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionFrame {
    /// Context node.
    pub context: CheckedNodeId,
    /// Operation name.
    pub operation: Box<str>,
    /// Rust function path.
    pub function: Vec<Box<str>>,
    /// Rust receiver name.
    pub receiver: Box<str>,
    /// Parameter mappings.
    pub parameters: Vec<CheckedAbstractionParameter>,
}

/// One abstraction parameter mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedAbstractionParameter {
    /// Model parameter.
    pub name: Box<str>,
    /// Rust parameter.
    #[serde(rename = "rust_parameter")]
    pub parameter_ident: Box<str>,
}

impl Encode for CheckedFrameModification {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        match self {
            Self::Relationship { declaration } => {
                writer.name("kind")?;
                writer.string("relationship")?;
                writer.name("declaration")?;
                declaration.encode_into(writer)?;
            }
            Self::Field {
                declaration,
                name,
                scope,
            } => {
                writer.name("kind")?;
                writer.string("field")?;
                writer.name("declaration")?;
                declaration.encode_into(writer)?;
                writer.name("name")?;
                writer.serialize(name)?;
                writer.name("scope")?;
                writer.serialize(scope)?;
            }
        }
        writer.end_object()
    }
}

impl Encode for CheckedAbstractionObject {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("type")?;
        self.type_id.encode_into(writer)?;
        writer.name("rust_type")?;
        writer.serialize(&self.type_path)?;
        writer.name("fields")?;
        writer.serialize(&self.fields)?;
        writer.end_object()
    }
}

impl Encode for CheckedAbstractionPopulation {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("population")?;
        self.population.encode_into(writer)?;
        writer.name("collection")?;
        writer.serialize(&self.collection)?;
        writer.end_object()
    }
}

impl Encode for CheckedAbstractionFrame {
    fn encode_into<S: Sink + ?Sized>(&self, writer: &mut Writer<'_, S>) -> Result<(), Error> {
        writer.begin_object()?;
        writer.name("context")?;
        self.context.encode_into(writer)?;
        writer.name("operation")?;
        writer.serialize(&self.operation)?;
        writer.name("function")?;
        writer.serialize(&self.function)?;
        writer.name("receiver")?;
        writer.serialize(&self.receiver)?;
        writer.name("parameters")?;
        writer.serialize(&self.parameters)?;
        writer.end_object()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApplicationOperator, CheckedApplicationOperation, CheckedBindingName,
        CheckedBindingNameKind, CheckedOperationMember, CheckedOperationRecord,
        CheckedOperationViews, CheckedTerm, ExecutionPoint, TermConversionCause,
    };
    use crate::checked_package::shared::{CheckedNodeId, CheckedNodePathSegment};
    use ix_trace_rs::trace;
    use quire_canonical::{to_vec, Limits};
    use serde_json::{error::Category, json};

    #[trace("FR-048-AC-7")]
    #[test]
    fn application_member_rewrite_keeps_wire_and_admitted_views_in_step() {
        let original = CheckedNodeId {
            package: "operation-map".into(),
            path: vec![CheckedNodePathSegment::Name("old".into())],
            ordinal: 0,
        };
        let replacement = CheckedNodeId {
            package: original.package.clone(),
            path: vec![CheckedNodePathSegment::Name("new".into())],
            ordinal: original.ordinal,
        };
        let member = CheckedOperationMember::Field {
            declaration: original.clone(),
            name: "field".into(),
            read_form: None,
        };
        let operation = CheckedApplicationOperation {
            record: json!({
                "identity": "quire.op.field.read", "laws": [], "mode": null,
                "member": {"kind": "field", "declaration": original, "name": "field"},
                "leaves": []
            }),
            execution_point: None,
            views: Some(CheckedOperationViews {
                record: Some(Box::new(CheckedOperationRecord {
                    identity: "quire.op.field.read".into(),
                    mode: None,
                    laws: Vec::new(),
                    member: Some(member),
                    constraints: Vec::new(),
                    leaves: Vec::new(),
                })),
                projection_field: Some((original.clone(), "field".into())),
                member_declaration: Some(original.clone()),
                ..CheckedOperationViews::default()
            }),
            #[cfg(feature = "test-support")]
            test_fixture: false,
        };
        let mut term = CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            result_type: original.clone(),
            arguments: Vec::new(),
        };
        let mut visits = 0;
        term.map_node_ids(|id| {
            assert_eq!(id, &original);
            visits += 1;
            Some(replacement.clone())
        });
        assert_eq!(visits, 2, "member and result are distinct logical slots");
        let CheckedTerm::Application {
            operation,
            result_type,
            ..
        } = term
        else {
            panic!("application changed form");
        };
        assert_eq!(result_type, replacement);
        assert_eq!(operation.member_declaration(), Ok(Some(&replacement)));
        assert_eq!(
            operation.projection_field(),
            Ok(Some((&replacement, "field")))
        );
        assert!(matches!(operation.read().expect("admitted view").member,
            Some(CheckedOperationMember::Field { declaration, .. }) if declaration == &replacement));
        assert_eq!(
            serde_json::to_value(&operation).expect("operation wire")["member"]["declaration"],
            serde_json::to_value(&replacement).expect("node ID wire")
        );
    }

    #[trace("FR-048-AC-7")]
    #[test]
    fn relationship_end_uses_one_callback_per_member_slot() {
        let original = CheckedNodeId {
            package: "operation-map".into(),
            path: vec![CheckedNodePathSegment::Name("shared".into())],
            ordinal: 0,
        };
        let member = CheckedOperationMember::RelationshipEnd {
            declaration: original.clone(),
            name: "original".into(),
            destination_type: original.clone(),
            population: Some(original.clone()),
        };
        let operation = CheckedApplicationOperation {
            record: json!({
                "identity": "quire.op.relationship.read", "laws": [], "mode": null,
                "member": {"kind": "relationship_end", "declaration": original,
                    "name": "original", "destination_type": original, "population": original},
                "leaves": []
            }),
            execution_point: None,
            views: Some(CheckedOperationViews {
                record: Some(Box::new(CheckedOperationRecord {
                    identity: "quire.op.relationship.read".into(),
                    mode: None,
                    laws: Vec::new(),
                    member: Some(member),
                    constraints: Vec::new(),
                    leaves: Vec::new(),
                })),
                member_declaration: Some(original.clone()),
                ..CheckedOperationViews::default()
            }),
            #[cfg(feature = "test-support")]
            test_fixture: false,
        };
        let mut term = CheckedTerm::Application {
            operator: ApplicationOperator::Call,
            operation,
            result_type: original.clone(),
            arguments: Vec::new(),
        };
        let mut visits = 0;
        term.map_node_ids(|seen| {
            assert_eq!(seen, &original);
            visits += 1;
            Some(CheckedNodeId {
                package: original.package.clone(),
                path: original.path.clone(),
                ordinal: visits,
            })
        });
        assert_eq!(visits, 4, "three member slots and the result type");
        let CheckedTerm::Application {
            operation,
            result_type,
            ..
        } = term
        else {
            panic!("application changed form");
        };
        let read = operation.read().expect("admitted operation view");
        let Some(CheckedOperationMember::RelationshipEnd {
            declaration,
            destination_type,
            population: Some(population),
            name,
        }) = read.member
        else {
            panic!("relationship member changed form");
        };
        assert_eq!(
            (
                declaration.ordinal,
                destination_type.ordinal,
                population.ordinal
            ),
            (1, 2, 3)
        );
        assert_eq!(result_type.ordinal, 4);
        assert_eq!(operation.member_declaration(), Ok(Some(declaration)));
        assert_eq!(name.as_ref(), "original");
        let wire = serde_json::to_value(&operation).expect("operation wire");
        for (field, id) in [
            ("declaration", declaration),
            ("destination_type", destination_type),
            ("population", population),
        ] {
            assert_eq!(
                wire["member"][field],
                serde_json::to_value(id).expect("node ID wire")
            );
        }
    }

    #[trace("FR-040-AC-31", "FR-040-AC-32", "FR-040-AC-36")]
    #[test]
    fn anchor_point_is_a_closed_bare_typed_object() {
        for wire in [
            json!({"kind": "pre", "operation": "scaled"}),
            json!({"kind": "post", "operation": "scaled"}),
        ] {
            let point = ExecutionPoint::from_wire(&wire).expect("closed point");
            assert!(CheckedTerm::decode_admitted(&wire).is_err());
            assert_eq!(serde_json::to_value(&point).expect("serialize point"), wire);
            assert_eq!(
                to_vec(&point, Limits::new(u64::MAX)).expect("encode point"),
                to_vec(&wire, Limits::new(u64::MAX)).expect("encode wire")
            );
        }
        for malformed in [
            json!({"kind": "pre", "operation": ""}),
            json!({"kind": "pre", "operation": "scaled", "extra": true}),
            json!({"kind": "unknown", "operation": "scaled"}),
        ] {
            assert!(ExecutionPoint::from_wire(&malformed).is_none());
            assert!(CheckedTerm::decode_admitted(&malformed).is_err());
        }
    }
    #[trace("TC-460")]
    #[test]
    fn admitted_binding_name_retains_its_class_without_changing_wire_text() {
        let known: CheckedBindingName = serde_json::from_value(json!("parameters")).unwrap();
        assert_eq!(known.kind(), CheckedBindingNameKind::Parameters);
        assert_eq!(serde_json::to_value(&known).unwrap(), json!("parameters"));
        let open: CheckedBindingName = serde_json::from_value(json!("user-field")).unwrap();
        assert_eq!(open.kind(), CheckedBindingNameKind::Other("user-field"));
        assert_eq!(open.as_ref(), "user-field");
    }

    #[trace("TC-048")]
    #[test]
    fn checked_term_conversion_causes_distinguish_invalid_inputs() {
        let missing = CheckedTerm::decode_admitted(&json!({})).expect_err("missing term");
        let unknown =
            CheckedTerm::decode_admitted(&json!({"term":"not_a_term"})).expect_err("unknown term");
        assert_ne!(missing, unknown, "missing and unknown term causes differ");
        assert_eq!(missing, TermConversionCause::MissingTerm);
        assert_eq!(unknown, TermConversionCause::UnknownTerm);
        for (body, expected) in [
            (
                json!({"term":false}),
                TermConversionCause::MalformedMember {
                    member: "term",
                    category: Category::Data,
                },
            ),
            (
                json!({"term":"reference"}),
                TermConversionCause::MissingMember { member: "target" },
            ),
            (
                json!({"term":"reference","target":0}),
                TermConversionCause::MalformedMember {
                    member: "target",
                    category: Category::Data,
                },
            ),
            (
                json!({"term":"aggregate"}),
                TermConversionCause::MissingMember { member: "members" },
            ),
            (
                json!({"term":"aggregate","members":false}),
                TermConversionCause::MalformedMember {
                    member: "members",
                    category: Category::Data,
                },
            ),
            (
                json!({"term":"binding","name":"x"}),
                TermConversionCause::MissingMember { member: "value" },
            ),
        ] {
            assert_eq!(CheckedTerm::decode_admitted(&body), Err(expected), "{body}");
        }
    }

    #[trace("TC-048")]
    #[test]
    fn checked_term_conversion_keeps_member_and_child_failure_order() {
        assert_eq!(
            CheckedTerm::decode_admitted(&json!({"term":"binding","name":0,"value":{}})),
            Err(TermConversionCause::MalformedMember {
                member: "name",
                category: Category::Data,
            }),
            "the binding name is decoded before its value"
        );
        assert_eq!(
            CheckedTerm::decode_admitted(&json!({"term":"literal","type":false})),
            Err(TermConversionCause::MalformedMember {
                member: "type",
                category: Category::Data,
            }),
            "the malformed type precedes the absent literal kind"
        );
        for (members, expected) in [
            (
                json!([{"term":"unknown"}, {}]),
                TermConversionCause::UnknownTerm,
            ),
            (
                json!([{}, {"term":"unknown"}]),
                TermConversionCause::MissingTerm,
            ),
        ] {
            let aggregate = json!({"term":"aggregate","members":members});
            assert_eq!(CheckedTerm::decode_admitted(&aggregate), Err(expected));
            assert_eq!(
                CheckedTerm::decode_admitted(
                    &json!({"term":"binding","name":"x","value":aggregate})
                ),
                Err(expected),
                "a recursive binding preserves the first child's exact cause"
            );
        }
    }

    #[trace("FR-349-AC-1")]
    #[test]
    fn checked_term_variants_round_trip_the_closed_wire_arms() {
        let node = json!({
            "package": "test-package", "path": ["node-a"], "ordinal": 0
        });
        let package = "dependency-package";
        let literal = json!({"term":"literal","type":node,"value_kind":"integer","value":"1"});
        let reference = json!({"term":"reference","target":node});
        let dependency = json!({"term":"dependency_reference","package":package,"node":node});
        let binding = json!({"term":"binding","name":"x","value":literal});
        let aggregate = json!({"term":"aggregate","members":[binding,reference,dependency]});
        let application = json!({
            "term":"application","operator":"binary",
            "operation":{"identity":"quire.op.test","laws":[],"mode":null,"member":null,"leaves":[]},
            "result_type":node,"arguments":[aggregate]
        });
        let frame = json!({
            "term":"frame","modifies":[],"creates":[],"deletes":[],"deletes_scoped":[]
        });
        let abstraction = json!({
            "term":"abstraction_relation","objects":[],"populations":[],"frames":[]
        });
        let refinement = json!({
            "term":"refinement", "abstract":node,
            "profile":{"authority":"test","identity":"trace"},
            "populations":[{"abstract":node,"concrete":node}],
            "objects":[{"abstract_type":node,"concrete_type":node,
                "fields":[{"name":"history","expression":reference}]}],
            "steps":[
                {"source_class":"finish","source_node":node,"target":{
                    "kind":"operation","operation":node,"receiver":reference,
                    "arguments":[{"term":"open"},reference]}},
                {"source_class":"memory flush","target":{
                    "kind":"protocol_step","class":"memory flush",
                    "arguments":[{"term":"open"}]}},
                {"source_class":"memory read","target":{"kind":"stutter"}},
                {"source_class":"memory write","target":{"kind":"any"}}
            ],
            "histories":[{"concrete_type":node,"name":"history","type":node,
                "initial":literal,"updates":[{"operation":node,"expression":reference}]}],
            "assume":[{"operation":node,"kind":"weak","granularity":"each"}],
            "ensure":[{"operation":node,"kind":"strong","granularity":"whole"}],
            "protocol":node,"abstract_internal":["history"],
            "class_catalog":{"concrete":[{"class":"finish","node":node,
                "binder_types":[node]},{"class":"memory flush","memory_model":"tso"}],
                "abstract":[{"class":"finish","node":node,"binder_types":[node]}]}
        });
        let extension = json!({
            "term":"extension_form",
            "definition":{"authority":"test","identity":"extension"},
            "node_kind":"retention","clauses":[]
        });

        for (body, expected) in [
            (literal, "literal"),
            (reference, "reference"),
            (dependency, "dependency_reference"),
            (binding, "binding"),
            (aggregate, "aggregate"),
            (application, "application"),
            (frame, "frame"),
            (abstraction, "abstraction_relation"),
            (refinement, "refinement"),
            (extension, "extension_form"),
        ] {
            let term = CheckedTerm::decode_admitted(&body).expect("closed term");
            let actual = match &term {
                CheckedTerm::Literal { .. } => "literal",
                CheckedTerm::Reference { .. } => "reference",
                CheckedTerm::DependencyReference { .. } => "dependency_reference",
                CheckedTerm::Binding { .. } => "binding",
                CheckedTerm::Aggregate { .. } => "aggregate",
                CheckedTerm::Application { .. } => "application",
                CheckedTerm::Frame { .. } => "frame",
                CheckedTerm::AbstractionRelation { .. } => "abstraction_relation",
                CheckedTerm::Refinement { .. } => "refinement",
                CheckedTerm::ExtensionForm { .. } => "extension_form",
                CheckedTerm::OperationAnchorBody(_) => "operation_anchor_body",
                CheckedTerm::NominalBody(_) => "nominal_body",
            };
            assert_eq!(actual, expected);
            if let CheckedTerm::Application { arguments, .. } = &term {
                let [CheckedTerm::Aggregate { members }] = arguments.as_slice() else {
                    panic!("application argument must be a typed aggregate");
                };
                assert!(matches!(members[0], CheckedTerm::Binding { .. }));
                assert!(matches!(members[1], CheckedTerm::Reference { .. }));
                assert!(matches!(
                    members[2],
                    CheckedTerm::DependencyReference { .. }
                ));
            }
            assert_eq!(serde_json::to_value(&term).expect("term serializes"), body);
            assert_eq!(
                to_vec(&term, Limits::new(u64::MAX)).expect("term encodes"),
                to_vec(&body, Limits::new(u64::MAX)).expect("wire encodes"),
                "typed {expected} canonical bytes"
            );
        }
        assert!(
            CheckedTerm::decode_admitted(&json!({"term":"group_reference","ordinal":0})).is_err(),
            "an internal group reference cannot become a wire term"
        );
    }

    #[trace("FR-349-AC-1", "FR-349-AC-3")]
    #[test]
    fn public_producer_helpers_build_typed_terms_and_visit_reference_targets() {
        let node = serde_json::from_value(json!({
            "package": "test-package", "path": ["node-a"], "ordinal": 0
        }))
        .expect("node identity");
        let replacement: crate::checked_package::shared::CheckedNodeId =
            serde_json::from_value(json!({
                "package": "test-package", "path": ["node-b"], "ordinal": 0
            }))
            .expect("replacement identity");
        let mut body = CheckedTerm::aggregate(vec![
            CheckedTerm::binding("member", CheckedTerm::reference(node)),
            CheckedTerm::literal_text(replacement.clone(), "plain text"),
        ]);
        let targets = body.reference_targets();
        assert_eq!(targets.len(), 1);
        let old = targets[0].clone();
        body.replace_node_id(&old, &replacement);
        assert_eq!(body.reference_targets(), [&replacement]);
        let wire = serde_json::to_value(&body).expect("producer term serializes");
        assert_eq!(
            wire["members"][0]["value"]["target"]["path"],
            json!(["node-b"])
        );
        assert_eq!(wire["members"][1]["value"], "plain text");

        let refinement_wire = json!({
            "term":"refinement", "abstract":old,
            "profile":{"authority":"test","identity":"trace"},
            "populations":[],"objects":[],
            "steps":[{"source_class":"finish","source_node":old,
                "target":{"kind":"stutter"}}],
            "histories":[],"assume":[],"ensure":[],
            "class_catalog":{"concrete":[{"class":"finish","node":old,
                "binder_types":[old]}],"abstract":[]}
        });
        let mut refinement = CheckedTerm::decode_admitted(&refinement_wire)
            .expect("typed refinement through the admitted decoder");
        refinement.replace_node_id(&old, &replacement);
        let wire = serde_json::to_value(&refinement).expect("refinement serializes");
        let replacement_wire = serde_json::to_value(&replacement).expect("node serializes");
        assert_eq!(wire["abstract"], replacement_wire);
        assert_eq!(wire["steps"][0]["source_node"], replacement_wire);
        assert_eq!(
            wire["class_catalog"]["concrete"][0]["node"],
            replacement_wire
        );
        assert_eq!(
            wire["class_catalog"]["concrete"][0]["binder_types"][0],
            replacement_wire
        );
        assert_eq!(wire["steps"][0]["source_class"], "finish");
    }
}
