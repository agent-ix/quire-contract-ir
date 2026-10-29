---
id: SR-12020
title: "IR-210 slice 1 code review (rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; tests/it/canonicalization.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-017
    type: references
  - target: ix://agent-ix/quire-contract-ir/StR-002
    type: references
---

## Summary

Ticket: IR-210, PR quire-contract-ir#426, over origin/main (the merge base). The one changed file, `tests/it/canonicalization.rs`, gains `use ix_trace_rs::trace;` and swaps the four tests' `/// Tracing:...` doc lines for one `/// Trace:...` line plus a matching `#[trace(...)]` attribute. Every changed line in `git diff origin/main...HEAD` is a doc comment, an attribute or the `use`. No test body or assertion changed. The rust-review lane is folded into this file. Each tag was read against its criterion's text and against the test body. The tag-honesty findings are here because they are defects in the changed lines; the coverage accounting is in the gap-analysis artifact (SR-12021).

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-016-AC-2 | examined | A semantic change to a clause changes that clause, requirement, and package digest while unrelated clause digests remain stable; repeated runs and reversed insertion order reproduce identical bytes without host-width or map-order fields. |
| FR-016-AC-4 | examined | Each of the five object kinds exposes a budgeted and a budget-free canonical operation; the budget-free result equals the budgeted result at `u64::MAX`; and a zero budget returns `canonicalization_resource_exhausted` with no bytes and no digest for every kind. |
| FR-017-AC-1 | examined | Version preflight accepts schema 1.1; schemas 1.0, 1.2 and 2.0 each fail `unsupported_schema_version` at the specified path, and a zero major fails `invalid_schema_version`, all before semantic interpretation. |
| FR-016-AC-1 | examined | Exact golden fixtures... pin the profile envelope, escaping, the string spelling of the eight integer members..., and SHA-256 digest for every closed object kind; equivalent supported permutations are byte/digest identical. |
| FR-016-AC-3 | examined | A deterministic reservation-failure harness forces canonical byte allocation failure and verifies `canonicalization_resource_exhausted`, no partial public bytes, and no digest. |
| FR-017-AC-2 | examined | Shallow, deep, uncovered, and each closed orphan reason have positive/negative fixtures; stale revision, missing, cross-package, duplicate, and over-limit inputs retain distinct diagnostics... Deep traces name the current requirement revision and carry no digest. |
| NFR-003-AC-1 | examined | Every declared invalid-input class returns a stable diagnostic code and no public panic. |
| NFR-003-AC-2 | examined | No malformed, unsupported, stale, undefined, or orphaned input contributes successful canonical identity or covered status. |
| StR-001-VC-2 | examined | Downstream artifacts can cite the same package, requirement, clause, and expression identities. |
| StR-002-VC-1 | examined | Semantically identical supported packages produce identical canonical bytes and digests on every supported platform. |
| StR-002-VC-2 | examined | Unsupported schema versions produce structured rejection. |
| tests/it/canonicalization.rs | examined | `use ix_trace_rs::trace;` plus four `/// Trace:` lines and `#[trace(...)]` attributes on the tc_017 tests |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `#[trace(..., "FR-016-AC-2"...)]` on `tc_017_canonical_bytes_digests_ordering_and_resource_failure_conform` claims more than the test asserts. The test compares package digests (changed against unchanged) and requirement digests (REQ_a differs, REQ_b equal), but it never compares a clause digest. The AC's "changes that clause... digest" and "unrelated clause digests remain stable" legs are not asserted. A `canonical_clause` that ignored the clause body would pass the test. The PR withheld FR-016-AC-1 because one leg ("every closed object kind") was missing; by the same standard this tag should also be withheld, or two clause-digest assertions added. The other legs are asserted: repeated equals, permuted equals, and the golden bytes rule out host-width and map-order fields. | tests/it/canonicalization.rs:77-78, tests/it/canonicalization.rs:112-139, spec/model/functional/FR-016-canonicalization-digests.md:156 |
| FND-002 | medium | StR-002-VC-2 ("Unsupported schema versions produce structured rejection") was dropped on the PR's stated ground that VC ids are "not AC ids". quire does count VC rows: `quire coverage --strict` shows StR-002 going from 2/2 to 0/2 backed on head. The claim is asserted. `tc_017_version_preflight_supports_only_schema_1_1` checks that 1.0, 1.2, 1.9, 2.0 and 2.1 each produce exactly one `UnsupportedSchemaVersion` diagnostic at the specified path. The canonical-API loop in the first test also asserts a structured rejection at `schema_version`. The old doc line was on the declaration/expression test, which asserts none of this, so removing it there is right. The claim should move to the preflight test instead of disappearing. This is a dropped true claim. | tests/it/canonicalization.rs:650-697, tests/it/canonicalization.rs:166-194, spec/core/stakeholder/StR-002-portable-interchange.md:27 |

