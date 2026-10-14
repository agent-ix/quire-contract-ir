---
id: SR-10820
title: "IR-238 PR 384 code-review at 2e739b5"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #384 at 2e739b5b7eef441766e72e69c7ad9dd371c304a8; FR-038 AC-215..218; checked_package v2 source and changed tests"
review_set: subset
---

## Summary

Diff-scoped Rust code review of the typed mismatch paths and their tests. The mismatch dispatch and typed identities follow the requested behavior; the FR-038 status narrative is stale.

## Verdict

**CONDITIONAL** for the frozen PR diff.

## Examined scope

- `FR-038-AC-215` — examined: A selected model document that reads as strict JSON, supplied under its row's `digest`, then edited in one member value so that its RFC 8785 SHA-256 changes, refuses `stale_dependency`/`content-mismatch` at `/lock/model_selections/0/digest` with `expected` equal to `{domain: "sha256-jcs", algorithm: "sha256", digest: <row digest>}` and `actual` equal to the edited document's RFC 8785 identity under that same domain and algorithm (the expected digest bytes are written out in the test, not recomputed by the reader's digest function), before identity or declaration checks and never as `byte-digest-mismatch`; the unedited document and a whitespace-respelled copy under their RFC 8785 digest admit; a parseable document selected under the plain SHA-256 of its raw bytes when that differs from its RFC 8785 SHA-256 refuses `content-mismatch`, not `wrong-model-selection`; a document holding a number FR-038-AC-93 or FR-038-AC-109 refuses `noncanonical_wire` first.
- `FR-038-AC-216` — examined: Bytes of a selected model document that `quire_canonical::read` refuses (a truncated array, invalid UTF-8, `{"a":1,"a":2}`, a lone surrogate escape), supplied under a row `digest` other than their plain SHA-256, refuse `stale_dependency`/`byte-digest-mismatch` at `/lock/model_selections/0/digest` with `expected` `{domain: "sha256-jcs", algorithm: "sha256", digest: <row digest>}` and `actual` `{domain: "raw-artifact-digest", algorithm: "sha256", digest: <plain SHA-256 of supplied bytes>}`; the same bytes supplied under the row whose digest bytes equal their plain SHA-256 refuse `invalid_model_binding`/`wrong-model-selection` at `/lock/model_selections/0/identity` with neither payload, admit no declaration and read no identity from the bytes. Equal digest bytes in those different domains remain unequal typed identities. The outcomes of FR-038-AC-93, AC-109 and AC-110 for a number past the double range and of FR-038-AC-26 for the byte limit are unchanged by the raw digest.
- `FR-038-AC-217` — examined: A `dependency_selections` entry whose supplied admitted package recomputes to another `package_id` than the entry's refuses `stale_dependency`/`content-mismatch` at `/lock/dependency_selections/0/package_id/digest` with `expected` equal to the entry's complete `CheckedSemanticId` (`quire.package.semantic/v2`, `sha256`, its digest) and `actual` equal to the supplied package's complete recomputed `package_id`, and never `byte-digest-mismatch`; no package supplied refuses `missing_import`/`missing-selection` at `/lock/dependency_selections/0` with neither payload, and a supplied package with the selected `package_id` admits.
- `FR-038-AC-218` — examined: The `expected` and `actual` members of `CheckedPackageRefusal` are either both absent or both complete `CheckedSemanticId` values with exact domain, `sha256` algorithm and lowercase 64-character hexadecimal digest, and are present exactly for `stale_dependency`/`content-mismatch` and `stale_dependency`/`byte-digest-mismatch`. A `noncanonical_wire`, `missing_import`/`missing-selection`, `digest_domain_mismatch`, `invalid_model_binding`/`wrong-model-selection`, `invalid_semantic_graph` and `malformed_wire` refusal each carries neither. `ContentMismatch` is paired only with `StaleDependency`; two mismatch refusals with the same code and cause tag but different expected/actual identities have different QSpec FR-272 cause keys, while changing only path leaves that cause key unchanged. Typed identities with equal algorithm and digest but different `sha256-jcs` and `raw-artifact-digest` domains compare unequal.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | FR-038 digest-mismatch inventory still says the own-package mismatch returns plain stale_dependency and typed payload is planned, while this PR emits ContentMismatch with expected/actual. This contradicts the implementation and AC-218; update the status narrative. | spec/checked_package/functional/FR-038-consume-checked-package-v2.md:1366-1370 | wrong-requirement |

## Trace and evidence

No source or test files were edited in this review. The PR diff changes only Rust source and tests; no spec, plan, CI workflow, or applicable AssuranceProfile was changed. The strict computed matrix is tagged for AC-215..218 but exits nonzero on inherited repository-wide gaps. The source paths for readable canonical content, unreadable raw bytes, dependency selection, and own-package identity were inspected with their changed assertions.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The FR-038 digest inventory now states all three content-mismatch sites, their typed pair, and current IR-238 implementation; the matrix status was updated in the same fix commit. |

## Disposition verdict

**PASS** for the three original findings at f49c37fba36dad96dbaa03d9a9a60e1a626df8f0. The reviewed fix delta changes only the stated spec, matrix row, and focused test files; no new review sweep or full gate was run.
