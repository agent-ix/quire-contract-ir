//! Typed, position-preserving operands of admitted scalar integer applications.

use super::operation_catalog::operation_catalog;
use super::{
    CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedPackageV2, CheckedSemanticNodeV2,
    CheckedTerm, LiteralKind, ValueForm,
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::BTreeMap;
use thiserror::Error;

/// Decode a raw candidate or borrow an already typed admitted body.
pub(super) trait ScalarBody {
    fn checked_term(&self) -> Result<Cow<'_, CheckedTerm>, ()>;
}

impl ScalarBody for Value {
    fn checked_term(&self) -> Result<Cow<'_, CheckedTerm>, ()> {
        CheckedTerm::decode_admitted(self)
            .map(Cow::Owned)
            .map_err(|_| ())
    }
}

impl ScalarBody for CheckedTerm {
    fn checked_term(&self) -> Result<Cow<'_, CheckedTerm>, ()> {
        Ok(Cow::Borrowed(self))
    }
}

#[cfg(test)]
mod scan_probe {
    use std::cell::Cell;

    thread_local! {
        static GRAPH_CANDIDATES: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn visited() {
        GRAPH_CANDIDATES.with(|count| count.set(count.get().saturating_add(1)));
    }

    pub(super) fn reset() {
        GRAPH_CANDIDATES.with(|count| count.set(0));
    }

    pub(super) fn count() -> u64 {
        GRAPH_CANDIDATES.with(Cell::get)
    }
}

/// The inclusive finite range of one integer operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedScalarOperandRange {
    /// Inclusive lower endpoint.
    pub lower: i128,
    /// Inclusive upper endpoint.
    pub upper: i128,
}

/// The identity of one operand, without inventing a graph node for an inline term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedScalarOperandChild {
    /// A referenced checked graph node.
    GraphChild(CheckedNodeId),
    /// A literal term within one occurrence and argument position.
    InlineLiteral {
        /// The containing application node.
        application: CheckedNodeId,
        /// The selected occurrence of that application.
        occurrence: CheckedOccurrence,
        /// Its zero-based argument position.
        ordinal: u64,
    },
}

/// One argument of an admitted scalar integer application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedScalarOperand {
    /// Zero-based position in `body.arguments`.
    pub ordinal: u64,
    /// The graph child or occurrence-qualified inline term.
    pub child: CheckedScalarOperandChild,
    /// Exact inclusive integer range.
    pub range: CheckedScalarOperandRange,
}

/// Why a scalar integer application's operands cannot be read.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CheckedScalarOperandError {
    /// The requested node is absent.
    #[error("unknown checked node")]
    UnknownNode,
    /// The requested node does not hold an application term.
    #[error("checked node is not an application")]
    NotApplication,
    /// The supplied occurrence is absent from the application.
    #[error("application occurrence is absent")]
    MissingOccurrence,
    /// The application's operation identity is absent from the catalog.
    #[error("unknown application operation")]
    UnknownOperator,
    /// The catalogued operation is outside scalar integer arithmetic.
    #[error("operation is not eligible for scalar integer operands")]
    IneligibleOperator,
    /// An argument names a graph node that is absent.
    #[error("referenced operand child is absent")]
    MissingChild,
    /// A child has no readable integer range.
    #[error("operand has no readable integer range")]
    MissingRange,
    /// An integer operand is valid but has no finite bounds.
    #[error("integer operand is unbounded")]
    UnboundedRange,
    /// A finite endpoint or literal cannot be represented as `i128`.
    #[error("integer operand is outside i128")]
    RangeOutOfI128,
}

/// One IR-owned parse of an admitted scalar argument, shared by both package views.
pub(super) struct ScalarArgument<'a, Body> {
    pub(super) ordinal: u64,
    pub(super) child: ScalarArgumentChild<'a, Body>,
    pub(super) own_range: Result<CheckedScalarOperandRange, CheckedScalarOperandError>,
}

pub(super) enum ScalarArgumentChild<'a, Body> {
    Graph(&'a CheckedSemanticNodeV2<Body>),
    InlineInteger(BigInt),
}

