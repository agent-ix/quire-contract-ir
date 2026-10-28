---
id: SR-612124
title: "IR-1087 PR #652 failure-domain review"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-contract-ir; spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-047-complete-v1-contract-package-output.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md"
review_set: subset
---

## Summary

IR-1087 PR #652 at 2df5c36ad6518438a95cfbcd57009b523f0491d6, base b712988b. Examined FR-035-AC-23..26, FR-038-AC-341, unchanged FR-035-AC-22 and TC-047/048 plus FR-019 inventory. Spec-only diff; no code or aggregate gate.

## Verdict

CONDITIONAL: authenticated source matching omits represented non-operation nodes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The authenticated handoff compares source identity/lock and every represented operation node, but not represented non-operation nodes. An internally canonical V1 package can change a represented non-operation body, recompute its V1 identity, retain the same V2 source identity/lock, and pass the stated operation-only match while violating AC-22 source-term equality and source/read-back association. Require source correspondence for every represented node (or an equivalent complete commitment), with a TC-047 non-operation tamper refusal control. | spec/checked_package/functional/FR-035-complete-v1-contract-package-lowering.md:150-169,252 |

## Coverage

All changed files and the unchanged AC-22 equality requirement examined. `quire validate --scope .` on the five changed files: 5/5 grammar-clean. `quoin advise --json` checked all five new AC methods; no mismatch. No backing test tags were added in this spec-only diff; the new criteria are marked Planned/UNRUN. The full computed matrix was not run for this scoped review. Plan completion: not assessed.

## Dispositions

Round 1, reviewed at ac26f2154a6a77f9010a176ce0977f59642258d2.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Authenticated handoff now compares every represented node, including non-operation nodes, and TC-047 includes a non-operation body tamper refusal control. |
