---
id: SR-662
title: "spec review of PR 240 (IR-486 FR-038 operation leaves)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir@5dddf2eaea8b4115ca611c915318bcd48d394d4e; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md"
review_set: base
---
# SR-662: spec review of PR 240

## Summary

Ticket: IR-486. The PR rewrites the "Operation leaves" prose of FR-038. The known gap (extra leaves and unrelated paths admitted) is removed. The exact-leaf rule, the refusal order and pointers, the no-source rule, and a note on what leaf-mode checking covers are added. FR-038-AC-44 is added, along with a TC-048 section and two matrix rows. The float32/float64 note stays. The rule text matches QSpec FR-322 lines 174-191 (untrusted external text, quire-specification main) and the reference `classify` order. The refusal pointers are IR's own choice, since the reference names none. The TC-048 section mirrors the AC clause for clause. The section was renamed from "Operation leaves" to "Operation leaf count" for AC-43, and nothing else in TC-048 changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The prose does not list three remaining deviations from QSpec. (1) A text leaf whose type pins no `text_profile` is admitted; the reference refuses it `operator-ineligible`. (2) A leaf with no `mode`, or a mode of another kind, is admitted; the reference refuses `operation-mode-mismatch`. "The mode kind vocabulary ... not checked" does not clearly cover an absent mode. (3) A compared type that does not resolve (an untyped first operand), or a non-set `result_inner` result, skips the leaf comparison. The reference refuses the first `operator-ineligible` and expects `[]` for the second. A reader of FR-038 would take the leaf rule as complete apart from float leaves and deep mode pins | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:494-518 |
| FND-002 | low | "Deriving them costs the supplied leaves times the nesting depth" understates the cost. The derivation visits every child of each composite on a leaf's path, including zero-count siblings, so the cost is leaves times depth times width. The visits are charged to the meter, so only the wording is wrong | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:506-508 |

## Verdict

The spec change is sound and honest about the float and deep-mode limits, but incomplete about the rest. FND-001: either close (3), which is SR-660 FND-001 and FND-002, or list it, and list (1) and (2) as stated limits next to the float32/float64 note, so that the FR states every place this reader admits what the reference refuses. FND-002 is a one-line wording fix. The matrix rows and TC-048 section are correct.

## Dispositions

Reviewed at agent-ix/quire-contract-ir@b2ba6c53bfb89f7a7bff9be7890ad92662452764.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 489d6a8 |
| FND-002 | fixed | 489d6a8 |
