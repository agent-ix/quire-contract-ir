---
id: SR-11189
title: IR-474 typed closed-member provenance code review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/common.rs, crates/quire-contract-model/src/checked_package/common/closed_members.rs, crates/quire-contract-model/src/checked_package/v2/identity.rs, crates/quire-contract-model/src/checked_package/v2/mod.rs, tests/it/checked_package_v2_reader.rs"
review_set: subset
---

## Summary

Ticket: IR-474. Reviewed the PR diff for Serde error provenance, closed-member classification, nominal preimage location, pointer preservation, bounded processing, and the public reader controls. The failed-decode observer uses Serde's derived struct field list, preserves the original successful decode path, and does not parse error prose.

## Verdict

**PASS** — no code or Rust findings in the changed paths. The changed public reader and nominal-preimage controls each pass; `cargo fmt --all -- --check`, workspace Clippy with `-D warnings`, and `git diff --check` pass. Full gates belong to the lead's pre-PR and pre-merge rounds.

## Examined units

- `FR-038-AC-2` (examined; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): In an otherwise valid package, replacing `/semantic_graph/nodes/0/occurrences/0/role` with the literal string `unknown field` refuses as `malformed_wire`, not `unknown_member`; a genuine extra member still refuses as `unknown_member` and a required member of the wrong kind still refuses as `malformed_wire`.
- `FR-038-AC-24` (examined; same path): The `unknown field` occurrence-role value points exactly at `/semantic_graph/nodes/0/occurrences/0/role`; genuine extra-member and wrong-kind controls point at their changed members or values.
- `TC-048` (examined; `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`): Mutate an admitted role value, genuine extra member, and wrong-kind dependency; compare codes and pointers against the public reader.
- `FR-038-AC-5` (context_only; `spec/checked_package/functional/FR-038-consume-checked-package-v2.md`): Nominal-preimage structural refusal and location precedence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