fn graph_node<'a, Body>(
    nodes: &'a [CheckedSemanticNodeV2<Body>],
    positions: &BTreeMap<CheckedNodeId, usize>,
    id: &CheckedNodeId,
) -> Option<(usize, &'a CheckedSemanticNodeV2<Body>)> {
    let position = *positions.get(id)?;
    nodes
        .get(position)
        .inspect(|_| {
            #[cfg(test)]
            scan_probe::visited();
        })
        .filter(|node| &node.node_id == id)
        .map(|node| (position, node))
}

/// Interpret the validated argument vector once, without requiring every own
/// range to be finite. The checked-package view imposes that older policy;
/// the admitted-package view retains each position when an own range is absent.
/// Per-position laziness preserves the checked view's first-error order when
/// an early range is absent and a later argument is malformed.
pub(super) fn scalar_arguments<'a, Body: ScalarBody>(
    nodes: &'a [CheckedSemanticNodeV2<Body>],
    positions: &'a BTreeMap<CheckedNodeId, usize>,
    kinds: &'a [CheckedNodeKind],
    node: &'a CheckedSemanticNodeV2<Body>,
    retained: Option<&'a CheckedTerm>,
    retained_terms: Option<&'a [CheckedTerm]>,
) -> Result<ScalarArguments<'a, Body>, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;

    // ContractPackageV1 does not retain a second body copy. Its admitted
    // root is decoded on demand; CheckedPackageV2 borrows its retained tree.
    let term = match retained {
        Some(term) => Cow::Borrowed(term),
        None => node
            .body
            .checked_term()
            .map_err(|_| Error::NotApplication)?,
    };
    let CheckedTerm::Application { operation, .. } = term.as_ref() else {
        return Err(Error::NotApplication);
    };
    let identity = operation
        .get("identity")
        .and_then(Value::as_str)
        .ok_or(Error::UnknownOperator)?;
    if operation_catalog().entry(identity).is_none() {
        return Err(Error::UnknownOperator);
    }
    if !matches!(
        identity,
        "quire.op.integer.add"
            | "quire.op.integer.sub"
            | "quire.op.integer.mul"
            | "quire.op.integer.negate"
    ) {
        return Err(Error::IneligibleOperator);
    }
    let arguments = match term {
        Cow::Borrowed(CheckedTerm::Application { arguments, .. }) => {
            Cow::Borrowed(arguments.as_slice())
        }
        Cow::Owned(CheckedTerm::Application { arguments, .. }) => Cow::Owned(arguments),
        _ => return Err(Error::NotApplication),
    };
    Ok(ScalarArguments {
        nodes,
        positions,
        kinds,
        retained_terms,
        arguments,
        next: 0,
    })
}

/// Lazy, position-preserving scalar projection over a borrowed or decoded
/// admitted application. It holds no second package body copy.
pub(super) struct ScalarArguments<'a, Body> {
    nodes: &'a [CheckedSemanticNodeV2<Body>],
    positions: &'a BTreeMap<CheckedNodeId, usize>,
    kinds: &'a [CheckedNodeKind],
    retained_terms: Option<&'a [CheckedTerm]>,
    arguments: Cow<'a, [CheckedTerm]>,
    next: usize,
}

impl<'a, Body: ScalarBody> Iterator for ScalarArguments<'a, Body> {
    type Item = Result<ScalarArgument<'a, Body>, CheckedScalarOperandError>;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.next;
        let argument = self.arguments.get(position)?;
        self.next = self.next.saturating_add(1);
        Some(self.project(position, argument))
    }
}

impl<'a, Body: ScalarBody> ScalarArguments<'a, Body> {
    fn project(
        &self,
        position: usize,
        argument: &CheckedTerm,
    ) -> Result<ScalarArgument<'a, Body>, CheckedScalarOperandError> {
        use CheckedScalarOperandError as Error;
        let ordinal = u64::try_from(position).map_err(|_| Error::MissingRange)?;
        let (child, own_range) = match argument {
            CheckedTerm::Reference { target } => {
                let (child_position, child) =
                    graph_node(self.nodes, self.positions, target).ok_or(Error::MissingChild)?;
                let kind = self.kinds.get(child_position).ok_or(Error::MissingRange)?;
                let range = if *kind == CheckedNodeKind::Value(ValueForm::Literal) {
                    let term = graph_term(
                        child,
                        self.retained_terms
                            .and_then(|terms| terms.get(child_position)),
                    )?;
                    typed_literal_integer(&term).and_then(singleton_range)
                } else {
                    type_range(
                        self.nodes,
                        self.positions,
                        self.kinds,
                        self.retained_terms,
                        &child.semantic_type,
                    )
                };
                (ScalarArgumentChild::Graph(child), range)
            }
            CheckedTerm::Literal { .. } => {
                let value = typed_literal_integer(argument)?;
                let range = value.to_i128().map_or(Err(Error::RangeOutOfI128), |value| {
                    Ok(CheckedScalarOperandRange {
                        lower: value,
                        upper: value,
                    })
                });
                (ScalarArgumentChild::InlineInteger(value), range)
            }
            _ => return Err(Error::MissingRange),
        };
        Ok(ScalarArgument {
            ordinal,
            child,
            own_range,
        })
    }
}

