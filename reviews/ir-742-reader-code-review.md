---
id: SR-11630
title: code-review and rust-review of IR-742 IEEE reader admission
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; branch code/ir742-reader-admission; diff origin/main... (.., commits and );
  crates/quire-contract-model/src/checked_package/v2/operations.rs, tests/it/checked_package_v2_ieee_admission.rs, tests/it/main.rs
review_set: subset
---

## Summary

Ticket: IR-742. Method: code-review with the rust-review lane folded in. PR: none yet.
Authority: spec PR #341  FR-038 normative bullet "IEEE literal type and
catalog result", FR-038-AC-222 and FR-038-AC-223, the published
`quire.checked-operation-catalog/v1` entries for `quire.op.ieee.to_float32` and
`quire.op.ieee.to_float64` (result `member`, member `type_argument`, constraint
`member_family` `float32` / `float64`), and the repository CLAUDE.md.

The branch adds `check_ieee_member_result` after `check_operands` and relies on
main's `literal_defect` (IR-484, #351) for the literal width rule; the rebase
dropped the original `check_ieee_literals` pass from. Reviewer measured
every case the dropped pass covered with a scratch scan through the public
`CheckedPackageV2::read` over every node of the all-families fixture as the
declared literal `type` (standalone body root, `arguments/0` and `arguments/1`
of `quire.op.ieee.numeric_equal`), then re-ran the scan with the dropped pass's
"no family is a mismatch" rule emulated.

## Verdict

**FAIL (one high finding).** AC-223 is implemented as specified: the result type
is compared with `operation.member.declaration`, the member family is taken from
the entry's catalogued `member_family` constraint through `family_fits`, and
every adverse case refuses `ill_typed`/`operator-ineligible` at the application's
`/body/result_type` with the application node as locus. For non-IEEE packages
the change is inert: the function returns `None` before any work for every
other identity, and the only other code change in `operation_defect` is a
removed blank line, so refusal precedence is unchanged versus main. The
"misplaced closing braces around StructuralTypes::resolve" fix leaves that
region byte-identical to main. Rust lane: no `unwrap`/`expect`/indexing on wire
data, no new wildcard arm, typed errors only, module docs updated. The new
check does not charge the work meter: it performs one `resolve_family` walk
(visited-set bounded) per IEEE conversion application, the same uncharged
pattern as `check_inner_result` and `literal_defect`, which matches the spec's
"resource accounting unchanged". Focused gates: `cargo test --test it
checked_package_v2` 435 passed; `cargo test -p quire-contract-model --lib
checked_package::v2` 170 passed.

The failure is the rebase resolution. The dropped pass refused an IEEE literal
whose declared `type` resolves to no operand family; main's `literal_defect`
skips such a literal, so those packages are now admitted (FND-001).

Deleted-pass coverage (scratch scan ):

| Case the dropped pass refused | At HEAD |
| --- | --- |
| float32_bits / float64_bits at the opposite width, standalone body root | refused, same code/cause/pointer `/body/type`/locus |
| same, at `arguments/0` or `arguments/1` of an application | refused, same, `/body/arguments/{i}/type` |
| declared at any other familied type (boolean, integer, text, option, integer_range, pure_function, object_type, temporal formula, reference) | refused, same |
| declared `type` absent from the graph, or not a node id | refused earlier, `invalid_semantic_graph` at the same `/type` pointer, no locus (wire stage precedes; unchanged from ) |
| declared at a node with no operand family (value/literal, value/parameter, model/namespace, model/process, relation/relationship, state/snapshot, state/frame, temporal/temporal_clause, protocol/protocol_clause, claim/verification_claim, correspondence/source_locus) | **ADMITTED**, standalone and in either argument position |

## Examined scope

```yaml
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages. Retaining each value kind and changing only its declared `type` to the opposite IEEE width, while re-deriving the affected node and package identities, refuses `ill_typed`/`operator-ineligible` at that literal's `type`, including in an operand position whose catalogued family would otherwise accept the declared type.
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node. For each identity, changing `result_type` and the enclosing application's root-node `semantic_type` together to the source-width node while retaining the valid destination `type_argument`... refuses `ill_typed`/`operator-ineligible` at the application's `result_type` by the `member` result form; changing `type_argument`, `result_type` and that root-node `semantic_type` together to the source-width node likewise refuses there by the catalogued `member_family` constraint.
- id: FR-038/IEEE-literal-type-and-catalog-result
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Independently of an enclosing operand position, a `float32_bits` literal requires a declared `type` in the `float32` family, and a `float64_bits` literal requires one in the `float64` family. A mismatch refuses `ill_typed`/`operator-ineligible` at that literal's `type`.
- id: operations.rs/check_ieee_member_result
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: fn check_ieee_member_result(application, operation, entry, graph, catalog) -> Option<ValidationFailure>
- id: operations.rs/literal_defect
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: 'let Some(family) = resolve_family(&type_id, graph.nodes, graph.kinds, graph.index) else { continue; };'
- id: tests/it/checked_package_v2_ieee_admission.rs
  path: tests/it/checked_package_v2_ieee_admission.rs
  role: examined
  excerpt: tc_048_ieee_bits_literal_requires_its_own_declared_family, tc_048_ieee_literal_width_refuses_before_an_accepting_operand_family, tc_048_cross_width_ieee_result_is_the_type_argument_of_the_catalogued_family
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Rebase dropped coverage: an IEEE literal whose declared `type` names a node with no operand family (for example a model namespace, value parameter, value literal or state frame) is admitted, standalone and as an argument, because `literal_defect` skips a literal whose type resolves to no family. The dropped `check_ieee_literals` refused it (`actual != Some(expected)`), and the FR-038 normative bullet requires a declared type in the `float32`/`float64` family. Reproduced through `CheckedPackageV2::read`; emulating the dropped rule refuses every such case `ill_typed`/`operator-ineligible` at the literal `type` with the containing node as locus. | crates/quire-contract-model/src/checked_package/v2/operations.rs:353-356 |
| FND-002 | low | Applicability is gated on the two identity names plus the entry's `result`/`member` shape and returns `None` (fails open) if the catalogued shape ever changes, while a missing `member_family` constraint fails closed with a package refusal; the doc says the family does not come from an operation name, but which entries are checked does. Correct for the current catalog; inconsistent failure direction for a catalog drift. | crates/quire-contract-model/src/checked_package/v2/operations.rs:524-546 |

## Dispositions

Round 1, verified (`git diff..`).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | `literal_defect` now keeps `continue` only for non-IEEE kinds with no resolved family; IEEE kinds compare `family == Some("float32"/"float64")`, so a familyless declared type refuses. Re-ran the reproduction through `CheckedPackageV2::read`: all 66 cases (11 familyless node kinds x float32_bits/float64_bits x standalone, arguments/0, arguments/1) refuse `ill_typed`/`operator-ineligible` at the literal `type` with the containing node as locus; only same-width types admit. An integer literal at the same familyless nodes still admits, as before, and the "refuse familyless literals of every kind" mutant is killed by the existing `checked_package_v2` suite, so non-IEEE behaviour is unchanged and pinned. Mutant "restore the old continue" is killed. |
| FND-002 | fixed | Catalog-shape drift now refuses at `result_type` (fail closed), the same direction as a missing `member_family` constraint. The "fail open on drift" mutant survives as an equivalent mutant: the catalog is compiled in from `quire-verification-contracts` and its two IEEE entries have result `member` and member `type_argument`, so the branch is unreachable through the public reader. The in-code comment stating the invariant is an acceptable disposition. |
