#!/usr/bin/env python3
"""Generate the checked-in v0.1 corpus from declarative fixture builders.

The script never invents expected results: it bootstraps schema-valid empty
expectations, executes the conformance runner, and freezes its reported actual
values. The runner independently rejects any `covers` token that it cannot
observe in the fixture.
"""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORPUS = ROOT / "corpus" / "contract-v0.1"
RUNNER = pathlib.Path("/tmp/quire-contract-ir-cargo-target/debug/quire-contract-conformance")


def span(at: int = 0, document: str = "contract", revision: int = 1) -> dict:
    source = {"document": document, "revision": revision}
    return {
        "start": {"source": source, "line": 1, "column": at + 1, "byte_offset": at},
        "end": {"source": source, "line": 1, "column": at + 2, "byte_offset": at + 1},
    }


OWNER = {"package": "agent-ix/conformance", "requirement": "REQ_alpha", "revision": 1}
BOOL = {"kind": "boolean"}
INT = {"kind": "integer", "domain": "signed", "minimum": -10, "maximum": 10, "overflow": "reject"}
UINT = {"kind": "integer", "domain": "unsigned", "minimum": 0, "maximum": 4, "overflow": "reject"}
RAT = {"kind": "rational", "numerator_minimum": -10, "numerator_maximum": 10, "maximum_denominator": 10}
COLL = {"kind": "collection", "element": INT, "maximum_items": 4}
OPT = {"kind": "option", "value": INT}


def expr(node: str, **fields: object) -> dict:
    return {"node": node, **fields, "source": span(20, "expression")}


def integer(value: int = 1, value_type: dict = INT) -> dict:
    return expr("integer_literal", value=value, value_type=copy.deepcopy(value_type))


def boolean(value: bool = True) -> dict:
    return expr("boolean_literal", value=value)


def value(name: str, observation: str = "current") -> dict:
    return expr("value_reference", name=name, observation=observation)


def compare(left: dict, right: dict, operator: str = "equal") -> dict:
    return expr("compare", operator=operator, left=left, right=right)


def numeric(left: dict, right: dict, operator: str = "add") -> dict:
    return expr("numeric", operator=operator, left=left, right=right)


def boolean_op(left: dict, right: dict, operator: str = "total_and") -> dict:
    return expr("boolean", operator=operator, left=left, right=right)


def environment() -> dict:
    return {
        "owner": copy.deepcopy(OWNER),
        "types": [
            {"kind": "enum", "name": "Color", "source": span(1, "expression"), "variants": [
                {"name": "red", "source": span(2, "expression")},
                {"name": "blue", "source": span(3, "expression")},
            ]},
            {"kind": "record", "name": "Sensor", "source": span(4, "expression"), "fields": [
                {"name": "reading", "value_type": copy.deepcopy(INT), "source": span(5, "expression")}
            ]},
        ],
        "values": [
            {"name": "divisor", "kind": "input", "value_type": copy.deepcopy(INT), "source": span(6, "expression")},
            {"name": "state_value", "kind": "state", "value_type": copy.deepcopy(INT), "source": span(7, "expression")},
            {"name": "maybe", "kind": "state", "value_type": copy.deepcopy(OPT), "source": span(8, "expression")},
            {"name": "items", "kind": "state", "value_type": copy.deepcopy(COLL), "source": span(9, "expression")},
            {"name": "position", "kind": "input", "value_type": copy.deepcopy(UINT), "source": span(10, "expression")},
            {"name": "sensor", "kind": "state", "value_type": {"kind": "record", "name": "Sensor"}, "source": span(11, "expression")},
        ],
        "functions": [{
            "name": "identity",
            "parameters": [{"name": "argument", "value_type": copy.deepcopy(INT), "source": span(12, "expression")}],
            "result_type": copy.deepcopy(INT),
            "source": span(13, "expression"),
        }],
        "expression": boolean(),
        "expected_type": copy.deepcopy(BOOL),
        "execution_point": {"kind": "pre", "operation": "check"},
        "clause_root": False,
    }


