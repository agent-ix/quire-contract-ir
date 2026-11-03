---
id: SR-2026110283822
title: "IR-1299 source and output frame specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-1299; spec/checked_package/functional/FR-049-admitted-scalar-result-context.md"
review_set: subset
---

## Summary

Reviewed FR-049 frame clarification at `02eb4bb6` against IR main `83864927`, FR-035, the admitted V1 reader, and the reported real-QSL separated-sibling reproduction. No applicable AssuranceProfile was found in the repository.

## Verdict

PASS at fix head `f5b40ee7`. The new source/output frame rule and AC-7 are concrete and follow the reader's authenticated structural comparison. The inherited envelope contradiction was corrected in the same branch; the six-member wire including `source_keys` now agrees with FR-035 and the reader.

## Reviewed units

- FR-049 Inputs and Admitted result-context query: examined. The typed `ContractSourceKeyV1` is derived from V2 at the lowering boundary; admitted source occurrence identity, role and ordinal are compared with the represented output entry by rebuilt structural key. The original graph is not queried after V1 admission.
- FR-049-AC-3: examined. A selected direct narrow must reference the represented expression output key; invalid conversion selection returns `IncompleteContext`.
- FR-049-AC-7: examined. Supported siblings separated by an excluded root and distinct raw V2/source/output indexes form a discriminating real-QSL control; wrong identity, role or ordinal and output reordering are adverse controls.
- FR-049-AC-1/4/6 and Dependencies: examined for envelope and disposition consistency.
- FR-035-AC-31/35 and FR-330 V1 wire: context only. They define the six-member package and independent indexed frames.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-049 still requires a five-member FR-330 package in its behavior, AC-1, AC-6 and Dependencies, while its new frame rule requires `source_keys` and FR-035-AC-31 plus the admitted `ContractPackageWireV1` define six members. A five-member output cannot satisfy the new source-frame identity test. Replace the five-member claims with six-member without changing the separate closure-node count. | spec/checked_package/functional/FR-049-admitted-scalar-result-context.md:123,201,234,239,245; FR-035-AC-31 |

## Coverage

Targeted `quire validate --scope . spec/checked_package/functional/FR-049-admitted-scalar-result-context.md --summary` passed 1/1 grammar-clean with zero findings. No Cargo or aggregate gate ran. The change adds no hash, digest, pin, copy or compatibility layer. The API design still needs the code stage to map admitted source occurrences through IR-owned reads at the lowering boundary; CG's current raw-key prefilter must consume that result.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Commit `f5b40ee7` changes all five FR-049 envelope claims at lines 123, 201, 234, 239 and 245 to the admitted six-member FR-330 package including `source_keys`; the separate six-node closure count remains unchanged. |
