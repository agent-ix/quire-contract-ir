---
id: SR-11660
title: IR-803 base spec review of the typed composite operator on the composite operand accessor
type: SpecReview
analysis: base
scope: agent-ix/quire-contract-ir (PR 407, diff vs origin/main ); spec/checked_package/functional/FR-038-consume-checked-package-v2.md,
  spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md
review_set: subset
---

## Summary

Ticket: IR-803. Base checklist over the three-file spec diff that adds a typed `operator: CheckedCompositeOperator` (`Eq`/`Ne`) to `CheckedCompositeOperands`, the new FR-038-AC-246 (PLANNED/UNRUN) and its TC-048 step. Every claim in the new text was checked against the implemented accessor `composite_application_operands` (crates/quire-contract-model/src/checked_package/v2/composite_operands.rs:351-492) and its tests (tests/it/checked_package_v2_composite_operands.rs). Review-only; no edits, no builds.

## Verdict

**CONDITIONAL**: one medium contradiction in the new eligibility sentence, two low wording defects.

What holds:

- The accessor resolves the application through `Graph::node`, which charges exactly one unit (composite_operands.rs:289-299, 366-370), and then reads `/operation/identity` from the already-resolved body with no charge (382-389). So "already resolves the application node under the graph lookup's one-unit work charge before reading its operation identity" is true, and "selecting the typed operator... shall add no work charge" can be met by mapping the string already matched at 390-393. Existing tests already pin the exact eq and ne work (`1 + 2 * expected_per_operand`, tests/it/checked_package_v2_composite_operands.rs:574-576) and ne == eq work (518), so any extra charge would fail an existing test.
- For catalogued identities, only `quire.op.structural.eq` and `quire.op.structural.ne` pass the `matches!` at 390-397; every other catalogued identity, including AC-199's reader-admitted `boolean.and` target, returns `IneligibleOperator`. No other success is possible.
- FR-038-AC-246 is falsifiable: a constant `Eq`, a swapped mapping, or an operator-dependent change to operands or `consumed_work` each fails it. It matches the TC-048 step, which builds both applications with the same ordered children, so the equalities are achievable. The id is unique (`git log --all -S FR-038-AC-246` finds only; PR 407 is the only open PR). The PLANNED/UNRUN marker is honest: no test is tagged, `quire matrix` shows the row `untagged`.
- AC-163, AC-177 through AC-182 and AC-199 are byte-identical to origin/main.
- No CG or QSL dependency in the new text ("the consumer maps these to its own equality operator vocabulary" names no crate). No hash, digest or pin.
- Gates: `quire validate` 876/876 docs grammar-clean. `quire coverage --strict` 49 unbacked rows at and 50; the only new one is FR-038-AC-246 (planned, expected). FR-038 row count 236 to 237.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "Every other operation identity shall refuse `IneligibleOperator`" is unscoped. It contradicts the same FR's error table, which gives `UnknownOperator` for a "Missing or unknown catalog identity" (FR-038:3316), the check order "node, application, occurrence and operation in that order" (3336), and the code, which returns `UnknownOperator` for an absent or uncatalogued identity (composite_operands.rs:382-389) and `UnknownNode`/`NotApplication`/`MissingOccurrence` first. Failure: a test author following 3126 asserts `IneligibleOperator` for a crate-internally mutated uncatalogued identity (the AC-181 twelve-variant probe) or for a non-eq application with an absent occurrence, and the test fails against correct code; or an implementer collapses `UnknownOperator` into `IneligibleOperator` to satisfy the sentence. Scope it: "Every other catalogued operation identity, on an application that passes the node, application and occurrence checks, shall refuse `IneligibleOperator`". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3126-3127,3316,3336 |
| FND-002 | low | The Outputs bullet drops "planned" and adds "returning the admitted comparison operator" in one unqualified sentence. The accessor is implemented (AC-177 to AC-182), but the operator is PLANNED/UNRUN (AC-246) and `CheckedCompositeOperands` has no `operator` field at this commit (composite_operands.rs:24-33). A reader of the Outputs list cannot tell which half exists. The edited line also points at a section titled "Typed authored domains of composite equality operands", but the section is titled "... composite equality and inequality operands" (3090). Mark the operator as planned (IR-803, AC-246) until the code lands, and fix the section title. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:130-133,3090 |
| FND-003 | low | The TC-048 step says "Assert the consumer reads no operation identity from the body and performs no second accessor lookup". An IR integration test has no consumer whose reads it can observe, so the step cannot fail. AC-246's "The consumer needs no separate operation lookup" is already entailed by the result carrying `operator`. Restate the step as something an IR test can check, for example: the external exhaustive-destructuring fixture reads `operator` from the result without body JSON. | spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:782-783 |

