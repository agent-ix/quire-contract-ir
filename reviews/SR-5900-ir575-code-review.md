---
id: SR-5900
title: "IR-575 PR 340 independent Rust and code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir PR 340; FR-038-AC-214; changed Rust, tests, and manifests"
review_set: subset
---

## Summary

The typed intake, canonical token scan, borrowed body decoding, dispatch, memory oracle, and existing refusal controls were reviewed against FR-038-AC-214. The fixed 100000-node fixture meets its measured memory ceiling, but the production lossless check still materializes the complete remaining wire as two `serde_json::Value` trees when the four omitted arrays are empty.

## Verdict

**FAIL** for the unconditional no-full-document-DOM obligation. The author must fix FND-001 and verify a package dominated by a non-omitted closed field. The inherited strict-coverage and QSpec conformance failures are not counted as this PR's defects or as green gates.

## Examined units

- `FR-038-AC-214`: direct typed read, refusal parity, memory method and ceiling.
- `FR-038-AC-117`: 256 KiB reader stack and deep-term controls.
- `crates/quire-contract-model/src/checked_package/common.rs`: strict scan, canonical token scan, closed decode.
- `crates/quire-contract-model/src/checked_package/dispatch.rs`: public version dispatch.
- `crates/quire-contract-model/src/checked_package/v2/intake.rs`: borrowed schema, lossless source comparison.
- `crates/quire-contract-model/src/checked_package/v2/mod.rs`: admission order and typed result.
- `tests/ir575_reader_memory.rs`: separate-process memory oracle and fixture.
- `tests/it/checked_package_v2_canonical_encoding.rs`: changed canonical adverse control.
- `tests/it/checked_package_v2_identity_digests.rs`: source guard.
- `Cargo.toml` and `crates/quire-contract-model/Cargo.toml`: target and serde feature.
- `spec/checked_package/functional/FR-038-consume-checked-package-v2.md` and `spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md`: criterion and procedure.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The lossless comparison still constructs two document-scale `Value` trees for closed fields outside the four omitted arrays; with those arrays empty, it constructs the full input document twice before validation, contrary to AC-214's unconditional no-DOM rule. | FR-038-AC-214; `v2/intake.rs:280-299` | implementation-bug-despite-evidence |

## Evidence and limits

`check_lossless_source` parses every root field other than `semantic_graph`, `identity_preimage`, `source_map`, and `diagnostics` with `serde_json::from_str` into `original`. It then serializes the entire remaining typed wire with `serde_json::to_value`, while retaining `original`. A syntactically closed V2 wire with empty `nodes`, `identity_projection`, `source_map`, and `diagnostics.entries` makes these two trees cover the full document, even if a later semantic validation refuses it. A large `lock`, `capability_report`, or selection list can dominate memory and is not exercised by the node-heavy measurement fixture. Compare closed fields in bounded units without either root `Value` object.

The final-target measurement was made, whose production reader source is unchanged through; logs record three valid 100022-node runs at 695712, 695712, and 695776 KiB additional peak. The explicit test target is excluded from ordinary CI and was separately invoked. Existing 256 KiB tests and adverse corpus passed in the author's focused/full evidence, which this reviewer inspected but did not rerun. `make ci` strict coverage and QSpec conformance remain red as stated in the PR; no full gate was run by this reviewer.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in the reviewed PR. |

At the reviewed local head, `v2/intake.rs` keeps the source as borrowed `RawValue` slices and calls the field and item comparator in `v2/lossless.rs`. The comparator parses one closed object into borrowed member slices or one array into borrowed item slices, then compares typed fields recursively. The former root `Value` and `serde_json::to_value(&*wire)` pair is gone. A 32,045,214-byte fixture dominated by 500,001 `capability_report` items passed the fresh-process reader-only control at 37,364 KiB additional peak, against its 786,432 KiB control ceiling; the old reader failed the same control at 810,652 KiB. The existing 176,923,194-byte node fixture admitted 100,022 nodes at 695,712 KiB additional peak, below AC-214's 1,572,864 KiB ceiling. The receipt logs predate the rebase and do not themselves name a SHA; the rebase from pre-rebase to this head changed only spec files, with the production and test Rust files byte-identical. This disposition did not rerun gates.
