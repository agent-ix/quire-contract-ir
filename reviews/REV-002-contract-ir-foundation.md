---
id: REV-002
title: "quire-contract-ir v0.1 composite specification review"
type: Review
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/11
    type: reviews
---
# quire-contract-ir v0.1 composite specification review

Date: 2026-08-30

| Review dimension | Result | Evidence or disposition |
|---|---|---|
| Dependency | clear | PLAN-002 orders issues #5, #6, #8, #9, and #10; only the dependency-ready child advances. |
| Risk and complexity | clear | Closed types, explicit definedness, staged canonicalization, and corpus publication isolate high-risk semantic decisions. |
| Evidence | clear | MP-001 names population, repetitions, per-case outcomes, review, gaps, identities, and limitations. |
| Integrity | clear | Wire values cannot bypass semantic validation; invalid values receive no canonical success identity or covered status. |
| Scope | clear | Runtime, code generation, solvers, Quoin, Quire, and certification decisions remain outside this crate. |
| Failure domains | clear | Malformed, unsupported, ill-typed, undefined, stale, orphaned, mismatch, and environment failures are explicit. |
| Architecture | clear | AD-001 separates wire, semantic, canonical, trace, runner, downstream, and human-decision boundaries. |
| Authority | clear | AA-001 remains open; only `@kreneskyp` can select and decide an exact source candidate. |

No implementation-blocking foundation finding remains in the authored
specification. Planned semantic criteria remain visibly planned until their
owning child issue supplies requirement-tagged implementation evidence.

## Independent Review Findings and Disposition

Three independent read-only review rounds of the issue #5 worktree reported 14
findings. Each was fixed before the candidate evidence record was minted.

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-R01 | high | New foundation inputs made the prior PGM-01 evidence record stale and no issue #5 evidence was planned. | TASK-005 now requires candidate-scoped evidence; a new revision-scoped record covers the final technical candidate. |
| FND-R02 | high | COR-001 was not consumed by the evidence verifier. | The verifier now schema-validates and authenticates corrections, requires resolvable affected records, rejects affected records, and reports enforced corrections. |
| FND-R03 | medium | COR-001 asserted an intermediate record name unavailable from reachable `main` history. | The unavailable record claim was removed; the correction is limited to the merged, permalinked `pgm-01-568bd05` record. |
| FND-R04 | medium | TC-022 claimed FR-009 without a requirement/matrix binding. | FR-009-AC-4 and both matrices now bind TC-022 to correction behavior. |
| FND-R05 | medium | The correction schema had no owning requirement or negative corpus. | FR-009 owns the schema; a manifest-complete positive/negative correction corpus is executed by the Python gate. |
| FND-R06 | low | NFR-004 verification omitted TC-022. | The verification section now includes TC-020 through TC-022. |
| FND-R07 | low | StR-003 coverage omitted its declared TC-016 verification. | TM-002 now lists TC-016, TC-018, and TC-020. |
| FND-R08 | medium | Correction enforcement and integrity failure branches lacked direct negative tests. | TC-022 now executes affected-record rejection plus checksum-mismatch, dangling-target, and malformed-record failures. |
| FND-R09 | low | The correction fixture manifest did not prove it listed every correction fixture. | The corpus test now requires exact equality between declared and discovered valid/invalid fixtures. |
| FND-R10 | low | An affected record name could contain path traversal and evade the record-name comparison. | The schema restricts affected records to `pgm-01-<seven hex>` and the loader additionally enforces a one-component safe relative name. |
| FND-R11 | high | A multiple-record assertion was unreachable inside an expected-exception block. | The assertion is restored to the unique-record test, and safe-record-name defense has a direct unit test. |
| FND-R12 | high | Draft evidence claimed its own still-pending exact-candidate review had passed. | Candidate evidence records review status as inconclusive and makes no self-certifying review claim. |
| FND-R13 | medium | Evidence and review artifacts disagreed about review-round counts. | REV-002 and retained evidence consistently record three rounds and 14 findings. |
| FND-R14 | medium | The verifier labeled `subjectRevision` but read all inputs from literal `HEAD`. | Verification now resolves and enumerates the subject revision, requires its non-evidence tree to equal current `HEAD`, and checks every digest against subject, `HEAD`, and worktree bytes. |
| FND-R15 | medium | A simultaneous worktree rewrite of COR-001 and its checksum was not compared to committed evidence bytes. | Correction JSON and checksum bytes must now match their `HEAD` blobs as well as each other. |
| FND-R16 | low | The first issue #5 record retained historical blank-at-EOF bytes that fail the PR-range whitespace check. | The record remains immutable; a path-scoped Git attribute preserves those bytes while keeping normal whitespace checking active elsewhere, and the superseding record uses canonical EOF layout. |