def expression_input(expression: dict, expected: dict, **changes: object) -> dict:
    result = environment()
    result.update({"expression": expression, "expected_type": copy.deepcopy(expected)})
    result.update(changes)
    return result


def package(requirements: int = 1) -> dict:
    clauses = [{
        "id": "note",
        "kind": "information",
        "source": span(0),
        "body": {"node": "literal"},
    }]
    values = []
    for index in range(requirements):
        values.append({
            "id": "REQ_alpha" if index == 0 else f"REQ_{index}",
            "revision": 1,
            "source": span(index * 2),
            "clauses": copy.deepcopy(clauses),
        })
    return {
        "id": "agent-ix/conformance",
        "schema_version": {"major": 1, "minor": 1},
        "source": {"document": "contract", "revision": 1},
        "requirements": values,
    }


def dependency(kind: str = "input", package_id: str = "agent-ix/conformance", requirement: str = "REQ_alpha", revision: int = 1) -> dict:
    value = {
        "requirement": {"package": package_id, "requirement": requirement, "revision": revision},
        "kind": kind,
        "path": ["value"],
    }
    if kind in ("input", "state"):
        value["observation"] = "current"
    return value


def package_with_clause(kind: str, anchor: dict | None, body: dict | None = None) -> dict:
    result = package()
    clause = result["requirements"][0]["clauses"][0]
    clause["kind"] = kind
    if anchor is not None:
        clause["anchor"] = anchor
    clause["body"] = body or {"node": "literal"}
    return result


def add_case(cases: list, fixture_id: str, operation: str, value: dict, *covers: str) -> None:
    cases.append({"id": fixture_id, "operation": operation, "value": value, "covers": sorted(set(covers) | {f"operation:{operation}"})})


