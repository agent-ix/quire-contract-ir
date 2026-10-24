//! Declarative inputs and declared coverage for the v0.1 conformance corpus.

use std::collections::BTreeSet;

use super::J;
use quire_contract_model::ConformanceOperation;

pub(super) struct Case {
    pub(super) id: String,
    pub(super) operation: ConformanceOperation,
    pub(super) value: J,
    pub(super) covers: BTreeSet<String>,
}

fn add<I, S>(
    cases: &mut Vec<Case>,
    id: impl Into<String>,
    operation: ConformanceOperation,
    value: J,
    covers: I,
) where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut covers = covers.into_iter().map(Into::into).collect::<BTreeSet<_>>();
    covers.insert(format!("operation:{}", operation.as_str()));
    cases.push(Case {
        id: id.into(),
        operation,
        value,
        covers,
    });
}

enum CaseCoverage {
    Construct(&'static str),
    Boundary(&'static str),
}
impl CaseCoverage {
    fn into_label(self) -> String {
        match self {
            Self::Construct(name) => format!("construct:{name}"),
            Self::Boundary(name) => format!("boundary:{name}"),
        }
    }
}

enum PackageMutation {
    Namespace,
    Schema,
    Identifier,
    RequirementRevision,
    SourceRevision,
    ReversedSpan,
    RequirementRevisionOver,
    SourceRevisionOver,
    SpanOffsetOver,
}
impl PackageMutation {
    fn apply(&self, candidate: &mut J) {
        match self {
            Self::Namespace => candidate.set("id", "agent/../bad"),
            Self::Schema => candidate["schema_version"].set("major", 0),
            Self::Identifier => candidate["requirements"][0].set("id", "bad id"),
            Self::RequirementRevision => candidate["requirements"][0].set("revision", 0),
            Self::SourceRevision => candidate["source"].set("revision", 0),
            Self::ReversedSpan => {
                candidate["requirements"][0]["source"]["start"].set("byte_offset", 100)
            }
            Self::RequirementRevisionOver => {
                candidate["requirements"][0].set("revision", (1_i64 << 53) + 1)
            }
            Self::SourceRevisionOver => candidate["source"].set("revision", (1_i64 << 53) + 1),
            Self::SpanOffsetOver => {
                candidate["requirements"][0]["source"]["end"].set("byte_offset", (1_i64 << 53) + 1)
            }
        }
    }
}

enum ExpressionMutation {
    DuplicateType,
    DuplicateValue,
    DuplicateFunction,
    DuplicateField,
    DuplicateVariant,
    DuplicateParameter,
    EmptyEnum,
    InvalidNumeric,
    UnboundedCollection,
    RecursiveType,
}
impl ExpressionMutation {
    fn apply(&self, request: &mut J) {
        match self {
            Self::DuplicateType => duplicate_first(&mut request["types"]),
            Self::DuplicateValue => duplicate_first(&mut request["values"]),
            Self::DuplicateFunction => duplicate_first(&mut request["functions"]),
            Self::DuplicateField => duplicate_first(&mut request["types"][1]["fields"]),
            Self::DuplicateVariant => duplicate_first(&mut request["types"][0]["variants"]),
            Self::DuplicateParameter => duplicate_first(&mut request["functions"][0]["parameters"]),
            Self::EmptyEnum => request["types"][0].set("variants", Vec::<J>::new()),
            Self::InvalidNumeric => {
                request["values"][0]["value_type"].set("minimum", "2");
                request["values"][0]["value_type"].set("maximum", "1");
            }
            Self::UnboundedCollection => request["values"][3]["value_type"].set("maximum_items", 0),
            Self::RecursiveType => {
                request.set("types", vec![obj!("kind" => "record", "name" => "Cycle", "source" => expression_span(1),
                    "fields" => vec![obj!("name" => "next", "value_type" => obj!("kind" => "option", "value" => obj!("kind" => "record", "name" => "Cycle")), "source" => expression_span(2))])]);
                request.set("values", Vec::<J>::new());
                request.set("functions", Vec::<J>::new());
            }
        }
    }
    fn additional_coverage(&self) -> Option<CaseCoverage> {
        match self {
            Self::InvalidNumeric => Some(CaseCoverage::Boundary("integer.out_of_range")),
            Self::UnboundedCollection => Some(CaseCoverage::Boundary("collection.minimum")),
            Self::DuplicateType
            | Self::DuplicateValue
            | Self::DuplicateFunction
            | Self::DuplicateField
            | Self::DuplicateVariant
            | Self::DuplicateParameter
            | Self::EmptyEnum
            | Self::RecursiveType => None,
        }
    }
}

fn span(at: i64, document: &str, revision: i64) -> J {
    let source = obj!("document" => document, "revision" => revision);
    obj!(
        "start" => obj!("source" => source.clone(), "line" => 1, "column" => at + 1, "byte_offset" => at),
        "end" => obj!("source" => source, "line" => 1, "column" => at + 2, "byte_offset" => at + 1)
    )
}

fn contract_span(at: i64) -> J {
    span(at, "contract", 1)
}
fn expression_span(at: i64) -> J {
    span(at, "expression", 1)
}
fn owner() -> J {
    obj!("package" => "agent-ix/conformance", "requirement" => "REQ_alpha", "revision" => 1)
}
fn bool_type() -> J {
    obj!("kind" => "boolean")
}
fn int_type() -> J {
    obj!("kind" => "integer", "domain" => "signed", "minimum" => "-10", "maximum" => "10", "overflow" => "reject")
}
fn uint_type() -> J {
    obj!("kind" => "integer", "domain" => "unsigned", "minimum" => "0", "maximum" => "4", "overflow" => "reject")
}
fn rational_type() -> J {
    obj!("kind" => "rational", "numerator_minimum" => "-10", "numerator_maximum" => "10", "maximum_denominator" => "10")
}
fn collection_type() -> J {
    obj!("kind" => "collection", "element" => int_type(), "maximum_items" => 4)
}
fn option_type() -> J {
    obj!("kind" => "option", "value" => int_type())
}

fn expr(node: &str, fields: Vec<(String, J)>) -> J {
    let mut entries = Vec::with_capacity(fields.len() + 2);
    entries.push(("node".to_owned(), J::from(node)));
    entries.extend(fields);
    entries.push(("source".to_owned(), expression_span(20)));
    J::Object(entries)
}

macro_rules! expression {
    ($node:expr $(, $key:expr => $value:expr)* $(,)?) => {
        expr($node, vec![$(($key.to_owned(), J::from($value))),*])
    };
}

fn integer(value: i64, value_type: J) -> J {
    expression!("integer_literal", "value" => value.to_string(), "value_type" => value_type)
}
fn one() -> J {
    integer(1, int_type())
}
fn boolean(value: bool) -> J {
    expression!("boolean_literal", "value" => value)
}
fn value(name: &str, observation: &str) -> J {
    expression!("value_reference", "name" => name, "observation" => observation)
}
fn compare(left: J, right: J, operator: &str) -> J {
    expression!("compare", "operator" => operator, "left" => left, "right" => right)
}
fn numeric(left: J, right: J, operator: &str) -> J {
    expression!("numeric", "operator" => operator, "left" => left, "right" => right)
}
fn boolean_op(left: J, right: J, operator: &str) -> J {
    expression!("boolean", "operator" => operator, "left" => left, "right" => right)
}

fn environment() -> J {
    obj!(
        "owner" => owner(),
        "types" => vec![
            obj!("kind" => "enum", "name" => "Color", "source" => expression_span(1),
                "variants" => vec![
                    obj!("name" => "red", "source" => expression_span(2)),
                    obj!("name" => "blue", "source" => expression_span(3))
                ]),
            obj!("kind" => "record", "name" => "Sensor", "source" => expression_span(4),
                "fields" => vec![obj!("name" => "reading", "value_type" => int_type(), "source" => expression_span(5))])
        ],
        "values" => vec![
            obj!("name" => "divisor", "kind" => "input", "value_type" => int_type(), "source" => expression_span(6)),
            obj!("name" => "state_value", "kind" => "state", "value_type" => int_type(), "source" => expression_span(7)),
            obj!("name" => "maybe", "kind" => "state", "value_type" => option_type(), "source" => expression_span(8)),
            obj!("name" => "items", "kind" => "state", "value_type" => collection_type(), "source" => expression_span(9)),
            obj!("name" => "position", "kind" => "input", "value_type" => uint_type(), "source" => expression_span(10)),
            obj!("name" => "sensor", "kind" => "state", "value_type" => obj!("kind" => "record", "name" => "Sensor"), "source" => expression_span(11))
        ],
        "functions" => vec![obj!(
            "name" => "identity",
            "parameters" => vec![obj!("name" => "argument", "value_type" => int_type(), "source" => expression_span(12))],
            "result_type" => int_type(),
            "source" => expression_span(13)
        )],
        "expression" => boolean(true),
        "expected_type" => bool_type(),
        "execution_point" => obj!("kind" => "pre", "operation" => "check"),
        "clause_root" => false
    )
}

fn expression_input(expression: J, expected: J) -> J {
    let mut result = environment();
    result.set("expression", expression);
    result.set("expected_type", expected);
    result
}

fn package(requirements: usize) -> J {
    let clause = obj!(
        "id" => "note", "kind" => "information", "source" => contract_span(0),
        "body" => obj!("node" => "literal")
    );
    let values = (0..requirements)
        .map(|index| {
            obj!(
                "id" => if index == 0 { "REQ_alpha".to_owned() } else { format!("REQ_{index}") },
                "revision" => 1,
                "source" => contract_span((index * 2) as i64),
                "clauses" => vec![clause.clone()]
            )
        })
        .collect::<Vec<_>>();
    obj!(
        "id" => "agent-ix/conformance",
        "schema_version" => obj!("major" => 1, "minor" => 1),
        "source" => obj!("document" => "contract", "revision" => 1),
        "requirements" => values
    )
}

fn dependency(kind: &str, package_id: &str, requirement: &str, revision: i64) -> J {
    let mut result = obj!(
        "requirement" => obj!("package" => package_id, "requirement" => requirement, "revision" => revision),
        "kind" => kind,
        "path" => vec![J::from("value")]
    );
    if matches!(kind, "input" | "state") {
        result.set("observation", "current");
    }
    result
}

fn package_with_clause(kind: &str, anchor: Option<J>, body: Option<J>) -> J {
    let mut result = package(1);
    let clause = &mut result["requirements"][0]["clauses"][0];
    clause.set("kind", kind);
    if let Some(anchor) = anchor {
        clause.set("anchor", anchor);
    }
    clause.set("body", body.unwrap_or_else(|| obj!("node" => "literal")));
    result
}

fn wire_document(request: &J) -> J {
    obj!("document_json" => serde_json::to_string(request).expect("fixture JSON serializes"))
}

fn duplicate_first(array: &mut J) {
    let first = array[0].clone();
    array.push(first);
}

pub(super) fn build_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    package_cases(&mut cases);
    expression_cases(&mut cases);
    coverage_cases(&mut cases);
    cases
}

fn package_cases(cases: &mut Vec<Case>) {
    let mut all_clauses = package(1);
    all_clauses["requirements"][0].set("clauses", vec![
        obj!("id" => "a_assert", "kind" => "assertion", "anchor" => obj!("kind" => "pre", "operation" => "check"), "source" => contract_span(1), "body" => obj!("node" => "literal")),
        obj!("id" => "b_case", "kind" => "case", "anchor" => obj!("kind" => "handler", "name" => "handle"), "source" => contract_span(2), "body" => obj!("node" => "composite", "children" => vec![obj!("node" => "composite", "children" => Vec::<J>::new()), obj!("node" => "literal")])),
        obj!("id" => "c_info", "kind" => "information", "source" => contract_span(3), "body" => obj!("node" => "literal")),
        obj!("id" => "d_inv", "kind" => "invariant", "anchor" => obj!("kind" => "initialization", "name" => "boot"), "source" => contract_span(4), "body" => obj!("node" => "literal")),
        obj!("id" => "e_post", "kind" => "postcondition", "anchor" => obj!("kind" => "post", "operation" => "check"), "source" => contract_span(5), "body" => obj!("node" => "literal")),
        obj!("id" => "f_pre", "kind" => "precondition", "anchor" => obj!("kind" => "pre", "operation" => "check"), "source" => contract_span(6), "body" => obj!("node" => "literal"))
    ]);
    all_clauses["requirements"][0]["clauses"]
        .as_array_mut()
        .unwrap()
        .reverse();
    add(
        cases,
        "package-constructs",
        ConformanceOperation::Package,
        all_clauses,
        [
            "construct:clause_kind.assertion",
            "construct:clause_kind.case",
            "construct:clause_kind.information",
            "construct:clause_kind.invariant",
            "construct:clause_kind.postcondition",
            "construct:clause_kind.precondition",
            "construct:reference_body.literal",
            "construct:reference_body.composite",
            "boundary:canonical.semantic_set_order",
            "boundary:canonical.sequence_order",
            "boundary:revision.current",
            "boundary:schema.1_1",
            "boundary:source_span.minimum",
        ],
    );
    add(
        cases,
        "package-reference",
        ConformanceOperation::Package,
        package_with_clause(
            "information",
            None,
            Some(
                obj!("node" => "reference", "identity" => dependency("input", "agent-ix/conformance", "REQ_alpha", 1)),
            ),
        ),
        [
            "construct:reference_body.reference",
            "construct:dependency.kind.input",
        ],
    );

    for (id, code, extra, mutation) in [
        (
            "package-invalid-namespace",
            "invalid_package_namespace",
            None,
            PackageMutation::Namespace,
        ),
        (
            "package-invalid-schema",
            "invalid_schema_version",
            Some("boundary:schema.zero_major"),
            PackageMutation::Schema,
        ),
        (
            "package-invalid-identifier",
            "invalid_identifier",
            None,
            PackageMutation::Identifier,
        ),
        (
            "package-invalid-requirement-revision",
            "invalid_requirement_revision",
            None,
            PackageMutation::RequirementRevision,
        ),
        (
            "package-invalid-source-revision",
            "invalid_source_revision",
            None,
            PackageMutation::SourceRevision,
        ),
        (
            "package-invalid-span",
            "invalid_source_span",
            Some("boundary:source_span.reversed"),
            PackageMutation::ReversedSpan,
        ),
    ] {
        let mut candidate = package(1);
        mutation.apply(&mut candidate);
        let mut covers = vec![format!("diagnostic:{code}")];
        if let Some(extra) = extra {
            covers.push(extra.to_owned());
        }
        add(cases, id, ConformanceOperation::Package, candidate, covers);
    }
    let mut duplicate = package(1);
    let requirement = duplicate["requirements"][0].clone();
    duplicate["requirements"].push(requirement);
    add(
        cases,
        "package-duplicate-requirement",
        ConformanceOperation::Package,
        duplicate,
        ["diagnostic:duplicate_requirement"],
    );
    let mut duplicate = package(1);
    let clause = duplicate["requirements"][0]["clauses"][0].clone();
    duplicate["requirements"][0]["clauses"].push(clause);
    add(
        cases,
        "package-duplicate-clause",
        ConformanceOperation::Package,
        duplicate,
        ["diagnostic:duplicate_clause"],
    );
    for (id, kind, anchor, code) in [
        (
            "package-floating",
            "precondition",
            None,
            "floating_executable_clause",
        ),
        (
            "package-information-anchored",
            "information",
            Some(obj!("kind" => "pre", "operation" => "check")),
            "informational_clause_anchored",
        ),
        (
            "package-incompatible-anchor",
            "postcondition",
            Some(obj!("kind" => "pre", "operation" => "check")),
            "incompatible_clause_anchor",
        ),
    ] {
        add(
            cases,
            id,
            ConformanceOperation::Package,
            package_with_clause(kind, anchor, None),
            [format!("diagnostic:{code}")],
        );
    }
    for (id, dep, code, boundary) in [
        (
            "package-cross-reference",
            dependency("input", "agent-ix/other", "REQ_alpha", 1),
            "cross_package_reference",
            None,
        ),
        (
            "package-malformed-reference",
            {
                let mut dep = dependency("input", "agent-ix/conformance", "REQ_alpha", 1);
                dep.set("path", Vec::<J>::new());
                dep
            },
            "malformed_reference",
            None,
        ),
        (
            "package-stale-reference",
            dependency("input", "agent-ix/conformance", "REQ_alpha", 2),
            "stale_requirement_revision",
            Some("boundary:revision.stale"),
        ),
        (
            "package-orphan-reference",
            dependency("input", "agent-ix/conformance", "REQ_missing", 1),
            "orphaned_requirement_reference",
            None,
        ),
    ] {
        let mut covers = vec![format!("diagnostic:{code}")];
        if let Some(boundary) = boundary {
            covers.push(boundary.to_owned());
        }
        add(
            cases,
            id,
            ConformanceOperation::Package,
            package_with_clause(
                "information",
                None,
                Some(obj!("node" => "reference", "identity" => dep)),
            ),
            covers,
        );
    }
    add(
        cases,
        "package-orphan-clause",
        ConformanceOperation::Package,
        obj!("package" => package(1),
        "clause_resolutions" => vec![obj!("requirement" => owner(), "clause" => "missing")]),
        ["diagnostic:orphaned_clause_reference"],
    );
    add(
        cases,
        "package-canonical-limit",
        ConformanceOperation::Package,
        obj!("package" => package(1), "canonical_maximum_bytes" => 0),
        [
            "diagnostic:canonicalization_resource_exhausted",
            "boundary:canonical.resource_failure",
        ],
    );
    let mut unknown = package(1);
    unknown.set("ignored", 0);
    add(
        cases,
        "package-unknown-field",
        ConformanceOperation::Package,
        wire_document(&unknown),
        ["diagnostic:invalid_wire_format"],
    );
    for (id, levels, boundary) in [
        (
            "package-wire-depth-maximum",
            576,
            "boundary:wire.depth.maximum",
        ),
        (
            "package-wire-depth-over",
            577,
            "boundary:wire.depth.over_maximum",
        ),
    ] {
        add(
            cases,
            id,
            ConformanceOperation::Package,
            obj!("document_json" => format!("{}0{}", "[".repeat(levels), "]".repeat(levels))),
            ["diagnostic:invalid_wire_format", boundary],
        );
    }
    let mut bound = package(1);
    bound["source"].set("revision", 1_i64 << 53);
    bound["requirements"][0].set("revision", 1_i64 << 53);
    bound["requirements"][0].set("source", span(0, "contract", 1_i64 << 53));
    bound["requirements"][0]["source"]["end"].set("byte_offset", 1_i64 << 53);
    bound["requirements"][0]["clauses"][0].set("source", span(0, "contract", 1_i64 << 53));
    add(
        cases,
        "package-revision-maximum",
        ConformanceOperation::Package,
        bound,
        std::iter::empty::<String>(),
    );
    for (id, code, mutation) in [
        (
            "package-invalid-requirement-revision-over",
            "invalid_requirement_revision",
            PackageMutation::RequirementRevisionOver,
        ),
        (
            "package-invalid-source-revision-over",
            "invalid_source_revision",
            PackageMutation::SourceRevisionOver,
        ),
        (
            "package-invalid-span-offset-over",
            "invalid_source_span",
            PackageMutation::SpanOffsetOver,
        ),
    ] {
        let mut candidate = package(1);
        mutation.apply(&mut candidate);
        add(
            cases,
            id,
            ConformanceOperation::Package,
            wire_document(&candidate),
            [format!("diagnostic:{code}")],
        );
    }
    for (id, major, minor, boundary) in [
        (
            "package-schema-1-0",
            1,
            0,
            "boundary:schema.unsupported_minor",
        ),
        (
            "package-schema-1-2",
            1,
            2,
            "boundary:schema.unsupported_minor",
        ),
        ("package-schema-2-0", 2, 0, "boundary:schema.unknown_major"),
    ] {
        let mut unsupported = package(1);
        unsupported.set("schema_version", obj!("major" => major, "minor" => minor));
        add(
            cases,
            id,
            ConformanceOperation::Package,
            unsupported,
            ["diagnostic:unsupported_schema_version", boundary],
        );
    }
}

fn expression_cases(cases: &mut Vec<Case>) {
    let constructs: Vec<(&str, J, J, Vec<CaseCoverage>, bool)> = vec![
        (
            "boolean-literal",
            boolean(true),
            bool_type(),
            vec![CaseCoverage::Construct("expression.boolean_literal")],
            true,
        ),
        (
            "integer-literal",
            one(),
            int_type(),
            vec![
                CaseCoverage::Construct("expression.integer_literal"),
                CaseCoverage::Construct("type.integer"),
            ],
            false,
        ),
        (
            "rational-literal",
            expression!("rational_literal", "numerator" => "2", "denominator" => "1", "value_type" => rational_type()),
            rational_type(),
            vec![
                CaseCoverage::Construct("expression.rational_literal"),
                CaseCoverage::Construct("type.rational"),
                CaseCoverage::Boundary("rational.normalized"),
            ],
            false,
        ),
        (
            "text-literal",
            expression!("text_literal", "value" => "line\nend"),
            obj!("kind" => "text"),
            vec![
                CaseCoverage::Construct("expression.text_literal"),
                CaseCoverage::Construct("type.text"),
                CaseCoverage::Boundary("canonical.escape_controls"),
            ],
            false,
        ),
        (
            "enum-literal",
            expression!("enum_literal", "enumeration" => "Color", "variant" => "red"),
            obj!("kind" => "enum", "name" => "Color"),
            vec![
                CaseCoverage::Construct("expression.enum_literal"),
                CaseCoverage::Construct("type.enum"),
                CaseCoverage::Construct("dependency.kind.enum_variant"),
            ],
            false,
        ),
        (
            "option-none",
            expression!("option_none", "value_type" => option_type()),
            option_type(),
            vec![
                CaseCoverage::Construct("expression.option_none"),
                CaseCoverage::Construct("type.option"),
            ],
            false,
        ),
        (
            "option-some",
            expression!("option_some", "value_type" => option_type(), "value" => one()),
            option_type(),
            vec![CaseCoverage::Construct("expression.option_some")],
            false,
        ),
        (
            "record-literal",
            expression!("record_literal", "record" => "Sensor", "fields" => vec![obj!("name" => "reading", "value" => one())]),
            obj!("kind" => "record", "name" => "Sensor"),
            vec![
                CaseCoverage::Construct("expression.record_literal"),
                CaseCoverage::Construct("type.record"),
                CaseCoverage::Construct("dependency.kind.field"),
            ],
            false,
        ),
        (
            "collection-literal",
            expression!("collection_literal", "value_type" => collection_type(), "items" => vec![one()]),
            collection_type(),
            vec![
                CaseCoverage::Construct("expression.collection_literal"),
                CaseCoverage::Construct("type.collection"),
            ],
            false,
        ),
        (
            "value-input",
            value("divisor", "current"),
            int_type(),
            vec![
                CaseCoverage::Construct("expression.value_reference"),
                CaseCoverage::Construct("dependency.kind.input"),
            ],
            false,
        ),
        (
            "value-state",
            value("state_value", "pre"),
            int_type(),
            vec![CaseCoverage::Construct("dependency.kind.state")],
            false,
        ),
        (
            "field-access",
            expression!("field_access", "base" => value("sensor", "pre"), "field" => "reading"),
            int_type(),
            vec![CaseCoverage::Construct("expression.field_access")],
            false,
        ),
        (
            "is-present",
            expression!("is_present", "option" => value("maybe", "pre")),
            bool_type(),
            vec![CaseCoverage::Construct("expression.is_present")],
            false,
        ),
        (
            "unwrap",
            expression!("unwrap", "option" => expression!("option_some", "value_type" => option_type(), "value" => one())),
            int_type(),
            vec![CaseCoverage::Construct("expression.unwrap")],
            false,
        ),
        (
            "length",
            expression!("length", "collection" => value("items", "pre")),
            uint_type(),
            vec![CaseCoverage::Construct("expression.length")],
            false,
        ),
        (
            "index",
            expression!("index", "collection" => expression!("collection_literal", "value_type" => collection_type(), "items" => vec![one()]), "index" => integer(0, uint_type())),
            int_type(),
            vec![CaseCoverage::Construct("expression.index")],
            false,
        ),
        (
            "call",
            expression!("call", "function" => "identity", "arguments" => vec![one()]),
            int_type(),
            vec![
                CaseCoverage::Construct("expression.call"),
                CaseCoverage::Construct("dependency.kind.pure_function"),
            ],
            false,
        ),
        (
            "numeric",
            numeric(one(), one(), "add"),
            int_type(),
            vec![CaseCoverage::Construct("expression.numeric")],
            false,
        ),
        (
            "numeric-negate",
            expression!("numeric_negate", "operand" => one()),
            int_type(),
            vec![CaseCoverage::Construct("expression.numeric_negate")],
            false,
        ),
        (
            "compare",
            compare(one(), one(), "equal"),
            bool_type(),
            vec![CaseCoverage::Construct("expression.compare")],
            false,
        ),
        (
            "boolean-not",
            expression!("boolean_not", "operand" => boolean(true)),
            bool_type(),
            vec![CaseCoverage::Construct("expression.boolean_not")],
            false,
        ),
        (
            "boolean",
            boolean_op(boolean(true), boolean(true), "total_and"),
            bool_type(),
            vec![CaseCoverage::Construct("expression.boolean")],
            false,
        ),
        (
            "quantifier",
            expression!("quantifier", "quantifier" => "for_all", "domain" => "elements", "collection" => value("items", "pre"), "local" => "element", "local_source" => expression_span(22), "predicate" => compare(expression!("local_reference", "name" => "element"), one(), "equal")),
            bool_type(),
            vec![
                CaseCoverage::Construct("expression.quantifier"),
                CaseCoverage::Construct("expression.local_reference"),
            ],
            false,
        ),
    ];
    for (id, expression, expected, tags, complete_declarations) in constructs {
        let mut covers = tags
            .into_iter()
            .map(CaseCoverage::into_label)
            .collect::<Vec<_>>();
        if complete_declarations {
            covers.extend(
                [
                    "construct:declaration.enum",
                    "construct:declaration.record",
                    "construct:declaration.input",
                    "construct:declaration.state",
                    "construct:declaration.function",
                    "construct:type.boolean",
                    "construct:execution.pre",
                ]
                .into_iter()
                .map(str::to_owned),
            );
        }
        add(
            cases,
            format!("expression-{id}"),
            ConformanceOperation::Expression,
            expression_input(expression, expected),
            covers,
        );
    }
    for (kind, field) in [
        ("post", "operation"),
        ("initialization", "name"),
        ("handler", "name"),
    ] {
        let mut input = expression_input(boolean(true), bool_type());
        input.set("execution_point", obj!("kind" => kind, field => "check"));
        add(
            cases,
            format!("expression-execution-{kind}"),
            ConformanceOperation::Expression,
            input,
            [format!("construct:execution.{kind}")],
        );
    }

    let diagnostics: Vec<(&str, J, J, &str, bool)> = vec![
        (
            "invalid-wire",
            expression!("integer_literal", "value" => "1", "value_type" => bool_type()),
            int_type(),
            "invalid_wire_format",
            false,
        ),
        (
            "orphan-value",
            value("missing", "current"),
            bool_type(),
            "orphaned_value_reference",
            false,
        ),
        (
            "orphan-function",
            expression!("call", "function" => "missing", "arguments" => Vec::<J>::new()),
            bool_type(),
            "orphaned_function_reference",
            false,
        ),
        (
            "invalid-state",
            value("state_value", "post"),
            int_type(),
            "invalid_state_observation",
            false,
        ),
        (
            "invalid-scope",
            expression!("local_reference", "name" => "missing"),
            bool_type(),
            "invalid_scope",
            false,
        ),
        (
            "arity",
            expression!("call", "function" => "identity", "arguments" => Vec::<J>::new()),
            int_type(),
            "arity_mismatch",
            false,
        ),
        (
            "ill-typed",
            numeric(boolean(true), boolean(true), "add"),
            int_type(),
            "ill_typed_expression",
            false,
        ),
        (
            "result-type",
            one(),
            bool_type(),
            "result_type_mismatch",
            false,
        ),
        (
            "non-boolean-root",
            one(),
            int_type(),
            "non_boolean_clause_root",
            true,
        ),
        (
            "collection-bound",
            expression!("collection_literal", "value_type" => obj!("kind" => "collection", "element" => bool_type(), "maximum_items" => 1), "items" => vec![boolean(true), boolean(false)]),
            obj!("kind" => "collection", "element" => bool_type(), "maximum_items" => 1),
            "collection_bound_exceeded",
            false,
        ),
        (
            "orphan-type",
            expression!("enum_literal", "enumeration" => "Missing", "variant" => "value"),
            obj!("kind" => "enum", "name" => "Missing"),
            "orphaned_type_reference",
            false,
        ),
    ];
    for (id, expression, expected, code, clause_root) in diagnostics {
        let mut request = expression_input(expression, expected);
        if clause_root {
            request.set("clause_root", true);
        }
        add(
            cases,
            format!("expression-{id}"),
            ConformanceOperation::Expression,
            request,
            [format!("diagnostic:{code}")],
        );
    }

    for (id, code, mutation) in [
        (
            "duplicate-type",
            "duplicate_type_declaration",
            ExpressionMutation::DuplicateType,
        ),
        (
            "duplicate-value",
            "duplicate_value_declaration",
            ExpressionMutation::DuplicateValue,
        ),
        (
            "duplicate-function",
            "duplicate_function_declaration",
            ExpressionMutation::DuplicateFunction,
        ),
        (
            "duplicate-field",
            "duplicate_field",
            ExpressionMutation::DuplicateField,
        ),
        (
            "duplicate-variant",
            "duplicate_variant",
            ExpressionMutation::DuplicateVariant,
        ),
        (
            "duplicate-parameter",
            "duplicate_parameter",
            ExpressionMutation::DuplicateParameter,
        ),
        ("empty-enum", "empty_enum", ExpressionMutation::EmptyEnum),
        (
            "invalid-numeric",
            "invalid_numeric_bounds",
            ExpressionMutation::InvalidNumeric,
        ),
        (
            "unbounded-collection",
            "unbounded_collection",
            ExpressionMutation::UnboundedCollection,
        ),
        (
            "recursive-type",
            "recursive_type",
            ExpressionMutation::RecursiveType,
        ),
    ] {
        let mut request = expression_input(boolean(true), bool_type());
        mutation.apply(&mut request);
        let mut covers = vec![format!("diagnostic:{code}")];
        if let Some(coverage) = mutation.additional_coverage() {
            covers.push(coverage.into_label());
        }
        add(
            cases,
            format!("expression-{id}"),
            ConformanceOperation::Expression,
            request,
            covers,
        );
    }

    for (id, expression, expected, obligation) in [
        (
            "non-zero-divisor",
            numeric(
                integer(10, int_type()),
                value("divisor", "current"),
                "divide",
            ),
            int_type(),
            "non_zero_divisor",
        ),
        (
            "option-presence",
            expression!("unwrap", "option" => value("maybe", "pre")),
            int_type(),
            "option_presence",
        ),
        (
            "index-in-bounds",
            expression!("index", "collection" => value("items", "pre"), "index" => value("position", "current")),
            int_type(),
            "index_in_bounds",
        ),
        (
            "checked-range",
            numeric(value("divisor", "current"), one(), "add"),
            int_type(),
            "checked_range",
        ),
    ] {
        add(
            cases,
            format!("expression-obligation-{id}"),
            ConformanceOperation::Expression,
            expression_input(expression, expected),
            [
                "diagnostic:potentially_undefined".to_owned(),
                format!("obligation:{obligation}"),
            ],
        );
    }

    expression_edges(cases);
}
fn nested_option(depth: usize) -> J {
    let mut result = bool_type();
    for _ in 1..depth {
        result = obj!("kind" => "option", "value" => result);
    }
    result
}

fn empty_function(index: usize) -> J {
    obj!("name" => format!("f{index}"), "parameters" => Vec::<J>::new(),
        "result_type" => bool_type(), "source" => expression_span((index % 100) as i64))
}

fn empty_value(index: usize) -> J {
    obj!("name" => format!("v{index}"), "kind" => "input",
        "value_type" => bool_type(), "source" => expression_span((index % 100) as i64))
}

fn expression_edges(cases: &mut Vec<Case>) {
    let mut edge = expression_input(boolean(true), bool_type());
    edge["values"][0].set("value_type", obj!("kind" => "integer", "domain" => "signed",
        "minimum" => i64::MIN.to_string(), "maximum" => i64::MAX.to_string(), "overflow" => "reject"));
    edge["values"][3].set(
        "value_type",
        obj!("kind" => "collection", "element" => bool_type(), "maximum_items" => u32::MAX),
    );
    edge["values"].push(obj!("name" => "wide_rational", "kind" => "input",
        "value_type" => obj!("kind" => "rational", "numerator_minimum" => "-1", "numerator_maximum" => "1", "maximum_denominator" => i64::MAX.to_string()),
        "source" => expression_span(14)));
    add(
        cases,
        "expression-numeric-edges",
        ConformanceOperation::Expression,
        edge,
        [
            "boundary:integer.minimum",
            "boundary:integer.maximum",
            "boundary:collection.declared_maximum",
            "boundary:rational.maximum_denominator",
        ],
    );

    let mut zero_denominator = expression_input(boolean(true), bool_type());
    zero_denominator["values"][0].set("value_type", obj!("kind" => "rational", "numerator_minimum" => "-1", "numerator_maximum" => "1", "maximum_denominator" => "0"));
    add(
        cases,
        "expression-rational-zero-denominator",
        ConformanceOperation::Expression,
        zero_denominator,
        [
            "diagnostic:invalid_numeric_bounds",
            "boundary:rational.zero_denominator",
        ],
    );
    let mut declared_over = expression_input(boolean(true), bool_type());
    declared_over["values"][3]["value_type"].set("maximum_items", u64::from(u32::MAX) + 1);
    add(
        cases,
        "expression-declared-collection-over",
        ConformanceOperation::Expression,
        declared_over,
        [
            "diagnostic:invalid_numeric_bounds",
            "boundary:collection.declared_out_of_range",
        ],
    );

    let mut number_bound = expression_input(boolean(true), bool_type());
    number_bound["values"][0]["value_type"].set("minimum", -10);
    add(
        cases,
        "expression-invalid-wire-number-bound",
        ConformanceOperation::Expression,
        wire_document(&number_bound),
        ["diagnostic:invalid_wire_format"],
    );
    let mut number_literal = expression_input(one(), int_type());
    number_literal["expression"].set("value", 1);
    add(
        cases,
        "expression-invalid-wire-number-literal",
        ConformanceOperation::Expression,
        wire_document(&number_literal),
        ["diagnostic:invalid_wire_format"],
    );
    let mut grammar_string = expression_input(boolean(true), bool_type());
    grammar_string["values"][0]["value_type"].set("maximum", "+10");
    add(
        cases,
        "expression-invalid-wire-string-grammar",
        ConformanceOperation::Expression,
        wire_document(&grammar_string),
        ["diagnostic:invalid_wire_format"],
    );
    let mut range_over = expression_input(boolean(true), bool_type());
    range_over["values"][0]["value_type"].set("maximum", (1_u128 << 127).to_string());
    add(
        cases,
        "expression-invalid-numeric-range",
        ConformanceOperation::Expression,
        range_over,
        ["diagnostic:invalid_numeric_bounds"],
    );

    add(
        cases,
        "expression-text-maximum",
        ConformanceOperation::Expression,
        expression_input(
            expression!("text_literal", "value" => "x".repeat(1_048_576)),
            obj!("kind" => "text"),
        ),
        ["boundary:text.maximum"],
    );
    add(
        cases,
        "expression-text-over",
        ConformanceOperation::Expression,
        expression_input(
            expression!("text_literal", "value" => "x".repeat(1_048_577)),
            obj!("kind" => "text"),
        ),
        [
            "diagnostic:text_bound_exceeded",
            "boundary:text.over_maximum",
        ],
    );

    let expression_max = expression!("collection_literal",
        "value_type" => obj!("kind" => "collection", "element" => bool_type(), "maximum_items" => 10_000),
        "items" => (0..9_999).map(|_| boolean(true)).collect::<Vec<_>>());
    add(
        cases,
        "expression-nodes-maximum",
        ConformanceOperation::Expression,
        expression_input(
            expression_max.clone(),
            obj!("kind" => "collection", "element" => bool_type(), "maximum_items" => 10_000),
        ),
        ["boundary:expression.nodes.maximum"],
    );
    let mut expression_over = expression_max;
    expression_over["items"].push(boolean(true));
    add(
        cases,
        "expression-nodes-over",
        ConformanceOperation::Expression,
        expression_input(
            expression_over,
            obj!("kind" => "collection", "element" => bool_type(), "maximum_items" => 10_000),
        ),
        [
            "diagnostic:expression_too_large",
            "boundary:expression.nodes.over_maximum",
        ],
    );

    let mut type_max = expression_input(boolean(true), bool_type());
    type_max.set("values", vec![obj!("name" => "deep", "kind" => "input", "value_type" => nested_option(256), "source" => expression_span(1))]);
    type_max.set("types", Vec::<J>::new());
    type_max.set("functions", Vec::<J>::new());
    add(
        cases,
        "expression-type-depth-maximum",
        ConformanceOperation::Expression,
        type_max.clone(),
        ["boundary:type.depth.maximum"],
    );
    let mut type_over = type_max;
    type_over["values"][0].set("value_type", nested_option(257));
    add(
        cases,
        "expression-type-depth-over",
        ConformanceOperation::Expression,
        type_over,
        [
            "diagnostic:semantic_input_too_large",
            "boundary:type.depth.over_maximum",
        ],
    );
    let mut embedded_type_over = expression_input(
        expression!("option_none", "value_type" => nested_option(257)),
        nested_option(257),
    );
    embedded_type_over.set("types", Vec::<J>::new());
    embedded_type_over.set("values", Vec::<J>::new());
    embedded_type_over.set("functions", Vec::<J>::new());
    add(
        cases,
        "expression-embedded-type-depth-over",
        ConformanceOperation::Expression,
        embedded_type_over,
        [
            "diagnostic:semantic_input_too_large",
            "boundary:type.depth.over_maximum",
        ],
    );

    let mut expression_depth_max = boolean(true);
    for _ in 0..255 {
        expression_depth_max = expression!("boolean_not", "operand" => expression_depth_max);
    }
    add(
        cases,
        "expression-depth-maximum",
        ConformanceOperation::Expression,
        expression_input(expression_depth_max.clone(), bool_type()),
        ["boundary:expression.depth.maximum"],
    );
    let expression_depth_over = expression!("boolean_not", "operand" => expression_depth_max);
    add(
        cases,
        "expression-depth-over",
        ConformanceOperation::Expression,
        expression_input(expression_depth_over, bool_type()),
        [
            "diagnostic:semantic_input_too_large",
            "boundary:expression.depth.over_maximum",
        ],
    );

    let mut semantic_max = expression_input(boolean(true), bool_type());
    semantic_max.set("types", Vec::<J>::new());
    semantic_max.set(
        "functions",
        (0..10_000).map(empty_function).collect::<Vec<_>>(),
    );
    semantic_max.set("values", (0..2_499).map(empty_value).collect::<Vec<_>>());
    add(
        cases,
        "expression-semantic-nodes-maximum",
        ConformanceOperation::Expression,
        semantic_max.clone(),
        [
            "boundary:collection.maximum",
            "boundary:semantic.nodes.maximum",
            "boundary:semantic_collection.maximum",
        ],
    );
    let mut semantic_over = semantic_max;
    semantic_over["values"][0].set(
        "value_type",
        obj!("kind" => "option", "value" => bool_type()),
    );
    add(
        cases,
        "expression-semantic-nodes-over",
        ConformanceOperation::Expression,
        semantic_over,
        [
            "diagnostic:semantic_input_too_large",
            "boundary:semantic.nodes.over_maximum",
        ],
    );
    let mut collection_over = expression_input(boolean(true), bool_type());
    collection_over.set("types", Vec::<J>::new());
    collection_over.set("values", Vec::<J>::new());
    collection_over.set(
        "functions",
        (0..10_001).map(empty_function).collect::<Vec<_>>(),
    );
    add(
        cases,
        "expression-semantic-collection-over",
        ConformanceOperation::Expression,
        collection_over,
        [
            "diagnostic:semantic_input_too_large",
            "boundary:semantic_collection.over_maximum",
            "boundary:collection.over_maximum",
        ],
    );
}

fn trace(artifact: &str, target: J, at: i64, deep: bool) -> J {
    let depth = obj!("kind" => if deep { "deep" } else { "shallow" });
    obj!("artifact_id" => artifact, "source" => span(at, "trace", 1), "target" => target,
        "target_span" => span(at + 1, "trace", 1), "depth" => depth)
}

fn coverage_cases(cases: &mut Vec<Case>) {
    let coverage_package = package(2);
    let req_a = owner();
    let req_b =
        obj!("package" => "agent-ix/conformance", "requirement" => "REQ_1", "revision" => 1);
    add(
        cases,
        "coverage-shallow",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package.clone(),
        "traces" => vec![trace("shallow", req_a.clone(), 1, false)]),
        [
            "construct:artifact.depth.shallow",
            "construct:coverage.class.shallow",
        ],
    );
    add(
        cases,
        "coverage-deep",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package.clone(),
        "traces" => vec![trace("deep", req_a.clone(), 3, true)]),
        [
            "construct:artifact.depth.deep",
            "construct:coverage.class.deep",
        ],
    );
    add(
        cases,
        "coverage-uncovered",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package.clone(), "traces" => Vec::<J>::new()),
        ["construct:coverage.class.uncovered"],
    );
    let mut cross = req_a.clone();
    cross.set("package", "agent-ix/other");
    add(
        cases,
        "coverage-cross-package",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package.clone(),
        "traces" => vec![trace("cross", cross, 5, false)]),
        [
            "construct:coverage.class.orphaned",
            "diagnostic:cross_package_reference",
            "boundary:artifact.cross_package",
        ],
    );
    let mut missing = req_a.clone();
    missing.set("requirement", "REQ_missing");
    add(
        cases,
        "coverage-missing",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package.clone(),
        "traces" => vec![trace("missing", missing, 7, false)]),
        [
            "diagnostic:orphaned_requirement_reference",
            "boundary:artifact.missing",
        ],
    );
    let mut stale = req_a.clone();
    stale.set("revision", 9);
    add(
        cases,
        "coverage-stale",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package.clone(),
        "traces" => vec![trace("stale", stale, 9, true)]),
        [
            "diagnostic:stale_requirement_revision",
            "boundary:artifact.stale",
        ],
    );
    add(
        cases,
        "coverage-duplicate",
        ConformanceOperation::Coverage,
        obj!("package" => coverage_package,
        "traces" => vec![trace("duplicate", req_b.clone(), 13, false), trace("duplicate", req_b, 15, false)]),
        [
            "diagnostic:duplicate_artifact_trace",
            "boundary:artifact.duplicate",
        ],
    );
}
