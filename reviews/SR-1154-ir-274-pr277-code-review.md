---
id: SR-1154
title: "code review of PR 277 (IR-274 part C: output-mapping identity steps metered through quire-canonical)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@fdfd4617773b27ad312cb59731164c5d3cfbea01; git diff origin/main...HEAD (base 6e67815086e1171290c9dfd9a46ec9b0cd4e24b6): crates/quire-contract-model/src/canonical.rs, crates/quire-contract-model/src/identity.rs, crates/quire-contract-model/src/output_mapping.rs, tests/it/output_mapping.rs, tests/it/checked_package_v2_identity_digests.rs, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/checked_package/matrix/tests.md, spec/output_mapping/matrix/tests.md, spec/tests.md; checked against quire-canonical b4bb97a5 (Cargo.lock) and spec/output_mapping/functional (FR-032, FR-033, FR-034, STD-003)"
review_set: subset
---
# SR-1154: code review of PR 277

## Summary

Ticket: IR-274 (code change C). Code review with the Rust-review lane folded in.
The PR replaces the three output-mapping identity encodes (request,
`record.identity`, `package.identity`) with `quire_canonical::to_vec` under
`Limits::new(maximum_request_bytes)`, spells the `MappingLimits` and
`OutputByteRegion` `u64`s as decimal strings with `serialize_with`, and derives
`FixedShape` on the types the three identity materials hold.

What was measured at the reviewed sha:

- `cargo test --workspace --all-targets -- --include-ignored`: 296 integration
  and 126 model unit tests pass. The 5 new unit tests and 5 new integration
  tests all ran and pass. `cargo fmt --check` and both `make lint` clippy lanes
  are clean.
- `make spec`: validate passes, with 1 grammar finding (FR-014, the baseline).
  `quire coverage --strict` reports 23 unbacked rows, which matches the
  baseline. Two new `oracle-resembles-implementation` warnings appear
  (FND-005).
- Ceilings before the change: the request was encoded under `u64::MAX` and
  then compared with `maximum_request_bytes`. Record and package were encoded
  under `u64::MAX` with no comparison at all. After the change all three use
  `maximum_request_bytes`. FR-034 Behavior and FR-033 require this, and no
  step used any other cap.
- `canonical_envelope_bytes` is still `pub(crate)` in `canonical.rs`. Its only
  caller is `binding.rs:382` (code change B), and `binding.rs` builds and its
  tests pass.
- Deriving `FixedShape` on public `identity.rs` and `canonical.rs` types only
  adds trait impls (and the blanket `Encode`). It changes no `Serialize`
  output. FR-038-AC-80 bans a hand-written `impl FixedShape` anywhere in the
  crate, so deriving is the only option. `serialize_with = serialize_decimal`
  emits a scalar, so the derived `DEPTH` stays correct, which is the hazard
  quire-canonical's docs warn about. The raw digest types write a hex scalar
  by hand while `DEPTH` comes from `[u8; 32]`, which overestimates the depth;
  that is harmless.
- `serialize_with` on the public `MappingLimits` and `OutputByteRegion` also
  changes the public `Serialize` of `GeneratedOutputPackage` and
  `OutputMappingRecord` (limits and regions become strings). Nothing in this
  repo, quire-contract-codegen, quire-spec-language or quire-driver consumes
  that JSON, and no corpus, schema or golden file holds an output-mapping
  digest. This is not the v1 wire, which is code change B.
