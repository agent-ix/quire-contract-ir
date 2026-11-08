//! Structural nominal bodies (enum declaration, enum value, dimension,
//! dimension equivalence and unit) and their semantic and cross-field rules.

/// Borrowed, bounded unit and enum projections.
#[path = "nominal_reads.rs"]
pub mod nominal_reads;

use super::dependency_references::SuppliedDependencies;
use super::graph_storage::{GraphIndex, SortedTable};
use super::model_members::DomainModel;
use super::natural::coprime;
use super::term_view::TermNode;
use super::{
    BoundedDomainForm, CheckedNodeKind, ClaimForm, CompositeTypeForm, CorrespondenceForm,
    ExpressionForm, FunctionForm, ModelForm, ProtocolForm,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, RelationForm, ScalarTypeForm, StateForm,
    TemporalForm, ValueForm, WorkMeter,
};
use super::{CheckedPackageLockV2, RawCheckedPackageWireV2};
use crate::checked_package::common::{node_pointer, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodePathSegment, CheckedPackageRefusal, CheckedPackageRefusalCause,
    CheckedPackageRefusalCode, JsonPointer, RefusalFields,
};
use quire_exact::Integer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;

/// Reader-owned view assembled from a node's body, owner and declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NominalRecord {
    /// Enum declaration.
    EnumDeclaration(EnumDeclarationRecord),
    /// Enum value.
    EnumMember(EnumMemberRecord),
    /// Dimension.
    Dimension(DimensionRecord),
    /// Dimension equivalence.
    DimensionEquivalence(DimensionEquivalenceRecord),
    /// Unit.
    Unit(UnitRecord),
}

/// Owner subject of a nominal declaration; joins an exact lock selection.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NominalOwner {
    /// A locked raw source, joined by authority and identity.
    Source {
        /// Source authority.
        authority: Box<str>,
        /// Source identity.
        identity: Box<str>,
    },
    /// A declaration in a selected domain package (QSpec `ModelOwner`),
    /// joined by the package identity; `node` is the IR node identity inside
    /// that package.
    Model {
        /// Domain package identity.
        identity: Box<str>,
        /// IR node identity within the domain package.
        node: Box<str>,
    },
}

/// Enum declaration semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct EnumDeclarationRecord {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Whether member order is semantic.
    pub ordered: bool,
    /// Member identifiers.
    pub members: Vec<Box<str>>,
}

/// Enum member semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct EnumMemberRecord {
    /// Declaring enum node key.
    pub declaration_node_id: CheckedNodeId,
    /// Exact semantic case name; model variants also admit nonidentifier UTF-8.
    pub case: Box<str>,
    /// Whether the node's body is exactly the enum literal of this member:
    /// `{term: literal, type: <declaring enum>, value_kind: enum, value: case}`.
    pub body_is_literal: bool,
}

/// Dimension semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DimensionRecord {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// The admitted source or selected-model base declaration flag.
    pub base: bool,
    /// Base-dimension terms; an empty derived dimension is dimensionless.
    pub terms: Vec<DimensionTerm>,
}

/// One base-dimension power in a derived dimension.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionTerm {
    /// Base dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Canonical nonzero integer exponent.
    pub exponent: Box<str>,
}

/// Exact equivalence between a structurally ordered pair of dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DimensionEquivalenceRecord {
    /// Source containing the equivalence declaration.
    pub owner: SourceOwner,
    /// Exactly two distinct dimension IDs, in structural order.
    pub dimension_node_ids: [CheckedNodeId; 2],
    /// Authored local or directly selected package operands.
    pub operands: [CheckedDimensionOperand; 2],
    /// Positive reduced ratio from the first canonical root to the second.
    pub root_scale: CheckedRational,
}

/// Source owner of a dimension equivalence.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceOwner {
    /// The only admitted owner kind.
    pub kind: SourceOwnerKind,
    /// Source authority selected by the lock.
    pub authority: Box<str>,
    /// Source identity selected by the lock.
    pub identity: Box<str>,
}

/// Closed source-owner discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceOwnerKind {
    /// A locked source.
    Source,
}

/// Declared unit semantic view.
#[derive(Clone, Debug, Eq, PartialEq)]
struct UnitRecord {
    /// Declaring owner.
    pub owner: NominalOwner,
    /// ASCII identifier segments.
    pub qualified_declaration: Vec<Box<str>>,
    /// Dimension node key.
    pub dimension_node_id: CheckedNodeId,
    /// Target unit key; `None` (wire `null`, required member) for the root.
    pub target_unit_node_id: Option<CheckedNodeId>,
    /// Exact reduced scale to the target.
    pub scale: CheckedRational,
    /// Exact reduced offset to the target.
    pub offset: CheckedRational,
}

/// A canonical decimal rational.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, quire_canonical::FixedShape)]
#[serde(deny_unknown_fields)]
pub struct CheckedRational {
    /// Canonical integer string.
    pub numerator: Box<str>,
    /// Canonical positive integer string.
    pub denominator: Box<str>,
}

/// A structural ID with the declared-id wire shape: a name-only path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CheckedDeclaredNodeId(CheckedNodeId);

impl CheckedDeclaredNodeId {
    /// Validate the declared-id shape; package admission checks the target.
    pub fn try_new(node: CheckedNodeId) -> Result<Self, Box<CheckedPackageRefusal>> {
        if node.is_well_formed()
            && node
                .path
                .iter()
                .all(|part| matches!(part, CheckedNodePathSegment::Name(_)))
        {
            Ok(Self(node))
        } else {
            Err(Box::new(CheckedPackageRefusal::new(RefusalFields {
                code: CheckedPackageRefusalCode::MalformedWire,
                path: Some(JsonPointer::root()),
                cause: None,
                locus: None,
                contract_version: None,
                document_pointer: None,
            })))
        }
    }

    /// Borrow the exact structural ID of the declared node.
    pub fn node_id(&self) -> &CheckedNodeId {
        &self.0
    }
}

/// A declared-form node reference whose package selection is checked on admission.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedPackageNodeRef {
    /// Selected dependency package name.
    pub package: Box<str>,
    /// Declared structural node in that package.
    pub node: CheckedDeclaredNodeId,
}

/// A dimension equivalence operand retains its local or package wire form.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CheckedDimensionOperand {
    /// A node of the importing graph.
    Local(CheckedNodeId),
    /// A declared-form node reference checked against a direct dependency on admission.
    Package(CheckedPackageNodeRef),
}

impl CheckedDimensionOperand {
    fn node_id(&self) -> &CheckedNodeId {
        match self {
            Self::Local(id) => id,
            Self::Package(reference) => reference.node.node_id(),
        }
    }

    /// [`Self::from_value`] over a node of the retained document: the
    /// package name and the node ID are charged before they are allocated.
    fn from_term(
        value: TermNode<'_>,
        row: &[u64],
        meter: &mut WorkMeter,
    ) -> Result<Option<Self>, ValidationFailure> {
        if !value.is_object() {
            return Ok(None);
        }
        if value.exact_members(&["package", "node"]) {
            let Some(package) = value.get("package").and_then(TermNode::as_str) else {
                return Ok(None);
            };
            let Some(node) = value.get("node") else {
                return Ok(None);
            };
            let Some(node) = meter.read_id(node, row)?.spelled() else {
                return Ok(None);
            };
            let Ok(node) = CheckedDeclaredNodeId::try_new(node) else {
                return Ok(None);
            };
            Ok(Some(Self::Package(CheckedPackageNodeRef {
                package: meter.boxed_str(package)?,
                node,
            })))
        } else {
            Ok(meter.read_id(value, row)?.spelled().map(Self::Local))
        }
    }

    fn from_value(value: &Value) -> Option<Self> {
        use crate::checked_package::common::exact_members;
        let object = value.as_object()?;
        if exact_members(object, &["package", "node"]) {
            let package = object.get("package")?.as_str()?;
            let node =
                CheckedDeclaredNodeId::try_new(CheckedNodeId::from_value(object.get("node")?)?)
                    .ok()?;
            Some(Self::Package(CheckedPackageNodeRef {
                package: package.into(),
                node,
            }))
        } else {
            CheckedNodeId::from_value(value).map(Self::Local)
        }
    }
}

/// A validated closed body of a source-authored nominal node. Constructors
/// check body shape, returning `malformed_wire` with a body-relative pointer
/// on failure; the package reader checks owner and semantic joins.
///
/// ```
/// use quire_contract_model::{CheckedTerm, NominalBody};
/// let body = NominalBody::try_enum(false, vec!["Ready".into()]).expect("valid body");
/// let term = CheckedTerm::NominalBody(Box::new(body));
/// assert_eq!(serde_json::to_value(term).expect("serializes")["members"], serde_json::json!(["Ready"]));
/// ```
///
/// Raw variants and mutable fields cannot bypass the constructors.
/// ```compile_fail
/// use quire_contract_model::NominalBody;
/// let _ = NominalBody::Enum { ordered: false, members: vec!["Ready".into()] };
/// ```
/// ```compile_fail
/// use quire_contract_model::NominalBody;
/// let mut body = NominalBody::try_enum(false, vec!["Ready".into()]).unwrap();
/// body.kind = body.kind.clone();
/// ```
/// The borrowed enum details cannot mutate or construct a body unchecked.
/// ```compile_fail
/// use quire_contract_model::NominalBody;
/// let body = NominalBody::try_enum(false, vec!["Ready".into()]).unwrap();
/// let (_, members) = body.enum_details().unwrap();
/// members[0] = "Other".into();
/// ```
/// ```compile_fail
/// use quire_contract_model::NominalBody;
/// let body = NominalBody::try_enum(false, vec!["Ready".into()]).unwrap();
/// let details = body.enum_details().unwrap();
/// let _: NominalBody = details.into();
/// ```
/// Unchecked conversion and direct deserialization are also unavailable.
/// ```compile_fail
/// use quire_contract_model::NominalBody;
/// let _: NominalBody = serde_json::json!({"term":"enum","ordered":false,"members":["Ready"]}).into();
/// ```
/// ```compile_fail
/// use quire_contract_model::NominalBody;
/// let _: NominalBody = serde_json::from_str(r#"{"term":"enum","ordered":false,"members":["Ready"]}"#).unwrap();
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct NominalBody {
    kind: NominalBodyKind,
}

/// The closed body forms, hidden so callers cannot construct unchecked terms.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "term", rename_all = "snake_case")]
pub(super) enum NominalBodyKind {
    /// Source enum declaration, preserving authored member order.
    Enum {
        /// Whether the enum is ordered.
        ordered: bool,
        /// Declared case names.
        members: Vec<Box<str>>,
    },
    /// Dimension terms over structural dimension IDs.
    Dimension {
        /// The source declaration's explicit base flag.
        base: bool,
        /// Authored base-dimension terms.
        terms: Vec<DimensionTerm>,
    },
    /// Unit conversion to its target in the named dimension.
    Unit {
        /// Structural dimension ID.
        dimension_node_id: CheckedNodeId,
        /// Target unit, or null for the root unit.
        target_unit_node_id: Option<CheckedNodeId>,
        /// Exact scale to the target.
        scale: CheckedRational,
        /// Exact offset to the target.
        offset: CheckedRational,
    },
    /// Exact ratio between two structurally ordered dimensions.
    DimensionEquivalence {
        /// The authored local or selected-package operand pair.
        dimension_node_ids: [CheckedDimensionOperand; 2],
        /// Exact positive ratio of their canonical roots.
        root_scale: CheckedRational,
    },
}

impl NominalBody {
    /// The declared base flag of a dimension body. Other nominal forms have no flag.
    pub fn dimension_base(&self) -> Option<bool> {
        match &self.kind {
            NominalBodyKind::Dimension { base, .. } => Some(*base),
            _ => None,
        }
    }

    /// Borrow the admitted direct terms of a dimension declaration.
    pub(crate) fn dimension_details(&self) -> Option<&[DimensionTerm]> {
        match &self.kind {
            NominalBodyKind::Dimension { terms, .. } => Some(terms),
            _ => None,
        }
    }

