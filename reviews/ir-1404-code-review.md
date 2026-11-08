---
id: SR-202611070951309517
title: "IR-1404 PR971 canonical diagnostic carrier code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/checked_package_v2_canonical_encoding.rs"
review_set: subset
---

## Summary

Ticket: IR-1404. Independent code-review with Rust lane of the frozen PR971 candidate against the supplied current main. One test file changed; no production, spec, plan, CI, or public API change. No applicable AssuranceProfile was found.

## Examined scope

- FR-038-AC-77 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A `CheckedSemanticGraphV2` node body and `CheckedDiagnosticsV2` detail nested 100000 levels deep each encode on a 256 KiB stack to independently written expected text without stack overflow. The canonical envelope encodes each supplied deep body once without retaining a second wire body.
- FR-038-AC-78 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: The encodes of FR-038-AC-77, run under a byte ceiling of the encoded text's exact length, return the bytes; run under a ceiling one byte lower, return the byte-limit error with no bytes; a body nested 20000 levels deep encodes and is not refused for its depth; and a body holding the integer 9007199254740993 returns the encoder's refusal naming that value with no bytes.
- FR-038-AC-79 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A package document holding 9007199254740993, -9007199254740993 or 2.0 in a node body, or an object with UTF-8-out-of-order member names, refuses noncanonical_wire before graph and body checks.
- FR-038-AC-111 (examined), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: A package document holding an inexact numeric JSON token in a node body refuses noncanonical_wire before graph or body checks; a canonical numeric token at the same position is not refused for that cause.
- FR-038-AC-511 (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: Whole-read accounting, including each read's owned catalog. A test binary whose global allocator counts, on the reading thread during the call, the size of each allocation request and the new size of each reallocation request reads, under an unlimited budget, an exact budget and each one-below budget of AC-497 through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance packages this repository reads, and the adverse shapes — the 100000-node reference chain, a 10000-node flat graph, a node with 10000 dependencies, a 1 MiB string member, 10000 source-map entries, 10000 d
- FR-038-AC-513 (context_only), spec/checked_package/functional/FR-038-consume-checked-package-v2.md: The V2 read path (`CheckedPackageV2::read` from entry through returning its result, including the owned catalog, and every function it reaches) calls no `serde_json` parser, deserializer or `RawValue`, constructs no `serde_json::Value`, calls no `num-bigint` parse or arithmetic, sorts only with `sort_unstable*`, and builds no `BTreeMap`, `BTreeSet`, `HashMap` or `serde_json::Map`; its tables are sorted exact-capacity slices and its containers are reserved at exact capacity before they are filled. Every read parses the catalog with the shared `quire_canonical` reader and charges its own catalog
- Carrier::place (examined), tests/it/checked_package_v2_canonical_encoding.rs: let held = quire_canonical::to_vec(&body, Limits::new(u64::MAX))
- Carrier::take (examined), tests/it/checked_package_v2_canonical_encoding.rs: self.entries[0].details[0] = CheckedJson::from_serializable(&Value::Null).expect("encodes");
- deferred-controls (examined), tests/it/checked_package_v2_canonical_encoding.rs: #[ignore = "IR-1404: deferred AC-511 whole-read allocation proof"]

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for this test-only runtime repair and explicitly authorized deferral scope. Both take callers only dismantle the returned value; neither uses it as an oracle. The checked handle retains a flat shared document, so replacement drops the arena without reconstructing recursive serde Values. The existing iterative canonical writer is reused; no new duplicate helper or API was added. Construction retains original encoder refusal errors. Expected deep bytes still come from independently repeated brackets and shallow envelope splicing, not from the production writer. Depth 100000, stack 256 KiB, exact ceiling, one-byte-short error kind/bound, depth 20000, integer refusals and boundary bytes remain unchanged.

Exactly four original scalar/UTF16 controls acquire tracked ignores; their bodies/assertions and trace tags are unchanged. Their pre-deferral logs show allocation accounting failure before the semantic oracle. Therefore this review makes no semantic-success claim for those four. AC-511 and AC-513/F1 remain deferred; the ignore attributes are not proof of either. Makefile test still uses --include-ignored and will execute the four; ordinary green is qualified, not the aggregate gate.

## Validation

Independently executed a scratch copy of the existing frozen worktree integration binary with RUST_TEST_THREADS=2: 17 passed, 0 failed, 4 ignored (test.log). This is a rerun, not an independent rebuild. Inspected recorded original stack-overflow RED, focused GREEN, and full qualified module evidence. No ignored known-red test was rerun. Aggregate workspace clippy, deny, full test and formatting remain lead-owned postmerge qualification as explicitly instructed; existing warnings are not represented as cleared. Gap-analysis does not apply to a test-only diff; spec-review and subanalyses do not apply because no spec/plan changes. No source edits or branch mutations were made.
