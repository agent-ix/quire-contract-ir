---
id: SR-12752
title: "IR-875 PR 467 integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir; IR-875; PR #467; spec/checked_package/non-functional/NFR-006-v2-producer-memory.md; spec/checked_package/matrix/TC-455-v2-producer-memory-and-identity.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/NFR-006
    type: reviews
---

## Summary

The 65,536 KiB threshold is internally consistent with the fresh 339,124 KiB control and 254,692 KiB local candidate. The changed files add no IDs or trace bindings, and the matrix stays identical to main.

## Scope and evidence

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| NFR-006 Statement | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md | examined | When a producer encodes a `quire.checked-package/v2` artifact, the Contract IR identity-projection encoding path SHALL permit it to borrow each semantic graph node body without retaining a second owned `serde_json::Value` body per node. When QSL emits with that view, the integrated producer SHALL reduce the median incremental peak resident memory in the fixed TC-455 emission window by at least 65,536 KiB (64 MiB) relative to a freshly measured owned-projection baseline on the same host, while preserving the exact canonical package bytes and `package_id`. |
| NFR-006 Measurement | spec/checked_package/non-functional/NFR-006-v2-producer-memory.md | examined | \| Metric \| Target \| Threshold \| Method \| \|---\|---\|---\|---\| \| Peak incremental resident memory in the TC-455 emission window \| At least 65,536 KiB (64 MiB) lower than the fresh owned-projection baseline median \| Candidate median ≤ fresh baseline median − 65,536 KiB; the measured control's ceiling is 273,588 KiB (339,124 − 65,536) \| Median of three fresh-process runs before and after the change, same fixture, toolchain, release profile, allocator, and host; sample `/proc/self/status` `VmRSS` from just before `emit_checked` until it returns and subtract the starting `VmRSS` \| \| |
| TC-455 Procedure | spec/checked_package/matrix/TC-455-v2-producer-memory-and-identity.md | examined | In a fresh process, construct the exact UTF-8 source as the prefix `language "ix:native" edition "1-draft";\nprofile v = "quire.value.complete/v1";\nfunction f using v(a: Boolean, x: Integer): Integer pure { `, followed by `x + ` repeated 12,499 times and the suffix `x }\n`. Use fixed source identity (`a`, `u`, `git`, `1`) and unit name `unit.native`. Set parser source-byte and token limits to at least the source byte length, and its node limit to at least sixteen times that length. Check with `CheckingLimits::new(u64::MAX).with_input_bytes(u64::MAX).with_work_budget(u64::MAX)`; |
| TC-455 Expected Results | spec/checked_package/matrix/TC-455-v2-producer-memory-and-identity.md | examined | All six fresh processes emit the same canonical bytes and `package_id`; both versions' bytes are reader-admitted. Once QSL adopts the borrowed view, the changed producer's median incremental peak RSS is at least 65,536 KiB (64 MiB) below the freshly remeasured owned-projection baseline median on the same host. The fresh control's median is 339,124 KiB, giving this measurement a candidate ceiling of 273,588 KiB. The local uncommitted candidate's median is 254,692 KiB, an 84,432 KiB (24.90%) reduction; this demonstrates the target with the trial patch but does not complete QSL-owned |

## Verification

Current PR head: `284db1c024ad361c27bd72919e433a7c0ce4e332`. Quire validation of both changed files passed. The computed matrix matches current main byte-for-byte, and strict coverage is 535/606 backed with 42 unbacked and zero contradicted statuses. The six local probe records yield medians of 339,124 and 254,692 KiB incremental sampled peak RSS, a reduction of 84,432 KiB (24.90%); median emission times are 1,137 and 1,544 ms (+35.8%). All six complete byte files match. These records are local candidate evidence; QSL adoption is pending. The exact command `cargo test --locked --workspace --all-targets -- --include-ignored` exited 0 on this PR head.

## Verdict

PASS. No contradictory status, duplicate ID, or criterion mutation found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