    /// Borrow the selected dimension, target and exact affine data of a source unit.
    pub(crate) fn unit_details(
        &self,
    ) -> Option<(
        &CheckedNodeId,
        Option<&CheckedNodeId>,
        &CheckedRational,
        &CheckedRational,
    )> {
        match &self.kind {
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                scale,
                offset,
            } => Some((
                dimension_node_id,
                target_unit_node_id.as_ref(),
                scale,
                offset,
            )),
            _ => None,
        }
    }

    /// Borrow an enum's ordered flag and members in their admitted order.
    /// Other nominal body forms have no enum view.
    pub fn enum_details(&self) -> Option<(bool, &[Box<str>])> {
        match &self.kind {
            NominalBodyKind::Enum { ordered, members } => Some((*ordered, members)),
            NominalBodyKind::Dimension { .. }
            | NominalBodyKind::Unit { .. }
            | NominalBodyKind::DimensionEquivalence { .. } => None,
        }
    }

    /// Construct a source enum body, retaining declared case order and refusing
    /// an empty, duplicate or nonidentifier member.
    pub fn try_enum(
        ordered: bool,
        members: Vec<Box<str>>,
    ) -> Result<Self, Box<CheckedPackageRefusal>> {
        Self::try_kind(
            NominalBodyKind::Enum { ordered, members },
            JsonPointer::root(),
        )
    }

    /// Construct a base or derived dimension body, checking every structural
    /// ID and canonical integer exponent.
    pub fn try_dimension(
        base: bool,
        terms: Vec<DimensionTerm>,
    ) -> Result<Self, Box<CheckedPackageRefusal>> {
        Self::try_kind(
            NominalBodyKind::Dimension { base, terms },
            JsonPointer::root(),
        )
    }

    /// Construct a unit body; an absent target is the canonical root unit.
    /// IDs and rational integer spellings must have the closed V2 shape.
    pub fn try_unit(
        dimension_node_id: CheckedNodeId,
        target_unit_node_id: Option<CheckedNodeId>,
        scale: CheckedRational,
        offset: CheckedRational,
    ) -> Result<Self, Box<CheckedPackageRefusal>> {
        Self::try_kind(
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                scale,
                offset,
            },
            JsonPointer::root(),
        )
    }

    /// Construct an equivalence body with two well-formed dimension operands
    /// and a positive root-scale numerator.
    pub fn try_dimension_equivalence(
        dimension_node_ids: [CheckedDimensionOperand; 2],
        root_scale: CheckedRational,
    ) -> Result<Self, Box<CheckedPackageRefusal>> {
        Self::try_kind(
            NominalBodyKind::DimensionEquivalence {
                dimension_node_ids,
                root_scale,
            },
            JsonPointer::root(),
        )
    }

    fn try_kind(
        kind: NominalBodyKind,
        at: JsonPointer,
    ) -> Result<Self, Box<CheckedPackageRefusal>> {
        Self::check_kind(&kind, &|| at.clone(), false)?;
        Ok(Self { kind })
    }

    /// The closed rules of a body, each path built only for a refusal.
    /// `names_checked` says the enum names were already checked to be
    /// distinct identifiers, as the charged reader does while it reads them.
    fn check_kind(
        kind: &NominalBodyKind,
        at: &dyn Fn() -> JsonPointer,
        names_checked: bool,
    ) -> Result<(), Box<CheckedPackageRefusal>> {
        let malformed = |path| {
            Box::new(CheckedPackageRefusal::new(RefusalFields {
                code: CheckedPackageRefusalCode::MalformedWire,
                path: Some(path),
                cause: None,
                locus: None,
                contract_version: None,
                document_pointer: None,
            }))
        };
        let valid_id = |id: &CheckedNodeId, path: &dyn Fn() -> JsonPointer| {
            id.is_well_formed()
                .then_some(())
                .ok_or_else(|| malformed(path()))
        };
        let valid_rational =
            |value: &CheckedRational, path: &dyn Fn() -> JsonPointer, positive: bool| {
                let numerator_ok = if positive {
                    positive_integer(&value.numerator)
                } else {
                    integer_magnitude(&value.numerator).is_some()
                };
                if !numerator_ok {
                    return Err(malformed(path().key("numerator")));
                }
                if !positive_integer(&value.denominator) {
                    return Err(malformed(path().key("denominator")));
                }
                Ok(())
            };
        match kind {
            NominalBodyKind::Enum { members, .. } => {
                if members.is_empty() {
                    return Err(malformed(at().key("members")));
                }
                if !names_checked {
                    if let Some(index) = first_bad_name(members.iter().map(AsRef::as_ref)) {
                        return Err(malformed(at().key("members").index(index)));
                    }
                }
            }
            NominalBodyKind::Dimension { base, terms } => {
                if *base && !terms.is_empty() {
                    return Err(malformed(at().key("terms")));
                }
                for (index, term) in terms.iter().enumerate() {
                    let path = || at().key("terms").index(index);
                    valid_id(&term.dimension_node_id, &|| path().key("dimension_node_id"))?;
                    if integer_magnitude(&term.exponent).is_none() {
                        return Err(malformed(path().key("exponent")));
                    }
                }
            }
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                scale,
                offset,
            } => {
                valid_id(dimension_node_id, &|| at().key("dimension_node_id"))?;
                if let Some(target) = target_unit_node_id {
                    valid_id(target, &|| at().key("target_unit_node_id"))?;
                }
                valid_rational(scale, &|| at().key("scale"), false)?;
                valid_rational(offset, &|| at().key("offset"), false)?;
            }
            NominalBodyKind::DimensionEquivalence {
                dimension_node_ids,
                root_scale,
            } => {
                for (index, operand) in dimension_node_ids.iter().enumerate() {
                    let path = || at().key("dimension_node_ids").index(index);
                    valid_id(operand.node_id(), &path)?;
                    if let CheckedDimensionOperand::Package(reference) = operand {
                        if reference.package.is_empty() {
                            return Err(malformed(path()));
                        }
                    }
                }
                valid_rational(root_scale, &|| at().key("root_scale"), true)?;
            }
        }
        Ok(())
    }

    pub(super) fn kind(&self) -> &NominalBodyKind {
        &self.kind
    }

    fn into_kind(self) -> NominalBodyKind {
        self.kind
    }

    /// Decode without cloning a caller-authored nested NodeId path. The
    /// NodeId decoder itself walks those paths on the heap.
    #[ix_trace_rs::string_edge]
    pub(super) fn from_value(body: &Value, at: JsonPointer) -> Result<Self, ValidationFailure> {
        use crate::checked_package::common::exact_members;
        let malformed =
            |path| ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, path);
        let object = body.as_object().ok_or_else(|| malformed(at.clone()))?;
        let term = object
            .get("term")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed(at.clone().key("term")))?;
        let node_ref = |member: &str| {
            object
                .get(member)
                .and_then(CheckedNodeId::from_value)
                .ok_or_else(|| malformed(at.clone().key(member)))
        };
        let rational = |member: &str, positive_numerator: bool| {
            let path = at.clone().key(member);
            let value = object.get(member).ok_or_else(|| malformed(path.clone()))?;
            let fields = value.as_object().ok_or_else(|| malformed(path.clone()))?;
            if !exact_members(fields, &["numerator", "denominator"]) {
                return Err(malformed(path));
            }
            let numerator = fields
                .get("numerator")
                .and_then(Value::as_str)
                .filter(|number| {
                    if positive_numerator {
                        positive_integer(number)
                    } else {
                        integer_magnitude(number).is_some()
                    }
                })
                .ok_or_else(|| malformed(path.clone().key("numerator")))?;
            let denominator = fields
                .get("denominator")
                .and_then(Value::as_str)
                .filter(|number| positive_integer(number))
                .ok_or_else(|| malformed(path.clone().key("denominator")))?;
            Ok(CheckedRational {
                numerator: numerator.into(),
                denominator: denominator.into(),
            })
        };
        let kind = match term {
            "enum" if exact_members(object, &["term", "ordered", "members"]) => {
                let ordered = object["ordered"]
                    .as_bool()
                    .ok_or_else(|| malformed(at.clone().key("ordered")))?;
                let values = object["members"]
                    .as_array()
                    .filter(|items| !items.is_empty())
                    .ok_or_else(|| malformed(at.clone().key("members")))?;
                let mut members: Vec<Box<str>> = Vec::with_capacity(values.len());
                for (index, value) in values.iter().enumerate() {
                    let member = value
                        .as_str()
                        .ok_or_else(|| malformed(at.clone().key("members").index(index)))?;
                    members.push(member.into());
                }
                if let Some(index) = first_bad_name(members.iter().map(AsRef::as_ref)) {
                    return Err(malformed(at.clone().key("members").index(index)));
                }
                Ok(NominalBodyKind::Enum { ordered, members })
            }
            "dimension" if exact_members(object, &["term", "base", "terms"]) => {
                let base = object["base"]
                    .as_bool()
                    .ok_or_else(|| malformed(at.clone().key("base")))?;
                let values = object["terms"]
                    .as_array()
                    .ok_or_else(|| malformed(at.clone().key("terms")))?;
                let mut terms = Vec::with_capacity(values.len());
                for (index, value) in values.iter().enumerate() {
                    let path = at.clone().key("terms").index(index);
                    let fields = value.as_object().ok_or_else(|| malformed(path.clone()))?;
                    if !exact_members(fields, &["dimension_node_id", "exponent"]) {
                        return Err(malformed(path));
                    }
                    let dimension_node_id = fields["dimension_node_id"]
                        .as_object()
                        .and_then(|_| CheckedNodeId::from_value(&fields["dimension_node_id"]))
                        .ok_or_else(|| malformed(path.clone().key("dimension_node_id")))?;
                    let exponent = fields["exponent"]
                        .as_str()
                        .filter(|text| integer_magnitude(text).is_some())
                        .ok_or_else(|| malformed(path.clone().key("exponent")))?;
                    terms.push(DimensionTerm {
                        dimension_node_id,
                        exponent: exponent.into(),
                    });
                }
                Ok(NominalBodyKind::Dimension { base, terms })
            }
            "unit"
                if exact_members(
                    object,
                    &[
                        "term",
                        "dimension_node_id",
                        "target_unit_node_id",
                        "scale",
                        "offset",
                    ],
                ) =>
            {
                let target_unit_node_id = match &object["target_unit_node_id"] {
                    Value::Null => None,
                    value => Some(
                        CheckedNodeId::from_value(value)
                            .ok_or_else(|| malformed(at.clone().key("target_unit_node_id")))?,
                    ),
                };
                Ok(NominalBodyKind::Unit {
                    dimension_node_id: node_ref("dimension_node_id")?,
                    target_unit_node_id,
                    scale: rational("scale", false)?,
                    offset: rational("offset", false)?,
                })
            }
            "dimension_equivalence"
                if exact_members(object, &["term", "dimension_node_ids", "root_scale"]) =>
            {
                let values = object["dimension_node_ids"]
                    .as_array()
                    .filter(|items| items.len() == 2)
                    .ok_or_else(|| malformed(at.clone().key("dimension_node_ids")))?;
                Ok(NominalBodyKind::DimensionEquivalence {
                    dimension_node_ids: [
                        CheckedDimensionOperand::from_value(&values[0]).ok_or_else(|| {
                            malformed(at.clone().key("dimension_node_ids").index(0))
                        })?,
                        CheckedDimensionOperand::from_value(&values[1]).ok_or_else(|| {
                            malformed(at.clone().key("dimension_node_ids").index(1))
                        })?,
                    ],
                    root_scale: rational("root_scale", true)?,
                })
            }
            _ => Err(malformed(at.clone())),
        }?;
        Self::try_kind(kind, at).map_err(ValidationFailure::Refused)
    }

    /// [`Self::from_value`] over a node of the retained document, whose
    /// operand slots read through `row`. Every list, name, number text and ID
    /// the body keeps is charged to the meter before it is allocated, and
    /// the same first defect is refused at the same body-relative path.
    #[ix_trace_rs::string_edge]
    pub(super) fn from_term(
        body: TermNode<'_>,
        row: &[u64],
        at: &dyn Fn() -> JsonPointer,
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let malformed =
            |path| ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, path);
        if !body.is_object() {
            return Err(malformed(at()));
        }
        let term = body
            .get("term")
            .and_then(TermNode::as_str)
            .ok_or_else(|| malformed(at().key("term")))?;
        let node_ref = |meter: &mut WorkMeter, member: &str| {
            let read = match body.get(member) {
                Some(value) => meter.read_id(value, row)?.spelled(),
                None => None,
            };
            read.ok_or_else(|| malformed(at().key(member)))
        };
        let rational = |meter: &mut WorkMeter, member: &str, positive_numerator: bool| {
            let path = || at().key(member);
            let value = body.get(member).ok_or_else(|| malformed(path()))?;
            if !value.is_object() {
                return Err(malformed(path()));
            }
            if !value.exact_members(&["numerator", "denominator"]) {
                return Err(malformed(path()));
            }
            let numerator = value
                .get("numerator")
                .and_then(TermNode::as_str)
                .filter(|number| {
                    if positive_numerator {
                        positive_integer(number)
                    } else {
                        integer_magnitude(number).is_some()
                    }
                })
                .ok_or_else(|| malformed(path().key("numerator")))?;
            let denominator = value
                .get("denominator")
                .and_then(TermNode::as_str)
                .filter(|number| positive_integer(number))
                .ok_or_else(|| malformed(path().key("denominator")))?;
            Ok(CheckedRational {
                numerator: meter.boxed_str(numerator)?,
                denominator: meter.boxed_str(denominator)?,
            })
        };
        let kind = match term {
            "enum" if body.exact_members(&["term", "ordered", "members"]) => {
                let ordered = body
                    .get("ordered")
                    .and_then(TermNode::as_bool)
                    .ok_or_else(|| malformed(at().key("ordered")))?;
                let values = body
                    .get("members")
                    .and_then(TermNode::items)
                    .filter(|items| items.len() != 0)
                    .ok_or_else(|| malformed(at().key("members")))?;
                // The distinct names, sorted, so a repeat is found without a
                // tree.
                let mut unique: Vec<&str> = meter.vec(values.len())?;
                let mut members = meter.vec(values.len())?;
                for (index, value) in values.enumerate() {
                    let member = value
                        .as_str()
                        .filter(|text| is_identifier(text))
                        .ok_or_else(|| malformed(at().key("members").index(index)))?;
                    if !meter.insert_sorted(&mut unique, member)? {
                        return Err(malformed(at().key("members").index(index)));
                    }
                    members.push(meter.boxed_str(member)?);
                }
                Ok(NominalBodyKind::Enum { ordered, members })
            }
            "dimension" if body.exact_members(&["term", "base", "terms"]) => {
                let base = body
                    .get("base")
                    .and_then(TermNode::as_bool)
                    .ok_or_else(|| malformed(at().key("base")))?;
                let values = body
                    .get("terms")
                    .and_then(TermNode::items)
                    .ok_or_else(|| malformed(at().key("terms")))?;
                let mut terms = meter.vec(values.len())?;
                for (index, value) in values.enumerate() {
                    let path = || at().key("terms").index(index);
                    if !value.is_object()
                        || !value.exact_members(&["dimension_node_id", "exponent"])
                    {
                        return Err(malformed(path()));
                    }
                    let dimension_node_id = match value.get("dimension_node_id") {
                        Some(id) => meter.read_id(id, row)?.spelled(),
                        None => None,
                    }
                    .ok_or_else(|| malformed(path().key("dimension_node_id")))?;
                    let exponent = value
                        .get("exponent")
                        .and_then(TermNode::as_str)
                        .filter(|text| integer_magnitude(text).is_some())
                        .ok_or_else(|| malformed(path().key("exponent")))?;
                    terms.push(DimensionTerm {
                        dimension_node_id,
                        exponent: meter.boxed_str(exponent)?,
                    });
                }
                Ok(NominalBodyKind::Dimension { base, terms })
            }
            "unit"
                if body.exact_members(&[
                    "term",
                    "dimension_node_id",
                    "target_unit_node_id",
                    "scale",
                    "offset",
                ]) =>
            {
                let target = body.get("target_unit_node_id");
                let target_unit_node_id = match target {
                    Some(value) if value.is_null() => None,
                    Some(value) => Some(
                        meter
                            .read_id(value, row)?
                            .spelled()
                            .ok_or_else(|| malformed(at().key("target_unit_node_id")))?,
                    ),
                    None => return Err(malformed(at().key("target_unit_node_id"))),
                };
                Ok(NominalBodyKind::Unit {
                    dimension_node_id: node_ref(meter, "dimension_node_id")?,
                    target_unit_node_id,
                    scale: rational(meter, "scale", false)?,
                    offset: rational(meter, "offset", false)?,
                })
            }
            "dimension_equivalence"
                if body.exact_members(&["term", "dimension_node_ids", "root_scale"]) =>
            {
                let values = body
                    .get("dimension_node_ids")
                    .and_then(TermNode::items)
                    .filter(|items| items.len() == 2)
                    .ok_or_else(|| malformed(at().key("dimension_node_ids")))?;
                let mut values = values;
                let mut operand = |meter: &mut WorkMeter, index: usize| {
                    let value = values
                        .next()
                        .ok_or_else(|| malformed(at().key("dimension_node_ids")))?;
                    CheckedDimensionOperand::from_term(value, row, meter)?
                        .ok_or_else(|| malformed(at().key("dimension_node_ids").index(index)))
                };
                let first = operand(meter, 0)?;
                let second = operand(meter, 1)?;
                Ok(NominalBodyKind::DimensionEquivalence {
                    dimension_node_ids: [first, second],
                    root_scale: rational(meter, "root_scale", true)?,
                })
            }
            _ => Err(malformed(at())),
        }?;
        Self::check_kind(&kind, at, true).map_err(ValidationFailure::Refused)?;
        Ok(Self { kind })
    }

    pub(super) fn replace_node_id(&mut self, old: &CheckedNodeId, new: &CheckedNodeId) {
        let replace = |id: &mut CheckedNodeId| {
            if id == old {
                *id = new.clone();
            }
        };
        match &mut self.kind {
            NominalBodyKind::Enum { .. } => {}
            NominalBodyKind::Dimension { terms, .. } => {
                for term in terms {
                    replace(&mut term.dimension_node_id);
                }
            }
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                replace(dimension_node_id);
                if let Some(id) = target_unit_node_id {
                    replace(id);
                }
            }
            NominalBodyKind::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for operand in dimension_node_ids {
                    if let CheckedDimensionOperand::Local(id) = operand {
                        replace(id);
                    }
                }
            }
        }
    }

    pub(super) fn map_node_ids(
        &mut self,
        map: &mut impl FnMut(&CheckedNodeId) -> Option<CheckedNodeId>,
    ) {
        let mut visit = |id: &mut CheckedNodeId| {
            if let Some(replacement) = map(id) {
                *id = replacement;
            }
        };
        match &mut self.kind {
            NominalBodyKind::Enum { .. } => {}
            NominalBodyKind::Dimension { terms, .. } => {
                for term in terms {
                    visit(&mut term.dimension_node_id);
                }
            }
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                visit(dimension_node_id);
                if let Some(id) = target_unit_node_id {
                    visit(id);
                }
            }
            NominalBodyKind::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for operand in dimension_node_ids {
                    if let CheckedDimensionOperand::Local(id) = operand {
                        visit(id);
                    }
                }
            }
        }
    }

    pub(super) fn visit_reference_targets<'a>(
        &'a self,
        visitor: &mut impl FnMut(&'a CheckedNodeId),
    ) {
        self.try_visit_reference_targets(&mut |id| {
            visitor(id);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {});
    }

    /// Visit each flat nominal reference and stop immediately on visitor refusal.
    pub(super) fn try_visit_reference_targets<'a, E>(
        &'a self,
        visitor: &mut impl FnMut(&'a CheckedNodeId) -> Result<(), E>,
    ) -> Result<(), E> {
        match &self.kind {
            NominalBodyKind::Enum { .. } => {}
            NominalBodyKind::Dimension { terms, .. } => {
                for term in terms {
                    visitor(&term.dimension_node_id)?;
                }
            }
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                ..
            } => {
                visitor(dimension_node_id)?;
                if let Some(id) = target_unit_node_id {
                    visitor(id)?;
                }
            }
            NominalBodyKind::DimensionEquivalence {
                dimension_node_ids, ..
            } => {
                for operand in dimension_node_ids {
                    if let CheckedDimensionOperand::Local(id) = operand {
                        visitor(id)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Which nominal semantic view a `(node_tag, semantic_form)` pair requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NominalKind {
    EnumDeclaration,
    EnumMember,
    Dimension,
    DimensionEquivalence,
    Unit,
}

impl NominalKind {
    /// The nominal semantic view a node of this kind requires, if any. Only the
    /// five nominal forms need one; every other form is decided here too, so
    /// a new form is a compile error until it is.
    fn required_by(kind: CheckedNodeKind) -> Option<Self> {
        use CheckedNodeKind as K;
        match kind {
            K::Extension(_) | K::Hybrid(_) | K::TimedModel(_) => None,
            K::ScalarType(ScalarTypeForm::Enum) => Some(Self::EnumDeclaration),
            K::Value(ValueForm::EnumValue) => Some(Self::EnumMember),
            K::ScalarType(ScalarTypeForm::Dimension) => Some(Self::Dimension),
            K::ScalarType(ScalarTypeForm::DimensionEquivalence) => Some(Self::DimensionEquivalence),
            K::ScalarType(ScalarTypeForm::Unit) => Some(Self::Unit),
            K::ScalarType(
                ScalarTypeForm::Boolean
                | ScalarTypeForm::Integer
                | ScalarTypeForm::Rational
                | ScalarTypeForm::Decimal
                | ScalarTypeForm::Float32
                | ScalarTypeForm::Float64
                | ScalarTypeForm::Text
                | ScalarTypeForm::String
                | ScalarTypeForm::Uuid
                | ScalarTypeForm::Timestamp
                | ScalarTypeForm::CompoundUnit
                | ScalarTypeForm::CarriedQuantity
                | ScalarTypeForm::BoundedQuantity,
            ) => None,
            K::CompositeType(
                CompositeTypeForm::Option
                | CompositeTypeForm::Sequence
                | CompositeTypeForm::Set
                | CompositeTypeForm::Bag
                | CompositeTypeForm::OrderedSet
                | CompositeTypeForm::Map
                | CompositeTypeForm::Record
                | CompositeTypeForm::Tuple
                | CompositeTypeForm::Union
                | CompositeTypeForm::Alias
                | CompositeTypeForm::Reference,
            ) => None,
            K::BoundedDomain(
                BoundedDomainForm::IntegerRange
                | BoundedDomainForm::RationalRange
                | BoundedDomainForm::DecimalRange
                | BoundedDomainForm::FloatRounding
                | BoundedDomainForm::TextBounds
                | BoundedDomainForm::CollectionBounds
                | BoundedDomainForm::ModelPopulation,
            ) => None,
            K::Value(
                ValueForm::Literal
                | ValueForm::CollectionValue
                | ValueForm::RecordValue
                | ValueForm::TupleValue
                | ValueForm::UnionValue
                | ValueForm::OptionValue
                | ValueForm::Parameter,
            ) => None,
            K::Expression(
                ExpressionForm::Reference
                | ExpressionForm::Call
                | ExpressionForm::Unary
                | ExpressionForm::Binary
                | ExpressionForm::Conditional
                | ExpressionForm::Case
                | ExpressionForm::Let
                | ExpressionForm::Quantify
                | ExpressionForm::Collection
                | ExpressionForm::Conversion
                | ExpressionForm::Query
                | ExpressionForm::PreRead
                | ExpressionForm::PresenceRead
                | ExpressionForm::ValueRead
                | ExpressionForm::Deref
                | ExpressionForm::Reachability,
            ) => None,
            K::Function(
                FunctionForm::PureFunction
                | FunctionForm::Predicate
                | FunctionForm::RecursiveFunction,
            ) => None,
            K::Model(
                ModelForm::ModelRoot
                | ModelForm::ModelImport
                | ModelForm::ObjectType
                | ModelForm::ValueType
                | ModelForm::VariantType
                | ModelForm::TupleValueType
                | ModelForm::RecordValueType
                | ModelForm::EventType
                | ModelForm::StateMachine
                | ModelForm::Process
                | ModelForm::PersistenceInterface
                | ModelForm::Namespace
                | ModelForm::SystemsInterface
                | ModelForm::SystemsPart
                | ModelForm::SystemsPort
                | ModelForm::SystemsConnection
                | ModelForm::SystemsAllocation,
            ) => None,
            K::Relation(
                RelationForm::Relationship
                | RelationForm::Population
                | RelationForm::Membership
                | RelationForm::CausalRelation
                | RelationForm::Refinement,
            ) => None,
            K::State(
                StateForm::StateClause
                | StateForm::Frame
                | StateForm::Transition
                | StateForm::OperationAnchor
                | StateForm::Snapshot,
            ) => None,
            K::Temporal(
                TemporalForm::TemporalClause
                | TemporalForm::Formula
                | TemporalForm::Fairness
                | TemporalForm::Clock
                | TemporalForm::Window
                | TemporalForm::Activation
                | TemporalForm::Deadline
                | TemporalForm::Terminal
                | TemporalForm::MonitorPlan,
            ) => None,
            K::Protocol(
                ProtocolForm::ProtocolClause
                | ProtocolForm::ProtocolReference
                | ProtocolForm::Role
                | ProtocolForm::Channel
                | ProtocolForm::Queue
                | ProtocolForm::Control
                | ProtocolForm::Obligation
                | ProtocolForm::Compensation,
            ) => None,
            K::Claim(
                ClaimForm::VerificationClaim
                | ClaimForm::AnalysisClaim
                | ClaimForm::Hyperproperty
                | ClaimForm::SynthesisRequest
                | ClaimForm::StateGraph
                | ClaimForm::StepRelation
                | ClaimForm::Schedulability
                | ClaimForm::ProbabilisticClaim,
            ) => None,
            K::Correspondence(
                CorrespondenceForm::SourceLocus
                | CorrespondenceForm::ModelCorrespondence
                | CorrespondenceForm::BindingRole
                | CorrespondenceForm::ProfileCorrespondence
                | CorrespondenceForm::AbstractionRelation,
            ) => None,
        }
    }

    fn of(record: &NominalRecord) -> Self {
        match record {
            NominalRecord::EnumDeclaration(_) => Self::EnumDeclaration,
            NominalRecord::EnumMember(_) => Self::EnumMember,
            NominalRecord::Dimension(_) => Self::Dimension,
            NominalRecord::DimensionEquivalence(_) => Self::DimensionEquivalence,
            NominalRecord::Unit(_) => Self::Unit,
        }
    }
}

fn invalid(path: JsonPointer) -> ValidationFailure {
    ValidationFailure::refused(CheckedPackageRefusalCode::InvalidSemanticGraph, path)
}

/// Refuses at the pointer `at` builds unless `condition` holds.
fn require(condition: bool, at: impl FnOnce() -> JsonPointer) -> Result<(), ValidationFailure> {
    if condition {
        Ok(())
    } else {
        Err(invalid(at()))
    }
}

/// One node under validation: the node, its graph position and pointers to
/// the members of its body, owner and declaration.
struct Site<'a> {
    node: &'a CheckedSemanticNodeV2,
    position: usize,
}

impl Site<'_> {
    fn pointer(&self) -> JsonPointer {
        node_pointer(self.position)
    }

    fn member(&self, member: &str) -> JsonPointer {
        node_pointer(self.position).key(member)
    }

    #[ix_trace_rs::string_edge]
    fn nominal_member(&self, members: &[&str]) -> JsonPointer {
        let (base, rest) = match members.split_first() {
            Some((first, rest)) if *first == "owner" => (self.member("owner"), rest),
            Some((first, rest)) if *first == "qualified_declaration" => {
                (self.member("declaration").key("qualified_name"), rest)
            }
            Some((first, rest)) if *first == "case" => (self.member("body").key("value"), rest),
            Some((first, rest)) if *first == "declaration_node_id" => {
                (self.member("semantic_type"), rest)
            }
            _ => (self.member("body"), members),
        };
        rest.iter()
            .fold(base, |pointer, member| pointer.key(member))
    }
}

type ModelNodes<'a> = SortedTable<(&'a str, &'a str), &'a CheckedNodeId>;

/// The node of each selected model declaration, by package identity and
/// declaration, as an exact-capacity sorted table charged first. Of two
/// nodes naming one declaration the later wins, as a map built by
/// insertion would hold.
fn model_node_table<'a>(
    nodes: impl Iterator<Item = (Option<&'a super::CheckedNodeOwner>, &'a CheckedNodeId)> + Clone,
    meter: &mut WorkMeter,
) -> Result<ModelNodes<'a>, ValidationFailure> {
    let owned = |(owner, id): (Option<&'a super::CheckedNodeOwner>, &'a CheckedNodeId)| match owner
    {
        Some(super::CheckedNodeOwner::Model {
            identity,
            node: owner_node,
        }) => Some(((identity.as_ref(), owner_node.as_ref()), id)),
        _ => None,
    };
    let count = nodes.clone().filter_map(owned).count();
    let mut entries = meter.vec(count)?;
    entries.extend(nodes.filter_map(owned));
    Ok(SortedTable::from_vec(entries, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    }))
}