def build_cases() -> list:
    cases: list[dict] = []

    all_clauses = package()
    all_clauses["requirements"][0]["clauses"] = [
        {"id": "a_assert", "kind": "assertion", "anchor": {"kind": "pre", "operation": "check"}, "source": span(1), "body": {"node": "literal"}},
        {"id": "b_case", "kind": "case", "anchor": {"kind": "handler", "name": "handle"}, "source": span(2), "body": {"node": "composite", "children": [{"node": "composite", "children": []}, {"node": "literal"}]}},
        {"id": "c_info", "kind": "information", "source": span(3), "body": {"node": "literal"}},
        {"id": "d_inv", "kind": "invariant", "anchor": {"kind": "initialization", "name": "boot"}, "source": span(4), "body": {"node": "literal"}},
        {"id": "e_post", "kind": "postcondition", "anchor": {"kind": "post", "operation": "check"}, "source": span(5), "body": {"node": "literal"}},
        {"id": "f_pre", "kind": "precondition", "anchor": {"kind": "pre", "operation": "check"}, "source": span(6), "body": {"node": "literal"}},
    ]
    all_clauses["requirements"][0]["clauses"].reverse()
    add_case(cases, "package-constructs", "package", all_clauses,
             *[f"construct:clause_kind.{name}" for name in ("assertion", "case", "information", "invariant", "postcondition", "precondition")],
             "construct:reference_body.literal", "construct:reference_body.composite",
             "boundary:canonical.semantic_set_order", "boundary:canonical.sequence_order",
             "boundary:revision.current", "boundary:schema.1_1", "boundary:source_span.minimum")

    referenced = package_with_clause("information", None, {"node": "reference", "identity": dependency()})
    add_case(cases, "package-reference", "package", referenced,
             "construct:reference_body.reference", "construct:dependency.kind.input")

    for fixture_id, mutate, code, extra in [
        ("package-invalid-namespace", lambda p: p.update(id="agent/../bad"), "invalid_package_namespace", []),
        ("package-invalid-schema", lambda p: p["schema_version"].update(major=0), "invalid_schema_version", ["boundary:schema.zero_major"]),
        ("package-invalid-identifier", lambda p: p["requirements"][0].update(id="bad id"), "invalid_identifier", []),
        ("package-invalid-requirement-revision", lambda p: p["requirements"][0].update(revision=0), "invalid_requirement_revision", []),
        ("package-invalid-source-revision", lambda p: p["source"].update(revision=0), "invalid_source_revision", []),
        ("package-invalid-span", lambda p: p["requirements"][0]["source"]["start"].update(byte_offset=100), "invalid_source_span", ["boundary:source_span.reversed"]),
    ]:
        candidate = package(); mutate(candidate)
        add_case(cases, fixture_id, "package", candidate, f"diagnostic:{code}", *extra)

    duplicate = package(); duplicate["requirements"].append(copy.deepcopy(duplicate["requirements"][0]))
    add_case(cases, "package-duplicate-requirement", "package", duplicate, "diagnostic:duplicate_requirement")
    duplicate = package(); duplicate["requirements"][0]["clauses"].append(copy.deepcopy(duplicate["requirements"][0]["clauses"][0]))
    add_case(cases, "package-duplicate-clause", "package", duplicate, "diagnostic:duplicate_clause")
    for fixture_id, kind, anchor, code in [
        ("package-floating", "precondition", None, "floating_executable_clause"),
        ("package-information-anchored", "information", {"kind": "pre", "operation": "check"}, "informational_clause_anchored"),
        ("package-incompatible-anchor", "postcondition", {"kind": "pre", "operation": "check"}, "incompatible_clause_anchor"),
    ]:
        add_case(cases, fixture_id, "package", package_with_clause(kind, anchor), f"diagnostic:{code}")
    for fixture_id, dep, code, boundary in [
        ("package-cross-reference", dependency(package_id="agent-ix/other"), "cross_package_reference", "boundary:artifact.cross_package"),
        ("package-malformed-reference", {**dependency(), "path": []}, "malformed_reference", None),
        ("package-stale-reference", dependency(revision=2), "stale_requirement_revision", "boundary:revision.stale"),
        ("package-orphan-reference", dependency(requirement="REQ_missing"), "orphaned_requirement_reference", "boundary:artifact.missing"),
    ]:
        candidate = package_with_clause("information", None, {"node": "reference", "identity": dep})
        tokens = [f"diagnostic:{code}"] + ([boundary] if boundary else [])
        add_case(cases, fixture_id, "package", candidate, *tokens)
    clause_probe = {"package": package(), "clause_resolutions": [{"requirement": OWNER, "clause": "missing"}]}
    add_case(cases, "package-orphan-clause", "package", clause_probe, "diagnostic:orphaned_clause_reference")
    add_case(cases, "package-canonical-limit", "package", {"package": package(), "canonical_maximum_bytes": 0},
             "diagnostic:canonicalization_resource_exhausted", "boundary:canonical.resource_failure")

    construct_cases = [
        ("boolean-literal", boolean(), BOOL, ["expression.boolean_literal"]),
        ("integer-literal", integer(), INT, ["expression.integer_literal", "type.integer"]),
        ("rational-literal", expr("rational_literal", numerator=2, denominator=1, value_type=copy.deepcopy(RAT)), RAT, ["expression.rational_literal", "type.rational", "boundary:rational.normalized"]),
        ("text-literal", expr("text_literal", value="line\nend"), {"kind": "text"}, ["expression.text_literal", "type.text", "boundary:canonical.escape_controls"]),
        ("enum-literal", expr("enum_literal", enumeration="Color", variant="red"), {"kind": "enum", "name": "Color"}, ["expression.enum_literal", "type.enum", "dependency.kind.enum_variant"]),
        ("option-none", expr("option_none", value_type=copy.deepcopy(OPT)), OPT, ["expression.option_none", "type.option"]),
        ("option-some", expr("option_some", value_type=copy.deepcopy(OPT), value=integer()), OPT, ["expression.option_some"]),
        ("record-literal", expr("record_literal", record="Sensor", fields=[{"name": "reading", "value": integer()}]), {"kind": "record", "name": "Sensor"}, ["expression.record_literal", "type.record", "dependency.kind.field"]),
        ("collection-literal", expr("collection_literal", value_type=copy.deepcopy(COLL), items=[integer()]), COLL, ["expression.collection_literal", "type.collection"]),
        ("value-input", value("divisor"), INT, ["expression.value_reference", "dependency.kind.input"]),
        ("value-state", value("state_value", "pre"), INT, ["dependency.kind.state"]),
        ("field-access", expr("field_access", base=value("sensor", "pre"), field="reading"), INT, ["expression.field_access"]),
        ("is-present", expr("is_present", option=value("maybe", "pre")), BOOL, ["expression.is_present"]),
        ("unwrap", expr("unwrap", option=expr("option_some", value_type=copy.deepcopy(OPT), value=integer())), INT, ["expression.unwrap"]),
        ("length", expr("length", collection=value("items", "pre")), UINT, ["expression.length"]),
        ("index", expr("index", collection=expr("collection_literal", value_type=copy.deepcopy(COLL), items=[integer()]), index=integer(0, UINT)), INT, ["expression.index"]),
        ("call", expr("call", function="identity", arguments=[integer()]), INT, ["expression.call", "dependency.kind.pure_function"]),
        ("numeric", numeric(integer(), integer()), INT, ["expression.numeric"]),
        ("numeric-negate", expr("numeric_negate", operand=integer()), INT, ["expression.numeric_negate"]),
        ("compare", compare(integer(), integer()), BOOL, ["expression.compare"]),
        ("boolean-not", expr("boolean_not", operand=boolean()), BOOL, ["expression.boolean_not"]),
        ("boolean", boolean_op(boolean(), boolean()), BOOL, ["expression.boolean"]),
        ("quantifier", expr("quantifier", quantifier="for_all", domain="elements", collection=value("items", "pre"), local="element", local_source=span(22, "expression"), predicate=compare(expr("local_reference", name="element"), integer())), BOOL, ["expression.quantifier", "expression.local_reference"]),
    ]
    for fixture_id, expression, expected, tags in construct_cases:
        tokens = [f"construct:{tag}" if not tag.startswith("boundary:") else tag for tag in tags]
        if fixture_id == "boolean-literal":
            tokens += [
                "construct:declaration.enum", "construct:declaration.record", "construct:declaration.input",
                "construct:declaration.state", "construct:declaration.function", "construct:type.boolean",
                "construct:execution.pre",
            ]
        add_case(cases, f"expression-{fixture_id}", "expression", expression_input(expression, expected), *tokens)
    for kind, field in [("post", "operation"), ("initialization", "name"), ("handler", "name")]:
        add_case(cases, f"expression-execution-{kind}", "expression",
                 expression_input(boolean(), BOOL, execution_point={"kind": kind, field: "check"}),
                 f"construct:execution.{kind}")

    diagnostic_cases = []
    def bad(fixture_id: str, expression: dict, expected: dict, code: str, **changes: object) -> None:
        diagnostic_cases.append((fixture_id, expression_input(expression, expected, **changes), code, []))
    bad("invalid-wire", expr("integer_literal", value=1, value_type=copy.deepcopy(BOOL)), INT, "invalid_wire_format")
    bad("orphan-value", value("missing"), BOOL, "orphaned_value_reference")
    bad("orphan-function", expr("call", function="missing", arguments=[]), BOOL, "orphaned_function_reference")
    bad("invalid-state", value("state_value", "post"), INT, "invalid_state_observation")
    bad("invalid-scope", expr("local_reference", name="missing"), BOOL, "invalid_scope")
    bad("arity", expr("call", function="identity", arguments=[]), INT, "arity_mismatch")
    bad("ill-typed", numeric(boolean(), boolean()), INT, "ill_typed_expression")
    bad("result-type", integer(), BOOL, "result_type_mismatch")
    bad("non-boolean-root", integer(), INT, "non_boolean_clause_root", clause_root=True)
    bad("collection-bound", expr("collection_literal", value_type={"kind": "collection", "element": BOOL, "maximum_items": 1}, items=[boolean(), boolean(False)]), {"kind": "collection", "element": BOOL, "maximum_items": 1}, "collection_bound_exceeded")
    bad("orphan-type", expr("enum_literal", enumeration="Missing", variant="value"), {"kind": "enum", "name": "Missing"}, "orphaned_type_reference")
    for fixture_id, request, code, tokens in diagnostic_cases:
        add_case(cases, f"expression-{fixture_id}", "expression", request, f"diagnostic:{code}", *tokens)

    env_errors = [
        ("duplicate-type", "duplicate_type_declaration", lambda r: r["types"].append(copy.deepcopy(r["types"][0]))),
        ("duplicate-value", "duplicate_value_declaration", lambda r: r["values"].append(copy.deepcopy(r["values"][0]))),
        ("duplicate-function", "duplicate_function_declaration", lambda r: r["functions"].append(copy.deepcopy(r["functions"][0]))),
        ("duplicate-field", "duplicate_field", lambda r: r["types"][1]["fields"].append(copy.deepcopy(r["types"][1]["fields"][0]))),
        ("duplicate-variant", "duplicate_variant", lambda r: r["types"][0]["variants"].append(copy.deepcopy(r["types"][0]["variants"][0]))),
        ("duplicate-parameter", "duplicate_parameter", lambda r: r["functions"][0]["parameters"].append(copy.deepcopy(r["functions"][0]["parameters"][0]))),
        ("empty-enum", "empty_enum", lambda r: r["types"][0].update(variants=[])),
        ("invalid-numeric", "invalid_numeric_bounds", lambda r: r["values"][0]["value_type"].update(minimum=2, maximum=1)),
        ("unbounded-collection", "unbounded_collection", lambda r: r["values"][3]["value_type"].update(maximum_items=0)),
        ("recursive-type", "recursive_type", lambda r: r.update(types=[{"kind": "record", "name": "Cycle", "source": span(1, "expression"), "fields": [{"name": "next", "value_type": {"kind": "option", "value": {"kind": "record", "name": "Cycle"}}, "source": span(2, "expression")}]}], values=[], functions=[])),
    ]
    for fixture_id, code, mutate in env_errors:
        request = expression_input(boolean(), BOOL); mutate(request)
        boundaries = {
            "invalid-numeric": ["boundary:integer.out_of_range"],
            "unbounded-collection": ["boundary:collection.minimum"],
        }.get(fixture_id, [])
        add_case(cases, f"expression-{fixture_id}", "expression", request, f"diagnostic:{code}", *boundaries)

    obligation_inputs = [
        ("non-zero-divisor", numeric(integer(10), value("divisor"), "divide"), INT, "non_zero_divisor"),
        ("option-presence", expr("unwrap", option=value("maybe", "pre")), INT, "option_presence"),
        ("index-in-bounds", expr("index", collection=value("items", "pre"), index=value("position")), INT, "index_in_bounds"),
        ("checked-range", numeric(value("divisor"), integer(1), "add"), INT, "checked_range"),
    ]
    for fixture_id, expression, expected, obligation in obligation_inputs:
        add_case(cases, f"expression-obligation-{fixture_id}", "expression", expression_input(expression, expected),
                 "diagnostic:potentially_undefined", f"obligation:{obligation}")

    # Exact resource and numeric edges. These are generated because checking in
    # hand-expanded million-character and ten-thousand-entry JSON is error prone.
    edge = expression_input(boolean(), BOOL)
    edge["values"][0]["value_type"] = {"kind": "integer", "domain": "signed", "minimum": -(2**63), "maximum": 2**63 - 1, "overflow": "reject"}
    edge["values"][3]["value_type"] = {"kind": "collection", "element": BOOL, "maximum_items": 2**32 - 1}
    edge["values"].append({"name": "wide_rational", "kind": "input", "value_type": {"kind": "rational", "numerator_minimum": -1, "numerator_maximum": 1, "maximum_denominator": 2**63 - 1}, "source": span(14, "expression")})
    add_case(cases, "expression-numeric-edges", "expression", edge,
             "boundary:integer.minimum", "boundary:integer.maximum",
             "boundary:collection.declared_maximum", "boundary:rational.maximum_denominator")

    zero_denominator = expression_input(boolean(), BOOL)
    zero_denominator["values"][0]["value_type"] = {"kind": "rational", "numerator_minimum": -1, "numerator_maximum": 1, "maximum_denominator": 0}
    add_case(cases, "expression-rational-zero-denominator", "expression", zero_denominator,
             "diagnostic:invalid_numeric_bounds", "boundary:rational.zero_denominator")
    declared_over = expression_input(boolean(), BOOL)
    declared_over["values"][3]["value_type"]["maximum_items"] = 2**32
    add_case(cases, "expression-declared-collection-over", "expression", declared_over,
             "diagnostic:invalid_numeric_bounds", "boundary:collection.declared_out_of_range")

    text_max = expr("text_literal", value="x" * 1_048_576)
    add_case(cases, "expression-text-maximum", "expression", expression_input(text_max, {"kind": "text"}),
             "boundary:text.maximum")
    text_over = expr("text_literal", value="x" * 1_048_577)
    add_case(cases, "expression-text-over", "expression", expression_input(text_over, {"kind": "text"}),
             "diagnostic:text_bound_exceeded", "boundary:text.over_maximum")

    expression_max = expr("collection_literal", value_type={"kind": "collection", "element": BOOL, "maximum_items": 10_000}, items=[boolean() for _ in range(9_999)])
    add_case(cases, "expression-nodes-maximum", "expression", expression_input(expression_max, {"kind": "collection", "element": BOOL, "maximum_items": 10_000}),
             "boundary:expression.nodes.maximum")
    expression_over = copy.deepcopy(expression_max)
    expression_over["items"].append(boolean())
    add_case(cases, "expression-nodes-over", "expression", expression_input(expression_over, {"kind": "collection", "element": BOOL, "maximum_items": 10_000}),
             "diagnostic:expression_too_large", "boundary:expression.nodes.over_maximum",
             "boundary:collection.maximum")

    def nested_option(depth: int) -> dict:
        result = copy.deepcopy(BOOL)
        for _ in range(depth - 1):
            result = {"kind": "option", "value": result}
        return result
    type_max = expression_input(boolean(), BOOL)
    type_max["values"] = [{"name": "deep", "kind": "input", "value_type": nested_option(256), "source": span(1, "expression")}]
    type_max["types"] = []; type_max["functions"] = []
    add_case(cases, "expression-type-depth-maximum", "expression", type_max, "boundary:type.depth.maximum")
    type_over = copy.deepcopy(type_max); type_over["values"][0]["value_type"] = nested_option(257)
    add_case(cases, "expression-type-depth-over", "expression", type_over,
             "diagnostic:semantic_input_too_large", "boundary:type.depth.over_maximum")
    embedded_type_over = expression_input(
        expr("option_none", value_type=nested_option(257)),
        nested_option(257),
        types=[], values=[], functions=[],
    )
    add_case(cases, "expression-embedded-type-depth-over", "expression", embedded_type_over,
             "diagnostic:semantic_input_too_large", "boundary:type.depth.over_maximum")

    expression_depth_max = boolean()
    for _ in range(255):
        expression_depth_max = expr("boolean_not", operand=expression_depth_max)
    add_case(cases, "expression-depth-maximum", "expression", expression_input(expression_depth_max, BOOL),
             "boundary:expression.depth.maximum")
    expression_depth_over = expr("boolean_not", operand=expression_depth_max)
    add_case(cases, "expression-depth-over", "expression", expression_input(expression_depth_over, BOOL),
             "diagnostic:semantic_input_too_large", "boundary:expression.depth.over_maximum")

    def empty_function(index: int) -> dict:
        return {"name": f"f{index}", "parameters": [], "result_type": copy.deepcopy(BOOL), "source": span(index % 100, "expression")}
    def empty_value(index: int) -> dict:
        return {"name": f"v{index}", "kind": "input", "value_type": copy.deepcopy(BOOL), "source": span(index % 100, "expression")}
    semantic_max = expression_input(boolean(), BOOL, types=[], functions=[empty_function(i) for i in range(10_000)], values=[empty_value(i) for i in range(2_499)])
    add_case(cases, "expression-semantic-nodes-maximum", "expression", semantic_max,
             "boundary:semantic.nodes.maximum", "boundary:semantic_collection.maximum")
    semantic_over = copy.deepcopy(semantic_max); semantic_over["values"].append(empty_value(2_499))
    add_case(cases, "expression-semantic-nodes-over", "expression", semantic_over,
             "diagnostic:semantic_input_too_large", "boundary:semantic.nodes.over_maximum")
    collection_over = expression_input(boolean(), BOOL, types=[], values=[], functions=[empty_function(i) for i in range(10_001)])
    add_case(cases, "expression-semantic-collection-over", "expression", collection_over,
             "diagnostic:semantic_input_too_large", "boundary:semantic_collection.over_maximum",
             "boundary:collection.over_maximum")

    migration = {"package": package(), "target_version": {"major": 1, "minor": 1}}
    migration["package"]["schema_version"] = {"major": 1, "minor": 0}
    add_case(cases, "migration-valid", "migration", migration,
             "construct:migration.reference_body_1_0_to_1_1", "boundary:schema.1_0")
    unregistered = copy.deepcopy(migration); unregistered["package"]["schema_version"] = {"major": 1, "minor": 2}
    add_case(cases, "migration-unregistered", "migration", unregistered,
             "diagnostic:unregistered_migration", "boundary:schema.unregistered_minor")
    unsupported = copy.deepcopy(migration); unsupported["package"]["schema_version"] = {"major": 2, "minor": 0}
    add_case(cases, "migration-unsupported", "migration", unsupported,
             "diagnostic:unsupported_schema_version", "boundary:schema.unknown_major")

    coverage_package = package(2)
    requirement_digest = "534e1e3e27345bd9a9fc7a9723793b76b9dfc6f3b35a43c23ba811bf0ef39046"
    def trace(artifact: str, target: dict, at: int, deep: str | None = None) -> dict:
        depth = {"kind": "shallow"} if deep is None else {"kind": "deep", "requirement_digest": deep, "digest_span": span(at + 1, "trace")}
        return {"artifact_id": artifact, "source": span(at, "trace"), "target": target, "target_span": span(at + 1, "trace"), "depth": depth}
    req_a = copy.deepcopy(OWNER)
    req_b = {"package": "agent-ix/conformance", "requirement": "REQ_1", "revision": 1}
    traces = [
        trace("a_shallow", req_a, 1), trace("b_deep", req_a, 3, requirement_digest),
        trace("c_cross", {**req_a, "package": "agent-ix/other"}, 5),
        trace("d_missing", {**req_a, "requirement": "REQ_missing"}, 7),
        trace("e_stale", {**req_a, "revision": 9}, 9),
        trace("f_digest", req_a, 11, "0" * 64),
        trace("g_duplicate", req_b, 13), trace("g_duplicate", req_b, 15),
    ]
    add_case(cases, "coverage-complete", "coverage", {"package": coverage_package, "traces": traces},
             "construct:artifact.depth.shallow", "construct:artifact.depth.deep",
             "construct:coverage.class.shallow", "construct:coverage.class.deep",
             "construct:coverage.class.uncovered", "construct:coverage.class.orphaned",
             "diagnostic:cross_package_reference", "diagnostic:orphaned_requirement_reference",
             "diagnostic:stale_requirement_revision", "diagnostic:stale_trace_digest",
             "diagnostic:duplicate_artifact_trace", "boundary:artifact.cross_package",
             "boundary:artifact.missing", "boundary:artifact.stale", "boundary:artifact.digest_mismatch",
             "boundary:artifact.duplicate")
    return cases


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def placeholder(operation: str) -> dict:
    if operation in ("package", "expression"):
        return {"valid": False, "diagnostics": [], "canonical": [], "dependencies": []}
    if operation == "migration":
        return {"valid": False, "diagnostics": [], "canonical": [], "migration_receipt": None}
    return {"diagnostics": [], "coverage": None}


