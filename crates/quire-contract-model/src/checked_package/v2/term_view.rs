// SPDX-License-Identifier: AGPL-3.0-or-later
//! The body-term views over the retained package document: the node API the
//! strict read's validators call instead of reading a `serde_json::Value`.
//!
//! A [`TermNode`] is one value of the shared `quire_canonical::Document`
//! (a node body, a diagnostic detail, any term below them). Reading it
//! allocates nothing: members and elements are the reader's own storage, so
//! the only requests are the owned values a caller builds from it, each of
//! which this module charges to the read's budget before it is made.
//!
//! The accessors mirror the `Value` projections they replace
//! (`common::WireTerm`, `terms::subterms`, `CheckedTerm::view`), with the same
//! answers on the same JSON.
//!
//! Implements: FR-038 "Budget-charged strict read".

use super::budgeted_read::denial;
use super::checked_json::CheckedJson;
use super::document_de::DecodeContext;
use super::native_join_budget::NativeJoinBudget;
use super::read_charge::{Denied, ReadStorage, Site};
use super::typed_terms::TermConversionCause;
use super::{
    ApplicationOperator, BodyTerm, CheckedNodeId, CheckedNodePathSegment,
    CheckedOperationAnchorBody, CheckedTerm, LiteralKind,
};
use crate::checked_package::common::{TermField, TermTag, ValidationFailure};
use crate::checked_package::shared::CheckedOperandEntry;
use crate::identity::ExecutionPoint;
use quire_canonical::{Document, Items, Members, Node, NodeRef};
use serde::Deserialize;
use serde_json::error::Category;
use std::sync::Arc;

/// What a value read as a node ID turned out to be.
#[derive(Debug, Eq, PartialEq)]
pub(in crate::checked_package) enum NodeIdRead {
    /// A valid node ID.
    Id(CheckedNodeId),
    /// The shape of a node ID whose package or path is empty or whose ordinal
    /// exceeds 2^53 - 1, with the ID it spells.
    Invalid(CheckedNodeId),
    /// Not the shape of a node ID.
    Malformed,
}

impl NodeIdRead {
    /// The valid ID, if there is one.
    pub(in crate::checked_package) fn valid(self) -> Option<CheckedNodeId> {
        match self {
            Self::Id(id) => Some(id),
            Self::Invalid(_) | Self::Malformed => None,
        }
    }

    /// The ID the value spells, valid or not.
    pub(in crate::checked_package) fn spelled(self) -> Option<CheckedNodeId> {
        match self {
            Self::Id(id) | Self::Invalid(id) => Some(id),
            Self::Malformed => None,
        }
    }
}

/// One value of the package document, read as a term.
#[derive(Clone, Copy)]
pub(in crate::checked_package) struct TermNode<'d> {
    node: NodeRef<'d>,
}

impl<'d> TermNode<'d> {
    /// The value a handle names.
    pub(in crate::checked_package) fn of(json: &'d CheckedJson) -> Self {
        Self { node: json.node() }
    }

    /// The document value `node`.
    pub(in crate::checked_package) fn from_ref(node: NodeRef<'d>) -> Self {
        Self { node }
    }

