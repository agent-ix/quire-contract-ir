---
id: SR-2106
title: "Criterion strength (manual, Jev unavailable) \u2014 IR-661 FR-038-AC-165..171"
type: SpecReview
analysis: criterion-strength
scope: agent-ix/quire-contract-ir@705cef3f7a07370f111f19e8e7d86e1e07fd31d7; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md; base 1540b3b6c0e4d167fe1ed9116c296e45e7dff258; Ticket IR-661
review_set: subset
---

## Summary

MANUAL reviewer judgement. The Jev client is unavailable: no `jev` executable, and no Jev subcommand in quoin 0.28.1. No calibrated `weakness_kind`, confidence score or `adverse_case_coverage` score was computed, and none is claimed. `criterion-strength` is present in the installed `SpecReview.analysis` enum. Each of the seven new criteria was judged by hand on whether it can fail. FR-level adverse-case coverage (manual, uncalibrated) is broad for ends, roles, direction and multiplicity, and absent for `reaches`, subtype receivers and malformed non-end members.

Ticket: IR-661. Method: `spec-review/spec-criterion-strength-analysis`. Reviewer model `claude-opus-5-5`, run `236a8098-415e-43ce-aa4b-9a892aa7e251`.

## Verdict

**CONDITIONAL**: two `medium` and one `low` finding, all from manual judgement. Clean, as examined (they can fail as worded): AC-167, AC-168, AC-170 and AC-171.

## Scope Examined

- `FR-038-AC-165` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387
- `FR-038-AC-166` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3388
- `FR-038-AC-167` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3389
- `FR-038-AC-168` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3390
- `FR-038-AC-169` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3392
- `FR-038-AC-170` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3393
- `FR-038-AC-171` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3394
- `FR-038-AC-155` (context_only) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3379
- `FR-038#object-node-declaration` (examined) spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1021-1027
- `TC-048#fcd-step-5-object-node` (examined) spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md:1006-1014

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-169 names no observable outcome for an object-node declaration | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3392 |
| FND-002 | medium | AC-166 'ends retain type, multiplicity and roles' has no surface on which retention is observed | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3388 |
| FND-003 | low | AC-165's 'cannot back model/object_type' restates AC-155 with no code or locus | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:3387 |

## Finding Detail

- **FND-001** (medium, confidence high, check `untestable-ac`, unit `FR-038-AC-169`): AC-169 says `declaration` "never substitutes a source or target object-type declaration" but names no code, cause or pointer for that case. Any refusal satisfies it, including the `operator-ineligible` that the FR-322-AC-30 precedent implies. The `missing-selection` at `member.declaration` that FR-038 and TC-048 step 5 require is not in the criterion. (Manual weakness_kind: unmeasurable outcome; not Jev-calibrated.)
- **FND-002** (medium, confidence medium, check `untestable-ac`, unit `FR-038-AC-166`): "Both ends retain their declared type and multiplicity and their authored role strings" names no surface on which retention can be observed. FR-038 adds no accessor for relationship declarations, so the clause can be checked only indirectly through navigation (AC-169, AC-170) and adds no failure condition of its own. (Manual weakness_kind: unfalsifiable as worded; not Jev-calibrated.)
- **FND-003** (low, confidence high, check `other`, unit `FR-038-AC-165`): "A relationship backs `relation`/`relationship` and cannot back `model`/`object_type`" restates existing FR-038-AC-155 ("a relationship cannot back `model`/`object_type`") without adding a code or locus, so it adds no discriminating power. (Manual weakness_kind: restates_requirement; not Jev-calibrated.)

## Dispositions

Round 1, reviewed at `agent-ix/quire-contract-ir@c50050da5f20d4af41ab2dd6ea573d5e4b9abacc` (prior review `705cef3f7a07370f111f19e8e7d86e1e07fd31d7`), reviewer model `claude-opus-5-5`, run `ae1a4470-06bc-44c7-90df-81c70be53842`. Every outcome was verified against the fix commit's own text, not the author's receipt. All new criteria and procedures remain PLANNED / UNRUN; no implementation, CI gate or procedure coverage is claimed.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | AC-169 now names the observable outcome: `ill_typed`/`operator-ineligible` at `member.declaration`, after owner recovery and before role lookup, without invalidating the object owner. (Manual judgement; Jev still unavailable.) |
| FND-002 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | The unobservable retention clause is replaced by observable mutations: renaming a role makes the old name `missing-name` and the new name resolve, and destination changes are observed through AC-170 result nodes. No accessor is invented. (Manual judgement; Jev still unavailable.) |
| FND-003 | fixed c50050da5f20d4af41ab2dd6ea573d5e4b9abacc | The clause duplicating AC-155 is removed from AC-165, and AC-155 keeps that obligation. (Manual judgement; Jev still unavailable.) |
