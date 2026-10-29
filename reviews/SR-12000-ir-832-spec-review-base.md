---
id: SR-12000
title: "IR-832 spec review (base)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir (PR #425); spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

Ticket: IR-832. Base spec review of the new FR-038 section "Typed parameter declaration", FR-038-AC-258 through FR-038-AC-261, the TC-048 procedure, the matrix rows and the FR-019 public-items row. Each factual claim was checked against `origin/main` code and against QSL `origin/main` FR-092.

Claims confirmed against `origin/main`:

- `crates/quire-contract-model/src/checked_package/v2/scalar_operands.rs`: `CheckedScalarOperandChild` has exactly `GraphChild(CheckedNodeId)` and `InlineLiteral { application, occurrence, ordinal }`. `scalar_application_operands` returns no parameter name.
- `v2/structural.rs` `parameter_defect`: the body is `aggregate_members == [name, level]`. `name` is `binding "name"` bound to a `Text` literal typed at `scalar_type/text`, whose value is a string identifier. `level` is `binding "level"` bound to an `Integer` literal typed at `scalar_type/integer`, whose value is a JSON string accepted by `is_non_negative_integer` (`^(0|[1-9][0-9]*)$`, canonical, unbounded). Dependencies are empty. "Canonical decimal text" and "adds no integer bound" therefore match the reader.
- No `parameter_declaration` method and no `CheckedParameterDeclaration*` type exist anywhere in `crates/`.
- Sibling conventions match. The method is an inherent method on `CheckedPackageV2` taking `&CheckedNodeId` and returning `Result<_, typed enum>`. `UnknownNode` and `Not<Form>` copy `CheckedScalarOperandError::{UnknownNode, NotApplication}`. Public-field structs copy `CheckedScalarOperand`. The `MalformedBody` arm is reachable only by a crate-internal post-admission mutation. That arm is honest and reachable: `CheckedPackageV2.wire` is private, but descendant-module tests can mutate it, as FR-038-AC-163 and FR-038-AC-233 already do.
- QSL FR-092 "Parameter nodes" gives a parameter node `declaration: null`, no `owner` and no function identity. Its level is defined, and "A reference to a parameter node of level `k` denotes the value bound by the one enclosing binder at level `k`". FR-092-AC-5 has `both2` reuse P1 and P2. The no-function-identity argument (AC-260) is faithful.
- The QSL planner decisions are reflected: `level` is in the result as canonical decimal `Box<str>`, and the method is named `parameter_declaration`.
- AC ids FR-038-AC-258 through FR-038-AC-261 are unused on `origin/main`. FR-038-AC-257 was the highest. No AC was renumbered, and no other open PR exists.
- The caller-join paragraph is descriptive and carries no `shall`. It states that the IR provides no name lookup, no value store and no evaluation, so it is not an IR obligation.

## Reviewed scope

| Unit | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-038 "Typed parameter declaration" API statement | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | `CheckedPackageV2::parameter_declaration(&self, node: &CheckedNodeId) -> Result<CheckedParameterDeclaration, CheckedParameterDeclarationError>` shall return it as typed Rust values, so the consumer does not decode the package's `body`. |
| FR-038 name/level semantics paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | `name` is the value of the `"name"` binding's text literal and `level` is the canonical non-negative decimal text of the `"level"` binding's integer literal, both exactly as the reader admitted them ... |
| FR-038 no-function-identity paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | The accessor shall identify a parameter by node id alone and shall return no enclosing-function identity. |
| FR-038 refusal paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | A parameter node whose body is no longer the admitted two-binding shape, which only a crate-internal post-admission mutation can produce, shall refuse `MalformedBody`. |
| FR-038 caller-join paragraph | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Binding concrete values is the caller's join and not an IR API ... The IR provides no lookup from a name to a parameter node or an operand ordinal, no value store, and no evaluation. |
| FR-038-AC-258 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For an admitted `value`/`parameter` node, `parameter_declaration` returns `Ok` whose `node` equals the supplied id ... a level of `10` is returned as `"10"` and not read as an integer. |
| FR-038-AC-259 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | `parameter_declaration` over an id absent from the graph refuses `UnknownNode`; ... `NotParameter` ... `MalformedBody` for each. |
| FR-038-AC-260 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Two functions in one admitted package whose parameters share a name, level and type reference one `value`/`parameter` node ... The result type has no member naming a function, owner or declaration. |
| FR-038-AC-261 | examined | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | For an admitted `quire.op.integer.add` application whose two operands reference parameters `a` and `b`, calling `parameter_declaration` on each `GraphChild` id ... yields `a` then `b` ... |
| FR-038 "Parameter and compound-unit nodes" | context_only | spec/checked_package/functional/FR-038-consume-checked-package-v2.md | Its body is exactly `aggregate{[binding "name" = a `text` literal ... binding "level" = an `integer` literal ... canonical non-negative decimal string]}`, and its `dependencies` is empty. |
| TC-048 "Typed parameter declaration" | examined | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md | Build a self-built admitted package with two functions that take parameters of one name, level and type ... |
| FR-019 V2 parameter declaration row | examined | spec/model/functional/FR-019-rust-library-interface.md | `checked_package` (V2 parameter declaration) - types `CheckedParameterDeclaration`, `CheckedParameterDeclarationError` - FR-038 |
| QSL FR-092 "Parameter nodes" | context_only | quire-spec-language:spec/functional/FR-092-key-type-parameter-and-declared-nodes.md | A parameter node carries no owner and no function identity, so its key does not depend on the function that declares it ... |
| QSL FR-092-AC-5 | context_only | quire-spec-language:spec/functional/FR-092-key-type-parameter-and-declared-nodes.md | `function both2 using v(a: Boolean, b: Boolean): Boolean pure { a and b }`, declared beside `both` in the same unit, reuses P1 and P2. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `MalformedBody` does not say how much of the admitted shape the accessor re-checks. A mutation can keep two bindings and a text `name` but change the literal's `type` node, make the name a non-identifier, or make `level` non-canonical (`"010"`). One implementation would refuse; another would return the text "exactly as admitted". AC-259 tests only a missing `name` binding and a non-text name. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3156-3160 |

## Verdict

PASS WITH FINDINGS for this method. The API, its body-shape claims, the no-function-identity design and the caller-join scoping are accurate and consistent with sibling accessors and QSL FR-092. The one base finding is a low ambiguity on a post-admission-only path. See SR-12002 for the blocking FR-019 finding.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: FR-038 now lists the full re-check (an aggregate of exactly the `"name"` and `"level"` bindings, a text name typed at `scalar_type`/`text` that is an identifier, an integer level typed at `scalar_type`/`integer` in canonical non-negative decimal) and adds "shall not otherwise re-check the package". AC-259 names five body mutations, and each maps to a check that `parameter_defect` (structural.rs) makes. |
