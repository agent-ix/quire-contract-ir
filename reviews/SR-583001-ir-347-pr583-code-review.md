---
id: SR-583001
title: "IR-347 PR #583 code-review (spec-only status closeout; source facts checked)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; PR #583 against origin/main; spec/assurance/AD-001-contract-ir-architecture.md, spec/assurance/AD-006-codegen-consumption-seam.md, spec/core/matrix/tests.md, spec/kani/matrix/tests.md, spec/tests.md; facts checked in src/kani/outcome.rs, tests/it/kani_outcome_tc223.rs, tests/it/kani_shared.rs, tests/it/main.rs; IR-347"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-030
    type: reviews
---

## Summary

Ticket: IR-347. This is the independent Opus review of draft PR #583 at head
, scoped to `git diff origin/main...HEAD`: three
commits and five Markdown files, 14 lines added and 14 removed. The diff changes no `.rs`,
`Cargo.toml` or other production file, so the `rust-review` lane and `gap-analysis` do not
apply. This code-review checks each source claim the new prose makes against the code at head.

Claims checked and found true at head:

- `KaniOutcome`'s fields are private. `src/kani/outcome.rs:152-166` carries a
  `compile_fail,E0451` struct-literal doctest. This backs AD-006 G-3 and the FR-030-AC-5
  struct-literal clause.
- `KaniOutcome::proved` takes a count. Zero returns `KaniOutcomeError` through
  `NonZeroUsize::new` (`src/kani/outcome.rs:232-252`). `KaniOutcomeError::code` is the constant
  `KANI_OUTCOME_INVALID` (`:75-77`). This backs AD-006 G-4.
- `tests/it/kani_outcome_tc223.rs` is wired by `tests/it/main.rs:73`. It holds four tests tagged
  `TC-223` with `FR-030-AC-4` and/or `FR-030-AC-5`. They cover the count of three, the
  zero-count vacuous classification, the zero-count `proved` refusal, the two absence causes,
  the invalid broad `non_success` requests, the exhaustive kind match and the wire-reader refusals.
- `tc_443_the_outcome_and_its_error_carry_a_typed_std001_code`
  (`tests/it/kani_shared.rs:538-577`) now reads the zero-count `proved` error code. It also
  checks the vacuous-proof code and wire member, the Boolean-kind errors and the `Bad-Code`
  refusal. These are the facts the new TC-443 row states.
- `quire matrix --scope.` at head reports FR-030-AC-4, AC-5 and AC-6 as `tagged`, at the test
  lines above.
- Codegen `origin/main` has `src/kani/terminal.rs`, which maps an IR `KaniOutcome` to
  `qsl_replay::TerminalValue`. IR `src/kani/` has no such map. The AD-001 ownership edit is
  correct.

No hash, digest, pin or compatibility shim is added. No Cargo, Kani or full-test gate was run,
per the brief.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean for this method. Every source fact the five edited files state about
`src/kani/outcome.rs`, `tests/it/kani_outcome_tc223.rs` and `tests/it/kani_shared.rs` holds at
head. The status-consistency gaps outside the diff are recorded under SR-583003.

## Coverage

```yaml
scope:
- id: src/kani/outcome.rs
  path: src/kani/outcome.rs
  role: examined
  excerpt: "pub struct KaniOutcome { kind: KaniOutcomeKind, code: Std001Code, source_id: String, context: Box<str>, success_checks: Option<NonZeroUsize> }"
- id: tests/it/kani_outcome_tc223.rs
  path: tests/it/kani_outcome_tc223.rs
  role: examined
  excerpt: "#[trace(\"TC-223\", \"FR-030-AC-4\")] fn proof_count_is_retained_and_zero_is_refused_or_classified_vacuous"
- id: tests/it/kani_shared.rs
  path: tests/it/kani_shared.rs
  role: examined
  excerpt: "#[trace(\"TC-443\", \"TC-055\", \"FR-030-AC-6\", \"FR-039-AC-4\")] fn tc_443_the_outcome_and_its_error_carry_a_typed_std001_code"
- id: quire-contract-codegen:src/kani/terminal.rs
  path: src/kani/terminal.rs
  role: context_only
  excerpt: "The terminal-value maps of a Kani run (FR-029) and of a Contract IR outcome (FR-030): one total match each... to QSL's FR-331 terminal value."
```