    /// The underlying document value.
    pub(in crate::checked_package) fn node_ref(self) -> NodeRef<'d> {
        self.node
    }

    pub(in crate::checked_package) fn is_null(self) -> bool {
        matches!(self.node.node(), Node::Null)
    }

    /// Whether this is a literal payload the V2 grammar admits: a boolean, a
    /// string, `null`, or an integer token within the 64-bit range.
    pub(in crate::checked_package) fn literal_value_ok(self) -> bool {
        match self.node.node() {
            Node::Bool(_) | Node::String(_) | Node::Null => true,
            Node::Number(number) => {
                number.text().parse::<i64>().is_ok() || number.text().parse::<u64>().is_ok()
            }
            Node::Array(_) | Node::Object(_) => false,
        }
    }

    pub(in crate::checked_package) fn is_object(self) -> bool {
        matches!(self.node.node(), Node::Object(_))
    }

    pub(in crate::checked_package) fn is_array(self) -> bool {
        matches!(self.node.node(), Node::Array(_))
    }

    pub(in crate::checked_package) fn as_str(self) -> Option<&'d str> {
        match self.node.node() {
            Node::String(text) => Some(text),
            _ => None,
        }
    }

    pub(in crate::checked_package) fn as_bool(self) -> Option<bool> {
        match self.node.node() {
            Node::Bool(flag) => Some(flag),
            _ => None,
        }
    }

    /// The value as a nonnegative integer, when its text is one.
    pub(in crate::checked_package) fn as_u64(self) -> Option<u64> {
        match self.node.node() {
            Node::Number(number) => number.text().parse().ok(),
            _ => None,
        }
    }

    /// The member `name` of an object.
    pub(in crate::checked_package) fn get(self, name: &str) -> Option<TermNode<'d>> {
        match self.node.node() {
            Node::Object(mut members) => members
                .find(|(member, _)| *member == name)
                .map(|(_, value)| Self { node: value }),
            _ => None,
        }
    }

    /// The elements of an array.
    pub(in crate::checked_package) fn items(self) -> Option<TermItems<'d>> {
        match self.node.node() {
            Node::Array(items) => Some(TermItems { items }),
            _ => None,
        }
    }

    /// The members of an object, in document order.
    pub(in crate::checked_package) fn members(self) -> Option<TermMembers<'d>> {
        match self.node.node() {
            Node::Object(members) => Some(TermMembers { members }),
            _ => None,
        }
    }

    /// Whether this is an object with exactly the members `expected`.
    pub(in crate::checked_package) fn exact_members(self, expected: &[&str]) -> bool {
        match self.node.node() {
            Node::Object(members) => {
                members.len() == expected.len()
                    && expected
                        .iter()
                        .all(|name| members.clone().any(|(member, _)| member == *name))
            }
            _ => false,
        }
    }

    /// The `term` discriminator, decoded.
    pub(in crate::checked_package) fn tag(self) -> TermTag {
        match self.get("term") {
            None => TermTag::Missing,
            Some(tag) => match tag.as_str() {
                Some("extension_form") => TermTag::ExtensionForm,
                Some(tag) => BodyTerm::from_wire(tag).map_or(TermTag::Unknown, TermTag::Known),
                None => TermTag::NonString,
            },
        }
    }

    /// The `term` tag when it names a body term of the closed vocabulary.
    pub(in crate::checked_package) fn kind(self) -> Option<BodyTerm> {
        match self.tag() {
            TermTag::Known(term) => Some(term),
            TermTag::ExtensionForm | TermTag::Missing | TermTag::NonString | TermTag::Unknown => {
                None
            }
        }
    }

    pub(in crate::checked_package) fn field(self, field: TermField) -> Option<TermNode<'d>> {
        self.get(field.as_wire())
    }

    pub(in crate::checked_package) fn array(self, field: TermField) -> Option<TermItems<'d>> {
        self.field(field)?.items()
    }

    pub(in crate::checked_package) fn literal_kind(self) -> Option<LiteralKind> {
        LiteralKind::from_wire(self.field(TermField::ValueKind)?.as_str()?)
    }

    pub(in crate::checked_package) fn operator(self) -> Option<ApplicationOperator> {
        ApplicationOperator::from_wire(self.field(TermField::Operator)?.as_str()?)
    }

    /// The closed shape of an ordinary term (an application, aggregate,
    /// binding, reference or literal with exactly its members), or `None`.
    pub(in crate::checked_package) fn view(self) -> Option<TermView<'d>> {
        match self.tag() {
            TermTag::Known(BodyTerm::Application)
                if self.exact_members(&[
                    "term",
                    "operator",
                    "operation",
                    "result_type",
                    "arguments",
                ]) =>
            {
                Some(TermView::Application {
                    operator: self.operator()?,
                    operation: self.field(TermField::Operation)?,
                    result_type: self.field(TermField::ResultType)?,
                    arguments: self.array(TermField::Arguments)?,
                })
            }
            TermTag::Known(BodyTerm::Aggregate) if self.exact_members(&["term", "members"]) => {
                Some(TermView::Aggregate {
                    members: self.array(TermField::Members)?,
                })
            }
            TermTag::Known(BodyTerm::Binding) if self.exact_members(&["term", "name", "value"]) => {
                Some(TermView::Binding {
                    name: self.field(TermField::Name)?.as_str()?,
                    value: self.field(TermField::Value)?,
                })
            }
            TermTag::Known(BodyTerm::Reference) if self.exact_members(&["term", "target"]) => {
                Some(TermView::Reference {
                    target: self.field(TermField::Target)?,
                })
            }
            TermTag::Known(BodyTerm::Literal)
                if self.exact_members(&["term", "type", "value_kind", "value"]) =>
            {
                Some(TermView::Literal {
                    type_id: self.field(TermField::Type)?,
                    value_kind: self.literal_kind()?,
                    value: self.field(TermField::Value)?,
                })
            }
            _ => None,
        }
    }

    /// The terms directly below this one, in document order: an application's
    /// `arguments`, an aggregate's `members` and a binding's `value`. Any other
    /// value, and a term whose member is absent or of the wrong kind, has none.
    pub(in crate::checked_package) fn subterms(self) -> TermSubterms<'d> {
        let (key, listed, valued) = match self.kind() {
            Some(BodyTerm::Application) => ("arguments", self.array(TermField::Arguments), None),
            Some(BodyTerm::Aggregate) => ("members", self.array(TermField::Members), None),
            Some(BodyTerm::Binding) => ("value", None, self.field(TermField::Value)),
            Some(
                BodyTerm::Literal
                | BodyTerm::Reference
                | BodyTerm::DependencyReference
                | BodyTerm::Frame
                | BodyTerm::AbstractionRelation
                | BodyTerm::ImplementationGuarantee
                | BodyTerm::ComponentComposition,
            )
            | None => ("", None, None),
        };
        TermSubterms {
            key,
            listed,
            position: 0,
            valued,
        }
    }

    /// Reads this value as a V2 wire node ID: an object of exactly a
    /// `package` string, a `path` of names and `{"node": i}` /
    /// `{"operand": k}` segments no larger than 2^53 - 1, and an `ordinal`.
    /// A value of another shape is [`NodeIdRead::Malformed`]; one of that shape
    /// whose package or path is empty or whose ordinal is above 2^53 - 1 is
    /// [`NodeIdRead::Invalid`], carrying the ID it spells. The ID's package,
    /// path and names are charged to `sink` at `site` before each is
    /// allocated.
    pub(in crate::checked_package) fn node_id<S: ReadStorage + ?Sized>(
        self,
        sink: &mut S,
        site: Site,
    ) -> Result<NodeIdRead, Denied> {
        self.node_id_in(sink, site, &[])
    }

    /// [`Self::node_id`] for an ID written in the node whose operand row is
    /// `row`: an operand slot `{"operand": k}` reads as the index segment of
    /// the node `row[k]` names, as the reader's resolved wire gives it. A slot
    /// the row does not name stays a slot.
    pub(in crate::checked_package) fn node_id_in<S: ReadStorage + ?Sized>(
        self,
        sink: &mut S,
        site: Site,
        row: &[CheckedOperandEntry],
    ) -> Result<NodeIdRead, Denied> {
        const MAXIMUM: u64 = 9_007_199_254_740_991;
        if !self.exact_members(&["package", "path", "ordinal"]) {
            return Ok(NodeIdRead::Malformed);
        }
        let (Some(package), Some(path), Some(ordinal)) = (
            self.get("package").and_then(TermNode::as_str),
            self.array_member("path"),
            self.get("ordinal").and_then(TermNode::as_u64),
        ) else {
            return Ok(NodeIdRead::Malformed);
        };
        // Check every segment before the first allocation.
        for segment in path.clone() {
            let valid = match segment.segment_form() {
                Some(SegmentForm::Name(_)) => true,
                Some(SegmentForm::Node(index) | SegmentForm::Operand(index)) => index <= MAXIMUM,
                None => false,
            };
            if !valid {
                return Ok(NodeIdRead::Malformed);
            }
        }
        let valid = !package.is_empty() && path.len() != 0 && ordinal <= MAXIMUM;
        sink.reserve(package.len() as u128, site)?;
        let package: Box<str> = package.into();
        sink.reserve(
            (path.len() as u128)
                .saturating_mul(std::mem::size_of::<CheckedNodePathSegment>() as u128),
            site,
        )?;
        let mut segments = Vec::with_capacity(path.len());
        for segment in path {
            segments.push(match segment.segment_form() {
                Some(SegmentForm::Name(name)) => {
                    sink.reserve(name.len() as u128, site)?;
                    CheckedNodePathSegment::Name(name.into())
                }
                Some(SegmentForm::Node(index)) => CheckedNodePathSegment::Index(index),
                Some(SegmentForm::Operand(slot)) => {
                    match usize::try_from(slot).ok().and_then(|slot| row.get(slot)) {
                        None => CheckedNodePathSegment::Operand(slot),
                        Some(CheckedOperandEntry::Node(entry)) => {
                            CheckedNodePathSegment::Index(*entry)
                        }
                        Some(CheckedOperandEntry::Dependency(operand)) => {
                            sink.reserve(operand.package.len() as u128, site)?;
                            CheckedNodePathSegment::Dependency {
                                package: operand.package.clone(),
                                node: operand.node,
                            }
                        }
                    }
                }
                None => return Ok(NodeIdRead::Malformed),
            });
        }
        let id = CheckedNodeId {
            package,
            path: segments,
            ordinal,
        };
        Ok(if valid {
            NodeIdRead::Id(id)
        } else {
            NodeIdRead::Invalid(id)
        })
    }

    /// Decodes this term, which the reader's flat and closed-shape checks
    /// already accepted, into its retained typed form: the node view of
    /// `CheckedTerm::decode_admitted`. Every string, sequence and box it
    /// requests is charged to `budget` at `site` before it is made; a refusal
    /// of the budget is the outer error, and a term that is not decodable is
    /// the inner one.
    pub(in crate::checked_package) fn decode(
        self,
        document: &'d Arc<Document>,
        budget: &mut NativeJoinBudget,
        site: Site,
    ) -> Result<Result<CheckedTerm, TermConversionCause>, ValidationFailure> {
        self.decode_in(document, budget, site, &[])
    }

    /// [`Self::decode`] for a body written in the node whose operand row is
    /// `row`: every node ID in it reads its operand slots through the row, as
    /// in [`Self::node_id_in`].
    pub(in crate::checked_package) fn decode_in(
        self,
        document: &'d Arc<Document>,
        budget: &mut NativeJoinBudget,
        site: Site,
        row: &'d [CheckedOperandEntry],
    ) -> Result<Result<CheckedTerm, TermConversionCause>, ValidationFailure> {
        let context = DecodeContext::at_locus(document, budget, site.locus()).with_operand_row(row);
        let decoded = decode_term(&context, document, self);
        let denied = context.denied();
        *budget = context.finish();
        if denied {
            return Err(denial(budget));
        }
        Ok(decoded)
    }

    /// Decodes an operation-anchor body, whose optional fourth binding holds
    /// a bare execution point rather than a semantic term. The three semantic
    /// members and the point's owned name are charged before allocation.
    pub(in crate::checked_package) fn decode_anchor_in(
        self,
        document: &'d Arc<Document>,
        budget: &mut NativeJoinBudget,
        site: Site,
        row: &'d [CheckedOperandEntry],
    ) -> Result<Result<CheckedTerm, TermConversionCause>, ValidationFailure> {
        let Some(TermView::Aggregate { mut members }) = self.view() else {
            return self.decode_in(document, budget, site, row);
        };
        let has_point = members.len() == 4;
        if members.len() != 3 && !has_point {
            return self.decode_in(document, budget, site, row);
        }
        let Some(context_term) = members.next() else {
            return self.decode_in(document, budget, site, row);
        };
        let Some(operation_term) = members.next() else {
            return self.decode_in(document, budget, site, row);
        };
        let Some(frame_term) = members.next() else {
            return self.decode_in(document, budget, site, row);
        };
        let point_value = if has_point {
            let Some(point_binding) = members.next() else {
                return self.decode_in(document, budget, site, row);
            };
            let Some(TermView::Binding {
                name: "execution_point",
                value,
            }) = point_binding.view()
            else {
                return Ok(Err(TermConversionCause::MalformedMember {
                    member: "execution_point",
                    category: Category::Data,
                }));
            };
            Some(value)
        } else {
            None
        };

        let decode_member = |member: TermNode<'d>, budget: &mut NativeJoinBudget| {
            member.decode_in(document, budget, site, row)
        };
        let context_member = match decode_member(context_term, budget)? {
            Ok(term) => term,
            Err(cause) => return Ok(Err(cause)),
        };
        let operation = match decode_member(operation_term, budget)? {
            Ok(term) => term,
            Err(cause) => return Ok(Err(cause)),
        };
        let frame = match decode_member(frame_term, budget)? {
            Ok(term) => term,
            Err(cause) => return Ok(Err(cause)),
        };

        let point = if let Some(value) = point_value {
            let decode_context =
                DecodeContext::at_locus(document, budget, site.locus()).with_operand_row(row);
            let point = ExecutionPoint::deserialize(decode_context.value_at(value.node_ref()));
            let denied = decode_context.denied();
            *budget = decode_context.finish();
            if denied {
                return Err(denial(budget));
            }
            match point {
                Ok(point) => Some(point),
                Err(_) => {
                    return Ok(Err(TermConversionCause::MalformedMember {
                        member: "execution_point",
                        category: Category::Data,
                    }));
                }
            }
        } else {
            None
        };
        super::read_charge::charge_items::<CheckedOperationAnchorBody>(budget, site, 1)?;
        Ok(Ok(CheckedTerm::OperationAnchorBody(Box::new(
            CheckedOperationAnchorBody {
                members: [context_member, operation, frame],
                execution_point: point,
            },
        ))))
    }

    fn array_member(self, name: &str) -> Option<TermItems<'d>> {
        self.get(name)?.items()
    }

    fn segment_form(self) -> Option<SegmentForm<'d>> {
        if let Some(name) = self.as_str() {
            return Some(SegmentForm::Name(name));
        }
        let mut members = self.members()?;
        let ((name, value), None) = (members.next()?, members.next()) else {
            return None;
        };
        let index = value.as_u64()?;
        match name {
            "node" => Some(SegmentForm::Node(index)),
            "operand" => Some(SegmentForm::Operand(index)),
            _ => None,
        }
    }
}