impl CheckedPackageV2 {
    /// Reads an admitted integer application's operands in wire argument order.
    ///
    /// Only `add`, `sub`, `mul` and `negate` are eligible. Every
    /// operand must have an exact finite `i128` range; on failure this returns
    /// one typed error and no partial list.
    pub fn scalar_application_operands(
        &self,
        application: &CheckedNodeId,
        occurrence: &CheckedOccurrence,
    ) -> Result<Vec<CheckedScalarOperand>, CheckedScalarOperandError> {
        use CheckedScalarOperandError as Error;

        let nodes = &self.wire.semantic_graph.nodes;
        let (_, node) =
            graph_node(nodes, &self.node_index, application).ok_or(Error::UnknownNode)?;
        let term = self.term(application).ok_or(Error::NotApplication)?;
        if !matches!(term, CheckedTerm::Application { .. }) {
            return Err(Error::NotApplication);
        }
        if !node.occurrences.contains(occurrence) {
            return Err(Error::MissingOccurrence);
        }
        scalar_arguments(
            nodes,
            &self.node_index,
            &self.kinds,
            node,
            Some(term),
            Some(&self.terms),
        )?
        .map(|argument| {
            let argument = argument?;
            let child = match argument.child {
                ScalarArgumentChild::Graph(node) => {
                    CheckedScalarOperandChild::GraphChild(node.node_id.clone())
                }
                ScalarArgumentChild::InlineInteger(_) => CheckedScalarOperandChild::InlineLiteral {
                    application: application.clone(),
                    occurrence: occurrence.clone(),
                    ordinal: argument.ordinal,
                },
            };
            Ok(CheckedScalarOperand {
                ordinal: argument.ordinal,
                child,
                range: argument.own_range?,
            })
        })
        .collect()
    }
}

fn singleton_range(value: BigInt) -> Result<CheckedScalarOperandRange, CheckedScalarOperandError> {
    let value = value
        .to_i128()
        .ok_or(CheckedScalarOperandError::RangeOutOfI128)?;
    Ok(CheckedScalarOperandRange {
        lower: value,
        upper: value,
    })
}

fn graph_term<'a, Body: ScalarBody>(
    node: &'a CheckedSemanticNodeV2<Body>,
    retained: Option<&'a CheckedTerm>,
) -> Result<Cow<'a, CheckedTerm>, CheckedScalarOperandError> {
    match retained {
        Some(term) => Ok(Cow::Borrowed(term)),
        None => node
            .body
            .checked_term()
            .map_err(|_| CheckedScalarOperandError::MissingRange),
    }
}

fn typed_literal_integer(term: &CheckedTerm) -> Result<BigInt, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    let CheckedTerm::Literal {
        value_kind: LiteralKind::Integer,
        value,
        ..
    } = term
    else {
        return Err(Error::MissingRange);
    };
    value
        .as_str()
        .ok_or(Error::MissingRange)?
        .parse::<BigInt>()
        .map_err(|_| Error::RangeOutOfI128)
}