## Verdict

CHANGES REQUESTED (two medium).

Clean units:

- **FR-016-AC-4.** The coder's split is genuine. The first test asserts budget-free equals budgeted at `u64::MAX`, and a zero budget gives `CanonicalizationResourceExhausted`, for package, requirement and clause. The second test asserts the same for declaration and expression. That is five kinds in total, and `unwrap_err` on the `Result` means no bytes and no digest are returned. The comment at line 196, "for every one of the five closed object kinds", sits on the first test, which covers three; the line predates this PR and is outside the diff.
- **FR-017-AC-1.** Fully asserted. 1.1 is accepted. 1.0 and 1.2 fail at `schema_version.minor` and 2.0 at `schema_version.major`. 0.0 and 0.1 fail `InvalidSchemaVersion` at `schema_version.major`. Each case has exactly one diagnostic while the id is invalid (`"bad id"`), which proves preflight runs before semantic validation.
- **Correct drops.** Dropping these claims was right:
  - FR-016-AC-1: only the package digest is pinned, and declaration has no exact golden.
  - FR-016-AC-3: there is no reservation-failure harness, only a zero budget. The crate's `tc_017_a_limit_and_an_encoder_fault_...` is not one either.
  - FR-017-AC-2: no requirement row is asserted `Shallow`, there is no over-limit fixture, and there is no deep-trace no-digest assertion.
  - NFR-003-AC-1: one invalid-input class against "every declared class".
  - NFR-003-AC-2: stale is not shown to be non-covering because REQ_a also has a deep trace, and there is no undefined input.
  - StR-001-VC-2: an interface-review claim. It is still backed by `tests/it/identity.rs`.
  - StR-002-VC-1: "every supported platform", while CI runs one `ubuntu-24.04` job.
- **Rust lane.** `use ix_trace_rs::trace;` is the repo's import form: it appears in at least 10 test files, including `tests/it/checked_package_v2_protocol_reference.rs`. The doc-line form `/// Trace: <ids>` with comma-separated ids and no trailing period matches the 230 existing `/// Trace:` lines. Both TC-first (59) and FR-first (171) orders are in use. The attribute ids equal the doc-line ids on every test.
- **Gates.** `cargo test --locked --workspace --test it canonicalization` was re-run on head: 4 passed. There is no panic, unsafe or integer-conversion surface in the diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, : `FR-016-AC-2` is removed from both the `/// Trace:` line and the `#[trace(...)]` attribute of `tc_017_canonical_bytes_digests_ordering_and_resource_failure_conform`, which now reads `/// Trace: TC-017, FR-016-AC-4` / `#[trace("TC-017", "FR-016-AC-4")]`. No other test carries the id; the FR-016 matrix row now says AC-2 has no test tagged. The test body is unchanged. |
| FND-002 | fixed | Round 1,: `tc_017_version_preflight_supports_only_schema_1_1` now carries `/// Trace: TC-017, FR-017-AC-1, StR-002-VC-2` / `#[trace("TC-017", "FR-017-AC-1", "StR-002-VC-2")]`. That body asserts the criterion: for 1.0, 1.2, 1.9, 2.0 and 2.1 the decode returns exactly one diagnostic, coded `UnsupportedSchemaVersion` and located at `schema_version.minor` or `.major`, which is a structured rejection. Leaving StR-002-VC-1 ("identical canonical bytes and digests on every supported platform") unbacked is correct, because CI runs one `ubuntu-24.04` job and no test compares platforms. Re-measured with `quire coverage --scope. --strict` on: 501/564 rows backed and 45 unbacked, with the same unbacked list as (only NFR-003-AC-1/2 added over base). StR-002 is 1/2, FR-016 6/9 and FR-017 2/6. |