/// Where the body of a node being read comes from: the retained document of
/// the graph being validated, or the copied `Value` of a node of an admitted
/// dependency package, which has no document here.
#[derive(Clone, Copy)]
enum BodyInput<'a> {
    Document(Option<(TermNode<'a>, &'a [u64])>),
    Value(&'a Value),
}

impl<'a> BodyInput<'a> {
    /// The body of graph node `position` of the graph being validated.
    fn of_graph(position: usize, meter: &WorkMeter<'a>) -> Self {
        Self::Document(meter.body(position))
    }

    /// The string member `name` of an object body.
    fn string_member(self, name: &str) -> Option<&'a str> {
        match self {
            Self::Document(body) => body?.0.get(name)?.as_str(),
            Self::Value(body) => body.as_object()?.get(name)?.as_str(),
        }
    }

    /// Whether the body is exactly the enum literal of `case` of the enum
    /// `declaration`: its `type` spells that node ID, read through the node's
    /// operand row with the ID's allocations charged.
    fn is_enum_literal(
        self,
        declaration: &CheckedNodeId,
        case: &str,
        position: usize,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        match self {
            Self::Document(Some((body, row))) => {
                if !(body.exact_members(&["term", "type", "value_kind", "value"])
                    && body.get("term").and_then(TermNode::as_str) == Some("literal")
                    && body.get("value_kind").and_then(TermNode::as_str) == Some("enum")
                    && body.get("value").and_then(TermNode::as_str) == Some(case))
                {
                    return Ok(false);
                }
                meter.at_node(position, "enum_member");
                let Some(spelled) = body.get("type") else {
                    return Ok(false);
                };
                Ok(meter.read_id(spelled, row)?.spelled().as_ref() == Some(declaration))
            }
            Self::Document(None) => Ok(false),
            Self::Value(body) => Ok(body.as_object().is_some_and(|body| {
                body.len() == 4
                    && body.get("term").and_then(Value::as_str) == Some("literal")
                    && body.get("value_kind").and_then(Value::as_str) == Some("enum")
                    && body.get("value").and_then(Value::as_str) == Some(case)
                    && body.get("type") == serde_json::to_value(declaration).ok().as_ref()
            })),
        }
    }

    /// Whether the body is exactly the empty aggregate term.
    fn is_empty_aggregate(self) -> bool {
        match self {
            Self::Document(body) => body.is_some_and(|(body, _)| {
                body.exact_members(&["term", "members"])
                    && body.get("term").and_then(TermNode::as_str) == Some("aggregate")
                    && body
                        .get("members")
                        .and_then(TermNode::items)
                        .is_some_and(|members| members.len() == 0)
            }),
            Self::Value(body) => body.as_object().is_some_and(|body| {
                body.len() == 2
                    && body.get("term").and_then(Value::as_str) == Some("aggregate")
                    && body
                        .get("members")
                        .and_then(Value::as_array)
                        .is_some_and(Vec::is_empty)
            }),
        }
    }

    /// The closed nominal body: a body outside the closed forms is refused at
    /// the node's body.
    fn nominal(
        self,
        position: usize,
        meter: &mut WorkMeter,
    ) -> Result<NominalBody, ValidationFailure> {
        let at = || node_pointer(position).key("body");
        match self {
            Self::Document(Some((body, row))) => NominalBody::from_term(body, row, &at, meter),
            Self::Document(None) => Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::MalformedWire,
                at(),
            )),
            Self::Value(body) => NominalBody::from_value(body, at()),
        }
    }
}

/// The closed nominal body of graph node `position`, read from the retained
/// document: a body outside the closed forms is refused at the node's body.
fn read_nominal_body(
    position: usize,
    meter: &mut WorkMeter,
) -> Result<NominalBody, ValidationFailure> {
    BodyInput::of_graph(position, meter).nominal(position, meter)
}

/// [`read_nominal_body`] for a caller that treats a body outside the closed
/// forms as absent; a refusal of the budget still stops the read.
pub(super) fn nominal_body_if_closed(
    position: usize,
    meter: &mut WorkMeter,
) -> Result<Option<NominalBody>, ValidationFailure> {
    match read_nominal_body(position, meter) {
        Ok(body) => Ok(Some(body)),
        Err(ValidationFailure::Refused(_)) => Ok(None),
        Err(stopped) => Err(stopped),
    }
}

fn record_from_node(
    node: &CheckedSemanticNodeV2,
    body: BodyInput<'_>,
    kind: NominalKind,
    position: usize,
    models: &[DomainModel],
    model_nodes: &ModelNodes<'_>,
    meter: &mut WorkMeter,
) -> Result<NominalRecord, ValidationFailure> {
    let at = || node_pointer(position);
    if kind == NominalKind::EnumMember {
        let case = body
            .string_member("value")
            .ok_or_else(|| invalid(at().key("body").key("value")))?;
        let body_is_literal = body.is_enum_literal(&node.semantic_type, case, position, meter)?;
        return Ok(NominalRecord::EnumMember(EnumMemberRecord {
            declaration_node_id: meter.clone_id(&node.semantic_type)?,
            case: meter.boxed_str(case)?,
            body_is_literal,
        }));
    }
    if let Some(super::CheckedNodeOwner::Model {
        identity,
        node: owner_node,
    }) = node.owner.as_ref()
    {
        return model_record_from_node(
            node,
            body,
            kind,
            position,
            identity,
            owner_node,
            models,
            model_nodes,
            meter,
        );
    }
    let source_owner = match node.owner.as_ref() {
        Some(super::CheckedNodeOwner::Source {
            authority,
            identity,
        }) => NominalOwner::Source {
            authority: meter.boxed_str(authority)?,
            identity: meter.boxed_str(identity)?,
        },
        Some(super::CheckedNodeOwner::Model { identity, node }) => NominalOwner::Model {
            identity: meter.boxed_str(identity)?,
            node: meter.boxed_str(node)?,
        },
        _ => return Err(invalid(at().key("owner"))),
    };
    let body = body.nominal(position, meter)?;
    let mut declared = || -> Result<Vec<Box<str>>, ValidationFailure> {
        let Some(declaration) = node.declaration.as_ref() else {
            return Err(invalid(at().key("declaration")));
        };
        let mut name = meter.vec(declaration.qualified_name.len())?;
        for segment in &declaration.qualified_name {
            name.push(meter.boxed_str(segment)?);
        }
        Ok(name)
    };
    match (kind, body.into_kind()) {
        (NominalKind::EnumDeclaration, NominalBodyKind::Enum { ordered, members }) => {
            Ok(NominalRecord::EnumDeclaration(EnumDeclarationRecord {
                owner: source_owner,
                qualified_declaration: declared()?,
                ordered,
                members,
            }))
        }
        (NominalKind::Dimension, NominalBodyKind::Dimension { base, terms }) => {
            Ok(NominalRecord::Dimension(DimensionRecord {
                owner: source_owner,
                qualified_declaration: declared()?,
                base,
                terms,
            }))
        }
        (
            NominalKind::Unit,
            NominalBodyKind::Unit {
                dimension_node_id,
                target_unit_node_id,
                scale,
                offset,
            },
        ) => Ok(NominalRecord::Unit(UnitRecord {
            owner: source_owner,
            qualified_declaration: declared()?,
            dimension_node_id,
            target_unit_node_id,
            scale,
            offset,
        })),
        (
            NominalKind::DimensionEquivalence,
            NominalBodyKind::DimensionEquivalence {
                dimension_node_ids,
                root_scale,
            },
        ) => {
            let NominalOwner::Source {
                authority,
                identity,
            } = source_owner
            else {
                return Err(invalid(at().key("owner")));
            };
            Ok(NominalRecord::DimensionEquivalence(
                DimensionEquivalenceRecord {
                    owner: SourceOwner {
                        kind: SourceOwnerKind::Source,
                        authority,
                        identity,
                    },
                    dimension_node_ids: [
                        meter.clone_id(dimension_node_ids[0].node_id())?,
                        meter.clone_id(dimension_node_ids[1].node_id())?,
                    ],
                    operands: dimension_node_ids,
                    root_scale,
                },
            ))
        }
        _ => Err(invalid(at().key("body"))),
    }
}

