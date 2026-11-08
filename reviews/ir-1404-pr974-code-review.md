---
id: SR-202611070951309529
title: "Code review of native encoder controls"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #974; two native encoder test bodies"
review_set: subset
---

## Summary

Ticket: IR-1404. Reviewed the two changed test bodies and their unchanged production encoder and error-mapping callees. No defects found.

## Verdict

**PASS** for the scoped code review. The deep fixture uses the iterative canonical writer and reader while retaining an independently constructed private-wire expected value, 100000 levels, a 256 KiB thread stack, equality and iterative cleanup. The refusing writer exercises the real canonical WriteSink and encoder, then the existing required_bytes helper, including saturation. The exact/short byte-ceiling behavioral test is unchanged. No production API, body graph, ignored test or CI lane changes.

## Scope examined

- FR-038-AC-95; role: examined; excerpt: PLANNED/UNRUN (IR-1141). Bounded canonical encoding of a complete typed node and package succeeds at the exact retained byte ceiling and returns a typed bytes-limit failure one byte below, without a partial value or panic. In a multi-request lowering call, a node-local byte failure preserves successfully lowered independent sibling records; a whole-package byte failure returns no lowered node, dependency node or package bytes. Each failure reports the configured limit and consumed work consistently; a work-budget failure remains distinct. No identity preimage or hash is constructed.
- FR-051-AC-4; role: examined; excerpt: Empty, nested and deeply nested node bodies and present or absent optional fields are serialized once and retain their full values and node order. The optional checked-term-data member is omitted for `None` and present with all four empty arrays for `Some(empty)`; neither case is normalized into the other. Exact-length and one-byte-short limits give their specified outcomes. No owned graph projection, sidecar clone or body clone is introduced on the borrowed encode path.

Changed files: crates/quire-contract-model/src/checked_package/v2/mod.rs and crates/quire-contract-model/src/checked_package/v2/lower/ceiling_tests.rs. Context: lower.rs required_bytes and measure_node; repository AGENTS.md and CLAUDE.md; Rust testing rules. No applicable AssuranceProfile found.

## Validation evidence and limits

Inspected author focused logs: deep test 1 passed; ceiling module 11 passed. No Cargo reruns: dispatch explicitly restricts redundant expensive gates while another native job is live. Final fmt check remains owned by lead and must be green before merge; clippy, deny and full workspace tests were not independently rerun. Whole-proof AC-511, static AC-513 and F1 migration are explicitly deferred by dispatch and are not evaluated here. This review does not certify those deferred obligations or the interrupted native batch.

Methods: code-review with Rust lane. Spec review and subanalyses do not apply because no requirements or plan changed. Separate gap-analysis does not apply under reviewer method selection because the diff changes only test bodies, not production code. Manually checked the two tagged criteria against the retained assertions; neither criterion is claimed fully proved by these two tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
