---
id: SR-2026110414560077
title: IR-1350 PR 918 spec-review-ears-conformance review
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-contract-ir; IR-1350; spec/checked_package/functional/FR-038-consume-checked-package-v2.md;
  spec/spec.md; directly related source and tests
review_set: subset
---

## Summary

Reviewed the new requirement-bearing body-rule prose and AC-472 for actor, trigger and measurable response. No changed-unit grammar or semantic EARS defect was found.

Ticket: IR-1350. Application version: 0.1.0. Scope is the frozen PR #918 diff against origin/main and the directly relevant surrounding code and requirements. This is a scoped review, not a repository-wide assurance verdict. No applicable AssuranceProfile was found in spec/. No source edits, builds, test runs, or Cargo gates were performed by this reviewer. The lead retains the single final scoped gate.

## Verdict

PASS within the stated scope. Final scoped execution remains with the lead.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Scoped Quire grammar validation of FR-038 succeeded and reported one existing ac:non-canonical-shape warning at line 5805, outside the changed units. No warning targets the new paragraph or AC-472. Manual inspection finds named subjects, explicit conditions and measurable refusal/preservation responses; separate sentences distinguish body validation, dependency validation and live-producer isolation. AC-472 is an outcome assertion, not a user-story template. This verdict excludes unrelated existing requirements and their grammar warning.

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