def main() -> None:
    cases = build_cases()
    for directory in (CORPUS / "inputs", CORPUS / "expectations", CORPUS / "canonical"):
        if directory.exists():
            shutil.rmtree(directory)
        directory.mkdir(parents=True)
    (CORPUS / "schemas").mkdir(parents=True, exist_ok=True)
    for name in ("contract-package-reference-v1.schema.json", "contract-conformance-manifest-v1.schema.json"):
        shutil.copyfile(ROOT / "schemas" / name, CORPUS / "schemas" / name)

    fixtures = []
    for case in cases:
        input_path = CORPUS / "inputs" / f"{case['id']}.json"
        expectation_path = CORPUS / "expectations" / f"{case['id']}.json"
        write_json(input_path, case["value"])
        write_json(expectation_path, placeholder(case["operation"]))
        fixtures.append({
            "id": case["id"], "operation": case["operation"],
            "input": f"inputs/{case['id']}.json",
            "expectation": f"expectations/{case['id']}.json",
            "covers": case["covers"],
        })

    inventory = json.loads((CORPUS / "inventory.json").read_text())
    manifest = {
        "corpus_id": "contract-v0.1",
        "package_schema": {"path": "schemas/contract-package-reference-v1.schema.json", "sha256": digest(CORPUS / "schemas/contract-package-reference-v1.schema.json")},
        "conformance_schema": {"path": "schemas/contract-conformance-manifest-v1.schema.json", "sha256": digest(CORPUS / "schemas/contract-conformance-manifest-v1.schema.json")},
        "inventory": {"path": "inventory.json", "sha256": digest(CORPUS / "inventory.json")},
        "canonical_profile": "quire.contract.canonical-json/v1",
        "protocol": "quire.contract.conformance-jsonl/v1",
        "fixtures": fixtures,
    }
    write_json(CORPUS / "manifest.json", manifest)
    completed = subprocess.run([str(RUNNER), "run", "--manifest", str(CORPUS / "manifest.json")], capture_output=True, check=False)
    if completed.returncode not in (0, 1):
        raise SystemExit(completed.stderr.decode())
    rows = [json.loads(line) for line in completed.stdout.splitlines()]
    if len(rows) != len(fixtures):
        raise SystemExit(f"runner returned {len(rows)} rows for {len(fixtures)} fixtures")
    for row in rows:
        actual = row["actual"]
        for index, canonical in enumerate(actual.get("canonical", [])):
            data = canonical.pop("bytes").encode()
            relative = f"canonical/{row['fixture_id']}-{index}.json"
            (CORPUS / relative).write_bytes(data)
            canonical["bytes_path"] = relative
        write_json(CORPUS / "expectations" / f"{row['fixture_id']}.json", actual)
    for path in sorted(CORPUS.rglob("*")):
        if path.is_file() and path.name != "README.md" and path.suffix != ".sha256":
            path.with_name(path.name + ".sha256").write_text(f"{digest(path)}  {path.name}\n")
    (ROOT / "schemas" / "contract-package-reference-v1.schema.json.sha256").write_text(
        f"{digest(ROOT / 'schemas' / 'contract-package-reference-v1.schema.json')}  contract-package-reference-v1.schema.json\n")
    (ROOT / "schemas" / "contract-conformance-manifest-v1.schema.json.sha256").write_text(
        f"{digest(ROOT / 'schemas' / 'contract-conformance-manifest-v1.schema.json')}  contract-conformance-manifest-v1.schema.json\n")


if __name__ == "__main__":
    main()
