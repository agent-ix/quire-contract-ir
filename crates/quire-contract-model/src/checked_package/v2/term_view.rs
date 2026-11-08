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

use super::checked_json::CheckedJson;
use super::native_join_budget::NativeJoinBudget;
use super::read_charge::{self, Site};
use super::{ApplicationOperator, BodyTerm, CheckedNodeId, CheckedNodePathSegment, LiteralKind};
use crate::checked_package::common::{TermField, TermTag, ValidationFailure};
use quire_canonical::{Items, Members, Node, NodeRef};

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
                | BodyTerm::AbstractionRelation,
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

    /// The node ID this value holds as a V2 wire ID, or `None` when it is not
    /// one: an object of exactly a `package` string, a `path` of names and
    /// `{"node": i}` / `{"operand": k}` segments no larger than 2^53 - 1, and an
    /// `ordinal`. With `well_formed`, the package and path are also nonempty
    /// and the ordinal no larger than 2^53 - 1. The ID's package, path and
    /// names are charged at `site` before each is allocated.
    pub(in crate::checked_package) fn node_id(
        self,
        budget: &mut NativeJoinBudget,
        site: Site,
        well_formed: bool,
    ) -> Result<Option<CheckedNodeId>, ValidationFailure> {
        const MAXIMUM: u64 = 9_007_199_254_740_991;
        if !self.exact_members(&["package", "path", "ordinal"]) {
            return Ok(None);
        }
        let (Some(package), Some(path), Some(ordinal)) = (
            self.get("package").and_then(TermNode::as_str),
            self.array_member("path"),
            self.get("ordinal").and_then(TermNode::as_u64),
        ) else {
            return Ok(None);
        };
        if well_formed && (package.is_empty() || path.len() == 0 || ordinal > MAXIMUM) {
            return Ok(None);
        }
        // Check every segment before the first allocation.
        for segment in path.clone() {
            let valid = match segment.segment_form() {
                Some(SegmentForm::Name(_)) => true,
                Some(SegmentForm::Node(index) | SegmentForm::Operand(index)) => index <= MAXIMUM,
                None => false,
            };
            if !valid {
                return Ok(None);
            }
        }
        let package = read_charge::boxed_str(budget, site, package)?;
        let mut segments = read_charge::with_capacity(budget, site, path.len())?;
        for segment in path {
            segments.push(match segment.segment_form() {
                Some(SegmentForm::Name(name)) => {
                    CheckedNodePathSegment::Name(read_charge::boxed_str(budget, site, name)?)
                }
                Some(SegmentForm::Node(index)) => CheckedNodePathSegment::Index(index),
                Some(SegmentForm::Operand(slot)) => CheckedNodePathSegment::Operand(slot),
                None => return Ok(None),
            });
        }
        Ok(Some(CheckedNodeId {
            package,
            path: segments,
            ordinal,
        }))
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
    use serde_json::{json, Value};

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
    /// both its strict and lenient forms, and its allocations are charged.
    ///
    /// Trace: TC-472, FR-038-AC-499.
    #[test]
    fn node_ids_parse_like_the_value_parse_and_are_charged() {
        use crate::checked_package::v2::native_join_budget::{NativeJoinBudget, PackageReadStage};
        use crate::checked_package::v2::read_charge::{node_id_from_value, Site};
        let site = Site::new(PackageReadStage::WireChecks, 0, "owner_schema");
        let ids = [
            json!({"package": "p", "path": ["a", {"node": 1}, {"operand": 2}], "ordinal": 4}),
            json!({"package": "", "path": ["a"], "ordinal": 0}),
            json!({"package": "p", "path": [], "ordinal": 0}),
            json!({"package": "p", "path": ["a"], "ordinal": 9007199254740992u64}),
            json!({"package": "p", "path": [{"node": 9007199254740992u64}], "ordinal": 0}),
            json!({"package": "p", "path": [{"node": 1, "operand": 2}], "ordinal": 0}),
            json!({"package": "p", "path": ["a"]}),
            json!({"package": "p", "path": ["a"], "ordinal": 0, "extra": 1}),
            json!("text"),
        ];
        for id in ids {
            for well_formed in [true, false] {
                let json = CheckedJson::from_value(&id).expect("sample");
                let mut budget = NativeJoinBudget::unlimited();
                let from_node = TermNode::of(&json)
                    .node_id(&mut budget, site, well_formed)
                    .expect("unlimited");
                let from_value =
                    node_id_from_value(&mut NativeJoinBudget::unlimited(), site, &id, well_formed)
                        .expect("unlimited");
                assert_eq!(from_node, from_value, "{id} {well_formed}");
                if from_node.is_some() {
                    assert!(budget.read_storage_consumed() > 0);
                    let mut exact = NativeJoinBudget::new(u64::MAX, 0)
                        .with_read_storage_limit(budget.read_storage_consumed());
                    assert!(TermNode::of(&json)
                        .node_id(&mut exact, site, well_formed)
                        .is_ok());
                    let mut short = NativeJoinBudget::new(u64::MAX, 0)
                        .with_read_storage_limit(budget.read_storage_consumed() - 1);
                    assert!(TermNode::of(&json)
                        .node_id(&mut short, site, well_formed)
                        .is_err());
                }
            }
        }
    }
}
