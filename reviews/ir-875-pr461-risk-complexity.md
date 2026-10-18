---
id: SR-12590
title: "IR-875 PR 461 risk-complexity review"
type: SpecReview
analysis: risk-complexity
scope: "agent-ix/quire-contract-ir; spec/checked_package/non-functional/NFR-006-v2-producer-memory.md; spec/checked_package/matrix/TC-455-v2-producer-memory-and-identity.md; spec/spec.md; external /tmp/ir875-probe"
review_set: subset
---

## Summary

Diff-scoped review of IR-875 PR 461 at 7c8e574026448c75a040bba7abf146872e0e72d2. Technical risk is high because the requirement includes a hard measured memory bound and byte identity preservation. Volatility is moderate across the QSL producer adoption; the same-host rebaseline, three-run median, byte oracle, and staged reduction target mitigate the risk.

## Examined units

- **NFR-006 Statement** (`examined`): producer borrowing, integrated no-regression memory, and unchanged canonical bytes and package identity.
- **NFR-006-M-1** (`examined`): same-host three-run incremental peak RSS comparison, with reduction target marked PLANNED.
- **NFR-006-M-2** (`examined`): no concurrent second owned copy per projected graph-node body.
- **NFR-006-M-3** (`examined`): exact byte and typed package identity equality.
- **TC-455** (`examined`): fixed 12,500-term source, fresh-process sampling, byte and identity oracle, V2 reader checks, nested body and refusal controls.
- **spec/spec.md index** (`examined`): links NFR-006 and TC-455 in the checked-package section.
- **FR-051 and FR-038** (`context_only`): typed envelope encoding and existing V2 reader boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS. Technical risk is high because the requirement includes a hard measured memory bound and byte identity preservation. Volatility is moderate across the QSL producer adoption; the same-host rebaseline, three-run median, byte oracle, and staged reduction target mitigate the risk.

## Verification

Targeted Quire validation is 3/3 grammar-clean and the exact-parent matrix comparison is 568 to 571 rows, adding only NFR-006-M-1 through NFR-006-M-3 with no changed prior row. The external public-API probe rerun is recorded with the reviewer handoff.

Independent offline release probe, built with its own target and two Cargo jobs under the shared lock against clean QSL checkout 77ab19b, produced 50,138 source bytes, 12,505 nodes and 40,631,408 bytes in each of three fresh processes. All three outputs match one another and the earlier control byte-for-byte; typed package_id is ecc79a978fb59047c27e588cb17ccce1d0385eaa8427817cc4591fe1e0dc079a in every run. Sampled RSS increments were 337,620, 337,388 and 337,416 KiB (median 337,416; range 232; fresh bound 337,648 KiB). This is a new baseline sample, not an optimized candidate result. The reduction target is still PLANNED.
