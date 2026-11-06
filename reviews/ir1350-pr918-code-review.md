---
id: SR-2026110414560074
title: IR-1350 PR 918 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1350; crates/quire-contract-model/src/checked_package/v2/flat_wire.rs;
  crates/quire-contract-model/src/checked_package/v2/mod.rs; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/spec.md; tests/conformance_qsl_exact_producer/literal_bodies.rs; tests/conformance_qsl_exact_producer/main.rs;
  tests/it/checked_package_v2_canonical_encoding.rs; tests/it/checked_package_v2_flat_wire.rs;
  tests/it/checked_package_v2_frame_entries.rs; tests/it/checked_package_v2_ieee_admission.rs;
  tests/it/checked_package_v2_literal_bodies.rs; tests/it/checked_package_v2_lowering.rs;
  tests/it/checked_package_v2_model_fields.rs; tests/it/checked_package_v2_parameters.rs;
  tests/it/checked_package_v2_protocol_body.rs; tests/it/checked_package_v2_reader.rs;
  tests/it/checked_package_v2_refinement.rs; tests/it/checked_package_v2_require_bounds.rs;
  tests/it/checked_package_v2_union.rs; tests/it/main.rs; tests/it/support/checked_package.rs
review_set: subset
---

## Summary

Reviewed strict Literal body dispatch, dependency validation, producer isolation and every changed fixture against Rust rules. One deterministic-resource defect was found in the new live test; production literal validation and fixture migration otherwise passed inspection.

Ticket: IR-1350. Application version: 0.1.0. Scope is the frozen PR #918 diff against origin/main and the directly relevant surrounding code and requirements. This is a scoped review, not a repository-wide assurance verdict. No applicable AssuranceProfile was found in spec/. No source edits, builds, test runs, or Cargo gates were performed by this reviewer. The lead retains the single final scoped gate.

## Verdict

CONDITIONAL — one medium finding in the new live producer execution path. Final scoped execution remains with the lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Live producer can execute a different helper's shared binary between build and run | tests/conformance_qsl_exact_producer/main.rs:498 |

## Coverage

The exhaustive body_grammar mapping assigns only ValueForm::Literal to Literal. The early flat-wire check rejects nonliteral bodies before node identity work; the graph pass rejects nonempty literal dependencies at dependencies. Literal payloads continue through existing closed term validation; no public API, new panic path, unsafe operation or lowering exception was added. Error paths retain the existing optional node-context policy. The new helper intentionally differs from prepare_consumer because that helper applies a shared model patch; its source/manifest setup does not introduce a second producer or consumer contract. CI and Cargo configuration are unchanged.

```yaml
scope:
- id: FR-038-AC-18
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: For a keyed input that reaches the graph-reference stage, a self-typed
    node's own body-root `literal.type` naming itself is exempt from the reference-cycle
    check and needs no declared `recursion_group`. The same self-reference nested
    in an `aggregate` member, `binding` value or `application` argument is not exempt
    and refuses as `invalid_semantic_graph` at `/semantic_graph/nodes/<n>` (the node
    object lacking `recursion_group`), as do a self-typed node's `reference` body
    and `application.result_type` naming itself. Public admission checks the structural
    id for shape and uniqueness before that s
- id: FR-038-AC-39
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under a bounds-required profile, `x + 1` over a parameter `x` typed at
    an `integer_range` domain over `integer` lowers although the parameter's name
    literal is annotated with the unbounded `text` type, and the annotation stays
    in the closure and in `dependencies`; a parameter typed at an unbounded `integer`
    or `rational` type refuses `requires_bound` naming that type.
- id: FR-038-AC-73
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Under a bounds-required profile, a quantity position declared directly
    at a `unit` or `compound_unit` returns `requires_bound` naming that exact underlying
    type node: a record field at a `unit`, a parameter at a `compound_unit`, and `Sequence<Quantity>[0,3]`
    with an element at a `unit` each refuse. Three distinct self-typed `scalar_type`/`carried_quantity`
    positions in QSpec FR-322''s `Quantity<u, C>` form each lower: a record field
    with an admitted `integer_range` carrier, a parameter with an admitted `decimal_range`
    carrier, and a record field with an admitted `rational_range` carrier. A seco'
- id: FR-038-AC-471
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Under a bounds-required profile, a reader-admitted QSL-shaped `quantity.pow`
    application with a quantity-literal base, an exact `compound_unit` result and
    a referenced `value`/`literal` Integer exponent whose body is the canonical literal
    `2` lowers without `requires_bound` from that exponent; its lowered dependencies
    retain both the literal and Integer type keys, its checked exponent retains value
    `2`, and its checked result retains the selected compound-unit type. A reader-admitted
    variant using a `value`/`parameter` typed at plain Integer as the exponent returns
    `requires_bound` naming Inte
- id: FR-038-AC-472
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A valid typed `value`/`literal` with empty dependencies admits with its
    literal body and type intact. Replacing its body with any nonliteral term refuses
    `malformed_wire` at `/semantic_graph/nodes/{node}/body`; a literal body with nonempty
    dependencies refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{node}/dependencies`,
    with no admitted package or partial view. A genuine QSL U12 square package admits
    with its Integer literal exponent intact; changing only that exponent body's term
    to a reference or aggregate refuses at that node's `/body` before bounds-required
    lowering.
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Under QSpec FR-322 "Node identity and dependencies", a `value`/`literal`

    node SHALL have a literal body and empty `dependencies`. During strict wire

    validation, any other body term SHALL refuse `malformed_wire` at that node''s

    `/body`, before graph joins or bounds-required lowering. A literal body with

    nonempty `dependencies` SHALL refuse `invalid_semantic_graph` at that node''s

    `/dependencies` during graph validation, following the existing parameter

    dependency policy. Strict wire body validation precedes this graph rule. Neither
    refusal admits a package or partial view. A valid

    typed literal w'