/// A one-segment declaration name, charged first.
fn single_segment(name: &str, meter: &mut WorkMeter) -> Result<Vec<Box<str>>, ValidationFailure> {
    let mut segments = meter.vec(1)?;
    segments.push(meter.boxed_str(name)?);
    Ok(segments)
}

fn model_record_from_node(
    node: &CheckedSemanticNodeV2,
    body: BodyInput<'_>,
    kind: NominalKind,
    position: usize,
    identity: &str,
    owner_node: &str,
    models: &[DomainModel],
    model_nodes: &ModelNodes<'_>,
    meter: &mut WorkMeter,
) -> Result<NominalRecord, ValidationFailure> {
    use crate::checked_package::shared::CheckedNodePathSegment as Segment;
    let at = || node_pointer(position);
    let empty_aggregate = body.is_empty_aggregate();
    if node.declaration.is_some() || !empty_aggregate {
        return Err(ValidationFailure::refused(
            CheckedPackageRefusalCode::MalformedWire,
            at(),
        ));
    }
    let expected_names = [
        node.node_tag.as_ref(),
        node.semantic_form.as_ref(),
        owner_node,
    ];
    require(
        node.node_id.path.len() == expected_names.len()
            && node.node_id.path.iter().zip(expected_names).all(
                |(part, expected)| matches!(part, Segment::Name(name) if name.as_ref() == expected),
            ),
        || at().key("node_id").key("path"),
    )?;
    let model = models
        .iter()
        .find(|model| model.identity.as_ref() == identity)
        .ok_or_else(|| invalid(at().key("owner")))?;
    let resolve = |meter: &mut WorkMeter, fcd_identity: &str| match model_nodes
        .get(&(identity, fcd_identity))
    {
        Some(id) => meter.clone_id(id),
        None => Err(invalid(at().key("owner"))),
    };
    let owner = NominalOwner::Model {
        identity: meter.boxed_str(identity)?,
        node: meter.boxed_str(owner_node)?,
    };
    match kind {
        NominalKind::Dimension => {
            let selected = model
                .dimensions
                .get(owner_node)
                .ok_or_else(|| invalid(at().key("owner")))?;
            let mut terms = meter.vec(selected.terms.len())?;
            for (target, exponent) in &selected.terms {
                terms.push(DimensionTerm {
                    dimension_node_id: resolve(meter, target)?,
                    exponent: meter.boxed_str(exponent)?,
                });
            }
            Ok(NominalRecord::Dimension(DimensionRecord {
                owner,
                qualified_declaration: single_segment(&selected.name, meter)?,
                base: selected.base,
                terms,
            }))
        }
        NominalKind::Unit => {
            let selected = model
                .units
                .get(owner_node)
                .ok_or_else(|| invalid(at().key("owner")))?;
            let ratio = |text: &str, meter: &mut WorkMeter| {
                let (numerator, denominator) = text.split_once('/').unwrap_or((text, "1"));
                Ok::<_, ValidationFailure>(CheckedRational {
                    numerator: meter.boxed_str(numerator)?,
                    denominator: meter.boxed_str(denominator)?,
                })
            };
            let dimension_node_id = resolve(meter, &selected.dimension)?;
            let target_unit_node_id = match selected.target.as_deref() {
                Some(target) => Some(resolve(meter, target)?),
                None => None,
            };
            Ok(NominalRecord::Unit(UnitRecord {
                owner,
                qualified_declaration: single_segment(&selected.name, meter)?,
                dimension_node_id,
                target_unit_node_id,
                scale: ratio(&selected.scale, meter)?,
                offset: ratio(&selected.offset, meter)?,
            }))
        }
        _ => Err(invalid(at().key("owner"))),
    }
}

/// Validates every node's nominal owner join, semantic rules and cross-field
/// joins. `kinds` holds each node's decoded kind. Equivalence refusals point
/// at their node; other nominal refusals point at the member at fault.
pub(super) fn validate_nominal_nodes<'e>(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    lock: &CheckedPackageLockV2,
    models: &[DomainModel],
    dependencies: &SuppliedDependencies<'e>,
    meter: &mut WorkMeter,
    bytes: u64,
) -> Result<ImportedNominals<'e>, ValidationFailure> {
    let model_nodes = model_node_table(
        nodes
            .iter()
            .map(|node| (node.owner.as_ref(), &node.node_id)),
        meter,
    )?;
    let mut records: Vec<Option<NominalRecord>> = meter.vec(nodes.len())?;
    records.resize_with(nodes.len(), || None);
    for &position in index.values() {
        if let Some(required) = NominalKind::required_by(kinds[position]) {
            records[position] = Some(record_from_node(
                &nodes[position],
                BodyInput::of_graph(position, meter),
                required,
                position,
                models,
                &model_nodes,
                meter,
            )?);
        }
    }
    // Of two models with one identity the later one is kept.
    let mut identified = meter.vec::<(&str, &DomainModel)>(models.len())?;
    identified.extend(models.iter().map(|model| (model.identity.as_ref(), model)));
    let selected_models = SortedTable::from_vec(identified, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let imported = imported_nominals(dependencies, meter)?;
    let mut imported_ids = meter.vec(imported.len())?;
    imported_ids.extend(imported.iter().map(|(id, _)| *id));
    let graph = NominalGraph {
        nodes,
        kinds,
        index,
        records: &records,
        selected_models,
        imported,
    };
    let mut roots: Vec<&CheckedNodeId> = Vec::new();
    let mut equivalence_pairs: Vec<&[CheckedNodeId; 2]> = Vec::new();
    let mut equivalences = Vec::new();
    for &position in index.values() {
        let node = &nodes[position];
        let kind = kinds[position];
        let site = Site { node, position };
        meter.at_node(position, super::graph_storage::node_form(&node.node_tag));
        let required = NominalKind::required_by(kind);
        let record = match (required, records[position].as_ref()) {
            (None, None) => continue,
            (Some(kind), Some(record)) if NominalKind::of(record) == kind => record,
            // Required and absent: the node lacks the member.
            (Some(_), None) => return Err(invalid(node_pointer(position))),
            (Some(NominalKind::DimensionEquivalence), Some(_)) => {
                return Err(invalid(site.pointer()))
            }
            _ => return Err(invalid(site.nominal_member(&[]))),
        };
        meter.charge(1, || site.nominal_member(&[]))?;
        match record {
            NominalRecord::EnumDeclaration(declaration) => {
                validate_enum_declaration(declaration, &site, &graph, lock, meter)?;
            }
            NominalRecord::EnumMember(member) => {
                validate_enum_member(&site, member, &graph)?;
            }
            NominalRecord::Dimension(dimension) => {
                validate_dimension(&site, dimension, &graph, lock, meter, bytes)?;
            }
            NominalRecord::DimensionEquivalence(equivalence) => {
                validate_dimension_equivalence(
                    &site,
                    equivalence,
                    &graph,
                    dependencies,
                    lock,
                    meter,
                )?;
                let first =
                    meter.insert_sorted(&mut equivalence_pairs, &equivalence.dimension_node_ids)?;
                require(first, || site.pointer())?;
                meter.push(&mut equivalences, (position, equivalence))?;
            }
            NominalRecord::Unit(unit) => {
                validate_unit(&site, unit, &graph, lock, meter)?;
                if unit.target_unit_node_id.is_none() {
                    // A dimension has exactly one targetless root unit.
                    let first = meter.insert_sorted(&mut roots, &unit.dimension_node_id)?;
                    require(first, || site.nominal_member(&["target_unit_node_id"]))?;
                }
            }
        }
    }
    validate_equivalence_graph(&graph, &equivalences, meter)?;
    Ok(ImportedNominals(imported_ids))
}

/// The ids of the nominal nodes imported from admitted dependencies, in id
/// order, as one block charged before it is requested.
pub(super) struct ImportedNominals<'e>(Vec<&'e CheckedNodeId>);

impl ImportedNominals<'_> {
    pub(super) fn contains(&self, id: &CheckedNodeId) -> bool {
        self.0.binary_search_by(|held| (*held).cmp(id)).is_ok()
    }
}

/// Reuses admitted dependency meanings for cross-package nominal references.
/// Each reachable package and nominal node is visited once; the current read
/// charges those visits before any local equivalence comparison.
fn imported_nominals<'e>(
    dependencies: &SuppliedDependencies<'e>,
    meter: &mut WorkMeter,
) -> Result<SortedTable<&'e CheckedNodeId, NominalRecord>, ValidationFailure> {
    // Requests made here are charged to the imported nominal, by its position
    // in its own package, and the caller's site returns when the walk ends.
    let caller_site = meter.site();
    let mut pending = dependencies.packages();
    let mut seen: Vec<&str> = Vec::new();
    let mut records: Vec<(&CheckedNodeId, NominalRecord)> = Vec::new();
    while let Some(package) = pending.pop() {
        meter.charge(1, || JsonPointer::root().key("lock"))?;
        if !meter.insert_sorted(&mut seen, package.name())? {
            continue;
        }
        meter.room(&mut pending, package.direct_dependencies.len())?;
        pending.extend(package.direct_dependencies.iter().map(AsRef::as_ref));
        meter.charge(
            u64::try_from(package.resolved_graph().nodes.len()).unwrap_or(u64::MAX),
            || JsonPointer::root().key("semantic_graph").key("nodes"),
        )?;
        let model_nodes = model_node_table(
            package
                .resolved_graph()
                .nodes
                .iter()
                .map(|node| (node.owner.as_ref(), &node.node_id)),
            meter,
        )?;
        for (position, (node, kind)) in package
            .resolved_graph()
            .nodes
            .iter()
            .zip(package.node_kinds())
            .enumerate()
        {
            let Some(required) = NominalKind::required_by(*kind) else {
                continue;
            };
            meter.at_row("imported_nominals", position);
            let raw = node.to_raw();
            let record = record_from_node(
                &raw,
                BodyInput::Value(&raw.body),
                required,
                position,
                package.models.selected_models(),
                &model_nodes,
                meter,
            )?;
            meter.push(&mut records, (&node.node_id, record))?;
        }
    }
    meter.restore_site(caller_site);
    // A node id names one node, so each id holds its one record.
    Ok(SortedTable::from_vec(records, |_, _| Ordering::Equal))
}

fn validate_dimension_equivalence(
    site: &Site<'_>,
    equivalence: &DimensionEquivalenceRecord,
    graph: &NominalGraph<'_>,
    dependencies: &SuppliedDependencies<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let ids = &equivalence.dimension_node_ids;
    for (position, operand) in equivalence.operands.iter().enumerate() {
        let id = operand.node_id();
        let at = site.nominal_member(&["dimension_node_ids"]).index(position);
        meter.charge(1, || {
            site.nominal_member(&["dimension_node_ids"]).index(position)
        })?;
        match operand {
            CheckedDimensionOperand::Local(_) => {
                require(
                    id.package == site.node.node_id.package && graph.index.contains_key(id),
                    || at.clone(),
                )?;
            }
            CheckedDimensionOperand::Package(reference) => {
                let selected = dependencies
                    .selected_package(&reference.package)
                    .ok_or_else(|| {
                        ValidationFailure::refused_because(
                            CheckedPackageRefusalCode::MissingDeclaration,
                            at.clone().key("package"),
                            CheckedPackageRefusalCause::MissingSelection,
                        )
                    })?;
                if selected.node(id).is_none() {
                    return Err(ValidationFailure::refused_because(
                        CheckedPackageRefusalCode::MissingDeclaration,
                        at.clone().key("node"),
                        CheckedPackageRefusalCause::MissingName,
                    ));
                }
            }
        }
        require(
            matches!(graph.nominal(id), Some(NominalRecord::Dimension(_))),
            || at,
        )?;
    }
    require(ids[0] < ids[1], || site.pointer())?;
    meter.charge(
        u64::try_from(lock.sources.len()).unwrap_or(u64::MAX),
        || site.nominal_member(&["owner"]),
    )?;
    require(
        lock.sources.iter().any(|source| {
            source.authority == equivalence.owner.authority
                && source.identity == equivalence.owner.identity
        }),
        || site.pointer(),
    )?;
    validate_rational(&equivalence.root_scale, meter, &|| site.pointer())?;
    require(site.node.semantic_type == site.node.node_id, || {
        site.pointer()
    })?;
    let local_ids = equivalence
        .operands
        .iter()
        .filter_map(|operand| match operand {
            CheckedDimensionOperand::Local(id) => Some(id),
            CheckedDimensionOperand::Package(_) => None,
        });
    require(site.node.dependencies.iter().eq(local_ids), || {
        site.pointer()
    })?;
    require(
        matches!(site.node.occurrences.as_slice(), [occurrence]
            if occurrence.role == crate::checked_package::shared::CheckedOccurrenceRole::Anchor),
        || site.pointer(),
    )?;
    Ok(())
}

/// An exact ratio. Every operation charges the work meter, and every limb
/// block it needs the read-storage budget, before allocating or multiplying
/// integers from the untrusted package.
#[derive(Debug, Eq, PartialEq)]
struct ExactRatio {
    numerator: Integer,
    denominator: Integer,
}

/// The bytes a big-endian rendering of `value` takes, at least one.
fn byte_length(value: &Integer) -> u64 {
    value.magnitude_bits().div_ceil(8).max(1)
}

impl ExactRatio {
    fn zero() -> Self {
        Self {
            numerator: Integer::zero(),
            denominator: Integer::one(),
        }
    }

    fn one() -> Self {
        Self {
            numerator: Integer::one(),
            denominator: Integer::one(),
        }
    }

    fn parse(
        value: &CheckedRational,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        Self::parse_parts(&value.numerator, &value.denominator, meter, at)
    }

