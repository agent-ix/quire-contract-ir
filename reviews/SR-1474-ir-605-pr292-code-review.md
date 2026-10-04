---
id: SR-1474
title: "code review of PR 292 (IR-605 typed STD-001 code as the Kani outcome code)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@fa67cf58512e2908256eef029345aa4989abcaac; crates/quire-contract-model/src/code.rs, crates/quire-contract-model/src/lib.rs, src/kani/abi.rs, src/kani/arithmetic.rs, src/kani/collections.rs, src/kani/mod.rs, src/kani/objects.rs, src/kani/outcome.rs, src/kani/profile.rs, tests/it/std001_code.rs, tests/it/kani_shared.rs, tests/it/kani_arithmetic.rs, tests/it/main.rs, Makefile, .github/workflows/ci.yml"
review_set: subset
---
# SR-1474: code review of PR 292

## Summary

Ticket: IR-605. Code review with the rust-review lane folded in, over
`git diff origin/main...HEAD` at fa67cf58512e2908256eef029345aa4989abcaac. The merge
base is origin/main 7e4dc54579e6d859e7fa224a63a15c4aeec4580d (the FR-044 spec merge, #291).
The PR, ticket and author report were treated as untrusted claims and re-measured.

What was checked, and what holds:

- **Gates, run by this review on the head.** `make fmt-check` and `make lint` (both
  clippy lanes, `-D warnings`) pass. `make test` passes: 367 integration, 149 unit and 12
  model-crate doctests, including the four `compile_fail` probes on `code.rs`.
  `cargo test -p quire-contract-ir --doc` (not part of `make test`, see FND-001) passes 3/3.
- **`Std001Code` representation.** A `Copy` struct over a private `[u8; 64]` with zero
  padding. `well_formed` checks the length before scanning and enforces lowercase start,
  `[a-z0-9_]`, no trailing `_`, no `__`. `pack` is the single build site; `new` and
  `from_static` both route through it. `as_str` uses `position(..).unwrap_or`,
  `get(..).unwrap_or_default` and `from_utf8(..).unwrap_or_default`: no `unwrap`, `expect`
  or panic in non-test code, and both fallbacks are unreachable because the field is
  private, the crate has no `unsafe`, and every byte is ASCII by construction.
  `PartialEq`, `Eq`, `Hash`, `Ord` and `PartialOrd` are all hand-written over `as_str`, so
  they agree with each other and with the string form.
- **Wire bytes.** `Serialize` is `serialize_str(as_str())`: the bytes the `String` field
  produced. `Deserialize` is `deserialize_str` with only `visit_str`, through `new`. Probed
  on this head: `"Bad"`, `""`, `7`, `null` (test), and a 65-byte string, `"x_"`, `"x__y"`
  (probe) all fail; `"kani_x"` reads as `kani_x`; `from_reader` works. The error text
  holds no copy of the input.
- **`std001_code!`.** Probed with throwaway examples (removed afterwards): `"Bad-Code"`
  fails with `E0080 evaluation panicked: not a well-formed STD-001 code`; a `&'static str`
  variable fails with `no rules expected 'text'`; `Std001Code::from("kani_proved")` fails
  with `E0277 ... From<&str> is not satisfied`; `std001_code!(7)` fails with `E0308`. Each
  `compile_fail` doctest therefore fails for the stated reason, and each is paired with the
  passing `const` doctest. The code comment that stable rustdoc does not check a named
  error code is accurate.
- **Registered constants.** One macro (`registered_codes!`) emits each constant and
  `REGISTERED` from one row list. The 15 rows equal STD-001's non-`DiagnosticCode` rows
  exactly: the union of `DiagnosticCode`'s 45 spellings and the 15 constants equals the set
  of every first-column code in STD-001 (`diff` empty). `REGISTERED` is sorted (the test
  compares it to the 15 codes written out in order).
- **`non_success`.** Exhaustive `match`, no wildcard. `Proved` and `Counterexample` return
  `Err(KaniOutcomeError)`; the eight non-success kinds return `Ok`. `kani_outcome_kind_invalid`
  is gone from all `.rs` code; one stale spec mention remains (SR-1475 FND-004).
- **`KaniOutcome::new` at raise sites.** All 30 call sites outside `outcome.rs` pass a
  literal non-success kind; `Proved`/`Counterexample` reach `new` only from
  `proved()`/`counterexample()`. No current call builds a Boolean-claim outcome through it.
  See FND-003 for the spec and future-validation problem.
- **`CapabilityDisposition` code as `Std001Code`.** FR-044 "Break and order" says any field
  that carried a code as `String` takes `Std001Code`, so this is in scope, not an excess.
  Profile deserialization now validates the code form; the profile tests pass.
- **No compatibility layer, no shim, no `unsafe` in `code.rs`, no build artifacts.** The
  diff is 20 source/spec files; no manifest or lock change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-030-AC-6 compile probes (a `&str` and a `String` as the code of `non_success`) are `compile_fail` doctests in the root crate, and no gate runs root-crate doctests. `make test` runs `cargo test --workspace --all-targets` (which excludes doctests) and then `cargo test -p quire-contract-model --doc` only; CI runs `make test`. The probes pass when run by hand (`cargo test -p quire-contract-ir --doc`: 3/3), but a regression that let a string reach `non_success` would stay green in CI while TC-223 and the matrix claim AC-6 is verified. Fix: add the root crate's doctests to `make test` (for example `cargo test --locked --workspace --doc`) | Makefile:67-71, src/kani/outcome.rs:190-215 |
| FND-002 | medium | `KaniOutcomeError(())` carries nothing. STD-001's `kani_outcome_invalid` row gives the required location as "requested kind and cause code, or the rejected count". A caller handed the error cannot tell which kind or code was refused. This PR introduces the type, so the required fields belong in it now: hold the requested `KaniOutcomeKind` and `Std001Code` (both cheap), and a test asserting them | src/kani/outcome.rs:98-99 |
| FND-003 | medium | Every raise site now builds through the unchecked `KaniOutcome::new`, made `pub(super)`, not through the non-success constructor. FR-044 "Break and order" says the family lowerings "emit their codes through the non-success constructor", and FR-030 says an outcome is built "only through its validated constructors". Today nothing is bypassed, because `non_success` checks only the kind and every site passes a literal non-success kind. But the `CapabilityDisposition::Inconclusive { code }` arms build an `Inconclusive` outcome with a profile-supplied code through `new`, which FR-030's rule (inconclusive carries exactly `kani_vacuous_proof`) forbids. When AC-5 adds cause validation to `non_success`, these sites will not reach it. Either route the sites through `non_success` (mapping the error explicitly), or amend FR-044's text and record the in-module constructor and its limit in FR-030 | src/kani/outcome.rs:240-246, src/kani/arithmetic.rs:78-85, src/kani/collections.rs:71-78, src/kani/objects.rs:75-82 |
| FND-004 | low | `Std001Code` derives `Debug` over its buffer, so `{:?}` prints `Std001Code { bytes: [107, 97, 110, ...64 numbers] }` (measured). `KaniOutcome`, `KaniProviderRecord` and `CapabilityDisposition` derive `Debug` too, so every `assert_eq!` failure and debug log on an outcome shows a byte array, not the code. Implement `Debug` as the code string (for example `f.debug_tuple("Std001Code").field(&self.as_str()).finish()`) | crates/quire-contract-model/src/code.rs:97-102 |
| FND-005 | low | `From<DiagnosticCode>` keeps an `Err` arm that returns `INVALID_CODE_FORM`, a wrong value if ever reached. Only a test guards it. `DiagnosticCode::as_str` is a `const fn`, so a `const` assertion that every `DiagnosticCode::ALL` spelling is `well_formed` would turn a future non-conforming variant into a compile error rather than a silently wrong conversion. The arm would still exist, but would be provably dead | crates/quire-contract-model/src/code.rs:226-236 |

## Verdict

The type is well built. Validation is by construction behind a private field, there are no
panics or `unwrap`s in non-test code, the comparison traits agree, the wire bytes are
unchanged, deserialization refuses everything it should, and the macro fails for the
right reasons. The constant set matches STD-001 exactly.

Three medium findings block merge:

- FND-001: AC-6's compile probes are not gated.
- FND-002: `KaniOutcomeError` lacks the location fields STD-001 requires.
- FND-003: the raise sites use the unchecked constructor, against FR-044's text and
  FR-030's direction.

FND-004 and FND-005 are cleanup.

Verdict: changes requested.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | The fix for FND-003 adds `KaniOutcome::raise`. When `non_success` refuses a request, `raise` returns a `refused` outcome whose cause is `kani_outcome_invalid`. That contradicts the spec twice. FR-030 says a refused request "returns a typed `KaniOutcomeError` ... and no outcome; it never substitutes another kind or cause". STD-001 says `kani_outcome_invalid` "never becomes a `KaniOutcome` cause code". This is the retired `kani_outcome_kind_invalid` pattern again, under the registered code. The branch is dead today: all 30 sites pass a literal non-success kind. But (a) a raise site that wrongly passed `Proved` or `Counterexample` would ship a `refused` outcome instead of failing a test or the build, and (b) once AC-5 adds cause checks to `non_success`, every `CapabilityDisposition::Inconclusive { code }` arm will start emitting `refused`/`kani_outcome_invalid` outcomes. That is FR-030's forbidden substitution made live, and nothing in the FR-030 status names it. No test reaches the branch. Fix: make the raise sites infallible by construction instead of by fallback, for example a private non-success kind type that `raise` takes and converts. Alternatively, have the lowerings return the `KaniOutcomeError`, or state the fallback in FR-030 and STD-001 through a spec change | src/kani/outcome.rs:259-282 |

## Dispositions

Round 1 at 165771b183fd96fb18791febd82f2732feb37796, one fix commit on top of fa67cf5.
Main has not moved (7e4dc54). Only the delta was reviewed: `git diff fa67cf5 165771b`.

Gates I ran on 165771b, all passing:

- `make fmt-check` and `make lint`.
- `make test`, exit 0. It ran 367 integration tests, 149 unit tests, 12 model doctests, and the new third step, "Doc-tests quire_contract_ir": `KaniOutcome::non_success` (line 206) compile fail ok, (line 211) compile fail ok, (line 220) ok, 3 passed.

FND-001: the Makefile's `test` target gains `$(CARGO) test $(LOCKED) -p quire-contract-ir --doc`, and CI runs `make test`.

FND-002: `KaniOutcomeError` now holds `requested_kind` and `requested_code`, with `const` accessors. The message names both. `tc_443_*` asserts the code, kind and cause for both a `Proved` and a `Counterexample` request.

FND-003: `new` is private to `outcome.rs`. Every raise site goes through `raise`, which calls `non_success`. The FR-030 status now states the `Inconclusive`-arm conflict. The routing is fixed, but the fallback it added is FND-006.

I checked that the coder's `sed` rename was only that. A `--word-diff` over abi, arithmetic, collections, objects and profile shows exactly 29 `KaniOutcome::new(` → `KaniOutcome::raise(` substitutions and nothing else. One more `Self::new` → `Self::raise` is in `proved_from_checks`.

FND-004: there is now a manual `Debug` that prints `Std001Code("…")`. `tc_442_new_*` asserts that format.

FND-005: a `const _` block asserts that every `DiagnosticCode::ALL` spelling is well formed, so a non-conforming variant fails the build.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-002 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-003 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-004 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |
| FND-005 | fixed | 165771b183fd96fb18791febd82f2732feb37796 |

Round 2 at 9a81ca3db229739b7b1ec848b3d7a2e0516ef2c2, one fix commit on top of 165771b. Main has not moved (7e4dc54).

Scope of the commit: `git diff --stat 165771b 9a81ca3` touches only the seven files under `src/kani/`. No spec, test or Makefile file changed.

FND-006:

- **Inexpressible.** The new private `NonSuccessKind` (`pub(super)`) has exactly the eight non-success variants. `raise` now takes it and is `Self::new(kind.kind(), code, ..)`, with no error path and no substitute. So a `Proved` or `Counterexample` request cannot be written at a raise site: the enum has no such variant. Neither `src/kani/*.rs` outside `outcome.rs` names `KaniOutcomeKind::Proved`, `KaniOutcomeKind::Counterexample` or `kani_outcome_invalid`. All 30 sites name `NonSuccessKind::X`, with the same kinds as before.
- **Mappings.** `kind()` and `TryFrom<KaniOutcomeKind>` are both exhaustive matches with no wildcard. `TryFrom` returns `Err(kind)` for `Proved` and `Counterexample`.
- **Public `non_success`.** It goes through `TryFrom`: `Err` gives a `KaniOutcomeError` with the requested kind and code, and `Ok` calls `raise`.
- **Remaining callers of private `new`.** Only `proved()`, `counterexample()` and `raise`.
- **Public API and wire.** Unchanged. `KaniOutcome`, `KaniOutcomeError` and `KaniOutcomeKind` and their serde derives are untouched, and `NonSuccessKind` is re-imported privately in `mod.rs` with `use`, not `pub use`.
- **Vacuous-proof path.** `proved_from_checks(0, ..)` still builds `Inconclusive` with `kani_vacuous_proof`, and `a_proved_run_with_zero_success_checks_settles_inconclusive_as_vacuous` passes.
- **Mutation.** Mapping `Counterexample` to `Ok(Self::Refused)` in `TryFrom`, applied in my review worktree and reverted afterwards, makes `tc_443_*` FAIL (tests/it/kani_shared.rs:302).

The FR-030 status is unchanged and still true. The `CapabilityDisposition::Inconclusive` arm "still builds an `inconclusive` outcome that AC-5 will forbid". With the fallback gone, nothing turns that into a substituted outcome. The conflict is tracked in Linear as IR-615 (Backlog). The spec does not cite IR-615, which is optional.

Gates I ran on 9a81ca3, all passing:

- `make fmt-check` and `make lint`.
- `make test`, exit 0. It ran 367 integration tests, 149 unit tests and 12 model doctests. "Doc-tests quire_contract_ir" ran 3 tests: two compile-fail probes and one passing doctest.
- `quire validate` passes.
- `quire coverage --strict` reports "23 unbacked row(s) and 0 contradicted status(es)", with 272/293 rows backed.

No regressions and no new findings.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed | 9a81ca3db229739b7b1ec848b3d7a2e0516ef2c2 |
