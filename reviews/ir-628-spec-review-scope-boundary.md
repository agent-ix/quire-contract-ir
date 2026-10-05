---
id: SR-1561
title: "Scope-boundary review of quire-contract-ir PR #296 (IR-628 typed model object fields accessor)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-contract-ir@7ffa956c25fe491767cb790d8168e958929000e4; spec/checked_package/functional/FR-038-consume-checked-package-v2.md (paragraphs 'What the accessor guarantees, stated exactly', 'What quire-contract-codegen can then retire', open questions IR-628-Q1..Q3)"
review_set: subset
---

## Summary

Ticket: IR-628. Boundary between Contract IR (owner of the reader and accessor), quire-contract-codegen
(CG, the consumer), and IR-627 (the body re-derivation). I checked CG's text at its
`origin/main`. CG FR-015 (`spec/kani/functional/FR-015-bounded-kani-obligations.md:440-470`,
`651-655`) and FR-024 (`spec/replay/functional/FR-024-counterexample-envelope-intake.md:317-321`,
`447`) mark FR-015-AC-77..81 and FR-024-AC-31..35 "GATED on IR-627 or IR-628". FR-015-AC-81's
`TypeNotRange` covers "a field whose type is known and is no `i64` `integer_range`". The accessor
returns `None` or a non-`i64` `IntRange`, and both map onto that reason, so the two are consistent.

The PR edits no CG file. The downstream ticket is described for the lead to file and is not filed
here. CG keeps the IR-627 gate for every range read from a node body: the scalar operand, a state
field's body target, a declared `bounded_domain`. IR-628-Q1..Q3 are framed as owner questions
decided on the merits. Q3 (no accessor for `systems_interface`/`relationship`; they return
`NotModelObjectType`) is consistent with item 4. Q2 is examined in SR-1563 FND-001, and Q1 in
SR-1559 FND-002.

## Verdict

Changes requested: one medium finding. The IR/CG boundary itself is drawn correctly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The paragraph claims that the GATED criteria FR-015-AC-77..81 and FR-024-AC-31..35 "are satisfied for the model declaration path by this accessor alone". As CG writes them, several criteria specify the route the accessor replaces. FR-015-AC-77 reads `balance` and `audit` "from the reads' `result_type`". FR-015-AC-78 defines `FieldNotRead`, `ConflictingFieldReads` and `MemberDisagreesWithRead`. FR-015-AC-81 defines `NoRead`. The PR's own downstream ticket retires all of these. The accessor lets CG amend those criteria and lift their IR-628 gate; it does not satisfy them as written. Say "lifts the gate, and CG amends AC-77, AC-78 and AC-81 to the accessor route", so that CG's lead does not read the text as an instruction to ungate unchanged criteria. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2123 |
| FND-002 | low | "What the accessor guarantees, stated exactly" says the bounds are those "`field_type` derives from the selected document". It does not say that the selected document is the one the caller's evidence supplied when the package was admitted. The accessor's answer is sound only relative to that evidence: a re-pointed `model_selections` row admitted with the re-pointed document in evidence returns that document's bounds (FR-038-AC-129). Cite the IR-627 "Trust root" paragraph here, so that "needs neither IR-627 nor evidence" is not read as "trusted without evidence". | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:2107 |


## Dispositions

Round 1, reviewed at bd47f6aa58501e5d88afff2657f634f9f2b4f382.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
| FND-002 | fixed | bd47f6aa58501e5d88afff2657f634f9f2b4f382 |
