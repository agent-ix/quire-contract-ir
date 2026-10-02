---
id: SR-716
title: "gap analysis of PR 246 (IR-475 KaniProfile validated on every construction path)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@1bd313563d7f6a1ea704e7c0868fb3b4d4f94f4e; spec/kani/functional/FR-029-versioned-bounded-kani-profile.md, spec/kani/functional/FR-039-root-crate-public-interface.md, spec/assurance/AD-006-codegen-consumption-seam.md, src/kani/profile.rs, tests/it/kani_shared.rs"
review_set: subset
---
# SR-716: gap analysis of PR 246

## Summary

Ticket: IR-475. Planless gap analysis. Plan completion: not assessed.

The PR changes no spec. Its claim is that FR-029 already owns the behaviour, and that
claim holds. FR-029's Behavior section says "A missing entry, unknown construct, version
mismatch, or duplicate/conflicting entry refuses before any partial lowered artifact is
exposed." FR-029-AC-2 says that "unknown, conflicting, and incomplete matrices refuse".
Refusing only in `new` left a deserialized profile that broke both. The PR closes that hole.

Other spec surface checked:

- FR-039's inventory row for `kani` lists types (`KaniProfile`, `ProfileSelection`, and so
  on), not fields, so making the fields private does not contradict it.
- AD-006 lists `KaniProfile` as a shared Kani contract type that CG consumes. It names no
  fields, and it says "IR refuses a selection it does not know". The PR strengthens that.

Binding: `tc_042_an_invalid_profile_is_refused_on_every_construction_path` is tagged
TC-042, FR-029-AC-1 and FR-029-AC-2. The duplicate and blank construct cases back AC-2
(conflicting or unknown matrix entries). The wrong-family and blank-revision cases back
AC-1 (the versioned profile identity is bound before lowering, and a version mismatch
refuses). Both bindings are correct. `make spec` shows one more bound symbol and the same
23 unbacked rows as main, with no new warning.

The PR has no unowned code: the accessors and the wire type both serve FR-029. It adds no
stub and no tautology. The test's oracle was checked by mutation (see SR-715).

Pre-existing gap, not caused by this PR: FR-029-AC-2 requires a refusal "with the
originating source identity", and `ProfileError` carries no source identity. FR-029's
Status section already marks AC-2 as planned, so this gap is tracked, not new.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- FR-029-AC-1, spec/kani/functional/FR-029-versioned-bounded-kani-profile.md, examined: "A selected profile binds its versioned lowering, harness, oracle/strategy, replay, bounds, and capability-matrix identities before lowering."
- FR-029-AC-2, spec/kani/functional/FR-029-versioned-bounded-kani-profile.md, examined: "Every encountered construct has exactly one supported, refused, or unsupported matrix entry; ... unknown, conflicting, and incomplete matrices refuse with the originating source identity."
- FR-039 `kani` inventory row, spec/kani/functional/FR-039-root-crate-public-interface.md, context_only.
- AD-006 shared Kani contract row, spec/assurance/AD-006-codegen-consumption-seam.md, context_only.

## Verdict

Clean. The spec already owns the change, the bindings are correct, and the coverage counts
match main.

## Disposition pass 1

Reviewed at 66967e8cec9449b75d668aeb07d8cf6708bfcd4e. This review had no findings, so there
are no dispositions. The delta from 1bd313563d7f6a1ea704e7c0868fb3b4d4f94f4e is one commit
that changes only the body of `tc_042_an_invalid_profile_is_refused_on_every_construction_path`.
Its `#[trace("TC-042", "FR-029-AC-1", "FR-029-AC-2")]` tag is unchanged, and so are the TC-042
bindings to FR-029-AC-1 and AC-2. The gate log for 66967e8cec9449b75d668aeb07d8cf6708bfcd4e
shows the same 23 strict-unbacked rows. The delta adds no new gap.