    /// The ratio `numerator / denominator` from its two decimal texts, read
    /// with the budget's exact-integer reservations; no text is copied.
    fn parse_parts(
        numerator: &str,
        denominator: &str,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        meter.charge(
            u64::try_from(numerator.len().saturating_add(denominator.len())).unwrap_or(u64::MAX),
            at,
        )?;
        let numerator = meter
            .exact_integer(numerator)?
            .ok_or_else(|| invalid(at().key("numerator")))?;
        let denominator = meter
            .exact_integer(denominator)?
            .ok_or_else(|| invalid(at().key("denominator")))?;
        if denominator.is_zero() {
            return Err(invalid(at().key("denominator")));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn duplicate(&self, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        Ok(Self {
            numerator: meter.exact_clone(&self.numerator)?,
            denominator: meter.exact_clone(&self.denominator)?,
        })
    }

    fn inverse(&self, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        Ok(Self {
            numerator: meter.exact_clone(&self.denominator)?,
            denominator: meter.exact_clone(&self.numerator)?,
        })
    }

    /// `numerator / denominator` divided through by their gcd.
    fn reduced(
        numerator: Integer,
        denominator: Integer,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let common = meter.exact_gcd(&numerator, &denominator)?;
        let (Some(numerator), Some(denominator)) = (
            meter.exact_div(&numerator, &common)?,
            meter.exact_div(&denominator, &common)?,
        ) else {
            return Err(invalid(at()));
        };
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn multiply(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let size = byte_length(&self.numerator)
            .saturating_add(byte_length(&self.denominator))
            .saturating_add(byte_length(&other.numerator))
            .saturating_add(byte_length(&other.denominator));
        meter.charge(size.saturating_mul(size).saturating_mul(size).max(1), at)?;
        let numerator = meter.exact_mul(&self.numerator, &other.numerator)?;
        let denominator = meter.exact_mul(&self.denominator, &other.denominator)?;
        Self::reduced(numerator, denominator, meter, at)
    }

    fn add(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let size = byte_length(&self.numerator)
            .saturating_add(byte_length(&self.denominator))
            .saturating_add(byte_length(&other.numerator))
            .saturating_add(byte_length(&other.denominator));
        meter.charge(size.saturating_mul(size).saturating_mul(size).max(1), at)?;
        let left = meter.exact_mul(&self.numerator, &other.denominator)?;
        let right = meter.exact_mul(&other.numerator, &self.denominator)?;
        let numerator = meter.exact_add(&left, &right)?;
        let denominator = meter.exact_mul(&self.denominator, &other.denominator)?;
        Self::reduced(numerator, denominator, meter, at)
    }

    fn divide(
        &self,
        other: &Self,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let inverse = other.inverse(meter)?;
        self.multiply(&inverse, meter, at)
    }

    fn power(
        &self,
        exponent: &str,
        meter: &mut WorkMeter,
        at: &dyn Fn() -> JsonPointer,
    ) -> Result<Self, ValidationFailure> {
        let (negative, magnitude) = match exponent.strip_prefix('-') {
            Some(magnitude) => (true, magnitude),
            None => (false, exponent),
        };
        meter.charge(u64::try_from(magnitude.len()).unwrap_or(u64::MAX), at)?;
        let mut exponent = meter
            .exact_integer(magnitude)?
            .ok_or_else(|| invalid(at()))?;
        let two = Integer::from(2_u64);
        let mut power = self.duplicate(meter)?;
        let mut result = Self::one();
        while !exponent.is_zero() {
            meter.charge(byte_length(&exponent), at)?;
            if !exponent.is_even() {
                result = result.multiply(&power, meter, at)?;
            }
            exponent = meter
                .exact_div(&exponent, &two)?
                .ok_or_else(|| invalid(at()))?;
            if !exponent.is_zero() {
                power = power.multiply(&power, meter, at)?;
            }
        }
        if negative {
            result.inverse(meter)
        } else {
            Ok(result)
        }
    }
}

/// Whether the named unit is its owner's one-second unit of its own base
/// Time dimension. The graph's nominal checks subsequently authenticate the
/// body, owner, and path; this projection only resolves the member's type.
#[ix_trace_rs::string_edge]
pub(super) fn duration_unit_key(
    nodes: &[CheckedSemanticNodeV2],
    name: &str,
    meter: &mut WorkMeter,
) -> Result<Option<CheckedNodeId>, ValidationFailure> {
    meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), || {
        JsonPointer::root().key("semantic_graph").key("nodes")
    })?;
    let Some((position, node)) = nodes.iter().enumerate().find(|(_, node)| {
        node.declaration.as_ref().is_some_and(|declaration| {
            name.strip_prefix("ix://quire/unit/")
                .is_some_and(|declared| joined_with_slashes(&declaration.qualified_name, declared))
        }) && node.node_tag.as_ref() == "scalar_type"
            && node.semantic_form.as_ref() == "unit"
    }) else {
        return Ok(None);
    };
    let Some(body) = nominal_body_if_closed(position, meter)? else {
        return Ok(None);
    };
    let NominalBodyKind::Unit {
        dimension_node_id,
        target_unit_node_id,
        scale: unit_scale,
        offset: unit_offset,
    } = body.into_kind()
    else {
        return Ok(None);
    };
    let owner_matches = match node.owner.as_ref() {
        Some(super::CheckedNodeOwner::Source { .. }) => name == "ix://quire/unit/s",
        Some(super::CheckedNodeOwner::Model { identity, .. }) => {
            name.strip_prefix("ix://quire/unit/")
                .and_then(|rest| rest.strip_suffix("/s"))
                == Some(identity.as_ref())
        }
        _ => false,
    };
    if !owner_matches {
        return Ok(None);
    }
    let at = || node_pointer(position);
    meter.at_node(position, "scalar_type");
    meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), at)?;
    let Some((dimension_position, dimension)) = nodes
        .iter()
        .enumerate()
        .find(|(_, candidate)| candidate.node_id == dimension_node_id)
    else {
        return Ok(None);
    };
    let Some(body) = nominal_body_if_closed(dimension_position, meter)? else {
        return Ok(None);
    };
    let NominalBodyKind::Dimension { base, .. } = body.into_kind() else {
        return Ok(None);
    };
    if dimension.owner != node.owner
        || dimension
            .declaration
            .as_ref()
            .map(|value| &value.qualified_name)
            .into_iter()
            .flatten()
            .last()
            .is_none_or(|name| name.as_ref() != "Time")
        || !base
    {
        return Ok(None);
    }
    let mut scale = ExactRatio::one();
    let mut offset = ExactRatio::zero();
    let mut current = (
        dimension_node_id,
        target_unit_node_id,
        unit_scale,
        unit_offset,
    );
    let mut seen: Vec<usize> = Vec::new();
    loop {
        meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), at)?;
        let edge_scale = ExactRatio::parse(&current.2, meter, &at)?;
        let edge_offset = ExactRatio::parse(&current.3, meter, &at)?;
        offset = offset
            .multiply(&edge_scale, meter, &at)?
            .add(&edge_offset, meter, &at)?;
        scale = scale.multiply(&edge_scale, meter, &at)?;
        let Some(target) = current.1.as_ref() else {
            break;
        };
        let Some((next_position, next)) = nodes
            .iter()
            .enumerate()
            .find(|(_, candidate)| &candidate.node_id == target)
        else {
            return Ok(None);
        };
        // A node id names one node, so a repeated target is a repeated position.
        if !meter.insert_sorted(&mut seen, next_position)? {
            return Ok(None);
        }
        let Some(body) = nominal_body_if_closed(next_position, meter)? else {
            return Ok(None);
        };
        let NominalBodyKind::Unit {
            dimension_node_id: next_dimension,
            target_unit_node_id: next_target,
            scale: next_scale,
            offset: next_offset,
        } = body.into_kind()
        else {
            return Ok(None);
        };
        if next.owner != node.owner || next_dimension != current.0 {
            return Ok(None);
        }
        current = (next_dimension, next_target, next_scale, next_offset);
    }
    if scale != ExactRatio::one() || offset != ExactRatio::zero() {
        return Ok(None);
    }
    Ok(Some(meter.clone_id(&node.node_id)?))
}

/// Resolve a selected model's `s` after its FCD document has been admitted.
/// The model-owned unit body is intentionally empty; conversion values come
/// from the selected declaration and the graph supplies exact structural IDs.
#[ix_trace_rs::string_edge]
pub(super) fn model_duration_unit_key(
    nodes: &[CheckedSemanticNodeV2],
    model: &DomainModel,
    meter: &mut WorkMeter,
) -> Result<Option<CheckedNodeId>, ValidationFailure> {
    let identity = model.identity.as_ref();
    let Some((unit_identity, unit)) = model
        .units
        .iter()
        .find(|(_, unit)| unit.name.as_ref() == "s")
    else {
        return Ok(None);
    };
    let Some(dimension) = model.dimensions.get(&unit.dimension) else {
        return Ok(None);
    };
    if dimension.name.as_ref() != "Time" || !dimension.base {
        return Ok(None);
    }
    let node_for = |owner_node: &str| {
        nodes.iter().find(|node| {
            matches!(
                node.owner.as_ref(),
                Some(super::CheckedNodeOwner::Model { identity: package, node: declaration })
                    if package.as_ref() == identity && declaration.as_ref() == owner_node
            )
        })
    };
    let Some(unit_node) = node_for(unit_identity) else {
        return Ok(None);
    };
    let Some(dimension_node) = node_for(&unit.dimension) else {
        return Ok(None);
    };
    if unit_node.semantic_type != dimension_node.node_id
        || dimension_node.node_tag.as_ref() != "scalar_type"
        || dimension_node.semantic_form.as_ref() != "dimension"
    {
        return Ok(None);
    }
    let at = || JsonPointer::root().key("semantic_graph").key("nodes");
    let mut scale = ExactRatio::one();
    let mut offset = ExactRatio::zero();
    let mut current = unit;
    let mut seen: Vec<&str> = Vec::new();
    loop {
        meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX), at)?;
        let ratio = |text: &str, meter: &mut WorkMeter| {
            let (numerator, denominator) = text.split_once('/').unwrap_or((text, "1"));
            ExactRatio::parse_parts(numerator, denominator, meter, &at)
        };
        let edge_scale = ratio(&current.scale, meter)?;
        let edge_offset = ratio(&current.offset, meter)?;
        offset = offset
            .multiply(&edge_scale, meter, &at)?
            .add(&edge_offset, meter, &at)?;
        scale = scale.multiply(&edge_scale, meter, &at)?;
        let Some(target) = current.target.as_deref() else {
            break;
        };
        if !meter.insert_sorted(&mut seen, target)? || node_for(target).is_none() {
            return Ok(None);
        }
        let Some(next) = model.units.get(target) else {
            return Ok(None);
        };
        if next.dimension != unit.dimension {
            return Ok(None);
        }
        current = next;
    }
    if scale != ExactRatio::one() || offset != ExactRatio::zero() {
        return Ok(None);
    }
    Ok(Some(meter.clone_id(&unit_node.node_id)?))
}

/// The canonical base component and exact root scale of each dimension an
/// equivalence reaches. Ids are held once, in order, as a charged exact
/// block; an entry names its component by that block's index.
struct Potentials<'a> {
    ids: Vec<&'a CheckedNodeId>,
    entries: Vec<Option<(usize, ExactRatio)>>,
}

impl<'a> Potentials<'a> {
    fn locate(&self, id: &CheckedNodeId) -> Option<usize> {
        self.ids.binary_search_by(|held| (*held).cmp(id)).ok()
    }

    /// The component and scale of `id`, when an equivalence reached it.
    fn get(&self, id: &CheckedNodeId) -> Option<(&'a CheckedNodeId, &ExactRatio)> {
        let at = self.locate(id)?;
        let (component, scale) = self.entries.get(at)?.as_ref()?;
        Some((self.ids[*component], scale))
    }
}

/// One equivalence edge as the walk takes it: from, to, the multiplier, the
/// node it was written at, and its place in the order the edges were added.
type Step = (usize, usize, ExactRatio, usize, usize);

/// Derives one potential per connected component and checks each edge against
/// the first path already established to its target. The first structural ID
/// in order starts each component with ratio one, so the result is deterministic.
fn equivalence_potentials<'a>(
    graph: &'a NominalGraph<'a>,
    edges: &'a [(usize, &'a DimensionEquivalenceRecord)],
    base_only: bool,
    meter: &mut WorkMeter,
) -> Result<Potentials<'a>, ValidationFailure> {
    meter.charge(u64::try_from(graph.nodes.len()).unwrap_or(u64::MAX), || {
        JsonPointer::root().key("semantic_graph").key("nodes")
    })?;
    // Every id the walk can reach: the graph's, the imported, and each
    // edge's two ends.
    let reachable = graph
        .index
        .len()
        .saturating_add(graph.imported.len())
        .saturating_add(edges.len().saturating_mul(2));
    let mut ids: Vec<&CheckedNodeId> = meter.vec(reachable)?;
    ids.extend(graph.index.keys().copied());
    ids.extend(graph.imported.iter().map(|(id, _)| *id));
    for (_, edge) in edges {
        ids.extend(edge.dimension_node_ids.iter());
    }
    ids.sort_unstable();
    ids.dedup();
    let count = ids.len();
    let mut potentials = Potentials {
        ids,
        entries: meter.vec(count)?,
    };
    potentials.entries.resize_with(count, || None);
    let located = |potentials: &Potentials<'_>, id: &CheckedNodeId| {
        potentials.locate(id).ok_or_else(|| {
            ValidationFailure::refused(CheckedPackageRefusalCode::MalformedWire, node_pointer(0))
        })
    };
    let mut adjacency: Vec<Step> = meter.vec(edges.len().saturating_mul(2))?;
    for (sequence, (position, edge)) in edges.iter().enumerate() {
        let ids = &edge.dimension_node_ids;
        let is_base = |id: &CheckedNodeId| matches!(graph.nominal(id), Some(NominalRecord::Dimension(dim)) if dim.base);
        if base_only && !(is_base(&ids[0]) && is_base(&ids[1])) {
            continue;
        }
        let at = || node_pointer(*position);
        meter.at_node(*position, "scalar_type");
        let ratio = ExactRatio::parse(&edge.root_scale, meter, &at)?;
        let inverse = ratio.inverse(meter)?;
        let (first, second) = (
            located(&potentials, &ids[0])?,
            located(&potentials, &ids[1])?,
        );
        adjacency.push((first, second, inverse, *position, sequence * 2));
        adjacency.push((second, first, ratio, *position, sequence * 2 + 1));
    }
    adjacency.sort_unstable_by(|left, right| left.0.cmp(&right.0).then(left.4.cmp(&right.4)));
    let mut queue: Vec<usize> = meter.vec(count)?;
    for id in graph
        .index
        .keys()
        .copied()
        .chain(graph.imported.iter().map(|(id, _)| *id))
    {
        let start = located(&potentials, id)?;
        if !matches!(graph.nominal(id), Some(NominalRecord::Dimension(_)))
            || potentials.entries[start].is_some()
        {
            continue;
        }
        queue.clear();
        queue.push(start);
        potentials.entries[start] = Some((start, ExactRatio::one()));
        let mut head = 0;
        while let Some(&current) = queue.get(head) {
            head += 1;
            let Some((component, current_ratio)) = potentials.entries[current].as_ref() else {
                continue;
            };
            let (component, current_ratio) = (*component, current_ratio.duplicate(meter)?);
            let first = adjacency.partition_point(|step| step.0 < current);
            for (_, neighbor, multiplier, position, _) in adjacency[first..]
                .iter()
                .take_while(|step| step.0 == current)
            {
                let at = || node_pointer(*position);
                meter.at_node(*position, "scalar_type");
                let candidate = current_ratio.multiply(multiplier, meter, &at)?;
                match &potentials.entries[*neighbor] {
                    Some((known_component, known_ratio)) => {
                        require(
                            *known_component == component && *known_ratio == candidate,
                            at,
                        )?;
                    }
                    None => {
                        potentials.entries[*neighbor] = Some((component, candidate));
                        queue.push(*neighbor);
                    }
                }
            }
        }
    }
    Ok(potentials)
}