- id: FR-322
  path: agent-ix/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md
  role: context_only
  excerpt: '| `value`/`literal`, `value`/`parameter` | `[]`; the body holds literals
    only |'
- id: spec/spec.md
  path: spec/spec.md
  role: examined
  excerpt: 'Its strict literal

    admission requires literal-only `value`/`literal` bodies with empty dependencies;

    nonliteral bodies refuse before bounds-required lowering.'
```

## Evidence

Inspected supplied execution logs: the initial affected-V2 sweep had 887 passes and 63 failures; the 63 original failure names exactly match the 63 replay records, all exit zero, and every current test name exists. This is a replay of original failures, not a fresh all-V2 green sweep. The separate protocol replay has 174 passes; literal-body controls 3, lowering 7, require-bounds 15, and the live U12 control 1 pass. Focused Clippy is successful with --workspace. These are supplied execution receipts, not reviewer reruns. Full workspace all-target/all-feature gates and cargo-deny were not run by this reviewer; the final scoped gate remains with the lead.

## Fixture assertion audit

The shared literal changes from an aggregate with a type dependency to a Boolean literal with an empty dependency list. Existing type reachability now comes through semantic_type and literal.type. Canonical encoding, IEEE family/width, scalar operands, parameters and refinement tests retain their original value/refusal assertions after removing only invalid literal dependencies. Flat-wire stress controls move to a general-term namespace host; union dangling-target control moves to a scalar so the missing-target oracle remains reachable. Reader dependency-cycle controls use nonliteral hosts, preserving cycle and recursion-group assertions; the meter edge moves to namespace. Bounds/claims overlap moves to namespace while list-subset assertions remain. The integer_typed fixture and forms_package use parameters for unbounded value positions; the former now reaches its integer_range through semantic_type and expected closure includes parameter annotation types.

The old Rational dependency-only value/literal lowering shape cannot pass strict admission. Its test is explicitly renamed/rebound from AC-39 to AC-472 and now asserts InvalidSemanticGraph at dependencies; no claim that this still proves dependency-only Rational lowering is made. The Boolean-literal-to-metre dependency case is likewise explicitly refused at admission; the existing compound-unit dependency and application-result controls continue to assert reached-unit versus quantity-position behavior. The self-typed root literal positive remains; nested nonliteral literal-node bodies now assert MalformedWire/body, while the unchanged private tc_048_only_a_self_typed_body_root_literal_omits_its_cycle_edge still asserts nested edges and graph refusal at the existing graph seam. These are disclosed replacements of unreachable literal-specific claims, as the ticket permits.

Protocol boolean_atoms now creates two distinct expression/binary nodes with opaque aggregate bodies. Their distinct keys preserve two independent valuation atoms, rather than replacing one with the newly constant shared Boolean literal. The existing truth-table, first-stop and exact-work assertions remain, including the separate 174-test protocol replay.

No diff touches lower.rs or canonical_referenced_integer_exponent_lowers_and_parameters_still_require_bounds. The new live helper owns its producer resolution, seeds its private Cargo.lock from the selected QSL checkout and has no IR model patch. The existing shared-model helpers remain unchanged. The fresh producer emits its original bytes; the host independently admits those bytes, asserts exponent key/form/type/value 2, then changes only its body to reference or aggregate and asserts exact MalformedWire/body refusal with unchanged envelope. The helper is independently authored integration code, not a copy of QSL source or a schema adapter.

## Changed paths examined

- `crates/quire-contract-model/src/checked_package/v2/flat_wire.rs`
- `crates/quire-contract-model/src/checked_package/v2/mod.rs`
- `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`
- `spec/spec.md`
- `tests/conformance_qsl_exact_producer/literal_bodies.rs`
- `tests/conformance_qsl_exact_producer/main.rs`
- `tests/it/checked_package_v2_canonical_encoding.rs`
- `tests/it/checked_package_v2_flat_wire.rs`
- `tests/it/checked_package_v2_frame_entries.rs`
- `tests/it/checked_package_v2_ieee_admission.rs`
- `tests/it/checked_package_v2_literal_bodies.rs`
- `tests/it/checked_package_v2_lowering.rs`
- `tests/it/checked_package_v2_model_fields.rs`
- `tests/it/checked_package_v2_parameters.rs`
- `tests/it/checked_package_v2_protocol_body.rs`
- `tests/it/checked_package_v2_reader.rs`
- `tests/it/checked_package_v2_refinement.rs`
- `tests/it/checked_package_v2_require_bounds.rs`
- `tests/it/checked_package_v2_union.rs`
- `tests/it/main.rs`
- `tests/it/support/checked_package.rs`

## Finding detail

The new test builds `--bin producer` into the shared CARGO_TARGET_DIR, releases cargo()'s flock, and then executes target/debug/producer directly. The four existing live tests use that same target and binary name. With the native test binary's default parallel threads, another helper can replace debug/producer after this build returns and before Command::new executes it, so this test consumes another source program's output. The supplied --exact filtered run cannot detect this race. Keep build-and-execution inside the existing cargo() flock (for example cargo run --workspace --offline --locked --bin producer), or give this producer an independently owned executable path/name.

Applicable rule: rust-style/testing requires deterministic resources assigned per test/process/worktree; rust-review requires inspecting concurrent-state and resource boundaries. This is a static code trace, not a reproduced parallel execution failure.