/// One member of `term` decoded as a `T`.
fn decode_field<'d, T: Deserialize<'d>>(
    context: &DecodeContext<'d>,
    term: TermNode<'d>,
    name: TermField,
) -> Result<T, TermConversionCause> {
    let member = term.field(name).ok_or(TermConversionCause::MissingMember {
        member: name.as_wire(),
    })?;
    T::deserialize(context.value_at(member.node_ref())).map_err(|_| {
        TermConversionCause::MalformedMember {
            member: name.as_wire(),
            category: Category::Data,
        }
    })
}

/// A typed flat-body list, prepaid before serde requests its final block.
fn decode_vec_field<'d, T: Deserialize<'d>>(
    context: &DecodeContext<'d>,
    term: TermNode<'d>,
    name: TermField,
) -> Result<Vec<T>, TermConversionCause> {
    let member = term.field(name).ok_or(TermConversionCause::MissingMember {
        member: name.as_wire(),
    })?;
    super::document_de::exact_vec(context.value_at(member.node_ref())).map_err(|_| {
        TermConversionCause::MalformedMember {
            member: name.as_wire(),
            category: Category::Data,
        }
    })
}

/// The terms of an array member, each decoded; the list is charged at its
/// exact length first.
fn decode_children<'d>(
    context: &DecodeContext<'d>,
    document: &'d Arc<Document>,
    term: TermNode<'d>,
    name: TermField,
) -> Result<Vec<CheckedTerm>, TermConversionCause> {
    let items = term
        .field(name)
        .ok_or(TermConversionCause::MissingMember {
            member: name.as_wire(),
        })?
        .items()
        .ok_or(TermConversionCause::MalformedMember {
            member: name.as_wire(),
            category: Category::Data,
        })?;
    let refused = || TermConversionCause::MalformedMember {
        member: name.as_wire(),
        category: Category::Data,
    };
    if !context.charge_bytes(
        items
            .len()
            .saturating_mul(std::mem::size_of::<CheckedTerm>()),
    ) {
        return Err(refused());
    }
    let mut children = Vec::with_capacity(items.len());
    for item in items {
        children.push(decode_term(context, document, item)?);
    }
    Ok(children)
}