/// A dimension as the exponents of its base components, in component order,
/// and its scale within them.
fn normalized_dimension<'a>(
    id: &CheckedNodeId,
    graph: &NominalGraph<'_>,
    bases: &Potentials<'a>,
    meter: &mut WorkMeter,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(Vec<(&'a CheckedNodeId, Integer)>, ExactRatio), ValidationFailure> {
    let Some(NominalRecord::Dimension(dimension)) = graph.nominal(id) else {
        return Err(invalid(at()));
    };
    if dimension.base {
        let Some((component, scale)) = bases.get(id) else {
            return Err(invalid(at()));
        };
        let mut map = meter.vec(1)?;
        map.push((component, Integer::one()));
        return Ok((map, scale.duplicate(meter)?));
    }
    let mut map: Vec<(&CheckedNodeId, Integer)> = meter.vec(dimension.terms.len())?;
    let mut scale = ExactRatio::one();
    for term in &dimension.terms {
        meter.charge(u64::try_from(term.exponent.len()).unwrap_or(u64::MAX), at)?;
        let exponent = meter
            .exact_integer(&term.exponent)?
            .ok_or_else(|| invalid(at()))?;
        let Some((component, base_scale)) = bases.get(&term.dimension_node_id) else {
            return Err(invalid(at()));
        };
        match map.binary_search_by(|(held, _)| (*held).cmp(component)) {
            Ok(held) => {
                let sum = meter.exact_add(&map[held].1, &exponent)?;
                map[held].1 = sum;
            }
            Err(held) => map.insert(held, (component, exponent)),
        }
        scale = scale.multiply(&base_scale.power(&term.exponent, meter, at)?, meter, at)?;
    }
    map.retain(|(_, exponent)| !exponent.is_zero());
    Ok((map, scale))
}

fn validate_equivalence_graph(
    graph: &NominalGraph<'_>,
    edges: &[(usize, &DimensionEquivalenceRecord)],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if edges.is_empty() {
        return Ok(());
    }
    let bases = equivalence_potentials(graph, edges, true, meter)?;
    for (position, edge) in edges {
        let ids = &edge.dimension_node_ids;
        let both_base = ids
            .iter()
            .all(|id| matches!(graph.nominal(id), Some(NominalRecord::Dimension(dim)) if dim.base));
        if both_base {
            continue;
        }
        let at = || node_pointer(*position);
        meter.at_node(*position, "scalar_type");
        let (left_map, left_scale) = normalized_dimension(&ids[0], graph, &bases, meter, &at)?;
        let (right_map, right_scale) = normalized_dimension(&ids[1], graph, &bases, meter, &at)?;
        require(left_map == right_map, at)?;
        let expected = left_scale.divide(&right_scale, meter, &at)?;
        let actual = ExactRatio::parse(&edge.root_scale, meter, &at)?;
        require(actual == expected, at)?;
    }
    // A derived edge may complete a cycle through other derived edges, so
    // check the whole equivalence graph after base-root consistency.
    equivalence_potentials(graph, edges, false, meter)?;
    Ok(())
}

/// Source selected by the lock for an equivalence declaration.
type EquivalenceSource<'a> = (&'a str, &'a str);

/// Source order, graph position and occurrence ordinal of one anchor.
type EquivalenceAnchor = (u64, usize, u64);

/// Anchors grouped by their declaring source.
type SourceEquivalenceAnchors<'a> = Vec<(EquivalenceSource<'a>, EquivalenceAnchor)>;

/// Anchor ordinals count equivalence declarations in source byte order, not
/// graph order. Source-map admission already established the exact node and
/// occurrence joins before this check runs.
pub(super) fn validate_equivalence_occurrences(
    wire: &RawCheckedPackageWireV2,
    kinds: &[CheckedNodeKind],
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if !kinds.contains(&CheckedNodeKind::ScalarType(
        ScalarTypeForm::DimensionEquivalence,
    )) {
        return Ok(());
    }
    meter.charge(
        u64::try_from(wire.source_map.len()).unwrap_or(u64::MAX),
        || JsonPointer::root().key("source_map"),
    )?;
    let is_anchor = |entry: &&crate::checked_package::shared::CheckedSourceMapEntry| {
        entry.role == crate::checked_package::shared::CheckedOccurrenceRole::Anchor
    };
    let mut anchor_entries = meter.vec(wire.source_map.iter().filter(is_anchor).count())?;
    anchor_entries.extend(
        wire.source_map
            .iter()
            .filter(is_anchor)
            .map(|entry| (&entry.node_id, entry)),
    );
    // Of two anchors for one node the later wins, as a map would hold.
    let anchors = SortedTable::from_vec(anchor_entries, |left, right| {
        std::ptr::from_ref(*right).cmp(&std::ptr::from_ref(*left))
    });
    let equivalent = kinds
        .iter()
        .filter(|kind| **kind == CheckedNodeKind::ScalarType(ScalarTypeForm::DimensionEquivalence))
        .count();
    let mut by_source: SourceEquivalenceAnchors<'_> = meter.vec(equivalent)?;
    for (position, (node, kind)) in wire.semantic_graph.nodes.iter().zip(kinds).enumerate() {
        if *kind != CheckedNodeKind::ScalarType(ScalarTypeForm::DimensionEquivalence) {
            continue;
        }
        let Some(super::CheckedNodeOwner::Source {
            authority,
            identity,
        }) = node.owner.as_ref()
        else {
            return Err(invalid(node_pointer(position).key("owner")));
        };
        meter.charge(1, || node_pointer(position).key("occurrences"))?;
        let Some(entry) = anchors.get(&&node.node_id).copied() else {
            return Err(invalid(node_pointer(position).key("occurrences")));
        };
        let Some(start) = entry
            .regions
            .iter()
            .filter(|region| {
                region.source.authority == *authority && region.source.identity == *identity
            })
            .map(|region| region.start)
            .min()
        else {
            return Err(invalid(node_pointer(position).key("occurrences")));
        };
        by_source.push((
            (authority.as_ref(), identity.as_ref()),
            (start, position, entry.ordinal),
        ));
    }
    // Sorted by source, then by anchor: each source's declarations are one
    // run, in source byte order.
    by_source.sort_unstable();
    let mut expected = 0_u64;
    let mut previous: Option<EquivalenceSource<'_>> = None;
    for (source, (_, position, actual)) in &by_source {
        if previous != Some(*source) {
            previous = Some(*source);
            expected = 0;
        }
        require(actual == &expected, || {
            node_pointer(*position).key("occurrences")
        })?;
        expected = expected.saturating_add(1);
    }
    Ok(())
}

struct NominalGraph<'a> {
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &'a [CheckedNodeKind],
    index: &'a GraphIndex<'a>,
    records: &'a [Option<NominalRecord>],
    selected_models: SortedTable<&'a str, &'a DomainModel>,
    imported: SortedTable<&'a CheckedNodeId, NominalRecord>,
}

impl NominalGraph<'_> {
    fn nominal(&self, id: &CheckedNodeId) -> Option<&NominalRecord> {
        self.index
            .get(id)
            .and_then(|position| self.records.get(*position))
            .and_then(Option::as_ref)
            .or_else(|| self.imported.find(|held| (*held).cmp(id)))
    }

    fn model_variant_cases(
        &self,
        id: &CheckedNodeId,
    ) -> Option<&super::model_members::SortedMap<Box<str>, super::model_members::SelectedVariantCase>>
    {
        let position = *self.index.get(id)?;
        if self.kinds.get(position) != Some(&CheckedNodeKind::Model(super::ModelForm::VariantType))
        {
            return None;
        }
        let node = self.nodes.get(position)?;
        let Some(super::CheckedNodeOwner::Model {
            identity,
            node: owner_node,
        }) = &node.owner
        else {
            return None;
        };
        self.selected_models
            .get(&identity.as_ref())
            .and_then(|model| model.variant_types.get(owner_node))
    }

    fn is_model_variant(&self, id: &CheckedNodeId) -> bool {
        let Some(&position) = self.index.get(id) else {
            return false;
        };
        self.kinds.get(position) == Some(&CheckedNodeKind::Model(super::ModelForm::VariantType))
            && self.nodes.get(position).is_some_and(|node| {
                matches!(
                    node.owner.as_ref(),
                    Some(super::CheckedNodeOwner::Model { .. })
                )
            })
    }
}

fn validate_owner(
    owner: &NominalOwner,
    lock: &CheckedPackageLockV2,
    graph: &NominalGraph<'_>,
    at: impl FnOnce() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let joined = match owner {
        NominalOwner::Source {
            authority,
            identity,
        } => lock
            .sources
            .iter()
            .any(|source| source.authority == *authority && source.identity == *identity),
        // A selected domain package is named by its `model_root` owner; the
        // node is not a lock member, so the join is by package identity, and
        // the owner's identity and node must each be nonempty (FR-038-AC-45).
        // Each identity has at most one root, so joining by identity alone is
        // sound.
        NominalOwner::Model { identity, node } => {
            !node.is_empty()
                && !identity.is_empty()
                && graph.selected_models.contains_key(&identity.as_ref())
        }
    };
    require(joined, at)
}

/// The position of the first name that is not an identifier or repeats an
/// earlier name, as a front-to-back scan with a set would find it. The names
/// are ordered once, unstably, with their positions.
fn first_bad_name<'a>(names: impl Iterator<Item = &'a str>) -> Option<usize> {
    let mut ordered: Vec<(&str, usize)> = names.zip(0..).collect();
    let mut first = ordered
        .iter()
        .find(|(name, _)| !is_identifier(name))
        .map(|&(_, index)| index);
    ordered.sort_unstable();
    for pair in ordered.windows(2) {
        if pair[0].0 == pair[1].0 {
            // Ordered by position within a name, `pair[1]` is a repeat.
            let index = pair[1].1;
            first = Some(first.map_or(index, |held| held.min(index)));
        }
    }
    first
}

/// ASCII identifier grammar shared with the closed `Declaration` member:
/// the schema's `Identifier`, `^[A-Za-z_][A-Za-z0-9_]*$`.
pub(super) fn is_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// A nonempty qualified name whose every segment is an ASCII identifier.
/// Shared by the three declared nominal kinds and by
/// `super::validate_declaration`'s `Declaration.qualified_name` check, so the
/// grammar is checked in exactly one place regardless of which member holds
/// the name. `at` names the array; an empty name refuses there and a bad
/// segment at that segment.
pub(super) fn validate_qualified_name(
    name: &[Box<str>],
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    require(!name.is_empty(), at)?;
    match name.iter().position(|segment| !is_identifier(segment)) {
        Some(segment) => Err(invalid(at().index(segment))),
        None => Ok(()),
    }
}

/// `^(0|-?[1-9][0-9]*)$`; returns the unsigned magnitude.
#[derive(Debug, Eq, PartialEq)]
struct IntegerMagnitude<'a> {
    digits: &'a str,
    zero: bool,
}

#[ix_trace_rs::string_edge]
fn integer_magnitude(value: &str) -> Option<IntegerMagnitude<'_>> {
    if value == "0" {
        return Some(IntegerMagnitude {
            digits: value,
            zero: true,
        });
    }
    let magnitude = value.strip_prefix('-').unwrap_or(value);
    positive_integer(magnitude).then_some(IntegerMagnitude {
        digits: magnitude,
        zero: false,
    })
}

/// `^[1-9][0-9]*$`.
fn positive_integer(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|first| (b'1'..=b'9').contains(&first))
        && bytes.all(|byte| byte.is_ascii_digit())
}

/// Validates the rational at `at`: a canonical numerator, a positive
/// denominator, and the pair reduced (its GCD work charged at `at`).
fn validate_rational(
    rational: &CheckedRational,
    meter: &mut WorkMeter,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    validate_rational_parts(&rational.numerator, &rational.denominator, meter, at)
}

/// Shared signed exact-rational syntax and metered reduction check.
pub(super) fn validate_rational_parts(
    numerator: &str,
    denominator: &str,
    meter: &mut WorkMeter,
    at: &dyn Fn() -> JsonPointer,
) -> Result<(), ValidationFailure> {
    let numerator = integer_magnitude(numerator).ok_or_else(|| invalid(at().key("numerator")))?;
    require(positive_integer(denominator), || at().key("denominator"))?;
    require(coprime(numerator.digits, denominator, meter, at)?, at)
}

fn validate_enum_declaration(
    declaration: &EnumDeclarationRecord,
    site: &Site<'_>,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    validate_owner(&declaration.owner, lock, graph, || {
        site.nominal_member(&["owner"])
    })?;
    validate_qualified_name(&declaration.qualified_declaration, &|| {
        site.nominal_member(&["qualified_declaration"])
    })?;
    require(!declaration.members.is_empty(), || {
        site.nominal_member(&["members"])
    })?;
    let member_at = |index: usize| site.nominal_member(&["members"]).index(index);
    let mut seen: Vec<&str> = meter.vec(declaration.members.len())?;
    for (index, member) in declaration.members.iter().enumerate() {
        meter.charge(1, || member_at(index))?;
        let first = is_identifier(member) && meter.insert_sorted(&mut seen, member.as_ref())?;
        require(first, || member_at(index))?;
    }
    if !declaration.ordered {
        // Identifiers are escape-free ASCII, so JCS key order is byte order.
        // The later member of the first out-of-order pair is at fault.
        if let Some(pair) = declaration
            .members
            .windows(2)
            .position(|pair| !matches!(pair, [left, right] if left < right))
        {
            return Err(invalid(member_at(pair.saturating_add(1))));
        }
    }
    Ok(())
}

fn validate_enum_member(
    site: &Site<'_>,
    member: &EnumMemberRecord,
    graph: &NominalGraph<'_>,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    if graph.is_model_variant(&member.declaration_node_id) {
        require(!member.case.is_empty(), || site.pointer())?;
        if let Some(cases) = graph.model_variant_cases(&member.declaration_node_id) {
            require(cases.values().all(|case| case.payload.is_none()), || {
                site.pointer()
            })?;
            require(cases.contains_key(&member.case), || site.pointer())?;
        }
    } else {
        require(is_identifier(&member.case), || {
            site.nominal_member(&["case"])
        })?;
        let Some(NominalRecord::EnumDeclaration(declaration)) =
            graph.nominal(&member.declaration_node_id)
        else {
            return Err(invalid(site.nominal_member(&["declaration_node_id"])));
        };
        require(declaration.members.contains(&member.case), || {
            site.nominal_member(&["case"])
        })?;
    }
    require(node.semantic_type == member.declaration_node_id, || {
        site.member("semantic_type")
    })?;
    require(
        node.dependencies.as_slice() == std::slice::from_ref(&member.declaration_node_id),
        || site.member("dependencies"),
    )?;
    require(member.body_is_literal, || site.member("body"))
}

fn validate_dimension(
    site: &Site<'_>,
    dimension: &DimensionRecord,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
    _bytes: u64,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    validate_owner(&dimension.owner, lock, graph, || {
        site.nominal_member(&["owner"])
    })?;
    validate_qualified_name(&dimension.qualified_declaration, &|| {
        site.nominal_member(&["qualified_declaration"])
    })?;
    require(node.semantic_type == node.node_id, || {
        site.member("semantic_type")
    })?;
    let term_at = |index: usize| site.nominal_member(&["terms"]).index(index);
    let mut bases: Vec<&CheckedNodeId> = meter.vec(dimension.terms.len())?;
    for (index, term) in dimension.terms.iter().enumerate() {
        meter.charge(1, || term_at(index))?;
        let exponent_at = || term_at(index).key("exponent");
        let magnitude = integer_magnitude(&term.exponent).ok_or_else(|| invalid(exponent_at()))?;
        require(!magnitude.zero, exponent_at)?;
        let base_at = || term_at(index).key("dimension_node_id");
        let first = meter.insert_sorted(&mut bases, &term.dimension_node_id)?;
        require(first, base_at)?;
        let Some(NominalRecord::Dimension(base)) = graph.nominal(&term.dimension_node_id) else {
            return Err(invalid(base_at()));
        };
        require(base.base, base_at)?;
    }
    // The later term of the first out-of-order node ID is at fault.
    if let Some(pair) = dimension.terms.windows(2).position(
        |pair| !matches!(pair, [left, right] if left.dimension_node_id < right.dimension_node_id),
    ) {
        return Err(invalid(term_at(pair.saturating_add(1))));
    }
    let same = same_ids(
        dimension.terms.iter().map(|term| &term.dimension_node_id),
        dimension.terms.len(),
        &node.dependencies,
        meter,
    )?;
    require(same, || site.member("dependencies"))
}

