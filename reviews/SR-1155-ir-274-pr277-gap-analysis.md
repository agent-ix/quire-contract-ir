---
id: SR-1155
title: "gap analysis of PR 277 (IR-274 part C: output-mapping identity steps metered through quire-canonical)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@fdfd4617773b27ad312cb59731164c5d3cfbea01; spec/output_mapping/functional/FR-032-admit-output-mapping-request.md, spec/output_mapping/functional/FR-033-account-for-output-obligations.md, spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md, spec/output_mapping/functional/STD-003-output-mapping-refusal-registry.md, spec/checked_package/functional/FR-038-consume-checked-package-v2.md (AC-80, AC-91), spec/output_mapping/matrix/tests.md, spec/checked_package/matrix/tests.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/tests.md, spec/core/matrix/tests.md (FR-011, FR-012 rows), crates/quire-contract-model/src/output_mapping.rs, tests/it/output_mapping.rs, tests/it/checked_package_v2_identity_digests.rs"
review_set: subset
---
# SR-1155: gap analysis of PR 277

## Summary

Ticket: IR-274 (code change C). Planless gap analysis of FR-032-AC-6,
FR-033-AC-6, FR-034-AC-6, FR-034-AC-7 and the `output_mapping.rs` part of
FR-038-AC-91. Each was checked against the code, the tagged tests and the
matrix rows the PR edits.

Per AC:

- FR-033-AC-6: implemented. The long-condition candidate admits one byte under
  the measured limit and refuses `request_limit_exceeded` at `record.identity`.
  It maps at the exact limit and at one byte over the limit. Tagged.
- FR-034-AC-6: implemented. The request and record steps go through the public
  calls (exact admits, one byte under refuses, not `allocation_failed`). The
  package step is unit-tested at the measured length and one byte lower, and
  the integration test reaches it through the public path.
- FR-038-AC-91 (`output_mapping.rs` part): implemented. The scan excludes the
  `#[cfg(test)]` module correctly and fails if `canonical_envelope_bytes`,
  `serde_json`, `json!` or the literal `u64::MAX` returns. `canonical.rs` and
  `binding.rs` stay planned for code change B, as the row says.
- FR-032-AC-6 and FR-034-AC-7: see findings.
- Matrix rows FR-033, TC-043, TC-048 and the `spec/tests.md` index: accurate.
- The FR-034 row's note that a source offset or revision above 2^53 still
  reaches the material until FR-011-AC-3 and FR-012-AC-6 bound those types
  (code change B) is accurate. Both rows already say planned in
  `spec/core/matrix/tests.md`. The refusal code it names at `record.identity`
  is unregistered, which is SR-1154 FND-002.
- No `.github`, no new constants, no pins, and no v1 numeric spelling change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-032 row marks AC-6 "implemented except its clause that a seam records the ceiling" and says the exact and one-byte-over tests "pin that ceiling by behavior". They do not. Encoding under an unbounded ceiling and comparing afterwards (the pre-change design) gives the same boundary results, and a probe of exactly that passed every test (SR-1154 FND-001). The missing clause is the AC's guard against that regression, so the AC is partial. The repo's convention for a partly met AC (FR-038-AC-110: "AC-110 is partial ... that clause stays planned (IR-555)") names the clause as planned with a ticket. The row should do the same, or the seam should land in this PR | spec/output_mapping/matrix/tests.md:12; spec/output_mapping/functional/FR-032-admit-output-mapping-request.md:94; spec/tests.md:16 |
| FND-002 | medium | The FR-034 row marks AC-7 implemented, but two of its clauses are not met. (a) "`MappingLimits` members, including ... `0` ... appear as `\"0\"`" is impossible to construct: `MappingLimits::new` refuses any zero member (`zero_limit`, FR-032-AC-2). Only an `OutputByteRegion` `start` of `0` is tested. This is a spec defect the row does not name. (b) "The request ... identities are the SHA-256 digests" and "have distinct request identities": admission computes no request digest at all, and `AdmittedMappingRequest` keeps only `request_bytes`. The row calls this "not exposed", which implies the identity exists. The test hashes the material inside the test. AC-7 should read partial with both clauses named, and (a) routed to a spec amendment | spec/output_mapping/matrix/tests.md:14; spec/output_mapping/functional/FR-034-assemble-output-package-atomically.md:99; crates/quire-contract-model/src/output_mapping.rs:624-647, 2268-2290 |

## Verdict

Changes requested on matrix honesty (two medium findings). The code meets
FR-033-AC-6, FR-034-AC-6 and the `output_mapping.rs` part of FR-038-AC-91. The
matrix overstates FR-032-AC-6 (the behavioral pin claim is false) and
FR-034-AC-7 (one clause cannot be constructed, one names an identity the code
never computes). Either fix the code or test so that these hold, or mark both
ACs partial with the open clauses named and ticketed.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: the seam clause is implemented. Each identity step takes the encoder as an argument, and a spy records that the ceiling equals the limits' `maximum_request_bytes` and varies with it. The FR-032 row now says that, and drops the false claim that behaviour pins the ceiling. The step-level clause of AC-6 is met as worded; the call-site residual is SR-1154 FND-006 (low) |
| FND-002 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: the FR-034 row marks AC-7 partial and names both unmet clauses (a zero `MappingLimits` member written as `"0"`, and the request digest admission does not compute) as planned under IR-567 (Backlog, spec amendment). The TC-043 row and the `spec/tests.md` index agree |