/// Borrowed shape guards keep owned serde diagnostics off the projection's
/// malformed leaf paths. The semantic validator still owns value meaning.
trait RefinementAtom<'d>: Deserialize<'d> {
    fn accepts(node: TermNode<'d>) -> bool;
}

impl<'d> RefinementAtom<'d> for Box<str> {
    fn accepts(node: TermNode<'d>) -> bool {
        node.as_str().is_some()
    }
}
impl<'d> RefinementAtom<'d> for CheckedNodeId {
    fn accepts(node: TermNode<'d>) -> bool {
        flat_shape(node, FlatShape::Id)
    }
}
impl<'d> RefinementAtom<'d> for super::CheckedArtifactRef {
    fn accepts(node: TermNode<'d>) -> bool {
        node.exact_members(&["authority", "identity"])
            && node.get("authority").and_then(TermNode::as_str).is_some()
            && node.get("identity").and_then(TermNode::as_str).is_some()
    }
}
impl<'d> RefinementAtom<'d> for super::refinement::RefinementFairnessKind {
    fn accepts(node: TermNode<'d>) -> bool {
        matches!(node.as_str(), Some("weak" | "strong"))
    }
}
impl<'d> RefinementAtom<'d> for super::refinement::RefinementGranularity {
    fn accepts(node: TermNode<'d>) -> bool {
        matches!(node.as_str(), Some("each" | "whole"))
    }
}
impl<'d> RefinementAtom<'d> for super::refinement::RefinementMemoryModel {
    fn accepts(node: TermNode<'d>) -> bool {
        matches!(node.as_str(), Some("sc" | "tso" | "ra"))
    }
}
impl<'d> RefinementAtom<'d> for super::refinement::RefinementTsoModel {
    fn accepts(node: TermNode<'d>) -> bool {
        matches!(node.as_str(), Some("tso"))
    }
}
impl<'d> RefinementAtom<'d> for super::refinement::RefinementDrain {
    fn accepts(node: TermNode<'d>) -> bool {
        matches!(node.as_str(), Some("store-buffer-flush"))
    }
}

/// The refinement wire has untagged alternatives and nested semantic terms.
/// Project those alternatives directly, avoiding serde's untagged buffering
/// while retaining the same semantic-term decoder used by every other body.
struct RefinementDocument<'a, 'd> {
    context: &'a DecodeContext<'d>,
    document: &'d Arc<Document>,
}

impl<'d> RefinementDocument<'_, 'd> {
    fn malformed() -> TermConversionCause {
        TermConversionCause::MalformedMember {
            member: "refinement",
            category: Category::Data,
        }
    }
    fn member(&self, node: TermNode<'d>, name: &str) -> Result<TermNode<'d>, TermConversionCause> {
        node.get(name).ok_or_else(Self::malformed)
    }
    fn shape(
        &self,
        node: TermNode<'d>,
        required: &[&str],
        optional: &[&str],
    ) -> Result<(), TermConversionCause> {
        if required.iter().all(|name| node.get(name).is_some())
            && node.members().is_some_and(|mut members| {
                members.all(|(name, _)| required.contains(&name) || optional.contains(&name))
            })
        {
            Ok(())
        } else {
            Err(Self::malformed())
        }
    }
    fn atom<T: RefinementAtom<'d>>(&self, node: TermNode<'d>) -> Result<T, TermConversionCause> {
        if !T::accepts(node) {
            return Err(Self::malformed());
        }
        T::deserialize(self.context.value_at(node.node_ref())).map_err(|_| Self::malformed())
    }
    fn field<T: RefinementAtom<'d>>(
        &self,
        node: TermNode<'d>,
        name: &str,
    ) -> Result<T, TermConversionCause> {
        self.atom(self.member(node, name)?)
    }
    fn list<T>(
        &self,
        node: TermNode<'d>,
        parse: impl Fn(TermNode<'d>) -> Result<T, TermConversionCause>,
    ) -> Result<Vec<T>, TermConversionCause> {
        let items = node.items().ok_or_else(Self::malformed)?;
        if !self
            .context
            .charge_bytes(items.len().saturating_mul(std::mem::size_of::<T>()))
        {
            return Err(Self::malformed());
        }
        let mut result = Vec::with_capacity(items.len());
        for item in items {
            result.push(parse(item)?);
        }
        Ok(result)
    }
    fn rows<T>(
        &self,
        node: TermNode<'d>,
        name: &str,
        parse: impl Fn(TermNode<'d>) -> Result<T, TermConversionCause>,
    ) -> Result<Vec<T>, TermConversionCause> {
        self.list(self.member(node, name)?, parse)
    }
    fn boxed<T>(
        &self,
        parse: impl FnOnce() -> Result<T, TermConversionCause>,
    ) -> Result<Box<T>, TermConversionCause> {
        if !self.context.charge_bytes(std::mem::size_of::<T>()) {
            return Err(Self::malformed());
        }
        Ok(Box::new(parse()?))
    }
    fn key(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementKey, TermConversionCause> {
        use super::refinement::{RefinementKey, RefinementPackageNodeKey};
        if node.get("node").is_some() {
            self.shape(node, &["package", "node"], &[])?;
            Ok(RefinementKey::External(RefinementPackageNodeKey {
                package: self.field(node, "package")?,
                node: self.field(node, "node")?,
            }))
        } else {
            self.atom(node).map(RefinementKey::Local)
        }
    }
    fn term(&self, node: TermNode<'d>) -> Result<CheckedTerm, TermConversionCause> {
        decode_term(self.context, self.document, node)
    }
    fn argument(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementStepArgument, TermConversionCause> {
        use super::refinement::{RefinementOpen, RefinementStepArgument};
        if node.get("term").and_then(TermNode::as_str) == Some("open") {
            self.shape(node, &["term"], &[])?;
            Ok(RefinementStepArgument::Open(RefinementOpen {
                term: self.field(node, "term")?,
            }))
        } else {
            self.boxed(|| self.term(node))
                .map(RefinementStepArgument::Term)
        }
    }
    fn target(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementStepTarget, TermConversionCause> {
        use super::refinement::RefinementStepTarget;
        match node.get("kind").and_then(TermNode::as_str) {
            Some("operation") => {
                self.shape(node, &["kind", "operation", "receiver", "arguments"], &[])?;
                Ok(RefinementStepTarget::Operation {
                    operation: self.key(self.member(node, "operation")?)?,
                    receiver: self.boxed(|| self.term(self.member(node, "receiver")?))?,
                    arguments: self.rows(node, "arguments", |n| self.argument(n))?,
                })
            }
            Some("protocol_step") => {
                self.shape(node, &["kind", "class", "arguments"], &["node"])?;
                Ok(RefinementStepTarget::ProtocolStep {
                    class: self.field(node, "class")?,
                    node: node.get("node").map(|n| self.key(n)).transpose()?,
                    arguments: self.rows(node, "arguments", |n| self.argument(n))?,
                })
            }
            Some("stutter") => {
                self.shape(node, &["kind"], &[])?;
                Ok(RefinementStepTarget::Stutter)
            }
            Some("any") => {
                self.shape(node, &["kind"], &[])?;
                Ok(RefinementStepTarget::Any)
            }
            _ => Err(Self::malformed()),
        }
    }
    fn memory(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementMemoryClass, TermConversionCause> {
        use super::refinement::RefinementMemoryClass;
        match node.get("class").and_then(TermNode::as_str) {
            Some("memory drain") => {
                self.shape(node, &["class", "memory_model", "drain"], &[])?;
                Ok(RefinementMemoryClass::Drain {
                    memory_model: self.field(node, "memory_model")?,
                    drain: self.field(node, "drain")?,
                })
            }
            Some("memory flush") => {
                self.shape(node, &["class", "memory_model"], &[])?;
                Ok(RefinementMemoryClass::Flush {
                    memory_model: self.field(node, "memory_model")?,
                })
            }
            _ => Err(Self::malformed()),
        }
    }
    fn population(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementPopulation, TermConversionCause> {
        self.shape(node, &["abstract", "concrete"], &[])?;
        Ok(super::refinement::RefinementPopulation {
            r#abstract: self.key(self.member(node, "abstract")?)?,
            concrete: self.field(node, "concrete")?,
        })
    }
    fn field_row(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementField, TermConversionCause> {
        self.shape(node, &["name", "expression"], &[])?;
        Ok(super::refinement::RefinementField {
            name: self.field(node, "name")?,
            expression: self.term(self.member(node, "expression")?)?,
        })
    }
    fn object(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementObject, TermConversionCause> {
        self.shape(node, &["abstract_type", "concrete_type", "fields"], &[])?;
        Ok(super::refinement::RefinementObject {
            abstract_type: self.key(self.member(node, "abstract_type")?)?,
            concrete_type: self.field(node, "concrete_type")?,
            fields: self.rows(node, "fields", |n| self.field_row(n))?,
        })
    }
    fn update(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementHistoryUpdate, TermConversionCause> {
        self.shape(node, &["operation", "expression"], &[])?;
        Ok(super::refinement::RefinementHistoryUpdate {
            operation: self.field(node, "operation")?,
            expression: self.term(self.member(node, "expression")?)?,
        })
    }
    fn history(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementHistory, TermConversionCause> {
        self.shape(
            node,
            &["concrete_type", "name", "type", "initial", "updates"],
            &[],
        )?;
        Ok(super::refinement::RefinementHistory {
            concrete_type: self.field(node, "concrete_type")?,
            name: self.field(node, "name")?,
            type_id: self.field(node, "type")?,
            initial: self.term(self.member(node, "initial")?)?,
            updates: self.rows(node, "updates", |n| self.update(n))?,
        })
    }
    fn assume(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementAssume, TermConversionCause> {
        self.shape(node, &["operation", "kind", "granularity"], &[])?;
        Ok(super::refinement::RefinementAssume {
            operation: self.field(node, "operation")?,
            kind: self.field(node, "kind")?,
            granularity: self.field(node, "granularity")?,
        })
    }
    fn ensure(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementEnsure, TermConversionCause> {
        self.shape(node, &["operation", "kind", "granularity"], &[])?;
        Ok(super::refinement::RefinementEnsure {
            operation: self.key(self.member(node, "operation")?)?,
            kind: self.field(node, "kind")?,
            granularity: self.field(node, "granularity")?,
        })
    }
    fn step(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementStep, TermConversionCause> {
        self.shape(node, &["source_class", "target"], &["source_node"])?;
        Ok(super::refinement::RefinementStep {
            source_class: self.field(node, "source_class")?,
            source_node: node.get("source_node").map(|n| self.atom(n)).transpose()?,
            target: self.target(self.member(node, "target")?)?,
        })
    }
    fn concrete_class(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementConcreteClass, TermConversionCause> {
        use super::refinement::RefinementConcreteClass;
        if node.get("node").is_none() {
            return self.memory(node).map(RefinementConcreteClass::Memory);
        }
        self.shape(node, &["class", "node", "binder_types"], &[])?;
        Ok(RefinementConcreteClass::Declared {
            class: self.field(node, "class")?,
            node: self.field(node, "node")?,
            binder_types: self.rows(node, "binder_types", |n| self.atom(n))?,
        })
    }
    fn abstract_class(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementAbstractClass, TermConversionCause> {
        use super::refinement::RefinementAbstractClass;
        if node.get("node").is_none() {
            return self.memory(node).map(RefinementAbstractClass::Memory);
        }
        self.shape(node, &["class", "node", "binder_types"], &[])?;
        Ok(RefinementAbstractClass::Declared {
            class: self.field(node, "class")?,
            node: self.key(self.member(node, "node")?)?,
            binder_types: self.rows(node, "binder_types", |n| self.key(n))?,
        })
    }
    fn catalog(
        &self,
        node: TermNode<'d>,
    ) -> Result<super::refinement::RefinementClassCatalog, TermConversionCause> {
        self.shape(node, &["concrete", "abstract"], &[])?;
        Ok(super::refinement::RefinementClassCatalog {
            concrete: self.rows(node, "concrete", |n| self.concrete_class(n))?,
            r#abstract: self.rows(node, "abstract", |n| self.abstract_class(n))?,
        })
    }
    fn body(&self, node: TermNode<'d>) -> Result<CheckedTerm, TermConversionCause> {
        self.shape(
            node,
            &[
                "term",
                "abstract",
                "profile",
                "populations",
                "objects",
                "steps",
                "histories",
                "assume",
                "ensure",
            ],
            &["protocol", "abstract_internal", "class_catalog"],
        )?;
        // Match the existing row decoder's construction order, including the
        // late abstract/profile decode after the ordered mapping tables.
        let populations = self.rows(node, "populations", |n| self.population(n))?;
        let objects = self.rows(node, "objects", |n| self.object(n))?;
        let steps = self.rows(node, "steps", |n| self.step(n))?;
        let histories = self.rows(node, "histories", |n| self.history(n))?;
        let assume = self.rows(node, "assume", |n| self.assume(n))?;
        let ensure = self.rows(node, "ensure", |n| self.ensure(n))?;
        let class_catalog = node
            .get("class_catalog")
            .map(|n| self.catalog(n))
            .transpose()?;
        Ok(CheckedTerm::Refinement {
            abstract_key: self.boxed(|| self.key(self.member(node, "abstract")?))?,
            profile: self.field(node, "profile")?,
            populations,
            objects,
            steps,
            histories,
            assume,
            ensure,
            protocol: node
                .get("protocol")
                .map(|n| self.boxed(|| self.key(n)))
                .transpose()?,
            abstract_internal: node
                .get("abstract_internal")
                .map(|n| self.list(n, |n| self.atom(n)))
                .transpose()?,
            class_catalog,
        })
    }
}

// Check closed flat shapes by borrowing the document. Serde's owned schema
// errors are never constructed for these bodies; malformed bodies retain the
// existing deferred flat-correspondence refusal in the graph validator.
#[derive(Clone, Copy)]
enum FlatShape {
    Guarantee,
    Composition,
    Progress,
    Fairness,
    Environment,
    Component,
    Clause,
    Premise,
    Text,
    Strength,
    Granularity,
    Measure,
    Supply,
    Id,
    List(&'static FlatShape),
}

fn flat_shape(node: TermNode<'_>, kind: FlatShape) -> bool {
    use FlatShape::*;
    let fields: &[(&str, FlatShape)] = match kind {
        Guarantee => &[
            ("term",Text),("clause",Id),("relation",Id),
            ("initial",List(&Text)),("progress",List(&Progress)),("environment",List(&Environment)),
        ],
        Composition => &[
            ("term",Text),("rule",Text),("components",List(&Component)),
            ("guarantees",List(&Clause)),("relies",List(&Premise)),
            ("liveness_dependencies",List(&Premise)),("supplies",List(&Fairness)),
            ("environment",List(&Environment)),
        ],
        Progress => &[("context",Id),("operation",Text),("measure",Measure)],
        Fairness => &[("kind",Strength),("granularity",Granularity),("context",Id),("operation",Text)],
        Environment => &[("name",Text),("supplier",Text),("supplies",Supply)],
        Component => &[("name",Text),("package",Text),("subject",Id)],
        Clause => &[("component",Text),("clause",Id)],
        Premise => &[("guarantee",Clause),("on",Clause)],
        Text => return node.as_str().is_some(),
        Strength => return matches!(node.as_str(),Some("weak"|"strong")),
        Granularity => return matches!(node.as_str(),Some("whole"|"each")),
        Measure => return (node.exact_members(&["decreases"]) && node.get("decreases").is_some_and(|n| flat_shape(n,List(&Text)))) || (node.exact_members(&["bound"]) && node.get("bound").and_then(TermNode::as_u64).is_some()),
        Supply => return (node.exact_members(&["fairness"]) && node.get("fairness").is_some_and(|n| flat_shape(n,Fairness))) || (node.exact_members(&["assumption"]) && node.get("assumption").is_some_and(|n| flat_shape(n,Id))),
        Id => return node.exact_members(&["package","path","ordinal"]) && node.get("package").and_then(TermNode::as_str).is_some() && node.get("ordinal").and_then(TermNode::as_u64).is_some() && node.get("path").and_then(TermNode::items).is_some_and(|mut path| path.all(|n| matches!(n.segment_form(),Some(SegmentForm::Name(_))) || matches!(n.segment_form(),Some(SegmentForm::Node(i)|SegmentForm::Operand(i)) if i <= 9_007_199_254_740_991))),
        List(kind) => return node.items().is_some_and(|mut items|items.all(|n|flat_shape(n,*kind))),
    };
    node.members()
        .is_some_and(|members| members.len() == fields.len())
        && fields
            .iter()
            .all(|(name, kind)| node.get(name).is_some_and(|n| flat_shape(n, *kind)))
}

fn decode_term<'d>(
    context: &DecodeContext<'d>,
    document: &'d Arc<Document>,
    term: TermNode<'d>,
) -> Result<CheckedTerm, TermConversionCause> {
    if let Some(kind @ ("implementation_guarantee" | "component_composition")) =
        term.get("term").and_then(TermNode::as_str)
    {
        let shape = if kind == "implementation_guarantee" {
            FlatShape::Guarantee
        } else {
            FlatShape::Composition
        };
        if !flat_shape(term, shape) {
            return Err(TermConversionCause::MalformedMember {
                member: "body",
                category: Category::Data,
            });
        }
    }
    if term.get("term").and_then(TermNode::as_str) == Some("refinement") {
        return RefinementDocument { context, document }.body(term);
    }
    match term.tag() {
        TermTag::Known(BodyTerm::Literal) => Ok(CheckedTerm::Literal {
            type_id: decode_field(context, term, TermField::Type)?,
            value_kind: decode_field(context, term, TermField::ValueKind)?,
            value: decode_field(context, term, TermField::Value)?,
        }),
        TermTag::Known(BodyTerm::Reference) => Ok(CheckedTerm::Reference {
            target: decode_field(context, term, TermField::Target)?,
        }),
        TermTag::Known(BodyTerm::Application) => Ok(CheckedTerm::Application {
            operator: decode_field(context, term, TermField::Operator)?,
            operation: decode_field(context, term, TermField::Operation)?,
            result_type: decode_field(context, term, TermField::ResultType)?,
            arguments: decode_children(context, document, term, TermField::Arguments)?,
        }),
        TermTag::Known(BodyTerm::Aggregate) => Ok(CheckedTerm::Aggregate {
            members: decode_children(context, document, term, TermField::Members)?,
        }),
        TermTag::Known(BodyTerm::Binding) => {
            let name = decode_field(context, term, TermField::Name)?;
            let value = term
                .field(TermField::Value)
                .ok_or(TermConversionCause::MissingMember { member: "value" })?;
            let refused = TermConversionCause::MalformedMember {
                member: "value",
                category: Category::Data,
            };
            if !context.charge_bytes(std::mem::size_of::<CheckedTerm>()) {
                return Err(refused);
            }
            Ok(CheckedTerm::Binding {
                name,
                value: Box::new(decode_term(context, document, value)?),
            })
        }
        TermTag::Known(BodyTerm::DependencyReference) => Ok(CheckedTerm::DependencyReference {
            package: decode_field(context, term, TermField::Package)?,
            node: decode_field(context, term, TermField::Node)?,
        }),
        TermTag::Known(BodyTerm::Frame) => Ok(CheckedTerm::Frame {
            modifies: decode_field(context, term, TermField::Modifies)?,
            creates: decode_field(context, term, TermField::Creates)?,
            deletes: decode_field(context, term, TermField::Deletes)?,
            deletes_scoped: decode_field(context, term, TermField::DeletesScoped)?,
        }),
        TermTag::Known(BodyTerm::AbstractionRelation) => Ok(CheckedTerm::AbstractionRelation {
            objects: decode_field(context, term, TermField::Objects)?,
            populations: decode_field(context, term, TermField::Populations)?,
            frames: decode_field(context, term, TermField::Frames)?,
        }),
        TermTag::ExtensionForm => Ok(CheckedTerm::ExtensionForm {
            definition: decode_field(context, term, TermField::Definition)?,
            node_kind: decode_field(context, term, TermField::NodeKind)?,
            clauses: decode_children(context, document, term, TermField::Clauses)?,
        }),
        TermTag::Missing => Err(TermConversionCause::MissingTerm),
        TermTag::NonString => Err(TermConversionCause::MalformedMember {
            member: "term",
            category: Category::Data,
        }),
        TermTag::Known(BodyTerm::ImplementationGuarantee) => {
            Ok(CheckedTerm::ImplementationGuarantee {
                clause: decode_field(context, term, TermField::Clause)?,
                relation: decode_field(context, term, TermField::Relation)?,
                initial: decode_vec_field(context, term, TermField::Initial)?,
                progress: decode_vec_field(context, term, TermField::Progress)?,
                environment: decode_vec_field(context, term, TermField::Environment)?,
            })
        }
        TermTag::Known(BodyTerm::ComponentComposition) => Ok(CheckedTerm::ComponentComposition {
            rule: decode_field(context, term, TermField::Rule)?,
            components: decode_vec_field(context, term, TermField::Components)?,
            guarantees: decode_vec_field(context, term, TermField::Guarantees)?,
            relies: decode_vec_field(context, term, TermField::Relies)?,
            liveness_dependencies: decode_vec_field(
                context,
                term,
                TermField::LivenessDependencies,
            )?,
            supplies: decode_vec_field(context, term, TermField::Supplies)?,
            environment: decode_vec_field(context, term, TermField::Environment)?,
        }),
        TermTag::Unknown => Err(TermConversionCause::UnknownTerm),
    }
}

/// A path segment as the wire spells it.
enum SegmentForm<'d> {
    Name(&'d str),
    Node(u64),
    Operand(u64),
}

/// The elements of an array term, as terms.
#[derive(Clone)]
pub(in crate::checked_package) struct TermItems<'d> {
    items: Items<'d>,
}

impl<'d> Iterator for TermItems<'d> {
    type Item = TermNode<'d>;

    fn next(&mut self) -> Option<TermNode<'d>> {
        self.items.next().map(TermNode::from_ref)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl ExactSizeIterator for TermItems<'_> {}

/// The members of an object term, as names and terms.
#[derive(Clone)]
pub(in crate::checked_package) struct TermMembers<'d> {
    members: Members<'d>,
}

impl<'d> Iterator for TermMembers<'d> {
    type Item = (&'d str, TermNode<'d>);

    fn next(&mut self) -> Option<Self::Item> {
        self.members
            .next()
            .map(|(name, value)| (name, TermNode::from_ref(value)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.members.size_hint()
    }
}

impl ExactSizeIterator for TermMembers<'_> {}

/// The closed shape of an ordinary term, with its members as terms. The
/// shapes are checked in their original order, with their own refusal
/// pointers, before the complete admitted tree is constructed.
pub(in crate::checked_package) enum TermView<'d> {
    Application {
        operator: ApplicationOperator,
        operation: TermNode<'d>,
        result_type: TermNode<'d>,
        arguments: TermItems<'d>,
    },
    Aggregate {
        members: TermItems<'d>,
    },
    Binding {
        name: &'d str,
        value: TermNode<'d>,
    },
    Reference {
        target: TermNode<'d>,
    },
    Literal {
        type_id: TermNode<'d>,
        value_kind: LiteralKind,
        value: TermNode<'d>,
    },
}

/// One term below another and where it sits: the member `key` of the parent
/// (`arguments`, `members` or `value`) and, for an array member, the `index`
/// within it.
#[derive(Clone, Copy)]
pub(in crate::checked_package) struct TermSubterm<'d> {
    pub(in crate::checked_package) value: TermNode<'d>,
    pub(in crate::checked_package) key: &'static str,
    pub(in crate::checked_package) index: Option<usize>,
}

/// The terms directly below a term.
pub(in crate::checked_package) struct TermSubterms<'d> {
    key: &'static str,
    listed: Option<TermItems<'d>>,
    position: usize,
    valued: Option<TermNode<'d>>,
}

impl<'d> Iterator for TermSubterms<'d> {
    type Item = TermSubterm<'d>;

    fn next(&mut self) -> Option<TermSubterm<'d>> {
        if let Some(value) = self.listed.as_mut().and_then(Iterator::next) {
            let index = self.position;
            self.position += 1;
            return Some(TermSubterm {
                value,
                key: self.key,
                index: Some(index),
            });
        }
        self.valued.take().map(|value| TermSubterm {
            value,
            key: self.key,
            index: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{TermNode, TermView};
    use crate::checked_package::common::{TermField, WireTerm};
    use crate::checked_package::terms::subterms;
    use crate::checked_package::v2::CheckedTermView as ValueView;
    use crate::checked_package::v2::{CheckedJson, CheckedTerm};
    use crate::identity::ExecutionPoint;
    use serde_json::{json, Value};

    /// The `Value` parse of a node ID the node read is checked against.
    fn node_id_from_value(
        _budget: &mut crate::checked_package::v2::native_join_budget::NativeJoinBudget,
        _site: crate::checked_package::v2::read_charge::Site,
        value: &Value,
        well_formed: bool,
    ) -> Result<Option<crate::checked_package::v2::CheckedNodeId>, ()> {
        use serde::Deserialize;
        let parsed = crate::checked_package::v2::CheckedNodeId::deserialize(value).ok();
        Ok(parsed.filter(|id| !well_formed || id.is_well_formed()))
    }

    fn samples() -> Vec<Value> {
        let id = json!({"package": "p", "path": ["a", {"node": 1}], "ordinal": 0});
        let leaf = json!({"term": "reference", "target": id.clone()});
        vec![
            leaf.clone(),
            json!({"term": "literal", "type": id.clone(), "value_kind": "integer", "value": 3}),
            json!({"term": "literal", "type": id, "value_kind": "bogus", "value": 3}),
            json!({"term": "aggregate", "members": [leaf.clone(), leaf.clone()]}),
            json!({"term": "aggregate", "members": 7}),
            json!({"term": "binding", "name": "n", "value": leaf.clone()}),
            json!({"term": "binding", "name": 5, "value": leaf.clone()}),
            json!({"term": "application", "operator": "call", "operation": {}, "result_type": {}, "arguments": [leaf.clone()]}),
            json!({"term": "application", "operator": "call", "arguments": [leaf]}),
            json!({"term": "extension_form"}),
            json!({"term": "nonsense"}),
            json!({"term": 5}),
            json!({"x": 1}),
            json!([1, 2]),
            json!("text"),
            json!(null),
        ]
    }

    /// The node views answer exactly as the `Value` projections do.
    ///
    /// Trace: TC-472, FR-038-AC-499.
    #[test]
    fn term_views_agree_with_the_value_projections() {
        for value in samples() {
            let json = CheckedJson::from_value(&value).expect("sample");
            let node = TermNode::of(&json);
            let wire = WireTerm::decode(&value);
            assert_eq!(node.kind(), wire.kind(), "{value}");
            assert_eq!(node.operator(), wire.operator(), "{value}");
            assert_eq!(node.literal_kind(), wire.literal_kind(), "{value}");
            assert_eq!(
                node.array(TermField::Arguments).map(|items| items.len()),
                wire.array(TermField::Arguments).map(<[Value]>::len),
                "{value}"
            );
            let shapes = (
                node.view().map(|view| match view {
                    TermView::Application { arguments, .. } => ("application", arguments.len()),
                    TermView::Aggregate { members } => ("aggregate", members.len()),
                    TermView::Binding { .. } => ("binding", 1),
                    TermView::Reference { .. } => ("reference", 1),
                    TermView::Literal { .. } => ("literal", 1),
                }),
                CheckedTerm::view(&value).map(|view| match view {
                    ValueView::Application { arguments, .. } => ("application", arguments.len()),
                    ValueView::Aggregate { members } => ("aggregate", members.len()),
                    ValueView::Binding { .. } => ("binding", 1),
                    ValueView::Reference { .. } => ("reference", 1),
                    ValueView::Literal { .. } => ("literal", 1),
                }),
            );
            assert_eq!(shapes.0, shapes.1, "{value}");
            let below = node
                .subterms()
                .map(|subterm| (subterm.key, subterm.index))
                .collect::<Vec<_>>();
            let expected = subterms(&value)
                .map(|subterm| (subterm.key, subterm.index))
                .collect::<Vec<_>>();
            assert_eq!(below, expected, "{value}");
        }
    }

    /// A node ID is read from the document as the `Value` parse reads it, in
    /// its valid and spelled forms, its shape and bounds failures are told
    /// apart, and its allocations are charged to the byte.
    ///
    /// Trace: TC-472, FR-038-AC-499.
    #[test]
    fn node_ids_parse_like_the_value_parse_and_are_charged() {
        use crate::checked_package::v2::native_join_budget::{NativeJoinBudget, PackageReadStage};
        use crate::checked_package::v2::read_charge::Site;
        use crate::checked_package::v2::term_view::NodeIdRead;
        let site = Site::new(PackageReadStage::WireChecks, 0, "owner_schema");
        let cases = [
            (
                json!({"package": "p", "path": ["a", {"node": 1}, {"operand": 2}], "ordinal": 4}),
                "id",
            ),
            (
                json!({"package": "", "path": ["a"], "ordinal": 0}),
                "invalid",
            ),
            (json!({"package": "p", "path": [], "ordinal": 0}), "invalid"),
            (
                json!({"package": "p", "path": ["a"], "ordinal": 9007199254740992u64}),
                "invalid",
            ),
            (
                json!({"package": "p", "path": [{"node": 9007199254740992u64}], "ordinal": 0}),
                "malformed",
            ),
            (
                json!({"package": "p", "path": [{"node": 1, "operand": 2}], "ordinal": 0}),
                "malformed",
            ),
            (json!({"package": "p", "path": ["a"]}), "malformed"),
            (
                json!({"package": "p", "path": ["a"], "ordinal": 0, "extra": 1}),
                "malformed",
            ),
            (json!("text"), "malformed"),
        ];
        for (id, expected) in cases {
            let json = CheckedJson::from_value(&id).expect("sample");
            let mut budget = NativeJoinBudget::unlimited();
            let read = TermNode::of(&json)
                .node_id(&mut budget, site)
                .expect("unlimited");
            let class = match &read {
                NodeIdRead::Id(_) => "id",
                NodeIdRead::Invalid(_) => "invalid",
                NodeIdRead::Malformed => "malformed",
            };
            assert_eq!(class, expected, "{id}");
            for well_formed in [true, false] {
                let old =
                    node_id_from_value(&mut NativeJoinBudget::unlimited(), site, &id, well_formed)
                        .expect("unlimited");
                let new = match (&read, well_formed) {
                    (NodeIdRead::Id(id), _) | (NodeIdRead::Invalid(id), false) => Some(id.clone()),
                    _ => None,
                };
                assert_eq!(new, old, "{id} {well_formed}");
            }
            let required = budget.read_storage_consumed();
            if required > 0 {
                let mut exact =
                    NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
                assert!(TermNode::of(&json).node_id(&mut exact, site).is_ok());
                let mut short =
                    NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required - 1);
                assert!(TermNode::of(&json).node_id(&mut short, site).is_err());
            }
        }
    }

    /// A term decodes as `decode_admitted` decodes it, and what it requests is
    /// charged to the byte.
    ///
    /// Trace: TC-472, FR-038-AC-499.
    #[test]
    fn terms_decode_like_decode_admitted_and_are_charged_to_the_byte() {
        use crate::checked_package::v2::native_join_budget::{NativeJoinBudget, PackageReadStage};
        use crate::checked_package::v2::read_charge::Site;
        let site = Site::new(PackageReadStage::WireChecks, 0, "flat_wire");
        for value in samples() {
            let json = CheckedJson::from_value(&value).expect("sample");
            let mut budget = NativeJoinBudget::unlimited();
            let decoded = TermNode::of(&json)
                .decode(json.document(), &mut budget, site)
                .expect("unlimited");
            assert_eq!(decoded, CheckedTerm::decode_admitted(&value), "{value}");
            let required = budget.read_storage_consumed();
            if decoded.is_ok() && required > 0 {
                let mut exact =
                    NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
                assert!(TermNode::of(&json)
                    .decode(json.document(), &mut exact, site)
                    .is_ok_and(|decoded| decoded.is_ok()));
                let mut short =
                    NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required - 1);
                assert!(TermNode::of(&json)
                    .decode(json.document(), &mut short, site)
                    .is_err());
            }
        }
    }

    /// An operation anchor's optional fourth binding decodes as a bare point,
    /// and both its point name and boxed result are charged before allocation.
    ///
    /// Trace: TC-472, FR-038-AC-499.
    #[test]
    fn operation_anchor_point_decodes_from_the_retained_document() {
        use crate::checked_package::v2::native_join_budget::{NativeJoinBudget, PackageReadStage};
        use crate::checked_package::v2::read_charge::Site;
        let body = json!({
            "term": "aggregate",
            "members": [
                {"term": "literal", "type": {"package":"p", "path":["t"], "ordinal":0}, "value_kind":"integer", "value":1},
                {"term": "literal", "type": {"package":"p", "path":["t"], "ordinal":0}, "value_kind":"integer", "value":2},
                {"term": "literal", "type": {"package":"p", "path":["t"], "ordinal":0}, "value_kind":"integer", "value":3},
                {"term":"binding", "name":"execution_point", "value":{"kind":"initialization", "name":"Start"}}
            ]
        });
        let handle = CheckedJson::from_value(&body).expect("canonical anchor body");
        let site = Site::new(PackageReadStage::Graph, 4, "term");
        let mut budget = NativeJoinBudget::unlimited();
        let decoded = TermNode::of(&handle)
            .decode_anchor_in(handle.document(), &mut budget, site, &[])
            .expect("unlimited budget")
            .expect("closed anchor body");
        let CheckedTerm::OperationAnchorBody(anchor) = decoded else {
            panic!("four-member anchor body keeps its bare execution point");
        };
        assert_eq!(
            anchor.execution_point,
            Some(ExecutionPoint::Initialization {
                name: crate::identity::ExecutionPointName::new("Start").expect("nonempty name"),
            })
        );

        let mut body_without_point = body;
        body_without_point["members"]
            .as_array_mut()
            .expect("members")
            .pop();
        let handle_without_point =
            CheckedJson::from_value(&body_without_point).expect("canonical anchor body");
        let decoded = TermNode::of(&handle_without_point)
            .decode_anchor_in(
                handle_without_point.document(),
                &mut NativeJoinBudget::unlimited(),
                site,
                &[],
            )
            .expect("unlimited budget")
            .expect("closed anchor body");
        let CheckedTerm::OperationAnchorBody(anchor) = decoded else {
            panic!("three-member anchor body uses its dedicated representation");
        };
        assert_eq!(anchor.execution_point, None);

        let required = budget.read_storage_consumed();
        let mut exact = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required);
        assert!(TermNode::of(&handle)
            .decode_anchor_in(handle.document(), &mut exact, site, &[])
            .is_ok_and(|result| result.is_ok()));
        let mut short =
            NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(required.saturating_sub(1));
        assert!(TermNode::of(&handle)
            .decode_anchor_in(handle.document(), &mut short, site, &[])
            .is_err());
    }

    /// Operand slots read through the holder's row at read time agree with the
    /// rewritten `Value` the reader's resolved wire holds, for node IDs and
    /// for decoded terms, and a lone `{"operand": k}` that is no path segment
    /// is left alone.
    ///
    /// Trace: TC-472, FR-038-AC-499.
    #[test]
    fn operand_slots_resolve_through_the_row_like_the_rewritten_value() {
        use crate::checked_package::v2::native_join_budget::{NativeJoinBudget, PackageReadStage};
        use crate::checked_package::v2::node_index::resolve_value_slots;
        use crate::checked_package::v2::read_charge::Site;
        let site = Site::new(PackageReadStage::WireChecks, 0, "owner_schema");
        let row = [
            crate::checked_package::shared::CheckedOperandEntry::Node(5),
            crate::checked_package::shared::CheckedOperandEntry::Node(7),
        ];
        let id =
            |slot: u64| json!({"package": "p", "path": ["x", {"operand": slot}], "ordinal": 0});
        let leaf = json!({"term": "reference", "target": id(1)});
        let bodies = [
            leaf.clone(),
            json!({"term": "aggregate", "members": [leaf.clone(), {"term": "reference", "target": id(0)}]}),
            json!({"term": "literal", "type": id(1), "value_kind": "integer", "value": 3}),
            json!({"term": "literal", "type": id(1), "value_kind": "text", "value": {"operand": 0}}),
            json!({"term": "reference", "target": id(9)}),
        ];
        for body in bodies {
            let mut rewritten = body.clone();
            resolve_value_slots(
                &mut rewritten,
                &row,
                &mut NativeJoinBudget::unlimited(),
                site,
            )
            .expect("unlimited");
            let json = CheckedJson::from_value(&body).expect("body");
            let by_row = TermNode::of(&json)
                .decode_in(
                    json.document(),
                    &mut NativeJoinBudget::unlimited(),
                    site,
                    &row,
                )
                .expect("unlimited");
            assert_eq!(by_row, CheckedTerm::decode_admitted(&rewritten), "{body}");
        }
        let target = id(1);
        let mut rewritten = target.clone();
        resolve_value_slots(
            &mut rewritten,
            &row,
            &mut NativeJoinBudget::unlimited(),
            site,
        )
        .expect("unlimited");
        let json = CheckedJson::from_value(&target).expect("id");
        let read = TermNode::of(&json)
            .node_id_in(&mut NativeJoinBudget::unlimited(), site, &row)
            .expect("unlimited")
            .valid();
        let expected =
            node_id_from_value(&mut NativeJoinBudget::unlimited(), site, &rewritten, true)
                .expect("unlimited");
        assert_eq!(read, expected);
    }
}

#[cfg(test)]
mod flat_document_allocation_tests {
    use super::super::PackageReadStage;
    use super::*;
    use ix_trace_rs::trace;
    use serde_json::json;

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn nested_guarantee_measure_is_prepaid_at_every_request_boundary() {
        let wire = json!({"term":"implementation_guarantee",
            "clause":{"package":"p","path":["c"],"ordinal":0},
            "relation":{"package":"p","path":["r"],"ordinal":0},
            "initial":["start"],"progress":[{"context":{"package":"p","path":["c"],"ordinal":0},"operation":"act","measure":{"decreases":["rank","step"]}}],"environment":[]});
        let body = CheckedJson::from_value(&wire).expect("document");
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut accepted = false;
            let measured = allocation_counter::measure(|| {
                let context = DecodeContext::at_locus(
                    body.document(),
                    &mut budget,
                    Site::new(PackageReadStage::Graph, 7, "body").locus(),
                );
                let outcome = decode_term(&context, body.document(), TermNode::of(&body));
                accepted = outcome.is_ok();
                if let Ok(CheckedTerm::ImplementationGuarantee { progress, .. }) = &outcome {
                    let super::super::guarantee_terms::CheckedGuaranteeMeasure::Decreases(path) =
                        &progress[0].measure
                    else {
                        panic!("decreases")
                    };
                    assert_eq!(path.len(), 2);
                    assert_eq!(path[0].as_ref(), "rank");
                    assert_eq!(path[1].as_ref(), "step");
                }
                drop(outcome);
                budget = context.finish();
            });
            (accepted, measured.bytes_total, budget)
        };
        let (ok, allocated, budget) = run(u64::MAX);
        assert!(ok);
        let total = budget.read_storage_consumed();
        assert_eq!(allocated, total);
        let mut limit = 0;
        loop {
            let (ok, allocated, budget) = run(limit);
            if ok {
                assert_eq!(limit, total);
                break;
            }
            assert_eq!(
                allocated,
                budget.read_storage_consumed(),
                "no unpaid allocation at {limit}"
            );
            let (_, request) = budget.first_unpaid_resource().expect("unpaid reservation");
            let next =
                budget.read_storage_consumed() + u64::try_from(request).expect("small fixture");
            assert!(next > limit);
            limit = next;
        }
        assert!(!run(total - 1).0);
    }

    #[trace("TC-472", "FR-038-AC-504")]
    #[test]
    fn refinement_document_projection_preserves_rows_and_prepays_owned_values() {
        let id = json!({"package":"p","path":["C"],"ordinal":0});
        let external = json!({"package":"dependency","node":id});
        let term = json!({"term":"reference","target":id});
        let wire = json!({"term":"refinement","abstract":external,"profile":{"authority":"a","identity":"trace"},
            "populations":[{"abstract":external,"concrete":id}],
            "objects":[{"abstract_type":external,"concrete_type":id,"fields":[{"name":"value","expression":term}]}],
            "steps":[{"source_class":"act","source_node":id,"target":{"kind":"operation","operation":external,"receiver":term,"arguments":[{"term":"open"},term]}},
                {"source_class":"other","target":{"kind":"protocol_step","class":"step","node":external,"arguments":[]}},
                {"source_class":"idle","target":{"kind":"stutter"}}, {"source_class":"free","target":{"kind":"any"}}],
            "histories":[{"concrete_type":id,"name":"history","type":id,"initial":term,"updates":[{"operation":id,"expression":term}]}],
            "assume":[{"operation":id,"kind":"weak","granularity":"whole"}],
            "ensure":[{"operation":external,"kind":"strong","granularity":"each"}],
            "protocol":external,"abstract_internal":["internal"],
            "class_catalog":{"concrete":[{"class":"act","node":id,"binder_types":[id]}, {"class":"memory drain","memory_model":"sc","drain":"store-buffer-flush"}],
                "abstract":[{"class":"step","node":external,"binder_types":[external]}, {"class":"memory flush","memory_model":"tso"}]}});
        let expected = super::super::refinement::decode_body(
            &wire,
            &crate::checked_package::common::Trail::Base(&[]),
        )
        .expect("existing refinement semantics");
        let body = CheckedJson::from_value(&wire).expect("document");
        let run = |limit| {
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut accepted = false;
            let measured = allocation_counter::measure(|| {
                let context = DecodeContext::at_locus(
                    body.document(),
                    &mut budget,
                    Site::new(PackageReadStage::Graph, 7, "body").locus(),
                );
                let outcome = decode_term(&context, body.document(), TermNode::of(&body));
                if let Ok(term) = &outcome {
                    assert_eq!(term, &expected);
                    accepted = true;
                }
                drop(outcome);
                budget = context.finish();
            });
            (accepted, measured.bytes_total, budget)
        };
        let (ok, allocated, budget) = run(u64::MAX);
        assert!(ok);
        let total = budget.read_storage_consumed();
        assert_eq!(allocated, total);
        let mut limit = 0;
        loop {
            let (ok, allocated, budget) = run(limit);
            assert!(
                allocated <= budget.read_storage_consumed(),
                "no unpaid allocation at {limit}: allocated {allocated}, reserved {}",
                budget.read_storage_consumed()
            );
            if ok {
                assert_eq!(limit, total);
                break;
            }
            let (_, request) = budget.first_unpaid_resource().expect("unpaid reservation");
            let next =
                budget.read_storage_consumed() + u64::try_from(request).expect("small fixture");
            assert!(next > limit);
            limit = next;
        }
        assert!(!run(total - 1).0);
    }

    #[trace("TC-472", "FR-038-AC-512")]
    #[test]
    fn refinement_optional_and_discriminator_shapes_do_not_fallback_or_allocate_errors() {
        let id = json!({"package":"p","path":["C"],"ordinal":0});
        let base = json!({"term":"refinement","abstract":id,"profile":{"authority":"a","identity":"trace"},"populations":[],"objects":[],"steps":[],"histories":[],"assume":[],"ensure":[]});
        let mut cases = Vec::new();
        for field in ["protocol", "abstract_internal", "class_catalog"] {
            let mut wire = base.clone();
            wire[field] = serde_json::Value::Null;
            cases.push(wire);
        }
        let mut external = base.clone();
        external["abstract"] = json!({"package":"dependency","node":null});
        cases.push(external);
        let mut catalog = base.clone();
        catalog["class_catalog"] = json!({"concrete":[{"class":"memory flush","node":null,"memory_model":"tso"}],"abstract":[]});
        cases.push(catalog);
        let mut memory = base.clone();
        memory["class_catalog"] =
            json!({"concrete":[{"class":"memory flush","memory_model":"sc"}],"abstract":[]});
        cases.push(memory);
        let mut target = base.clone();
        target["steps"] = json!([{"source_class":"act","source_node":id,"target":{"kind":"stutter","arguments":[]}}]);
        cases.push(target);
        let mut source = base.clone();
        source["steps"] =
            json!([{"source_class":"act","source_node":null,"target":{"kind":"any"}}]);
        cases.push(source);
        let mut protocol_step = base.clone();
        protocol_step["steps"] = json!([{"source_class":"act","source_node":id,"target":{"kind":"protocol_step","class":"step","node":null,"arguments":[]}}]);
        cases.push(protocol_step);
        for wire in cases {
            assert!(super::super::refinement::decode_body(
                &wire,
                &crate::checked_package::common::Trail::Base(&[])
            )
            .is_err());
            let body = CheckedJson::from_value(&wire).expect("document");
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
            let measured = allocation_counter::measure(|| {
                let context = DecodeContext::at_locus(
                    body.document(),
                    &mut budget,
                    Site::new(PackageReadStage::Graph, 7, "body").locus(),
                );
                assert!(decode_term(&context, body.document(), TermNode::of(&body)).is_err());
                assert!(!context.denied());
                budget = context.finish();
            });
            assert!(
                measured.bytes_total <= budget.read_storage_consumed(),
                "owned error allocation: {} > {}",
                measured.bytes_total,
                budget.read_storage_consumed()
            );
        }
    }

    #[trace("TC-472", "FR-038-AC-512")]
    #[test]
    fn malformed_flat_shapes_refuse_without_owned_schema_errors() {
        let valid = json!({"term":"implementation_guarantee","clause":{"package":"p","path":["c"],"ordinal":0},"relation":{"package":"p","path":["r"],"ordinal":0},"initial":[],"progress":[],"environment":[]});
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove("initial");
        let mut extra = valid.clone();
        extra["extra"] = json!(0);
        let mut unknown = valid.clone();
        unknown["progress"] = json!([{"context":{"package":"p","path":["c"],"ordinal":0},"operation":"act","measure":{"unknown":[]}}]);
        let mut nested = unknown.clone();
        nested["progress"][0]["measure"] = json!({"decreases":[12]});
        for wire in [missing, extra, unknown, nested] {
            let body = CheckedJson::from_value(&wire).expect("document");
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(0);
            let measured = allocation_counter::measure(|| {
                let context = DecodeContext::at_locus(
                    body.document(),
                    &mut budget,
                    Site::new(PackageReadStage::Graph, 7, "body").locus(),
                );
                assert!(decode_term(&context, body.document(), TermNode::of(&body)).is_err());
                assert!(!context.denied());
                budget = context.finish();
            });
            assert_eq!(measured.bytes_total, 0);
            assert_eq!(budget.read_storage_consumed(), 0);
        }
    }
}