- Mutation probe (reverted): in admission, replace the request ceiling with
  `!0_u64` plus a post-hoc `check_limit`. Every `tc_043` test and the TC-048
  scan still pass (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test fails on the regression FR-032-AC-6 and FR-034-AC-6 exist to forbid: encoding under an effectively unbounded ceiling and comparing the length afterwards. The exact-length and one-byte-lower boundary results are identical either way. A probe that changed admission to `request_identity_bytes(&material, !0_u64)` plus a post-hoc `check_limit` passed all 27 `tc_043` tests and the TC-048 scan. The scan only bans the literal text `u64::MAX`, so `!0`, `u64::max_value()` or an imported `MAX` slips through. The same holds for the record and package steps. FR-032-AC-6's recording seam is the oracle that would catch this, and it is not implemented | crates/quire-contract-model/src/output_mapping.rs:2268, 2631-2655; tests/it/checked_package_v2_identity_digests.rs:375-385 |
| FND-002 | medium | A source byte offset or a requirement or source revision above 2^53 in a record's material makes `canonical_identity_bytes` emit `arithmetic_overflow` at `record.identity`. STD-003 does not register `arithmetic_overflow` at `record.identity` or `package.identity`, and its rule is "every path it may carry is listed and no other path is permitted". STD-003 does register `allocation_failed` at both paths for a failed canonicalization step. The new unit test pins the unregistered pair | crates/quire-contract-model/src/output_mapping.rs:2642-2648, 3025-3030; spec/output_mapping/functional/STD-003-output-mapping-refusal-registry.md:34-36, 91-92 |
| FND-003 | low | Every `quire_canonical::Error::Limit(_)` maps to `request_limit_exceeded` with the message "exceeds maximum_request_bytes", whatever its `LimitKind`. `LimitKind::ObjectBytes` is a fixed `u32::MAX` object-buffer bound, reachable once `maximum_request_bytes` is set above 4 GiB, which the admitted `u64::MAX` limits allow. That bound is not the request byte limit. Match `LimitKind::CanonicalBytes` and send the rest to `allocation_failed` | crates/quire-contract-model/src/output_mapping.rs:2637-2641 |
| FND-004 | low | The five new unit tests in `output_mapping.rs` carry only a `/// Tracing:` doc comment, no `#[trace]` attribute, so `quire coverage` does not count them. Other model-crate unit tests (`temporal.rs`, `operation_catalog.rs`) use `#[trace]`. The matrix credits these tests for FR-034-AC-6's package-step seam clause and FR-034-AC-7's hand-written request material, which no tagged test covers | crates/quire-contract-model/src/output_mapping.rs:2874-3083 |
| FND-005 | low | The PR adds two `oracle-resembles-implementation` coverage warnings that origin/main does not have: `tests/it/output_mapping.rs::ocl_profile` resembles the new unit-test helper `tests::profile` (similarity 0.82). Both are fixtures, not oracles. Reusing or renaming one fixture clears the noise without hiding a real oracle problem | crates/quire-contract-model/src/output_mapping.rs:2737; tests/it/output_mapping.rs:98 |

## Verdict

Changes requested (two medium findings). The metering is correct. All three
steps pass `maximum_request_bytes` and no other cap, as FR-033 and FR-034
require. The decimal-string spelling matches FR-034-AC-7 and stays inside
output mapping, without touching the v1 wire. The hand-written expected texts
in both test files come from inputs, not from the encoder, and the
exact-length and one-byte-lower pairs exist for every step. The
`allocation_failed` fallback is preserved for every other encoder error.
`FixedShape` derives are additive and justified by FR-038-AC-80.

Before merge:

- FND-001: add the recording seam that FR-032-AC-6 names, or otherwise make
  the "no unbounded ceiling" property testable.
- FND-002: map the integer-magnitude refusal to a registered path/code, or
  amend STD-003 in a spec change.

FND-003 to FND-005 are cheap fixes for the same round.

## New findings (disposition pass 1)

Reviewed at fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df. Mutation probes ran in a
throwaway copy, not on the PR branch:

- M1: `identity_bytes` encodes under `Limits::new(!0_u64)` and compares the
  length afterwards. The spy test and the TC-048 scan both catch it.
- M3: the production call site passes a closure that ignores the `Limits` it
  is given. The scan's single-`Limits::new` rule catches it.
- M2: the call site passes a substituted `MappingLimits` whose request byte
  limit is unbounded, then compares the length afterwards. Every test still
  passes (FND-006).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The spy observes the ceiling at the step function, not at its three production call sites. The call sites hand the step the request's own `MappingLimits` and `quire_canonical::to_vec`, and nothing checks that. Probe M2 (admission passes `&MappingLimits { maximum_request_bytes: !0_u64, ..limits.clone() }` and then calls `check_limit` on the length) passed all 28 `tc_043` tests and the TC-048 scan. The scan could also require that the three call sites pass `&limits` or `request.limits()` together with `quire_canonical::to_vec`, or the spy could be driven through admission | crates/quire-contract-model/src/output_mapping.rs:1967, 2269, 2557; tests/it/checked_package_v2_identity_digests.rs:387-399 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: each step takes the encoder as an argument, `identity_bytes` is the one place a ceiling is chosen (`Limits::new(limits.maximum_request_bytes)`), and a spy test records the ceiling at 4096 and 5000 for all three steps. The TC-048 scan allows exactly one `Limits::new(`. Probe M1 (unbounded encode, then compare) now fails both. Residual at the call sites is FND-006 |
| FND-002 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: an integer past 2^53 now gives `allocation_failed` at `record.identity`, which STD-003 registers there. QSpec origin/main 2f846f8 says nothing about these paths |
| FND-003 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: `identity_refusal` maps only `LimitKind::CanonicalBytes` to `request_limit_exceeded` and everything else to `allocation_failed`, unit-tested over CanonicalBytes, ObjectBytes, integer magnitude and allocation |
| FND-004 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: every new unit test carries `#[trace]`, and coverage counts them |
| FND-005 | fixed | fe5d5e20d224ec10ccd0ba4a84a71647fbe8e2df: the unit fixture is now the FRETish profile, and `quire coverage` shows only the two baseline `oracle-resembles` warnings. Strict stays at 23 unbacked |
| FND-006 | fixed | 7f7782023c6ea38123bf580808e07ecbce4ac480 (round 2): the TC-048 scan now requires, after collapsing whitespace, each of the three call sites exactly once, passing the caller's own limits (`&limits` or `request.limits()`) and `quire_canonical::to_vec`, and no other call site. In a throwaway copy, probes M1 (caught by the spy and the scan), M2 (caught by the scan) and M3 (caught by the scan) all fail. Residual, not a finding: shadowing `limits` in an inner block before an unchanged call (M2b) still passes, because no text scan can rule out deliberate shadowing. The scan only fails on purpose-built evasion like that, and any behaviour-preserving rename of a call-site argument trips it loudly rather than passing silently. That matches this file's existing text scans (symbol counts, exact derive lines). Only `tests/it/checked_package_v2_identity_digests.rs` moved since fe5d5e2. Tests (296 + 128), fmt and clippy pass; spec baseline is unchanged (grammar 1, strict 23, 2 baseline oracle warnings) |