/// Whether `expected` and `actual` hold the same ids as multisets, compared
/// through two exact blocks charged first and sorted in place.
fn same_ids<'a>(
    expected: impl Iterator<Item = &'a CheckedNodeId>,
    expected_len: usize,
    actual: &'a [CheckedNodeId],
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let mut wanted = meter.vec(expected_len)?;
    wanted.extend(expected);
    let mut held = meter.vec(actual.len())?;
    held.extend(actual.iter());
    wanted.sort_unstable();
    held.sort_unstable();
    Ok(wanted == held)
}

fn validate_unit(
    site: &Site<'_>,
    unit: &UnitRecord,
    graph: &NominalGraph<'_>,
    lock: &CheckedPackageLockV2,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let node = site.node;
    validate_owner(&unit.owner, lock, graph, || site.nominal_member(&["owner"]))?;
    validate_qualified_name(&unit.qualified_declaration, &|| {
        site.nominal_member(&["qualified_declaration"])
    })?;
    require(
        matches!(
            graph.nominal(&unit.dimension_node_id),
            Some(NominalRecord::Dimension(_))
        ),
        || site.nominal_member(&["dimension_node_id"]),
    )?;
    validate_rational(&unit.scale, meter, &|| site.nominal_member(&["scale"]))?;
    validate_rational(&unit.offset, meter, &|| site.nominal_member(&["offset"]))?;
    require(node.semantic_type == unit.dimension_node_id, || {
        site.member("semantic_type")
    })?;
    match &unit.target_unit_node_id {
        None => {
            require(is_rational(&unit.scale, "1", "1"), || {
                site.nominal_member(&["scale"])
            })?;
            require(is_rational(&unit.offset, "0", "1"), || {
                site.nominal_member(&["offset"])
            })?;
        }
        Some(_) => {
            require(
                integer_magnitude(&unit.scale.numerator).is_none_or(|m| !m.zero),
                || site.nominal_member(&["scale", "numerator"]),
            )?;
            validate_unit_path(site, unit, graph, meter)?;
        }
    }
    let same = same_ids(
        std::iter::once(&unit.dimension_node_id).chain(unit.target_unit_node_id.as_ref()),
        1 + usize::from(unit.target_unit_node_id.is_some()),
        &node.dependencies,
        meter,
    )?;
    require(same, || site.member("dependencies"))
}

/// Whether `segments` joined by `/` spell `text`, compared in place.
fn joined_with_slashes(segments: &[Box<str>], text: &str) -> bool {
    let mut rest = text;
    for (position, segment) in segments.iter().enumerate() {
        if position > 0 {
            let Some(after) = rest.strip_prefix('/') else {
                return false;
            };
            rest = after;
        }
        let Some(after) = rest.strip_prefix(segment.as_ref()) else {
            return false;
        };
        rest = after;
    }
    rest.is_empty()
}

fn is_rational(rational: &CheckedRational, numerator: &str, denominator: &str) -> bool {
    rational.numerator.as_ref() == numerator && rational.denominator.as_ref() == denominator
}