fn type_range<Body: ScalarBody>(
    nodes: &[super::CheckedSemanticNodeV2<Body>],
    positions: &BTreeMap<CheckedNodeId, usize>,
    kinds: &[CheckedNodeKind],
    retained_terms: Option<&[CheckedTerm]>,
    ty: &CheckedNodeId,
) -> Result<CheckedScalarOperandRange, CheckedScalarOperandError> {
    use CheckedScalarOperandError as Error;
    let (position, node) = graph_node(nodes, positions, ty).ok_or(Error::MissingRange)?;
    match kinds.get(position) {
        Some(CheckedNodeKind::ScalarType(super::ScalarTypeForm::Integer)) => {
            return Err(Error::UnboundedRange);
        }
        Some(CheckedNodeKind::BoundedDomain(super::BoundedDomainForm::IntegerRange)) => {}
        _ => return Err(Error::MissingRange),
    }
    let term = graph_term(node, retained_terms.and_then(|terms| terms.get(position)))?;
    let CheckedTerm::Aggregate { members } = term.as_ref() else {
        return Err(Error::MissingRange);
    };
    let endpoint = |name: &str| {
        let text = members.iter().find_map(|member| match member {
            CheckedTerm::Binding { name: found, value } if found.as_ref() == name => {
                match value.as_ref() {
                    CheckedTerm::Literal { value, .. } => value.as_str(),
                    _ => None,
                }
            }
            _ => None,
        });
        text.ok_or(Error::MissingRange)?
            .parse::<i128>()
            .map_err(|_| Error::RangeOutOfI128)
    };
    Ok(CheckedScalarOperandRange {
        lower: endpoint("min")?,
        upper: endpoint("max")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::v2::{CheckedPackageWireV2, RetainedModels};
    use ix_trace_rs::trace;
    use serde_json::json;

    fn id(digit: char) -> Value {
        json!({"package": "test", "path": [digit.to_string()], "ordinal": 0})
    }

    /// The accessor reads only the semantic graph. This private fixture has
    /// the other wire members needed to construct a package, then tests may
    /// mutate the graph in states admission would ordinarily reject.
    fn fixture_package() -> CheckedPackageV2 {
        let node = |digit: char, tag: &str, form: &str, ty: char, body: Value| {
            json!({
                "node_id": id(digit), "schema_version": "quire.checked-semantic-graph/v2",
                "node_tag": tag, "semantic_form": form, "semantic_type": id(ty),
                "dependencies": [], "occurrences": [{"role": "expression", "ordinal": 0}],
                "body": body,
            })
        };
        let application = json!({
            "term": "application", "operator": "binary",
            "operation": {"identity": "quire.op.integer.add", "laws": [], "mode": null,
                "member": null, "leaves": []},
            "result_type": id('2'),
            "arguments": [
                {"term": "reference", "target": id('3')},
                {"term": "literal", "type": id('1'), "value_kind": "integer", "value": "7"},
            ],
        });
        let wire: CheckedPackageWireV2 = super::super::test_support::private_wire(json!({
            "contract_version": "quire.checked-package/v2",
            "name": "test",
            "lock": {
                "sources": [],
                "edition": {"role": "edition", "definition": {"authority": "test", "identity": "edition"}},
                "profile_selections": [], "definition_selections": [],
                "required_features": [], "dependency_selections": [],
            },
            "semantic_graph": {"graph_version": "quire.checked-semantic-graph/v2", "nodes": [
                node('1', "scalar_type", "integer", '1', json!({"term": "aggregate", "members": []})),
                node('2', "bounded_domain", "integer_range", '1', json!({"term": "aggregate", "members": [
                    {"term": "binding", "name": "min", "value": {"term": "literal", "type": id('1'), "value_kind": "integer", "value": "0"}},
                    {"term": "binding", "name": "max", "value": {"term": "literal", "type": id('1'), "value_kind": "integer", "value": "1000"}},
                ]})),
                node('3', "value", "parameter", '2', json!({"term": "aggregate", "members": []})),
                node('4', "expression", "binary", '2', application),
            ]},
            "source_map": [], "capability_report": [],
            "diagnostics": {"catalog": {"authority": "test", "identity": "diagnostics"}, "entries": []},
        }));
        let terms = wire
            .semantic_graph
            .nodes
            .iter()
            .map(|node| node.body.clone())
            .collect();
        CheckedPackageV2 {
            node_index: super::super::retained_node_index(&wire),
            structural_keys: Vec::new(),
            source_index: super::super::retained_source_index(&wire),
            lowering_facts: Vec::new(),
            wire,
            terms,
            kinds: vec![
                super::super::CheckedNodeKind::ScalarType(super::super::ScalarTypeForm::Integer),
                super::super::CheckedNodeKind::BoundedDomain(
                    super::super::BoundedDomainForm::IntegerRange,
                ),
                super::super::CheckedNodeKind::Value(super::super::ValueForm::Parameter),
                super::super::CheckedNodeKind::Expression(super::super::ExpressionForm::Binary),
            ],
            bytes: 1 << 20,
            models: RetainedModels::default(),
            direct_dependencies: Vec::new(),
            closure_index: Default::default(),
        }
    }

    /// Vary graph size while keeping the same admitted scalar argument shape.
    fn padded_package(graph_nodes: usize) -> CheckedPackageV2 {
        let mut package = fixture_package();
        let application = package
            .wire
            .semantic_graph
            .nodes
            .pop()
            .expect("application");
        let application_term = package.terms.pop().expect("application term");
        let application_kind = package.kinds.pop().expect("application kind");
        for offset in 0..(graph_nodes - 4) {
            let mut filler = package.wire.semantic_graph.nodes[0].clone();
            filler.node_id = serde_json::from_value(json!({
                "package": "test", "path": [format!("filler-{offset}")], "ordinal": 0,
            }))
            .expect("filler identity");
            package.wire.semantic_graph.nodes.push(filler);
            package.terms.push(package.terms[0].clone());
            package.kinds.push(package.kinds[0]);
        }
        package.wire.semantic_graph.nodes.push(application);
        package.terms.push(application_term);
        package.kinds.push(application_kind);
        package.node_index = super::super::retained_node_index(&package.wire);
        package
    }

    /// Graph candidate visits stay local to the selected application, child,
    /// and type when one package answers repeated operand requests.
    #[trace("TC-048", "FR-038-AC-162")]
    #[test]
    fn repeated_scalar_operand_reads_do_not_walk_the_whole_graph() {
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let mut measured = Vec::new();
        for graph_nodes in [4, 8, 16, 64] {
            let package = padded_package(graph_nodes);
            for requests in [2, 4, 8] {
                scan_probe::reset();
                crate::checked_package::shared::checked_node_comparisons::reset();
                for _ in 0..requests {
                    assert_eq!(
                        package
                            .scalar_application_operands(&application, &occurrence)
                            .expect("operands")
                            .len(),
                        2
                    );
                }
                let visits = scan_probe::count();
                let comparisons = crate::checked_package::shared::checked_node_comparisons::count();
                println!("scalar_operands K={requests} R={graph_nodes} graph_candidates={visits} id_comparisons={comparisons}");
                if graph_nodes == 64 {
                    assert!(comparisons < requests * 64, "a full graph scan must fail");
                }
                measured.push((graph_nodes, requests, visits));
            }
            if graph_nodes == 64 {
                crate::checked_package::shared::checked_node_comparisons::reset();
                for _ in 0..8 {
                    assert!(package
                        .wire
                        .semantic_graph
                        .nodes
                        .iter()
                        .any(|candidate| candidate.node_id == application));
                }
                assert!(
                    crate::checked_package::shared::checked_node_comparisons::count() >= 8 * 64,
                    "the retired full scan exceeds the bound"
                );
            }
        }
        for (graph_nodes, requests, visits) in measured {
            assert_eq!(
                visits,
                requests * 3,
                "graph_nodes={graph_nodes}, requests={requests}"
            );
        }
    }

    #[trace("TC-048", "FR-038-AC-162", "FR-038-AC-163")]
    #[test]
    fn tc_048_post_admission_graph_mutations_return_distinct_errors() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        assert_eq!(
            package
                .scalar_application_operands(&application, &occurrence)
                .expect("healthy")
                .len(),
            2
        );
        package.wire.semantic_graph.nodes[2].node_id =
            serde_json::from_value(id('5')).expect("replacement identity");
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::MissingChild)
        );
        let mut package = fixture_package();
        package.terms[1] = CheckedTerm::Aggregate {
            members: Vec::new(),
        };
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::MissingRange)
        );
        let mut package = fixture_package();
        let CheckedTerm::Application { operation, .. } = &mut package.terms[3] else {
            panic!("application term");
        };
        operation.record_mut()["identity"] = json!("test.unknown");
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::UnknownOperator)
        );
    }

    /// Private damaged-state discriminator only; this fixture is not a new
    /// reader admission and provides no admission evidence.
    #[trace("FR-038-AC-162", "FR-349-AC-2")]
    #[test]
    fn retained_parameter_kind_cannot_be_overridden_by_raw_literal_spelling() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("application");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let expected = package
            .scalar_application_operands(&application, &occurrence)
            .expect("private baseline");
        assert_eq!(
            expected[0].range,
            CheckedScalarOperandRange {
                lower: 0,
                upper: 1000
            }
        );
        package.wire.semantic_graph.nodes[2].semantic_form = "literal".into();
        assert_eq!(
            package.kinds[2],
            super::super::CheckedNodeKind::Value(super::super::ValueForm::Parameter)
        );
        scan_probe::reset();
        let actual = package.scalar_application_operands(&application, &occurrence);
        assert_eq!(
            actual,
            Ok(expected),
            "raw literal spelling cannot override the retained parameter kind"
        );
        assert_eq!(
            scan_probe::count(),
            3,
            "application, selected child and its type only"
        );
    }

    /// The inverse private discriminator retains a literal while only its
    /// display metadata is damaged. It is not newly admitted input.
    #[trace("FR-038-AC-162", "FR-349-AC-2")]
    #[test]
    fn retained_literal_kind_cannot_be_overridden_by_raw_parameter_spelling() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("application");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let value = CheckedTerm::Literal {
            type_id: serde_json::from_value(id('1')).expect("integer type"),
            value_kind: LiteralKind::Integer,
            value: json!("13"),
        };
        package.terms[2] = value.clone();
        package.wire.semantic_graph.nodes[2].body = value;
        package.wire.semantic_graph.nodes[2].semantic_type =
            serde_json::from_value(id('1')).expect("integer type");
        package.wire.semantic_graph.nodes[2].semantic_form = "literal".into();
        package.kinds[2] = super::super::CheckedNodeKind::Value(super::super::ValueForm::Literal);
        let expected = package
            .scalar_application_operands(&application, &occurrence)
            .expect("private literal baseline");
        assert_eq!(
            expected[0].range,
            CheckedScalarOperandRange {
                lower: 13,
                upper: 13
            }
        );
        package.wire.semantic_graph.nodes[2].semantic_form = "parameter".into();
        scan_probe::reset();
        let actual = package.scalar_application_operands(&application, &occurrence);
        assert_eq!(
            actual,
            Ok(expected),
            "raw parameter spelling cannot override the retained literal kind"
        );
        assert_eq!(
            scan_probe::count(),
            2,
            "application and selected literal only"
        );
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn admitted_scalar_arguments_use_retained_terms_after_raw_body_changes() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let expected = package
            .scalar_application_operands(&application, &occurrence)
            .expect("original operands");
        package.wire.semantic_graph.nodes[3].body = CheckedTerm::aggregate(Vec::new());
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Ok(expected)
        );
    }

    #[trace("FR-349-AC-2")]
    #[test]
    fn admitted_scalar_ranges_use_retained_graph_terms_after_raw_body_changes() {
        let mut package = fixture_package();
        let application: CheckedNodeId = serde_json::from_value(id('4')).expect("id");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let expected = package
            .scalar_application_operands(&application, &occurrence)
            .expect("original ranges");
        package.wire.semantic_graph.nodes[1].body = CheckedTerm::aggregate(Vec::new());
        assert_eq!(
            package.scalar_application_operands(&application, &occurrence),
            Ok(expected)
        );
    }

    /// These private damaged states are not reader admissions. The indexed
    /// retained type kind, rather than its raw spelling, controls projection.
    #[trace("TC-048")]
    #[test]
    fn scalar_range_retained_bounds_cannot_be_erased_by_raw_integer_spelling() {
        let application = serde_json::from_value(id('4')).expect("application");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let mut bounded = fixture_package();
        let expected = bounded
            .scalar_application_operands(&application, &occurrence)
            .expect("private bounded baseline");
        assert_eq!(expected.len(), 2);
        assert_eq!(expected[0].ordinal, 0);
        assert_eq!(
            expected[0].range,
            CheckedScalarOperandRange {
                lower: 0,
                upper: 1000
            }
        );
        assert_eq!(expected[1].ordinal, 1);
        assert_eq!(
            expected[1].range,
            CheckedScalarOperandRange { lower: 7, upper: 7 }
        );
        bounded.wire.semantic_graph.nodes[1].node_tag = "scalar_type".into();
        bounded.wire.semantic_graph.nodes[1].semantic_form = "integer".into();
        assert_eq!(
            bounded.scalar_application_operands(&application, &occurrence),
            Ok(expected),
            "a raw scalar-integer spelling cannot erase retained finite bounds"
        );
    }

    /// The inverse private damaged-state control supplies readable finite
    /// bounds, while the retained type kind remains an unbounded integer.
    #[trace("TC-048")]
    #[test]
    fn scalar_range_retained_integer_cannot_be_bounded_by_raw_range_spelling() {
        let application = serde_json::from_value(id('4')).expect("application");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        let mut unbounded = fixture_package();
        unbounded.wire.semantic_graph.nodes[2].semantic_type =
            serde_json::from_value(id('1')).expect("integer type");
        assert_eq!(
            unbounded.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::UnboundedRange),
            "private unbounded baseline"
        );
        unbounded.wire.semantic_graph.nodes[0].node_tag = "bounded_domain".into();
        unbounded.wire.semantic_graph.nodes[0].semantic_form = "integer_range".into();
        unbounded.terms[0] = unbounded.terms[1].clone();
        assert_eq!(
            unbounded.scalar_application_operands(&application, &occurrence),
            Err(CheckedScalarOperandError::UnboundedRange),
            "raw bounded-range spelling and readable bounds cannot bound a retained integer"
        );
    }

    /// Preserve the checked view's first argument error and the shared
    /// iterator's ability to retain later positions when an own range fails.
    #[trace("TC-048")]
    #[test]
    fn scalar_range_errors_remain_lazy_and_keep_their_distinct_dispositions() {
        let application = serde_json::from_value(id('4')).expect("application");
        let occurrence = CheckedOccurrence {
            role: super::super::CheckedOccurrenceRole::Expression,
            ordinal: 0,
        };
        for error in [
            CheckedScalarOperandError::UnboundedRange,
            CheckedScalarOperandError::MissingRange,
            CheckedScalarOperandError::RangeOutOfI128,
        ] {
            let mut package = fixture_package();
            match error {
                CheckedScalarOperandError::UnboundedRange => {
                    package.wire.semantic_graph.nodes[2].semantic_type =
                        serde_json::from_value(id('1')).expect("integer type");
                }
                CheckedScalarOperandError::MissingRange => {
                    package.terms[1] = CheckedTerm::Aggregate {
                        members: Vec::new(),
                    };
                }
                CheckedScalarOperandError::RangeOutOfI128 => {
                    let CheckedTerm::Aggregate { members } = &mut package.terms[1] else {
                        panic!("range body");
                    };
                    let CheckedTerm::Binding { value, .. } = &mut members[0] else {
                        panic!("minimum binding");
                    };
                    let CheckedTerm::Literal { value, .. } = value.as_mut() else {
                        panic!("minimum literal");
                    };
                    *value = json!("-170141183460469231731687303715884105729");
                }
                _ => unreachable!("the test varies only range dispositions"),
            }
            let node = &package.wire.semantic_graph.nodes[3];
            let mut arguments = scalar_arguments(
                &package.wire.semantic_graph.nodes,
                &package.node_index,
                &package.kinds,
                node,
                Some(&package.terms[3]),
                Some(&package.terms),
            )
            .expect("scalar iterator");
            let first = arguments
                .next()
                .expect("first position")
                .expect("graph child");
            assert_eq!(first.ordinal, 0);
            assert_eq!(first.own_range, Err(error));
            let second = arguments
                .next()
                .expect("second position")
                .expect("inline literal");
            assert_eq!(second.ordinal, 1);
            assert_eq!(
                second.own_range,
                Ok(CheckedScalarOperandRange { lower: 7, upper: 7 })
            );
            assert!(arguments.next().is_none());

            let CheckedTerm::Application { arguments, .. } = &mut package.terms[3] else {
                panic!("application");
            };
            arguments[1] = CheckedTerm::Aggregate {
                members: Vec::new(),
            };
            assert_eq!(
                package.scalar_application_operands(&application, &occurrence),
                Err(error),
                "the first range error wins over the later malformed argument"
            );
        }
    }
}