## Coverage

Examined: the FR-038 Outputs bullet (130-133), the public-type table and the new prose (3105-3132), the error table and check order (3305-3340), AC-163, AC-177 to AC-182, AC-199 and AC-246, the TC-048 section (764-784) and both tests.md rows. Six coverage rules: AC-246 has a TC (TC-048), the TC row in tests.md lists it, and both tests.md rows carry its planned status. No hash/pin use.

## Examined units

```yaml
scope:
- id: FR-038-AC-246
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "PLANNED/UNRUN (IR-803). For a reader-admitted `quire.op.structural.eq` application, `composite_application_operands` returns `CheckedCompositeOperator::Eq` together with its operands; for an otherwise equivalent reader-admitted `quire.op.structural.ne` application it returns `CheckedCompositeOperator::Ne` together with its operands. Changing the operation identity between those two admitted cases changes the returned operator and no other operator-dependent projection field:..."
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Every other operation identity shall refuse `IneligibleOperator`, so no other successful operator value is possible. The operator shall be returned together with the operands; the consumer shall not need a separate operation lookup."
- id: FR-038-AC-177
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Implemented by `composite_application_operands`; verified by the TC-048 composite operand tests. An admitted structural.eq over supported parameter references returns the actual application and supplied occurrence with two typed operands in argument order, ordinals 0 and 1."
- id: FR-038-AC-181
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "The public IneligibleOperator assertion uses a reader-admitted `boolean.and` application outside eq/ne (AC-199). Every one of the twelve specified error variants exposes exactly its tabled payload and no partial result or panic."
- id: FR-038-AC-182
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "An external consumer calls the named public accessor and exhaustively matches all named output, descriptor, edge, key and error types using the specified fields without body JSON or a QSL model dependency."
- id: FR-038-AC-199
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: "Implemented (IR-690); TC-048 checks MissingOccurrence with the supplied application and occurrence, exact refusal variants... and a reader-admitted `boolean.and` IneligibleOperator target."
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: "For AC246, admit otherwise equivalent structural.eq and structural.ne applications through the public reader, with the same ordered operand children and an authentic occurrence in each. Call `composite_application_operands` once per application. Assert `operator` is `CheckedCompositeOperator::Eq` for eq and `CheckedCompositeOperator::Ne` for ne..."
bindings: []
```

## Dispositions

Round 1; reviewed (diff..).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |

## Disposition round 1 verdict

**PASS**.

- FND-001: the sentence is now "For a catalogued operation identity other than `quire.op.structural.eq` or `quire.op.structural.ne`, on an application that passes the node, application and occurrence checks, the accessor shall refuse `IneligibleOperator`. The existing refusal order and variants are unchanged". This matches composite_operands.rs:366-397 and the error table. `UnknownOperator` (FR-038:3318) is unchanged.
- FND-002: the bullet marks the accessor implemented (AC-177 to AC-182) and the operator PLANNED/UNRUN under AC-246. It cites the real heading "Typed authored domains of composite equality and inequality operands" (FR-038:3092).
- FND-003: the TC-048 step now asserts on things the IR fixture itself does: it reads `operator` from the result, reads no body JSON for the identity, and matches the variants exhaustively.

Re-checked:
- AC-163, AC-177 to AC-182 and AC-199 are byte-identical to.
- `quire validate`: 876/876 grammar-clean.
- `quire coverage --strict`: 50 unbacked rows. The only new one is AC-246, which is planned.
- No new findings.
