---
id: SR-2026103002
title: "IR-1059 PR #725 spec-review-base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-069-admit-model-tuple-and-union-payload-declarations.md"
review_set: subset
---

## Summary

Independent spec-review-base review of PR #725 at 529c941d against QSpec FR-322 at 8872fd2d and merged IR #721/#723. The AC-2 correction, AC-6 withdrawal, and AC-8 replay obligation are consistent with structural V2 identity.

## Verdict

**PASS** — no findings in the PR diff.

## Scope

- `FR-069-AC-2` examined: Changing only a tuple position type or order changes the model-owned tuple declaration's content while its authored structural node id can remain unchanged. Two named tuples of identical shape have distinct owner-nested structural paths under the same owning package. A test compares each declared node path and owning package name as the tuple declaration identity, and compares the retained ordered positions and type references as its content.
- `FR-069-AC-6` examined: Withdrawn: this historical numbered intake and wire replay criterion included a stale tuple key refusal. No content-derived tuple key or stale-key comparison remains under QSpec FR-322-AC-160 and FR-322-AC-169 (withdrawn); the refusal has no subject. Its former `TK-01` through `TK-35` and `TW-00` through `TW-26` case numbers remain here for trace history. FR-069-AC-8 requires replay of every currently published tuple/union intake and wire case, including admitted and refused controls, without a fixed count.
- `FR-069-AC-8` examined: PLANNED/UNRUN (IR-1141). make conformance-qspec loads the current positive fixture, model-tuple-union vectors and selected document directly from QUIRE_SPECIFICATION_DIR; it replays every published tuple/union intake and wire case against production intake and reader paths, compares admission or refusal code, cause, RFC 6901 location and reader stage with the published result, and fails on an admitted partial package or panic for a malformed declaration. Missing or malformed files and omitted cases fail. No QSpec artifact is copied into IR, and no identity hash is computed.
- QSpec FR-322-AC-160 and FR-322-AC-169 (withdrawn) examined as authoritative context.
- No applicable AssuranceProfile found in the changed module.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