/// Follows targets to a root; every hop is a same-dimension unit, acyclic.
/// Each hop is charged, and refused, at the `target_unit_node_id` that names
/// it — this node's own for the first hop, the previous unit's after that.
fn validate_unit_path(
    site: &Site<'_>,
    unit: &UnitRecord,
    graph: &NominalGraph<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let link_at = |position: usize| {
        node_pointer(position)
            .key("body")
            .key("target_unit_node_id")
    };
    let mut visited: Vec<&CheckedNodeId> = meter.vec(1)?;
    visited.push(&site.node.node_id);
    let mut from = site.position;
    let mut next = unit.target_unit_node_id.as_ref();
    while let Some(target) = next {
        meter.charge(1, || link_at(from))?;
        let first = meter.insert_sorted(&mut visited, target)?;
        require(first, || link_at(from))?;
        let Some(NominalRecord::Unit(target_unit)) = graph.nominal(target) else {
            return Err(invalid(link_at(from)));
        };
        require(
            target_unit.dimension_node_id == unit.dimension_node_id,
            || link_at(from),
        )?;
        // `graph.nominal` resolved `target`, so it is indexed.
        from = graph.index.get(target).copied().unwrap_or(from);
        next = target_unit.target_unit_node_id.as_ref();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::checked_json::CheckedJson;
    use super::*;
    use crate::NativeJoinBudget;
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(package: &str, name: &str) -> CheckedNodeId {
        CheckedNodeId {
            package: package.into(),
            path: vec![crate::checked_package::shared::CheckedNodePathSegment::Name(name.into())],
            ordinal: 0,
        }
    }

    /// The in-place slash join and the dependency multiset comparison agree
    /// with the strings and sorted collections they replace, and the
    /// comparison's blocks are charged first (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn slash_joins_and_id_multisets_agree_with_their_allocating_forms() {
        let names: Vec<Box<str>> = vec!["a".into(), "bc".into(), "d".into()];
        for text in [
            "a/bc/d", "a/bc/", "a/bc/dd", "a/b/d", "a//bc/d", "", "a/bc/d/",
        ] {
            let joined = names
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join("/");
            assert_eq!(joined_with_slashes(&names, text), joined == text, "{text}");
        }
        assert!(joined_with_slashes(&[], ""));
        let (a, b, c) = (id("p", "a"), id("p", "b"), id("q", "a"));
        let run = |limit: u64, expected: &[&CheckedNodeId], actual: &[CheckedNodeId]| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut meter = WorkMeter::over(&mut budget);
            let same = same_ids(expected.iter().copied(), expected.len(), actual, &mut meter).ok();
            drop(meter);
            (same, budget.read_storage_consumed())
        };
        let cases: [(&[&CheckedNodeId], Vec<CheckedNodeId>); 5] = [
            (&[&a, &b], vec![b.clone(), a.clone()]),
            (&[&a, &b], vec![a.clone(), c.clone()]),
            (&[&a, &a], vec![a.clone()]),
            (&[&a], vec![a.clone(), a.clone()]),
            (&[], vec![]),
        ];
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut meter = WorkMeter::over(&mut budget);
        assert_eq!(meter.clone_id(&a).ok(), Some(a.clone()));
        drop(meter);
        let spent = budget.read_storage_consumed();
        assert!(spent > 0, "a cloned id is charged");
        let mut short = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(spent - 1);
        assert!(WorkMeter::over(&mut short).clone_id(&a).is_err());
        for (expected, actual) in &cases {
            let mut left: Vec<_> = expected.iter().copied().collect();
            let mut right: Vec<_> = actual.iter().collect();
            left.sort();
            right.sort();
            let (same, spent) = run(u64::MAX, expected, actual);
            assert_eq!(same, Some(left == right));
            if spent > 0 {
                assert_eq!(run(spent - 1, expected, actual).0, None, "one byte short");
            }
        }
    }

    /// Resolving the one-second Time unit asks for every block before it
    /// makes it: the measured requirement admits and one byte below is
    /// denied, for a chain of units, for a unit that stops resolving, and for
    /// the resolved ID's own copy (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn duration_unit_resolution_allocates_exactly_what_it_requests() {
        let source = json!({"kind":"source","authority":"agent-ix","identity":"example"});
        let dimension: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id":{"package":"example","path":["Time"],"ordinal":0},
            "schema_version":"quire.checked-semantic-graph/v2", "node_tag":"scalar_type", "semantic_form":"dimension",
            "semantic_type":{"package":"example","path":["Time"],"ordinal":0},
            "dependencies":[], "occurrences":[], "owner":source,
            "declaration":{"qualified_name":["Time"]},
            "body":{"term":"dimension","base":true,"terms":[]}
        })).expect("dimension node");
        let unit = |name: &str,
                    target: Option<&CheckedNodeId>,
                    scale: (&str, &str)|
         -> CheckedSemanticNodeV2 {
            serde_json::from_value(json!({
                "node_id":{"package":"example","path":[name],"ordinal":0},
                "schema_version":"quire.checked-semantic-graph/v2", "node_tag":"scalar_type", "semantic_form":"unit",
                "semantic_type":dimension.node_id, "dependencies":[dimension.node_id], "occurrences":[],
                "owner":source, "declaration":{"qualified_name":[name]},
                "body":{"term":"unit","dimension_node_id":dimension.node_id,
                    "target_unit_node_id":target,"scale":{"numerator":scale.0,"denominator":scale.1},
                    "offset":{"numerator":"0","denominator":"1"}}
            })).expect("unit node")
        };
        let root = unit("root", None, ("1", "1"));
        let second = unit("s", Some(&root.node_id), ("3", "3"));
        let wrong = unit("s", Some(&root.node_id), ("2", "1"));
        let measure = |nodes: &[CheckedSemanticNodeV2], limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let bodies =
                super::super::graph_storage::OwnedBodies::of(nodes).expect("canonical bodies");
            let mut meter = WorkMeter::over(&mut budget);
            bodies.attach(&mut meter);
            let found = duration_unit_key(nodes, "ix://quire/unit/s", &mut meter);
            drop(meter);
            (found.ok(), budget.read_storage_consumed())
        };
        for (nodes, resolves) in [
            (vec![dimension.clone(), root.clone(), second.clone()], true),
            (vec![dimension.clone(), root.clone(), wrong], false),
        ] {
            let (found, spent) = measure(&nodes, u64::MAX);
            assert_eq!(found, Some(resolves.then(|| second.node_id.clone())));
            assert!(spent > 0, "resolution is charged");
            assert_eq!(measure(&nodes, spent).0, found, "the measured limit admits");
            assert_eq!(
                measure(&nodes, spent - 1).0,
                None,
                "one byte short is denied"
            );
        }
    }

    #[trace("TC-280", "FR-038-AC-387")]
    #[test]
    fn published_dimension_operands_keep_their_authored_wire_forms() {
        let root = match std::env::var("QUIRE_SPECIFICATION_DIR") {
            Ok(root) => root,
            Err(_) if std::env::var_os("QSPEC_CONFORMANCE").is_none() => return,
            Err(_) => panic!("QUIRE_SPECIFICATION_DIR is required for QSpec conformance"),
        };
        let path = std::path::Path::new(&root)
            .join("proposals/checked-package-v2/dimension-equivalence-vectors.json");
        let bytes = std::fs::read(&path).expect("published dimension vectors");
        let vectors: Value = serde_json::from_slice(&bytes).expect("published vector JSON");
        let cases = vectors["cases"].as_array().expect("published cases");
        assert_eq!(cases.len(), 15, "XP-01 through XP-15 are present");
        for (position, case) in cases.iter().enumerate() {
            let id = format!("XP-{:02}", position + 1);
            assert_eq!(
                case["id"].as_str(),
                Some(id.as_str()),
                "published case order"
            );
            for equivalence in case["equivalences"].as_array().expect("equivalences") {
                let body = json!({
                    "term":"dimension_equivalence",
                    "dimension_node_ids":equivalence["pair"],
                    "root_scale":equivalence["root_scale"]
                });
                let parsed = NominalBody::from_value(&body, JsonPointer::root())
                    .unwrap_or_else(|failure| panic!("{id}: {failure:?}"));
                assert_eq!(
                    serde_json::to_value(parsed).expect("typed nominal body serializes"),
                    body,
                    "{id}: the typed body retains local and package operands"
                );
            }
        }
    }

    #[trace("TC-048", "FR-038-AC-5")]
    #[test]
    fn model_owned_nominals_join_selected_fcd_values() {
        let document = json!({
            "contractVersion":"2.0.0",
            "package":{"identity":"acme/rpc","version":"1.0.0"},
            "constructs":[], "types":[],
            "dimensions":[{"identity":"ix://acme/rpc/dimension/Time","name":"Time","base":true,"terms":[]}],
            "units":[{"identity":"ix://acme/rpc/unit/ms","name":"ms",
                "dimension":"ix://acme/rpc/dimension/Time","scale":"1/1000","offset":"0"}]
        });
        let mut meter = WorkMeter::new(u64::MAX);
        let mut budget = super::super::model_members::Budget::new(&mut meter, 0, 1 << 20);
        let unit_refs = crate::checked_package::v2::model_members::UnitRefs::default();
        let model = super::super::model_members::read_semantic_ir(
            super::super::model_members::view_of(&document),
            "example",
            &mut budget,
            &unit_refs,
        )
        .expect("selected declarations");
        let dimension: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id":{"package":"acme/rpc","path":["scalar_type","dimension","ix://acme/rpc/dimension/Time"],"ordinal":0},
            "schema_version":"quire.checked-semantic-graph/v2", "node_tag":"scalar_type", "semantic_form":"dimension",
            "semantic_type":{"package":"acme/rpc","path":["scalar_type","dimension","ix://acme/rpc/dimension/Time"],"ordinal":0},
            "dependencies":[], "occurrences":[],
            "owner":{"kind":"model","identity":"acme/rpc","node":"ix://acme/rpc/dimension/Time"},
            "body":{"term":"aggregate","members":[]}
        })).expect("dimension node");
        let unit: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id":{"package":"acme/rpc","path":["scalar_type","unit","ix://acme/rpc/unit/ms"],"ordinal":0},
            "schema_version":"quire.checked-semantic-graph/v2", "node_tag":"scalar_type", "semantic_form":"unit",
            "semantic_type":dimension.node_id, "dependencies":[dimension.node_id], "occurrences":[],
            "owner":{"kind":"model","identity":"acme/rpc","node":"ix://acme/rpc/unit/ms"},
            "body":{"term":"aggregate","members":[]}
        })).expect("unit node");
        let nodes = [&dimension, &unit];
        let model_nodes = model_node_table(
            nodes
                .iter()
                .map(|node| (node.owner.as_ref(), &node.node_id)),
            &mut WorkMeter::new(u64::MAX),
        )
        .expect("model node table");
        let bodies = super::super::graph_storage::OwnedBodies::of(nodes).expect("canonical bodies");
        let mut meter = WorkMeter::new(u64::MAX);
        bodies.attach(&mut meter);
        let record = record_from_node(
            &unit,
            BodyInput::of_graph(1, &meter),
            NominalKind::Unit,
            1,
            &[model],
            &model_nodes,
            &mut meter,
        )
        .expect("model unit join");
        let NominalRecord::Unit(record) = record else {
            panic!("unit record")
        };
        assert_eq!(record.dimension_node_id, dimension.node_id);
        assert_eq!(
            (&*record.scale.numerator, &*record.scale.denominator),
            ("1", "1000")
        );
        assert_eq!(
            (&*record.offset.numerator, &*record.offset.denominator),
            ("0", "1")
        );

        let mut malformed = unit.clone();
        malformed.body = json!({"term":"aggregate","members":[{"term":"literal","kind":"text","value":"extra"}]});
        assert!(matches!(
            record_from_node(
                &malformed,
                BodyInput::Value(&malformed.body),
                NominalKind::Unit,
                1,
                &[],
                &model_nodes,
                &mut WorkMeter::new(u64::MAX)
            ),
            Err(ValidationFailure::Refused(refusal))
                if refusal.code == CheckedPackageRefusalCode::MalformedWire
                    && refusal.loci[0].path.as_ref().is_some_and(|path| path.as_str() == "/semantic_graph/nodes/1")
        ));

        let source_unit: CheckedSemanticNodeV2 = serde_json::from_value(json!({
            "node_id":{"package":"example","path":["acme","rpc","ms"],"ordinal":0},
            "schema_version":"quire.checked-semantic-graph/v2", "node_tag":"scalar_type", "semantic_form":"unit",
            "semantic_type":dimension.node_id, "dependencies":[dimension.node_id], "occurrences":[],
            "owner":{"kind":"source","authority":"agent-ix","identity":"example"},
            "declaration":{"qualified_name":["acme","rpc","ms"]},
            "body":{"term":"unit","dimension_node_id":dimension.node_id,
                "target_unit_node_id":null,"scale":{"numerator":"1","denominator":"1"},
                "offset":{"numerator":"0","denominator":"1"}}
        })).expect("source unit node");
        let declarations = [dimension, unit, source_unit];
        let kinds = [
            CheckedNodeKind::ScalarType(ScalarTypeForm::Dimension),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Unit),
        ];
        let index = declarations
            .iter()
            .enumerate()
            .map(|(position, node)| (&node.node_id, position))
            .collect::<GraphIndex<'_>>();
        assert!(matches!(
            super::super::validate_declaration_names(&declarations, &kinds, &index),
            Err(ValidationFailure::Refused(refusal))
                if refusal.code == CheckedPackageRefusalCode::AmbiguousDeclaration
                    && refusal.cause == Some(crate::CheckedPackageRefusalCause::AmbiguousName)
        ));
    }

    #[trace("TC-048", "FR-038-AC-5")]
    #[test]
    fn merged_qspec_structural_nominal_fixture_reads() {
        let Some(root) = std::env::var_os("QUIRE_SPECIFICATION_DIR") else {
            return;
        };
        let path = std::path::Path::new(&root)
            .join("proposals/checked-package-v2/fixtures/positive-nominal-identities.json");
        let source = std::fs::read(path).expect("authoritative QSpec nominal fixture");
        let fixture: Value = serde_json::from_slice(&source).expect("QSpec JSON");
        let bytes = serde_json::to_vec(&fixture).expect("canonical test JSON");
        let mut evidence = crate::CheckedPackageEvidence::new();
        evidence.support_feature("quire.value.complete/v1");
        let result = super::super::CheckedPackageV2::read(
            &bytes,
            crate::CheckedPackageReadLimits::bounded(),
            &evidence,
        );
        assert!(
            matches!(
                result,
                super::super::CheckedPackageV2ReadResult::Admitted(_)
            ),
            "{result:?}"
        );
    }

    #[trace("TC-048", "FR-038-AC-5")]
    #[test]
    fn nominal_body_keeps_closed_structural_references() {
        let first = json!({"package":"acme/clock","path":["Time"],"ordinal":0});
        let second = json!({"package":"acme/flow","path":["Time"],"ordinal":0});
        let body = json!({"term":"dimension_equivalence",
            "dimension_node_ids":[first,second],
            "root_scale":{"numerator":"3","denominator":"2"}});
        let parsed = NominalBody::from_value(&body, JsonPointer::root()).expect("closed body");
        let NominalBodyKind::DimensionEquivalence {
            dimension_node_ids,
            root_scale,
        } = parsed.into_kind()
        else {
            panic!("equivalence body");
        };
        assert_eq!(
            dimension_node_ids[0].node_id().package.as_ref(),
            "acme/clock"
        );
        assert_eq!(
            dimension_node_ids[1].node_id().package.as_ref(),
            "acme/flow"
        );
        assert_eq!(
            (&*root_scale.numerator, &*root_scale.denominator),
            ("3", "2")
        );
        let malformed = json!({"term":"dimension_equivalence",
            "dimension_node_ids":[first,second],
            "root_scale":{"numerator":"0","denominator":"1"}});
        assert!(matches!(
            NominalBody::from_value(&malformed, JsonPointer::root()),
            Err(ValidationFailure::Refused(refusal))
                if refusal.code == CheckedPackageRefusalCode::MalformedWire
                    && refusal.loci[0].path.as_ref().is_some_and(|path| path.as_str() == "/root_scale/numerator")
        ));
    }

    #[trace("TC-048", "FR-038-AC-24")]
    #[test]
    fn nominal_body_node_ref_requires_graph_local_index() {
        let valid = json!({
            "term": "dimension",
            "base": false,
            "terms": [{
                "dimension_node_id": {
                    "package": "example",
                    "path": ["dimension", {"node": 0}],
                    "ordinal": 0
                },
                "exponent": "1"
            }]
        });
        let parsed = NominalBody::from_value(&valid, JsonPointer::root()).expect("indexed ref");
        let term = super::super::CheckedTerm::NominalBody(Box::new(parsed));
        let encoded = quire_canonical::to_vec(&term, quire_canonical::Limits::new(1 << 22))
            .expect("indexed encoding");
        assert!(encoded
            .windows(b"\"node\":0".len())
            .any(|part| part == b"\"node\":0"));

        // The old inline operand spelling is not a V2 node ID. The NI-13
        // conformance control covers 100,001 valid graph-local indexed keys
        // on a 256 KiB stack (FR-038-AC-117/380).
        let mut id = json!({"package":"example","path":["base"],"ordinal":0});
        for _ in 0..10_000 {
            let mut wrapper = serde_json::Map::new();
            wrapper.insert("package".into(), Value::String("example".into()));
            wrapper.insert("path".into(), Value::Array(vec![id]));
            wrapper.insert("ordinal".into(), Value::from(0));
            id = Value::Object(wrapper);
        }
        let mut term = serde_json::Map::new();
        term.insert("dimension_node_id".into(), id);
        term.insert("exponent".into(), Value::String("1".into()));
        let mut body_members = serde_json::Map::new();
        body_members.insert("term".into(), Value::String("dimension".into()));
        body_members.insert("base".into(), Value::Bool(false));
        body_members.insert("terms".into(), Value::Array(vec![Value::Object(term)]));
        let body = Value::Object(body_members);
        assert!(matches!(
            NominalBody::from_value(&body, JsonPointer::root()),
            Err(ValidationFailure::Refused(refusal))
                if refusal.code == CheckedPackageRefusalCode::MalformedWire
                    && refusal.loci[0].path.as_ref().is_some_and(
                        |path| path.as_str() == "/terms/0/dimension_node_id"
                    )
        ));
        quire_canonical::drop_value(body);
    }

    #[trace("TC-048")]
    #[test]
    fn tc_048_integer_and_identifier_grammars_are_exact() {
        assert_eq!(
            integer_magnitude("0").map(|m| (m.digits, m.zero)),
            Some(("0", true))
        );
        assert_eq!(
            integer_magnitude("-12").map(|m| (m.digits, m.zero)),
            Some(("12", false))
        );
        assert_eq!(integer_magnitude("-0"), None);
        assert_eq!(integer_magnitude("012"), None);
        assert_eq!(integer_magnitude(""), None);
        assert!(!positive_integer("0"));
        assert!(is_identifier("_a1"));
        assert!(!is_identifier("1a"));
        assert!(!is_identifier(""));
    }

    /// The charged reader of a nominal body answers as the `Value` reader
    /// does on the same JSON, operand slots resolved through the node's row:
    /// the same body when it is closed, the same refusal when it is not
    /// (FR-038-AC-504).
    /// The first bad enum name is the one a front-to-back scan with a set
    /// finds: a non-identifier, or a repeat of an earlier name.
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn first_bad_name_agrees_with_the_set_scan() {
        let pool = ["a", "b", "", "a1", "9", "_x"];
        let mut cases: Vec<Vec<&str>> = vec![vec![]];
        for _ in 0..4 {
            cases = cases
                .into_iter()
                .flat_map(|prefix| {
                    pool.iter().map(move |name| {
                        let mut next = prefix.clone();
                        next.push(name);
                        next
                    })
                })
                .collect();
            for names in &cases {
                let mut seen = std::collections::BTreeSet::new();
                let expected = names
                    .iter()
                    .position(|name| !(is_identifier(name) && seen.insert(*name)));
                assert_eq!(first_bad_name(names.iter().copied()), expected, "{names:?}");
            }
        }
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn nominal_body_reads_as_the_value_reader_does() {
        let id = |name: &str| json!({"package":"example","path":[name],"ordinal":0});
        let slot = |k: u64| json!({"package":"example","path":["a",{"operand":k}],"ordinal":0});
        let node = |k: u64| json!({"package":"example","path":["a",{"node":k}],"ordinal":0});
        let rational = |n: &str, d: &str| json!({"numerator":n,"denominator":d});
        let unit = |dimension: Value, target: Value, scale: Value| {
            json!({"term":"unit","dimension_node_id":dimension,"target_unit_node_id":target,
                "scale":scale,"offset":rational("0","1")})
        };
        let bodies: Vec<(Value, Value)> = vec![
            // (body with operand slots, the same body with the slots resolved)
            (
                json!({"term":"enum","ordered":true,"members":["A","B","C"]}),
                json!({"term":"enum","ordered":true,"members":["A","B","C"]}),
            ),
            (
                json!({"term":"enum","ordered":true,"members":["A","A"]}),
                json!({"term":"enum","ordered":true,"members":["A","A"]}),
            ),
            (
                json!({"term":"enum","ordered":true,"members":["A","1b"]}),
                json!({"term":"enum","ordered":true,"members":["A","1b"]}),
            ),
            (
                json!({"term":"enum","ordered":true,"members":[]}),
                json!({"term":"enum","ordered":true,"members":[]}),
            ),
            (
                json!({"term":"dimension","base":false,"terms":[
                    {"dimension_node_id":slot(1),"exponent":"2"},
                    {"dimension_node_id":id("x"),"exponent":"-3"}]}),
                json!({"term":"dimension","base":false,"terms":[
                    {"dimension_node_id":node(11),"exponent":"2"},
                    {"dimension_node_id":id("x"),"exponent":"-3"}]}),
            ),
            (
                json!({"term":"dimension","base":true,"terms":[
                    {"dimension_node_id":id("x"),"exponent":"1"}]}),
                json!({"term":"dimension","base":true,"terms":[
                    {"dimension_node_id":id("x"),"exponent":"1"}]}),
            ),
            (
                json!({"term":"dimension","base":false,"terms":[
                    {"dimension_node_id":id("x"),"exponent":"01"}]}),
                json!({"term":"dimension","base":false,"terms":[
                    {"dimension_node_id":id("x"),"exponent":"01"}]}),
            ),
            (
                json!({"term":"dimension","base":false,"terms":[
                    {"dimension_node_id":{"package":"","path":["x"],"ordinal":0},"exponent":"1"}]}),
                json!({"term":"dimension","base":false,"terms":[
                    {"dimension_node_id":{"package":"","path":["x"],"ordinal":0},"exponent":"1"}]}),
            ),
            (
                unit(slot(0), Value::Null, rational("3", "7")),
                unit(node(10), Value::Null, rational("3", "7")),
            ),
            (
                unit(id("d"), slot(2), rational("3", "7")),
                unit(id("d"), node(12), rational("3", "7")),
            ),
            (
                unit(id("d"), id("t"), rational("3", "0")),
                unit(id("d"), id("t"), rational("3", "0")),
            ),
            (
                unit(id("d"), json!(7), rational("3", "7")),
                unit(id("d"), json!(7), rational("3", "7")),
            ),
            (
                json!({"term":"dimension_equivalence",
                    "dimension_node_ids":[slot(0), {"package":"dep","node":id("m")}],
                    "root_scale":rational("5","1")}),
                json!({"term":"dimension_equivalence",
                    "dimension_node_ids":[node(10), {"package":"dep","node":id("m")}],
                    "root_scale":rational("5","1")}),
            ),
            (
                json!({"term":"dimension_equivalence",
                    "dimension_node_ids":[id("a"), slot(1), id("c")],
                    "root_scale":rational("5","1")}),
                json!({"term":"dimension_equivalence",
                    "dimension_node_ids":[id("a"), node(11), id("c")],
                    "root_scale":rational("5","1")}),
            ),
            (
                json!({"term":"dimension_equivalence",
                    "dimension_node_ids":[id("a"), {"package":"dep","node":slot(0)}],
                    "root_scale":rational("5","1")}),
                json!({"term":"dimension_equivalence",
                    "dimension_node_ids":[id("a"), {"package":"dep","node":node(10)}],
                    "root_scale":rational("5","1")}),
            ),
            (json!([]), json!([])),
            (json!({"term":7}), json!({"term":7})),
            (json!({"term":"enum"}), json!({"term":"enum"})),
            (json!({"term":"other"}), json!({"term":"other"})),
        ];
        let row = [10_u64, 11, 12];
        let at = || JsonPointer::root().key("body");
        let mut closed = 0;
        for (slotted, resolved) in &bodies {
            let handle = CheckedJson::from_value(slotted).expect("canonical");
            let mut meter = WorkMeter::new(u64::MAX);
            let read = NominalBody::from_term(TermNode::of(&handle), &row, &at, &mut meter);
            let expected = NominalBody::from_value(resolved, at());
            match (&read, &expected) {
                (Ok(read), Ok(expected)) => {
                    closed += 1;
                    assert_eq!(read, expected, "{slotted}");
                }
                (Err(read), Err(expected)) => {
                    assert_eq!(format!("{read:?}"), format!("{expected:?}"), "{slotted}");
                }
                _ => panic!("{slotted}: {read:?} against {expected:?}"),
            }
        }
        assert!(closed >= 5, "the corpus holds closed bodies of every form");
    }

    /// Every list, name, number text and ID a nominal body keeps is charged
    /// before it is allocated: the bytes the allocator is asked for equal the
    /// bytes charged, the measured requirement admits and one byte below is
    /// denied at the graph locus (FR-038-AC-504).
    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn nominal_body_allocates_exactly_what_it_charges() {
        use super::super::native_join_budget::{NativeJoinLocusSource, PackageReadStage};
        let id = |name: &str| json!({"package":"example","path":[name,{"operand":0}],"ordinal":0});
        let rational = |n: &str, d: &str| json!({"numerator":n,"denominator":d});
        let bodies = [
            json!({"term":"enum","ordered":true,"members":["Red","Green","Blue","Alpha","Beta"]}),
            json!({"term":"dimension","base":false,"terms":[
                {"dimension_node_id":id("x"),"exponent":"2"},
                {"dimension_node_id":id("y"),"exponent":"-30"},
                {"dimension_node_id":id("z"),"exponent":"1"}]}),
            json!({"term":"unit","dimension_node_id":id("d"),"target_unit_node_id":id("t"),
                "scale":rational("3","7"),"offset":rational("-12345678901234567890","1")}),
            json!({"term":"dimension_equivalence",
                "dimension_node_ids":[id("a"), {"package":"dependency","node":{"package":"p","path":["m"],"ordinal":0}}],
                "root_scale":rational("5","1")}),
        ];
        let handles: Vec<CheckedJson> = bodies
            .iter()
            .map(|body| CheckedJson::from_value(body).expect("canonical"))
            .collect();
        let row = [4_u64];
        let at = || JsonPointer::root().key("body");
        let run = |limit: u64| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut read = Vec::with_capacity(handles.len());
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(2, "scalar_type");
                for handle in &handles {
                    match NominalBody::from_term(TermNode::of(handle), &row, &at, &mut meter) {
                        Ok(body) => read.push(body),
                        Err(_) => break,
                    }
                }
            });
            let complete = read.len() == handles.len();
            (
                complete.then(|| read.len()),
                measured.bytes_total,
                budget.read_storage_consumed(),
                budget.first_unpaid_locus(),
            )
        };
        let (read, allocated, charged, _) = run(u64::MAX);
        assert_eq!(read, Some(4));
        assert!(charged > 0);
        assert_eq!(allocated, charged, "allocated == requested");
        assert_eq!(run(charged).0, read, "the measured limit admits");
        let (short, _, _, locus) = run(charged - 1);
        assert_eq!(short, None, "one byte short is denied");
        let locus = locus.expect("a denied request has a locus");
        assert_eq!(
            locus.source,
            NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 2)
        );
    }
}
