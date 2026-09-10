---
id: SR-036
title: "Independent code review — exact Rust 1.98.1 qualification at a12ba79"
type: SpecReview
analysis: code-review
scope: "PR #62 at a12ba79; NFR-005; ADR-0055; tests/toolchain_policy.rs; Makefile; .github/workflows/ci.yml; deny.toml; Cargo.lock; src/canonical.rs; SR-035 FND-1512"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-contract-ir/spec/nonfunctional/NFR-005"
    type: reviews
---

## Summary

The compiler migration itself is sound and its supply-chain half is genuinely
good: I confirmed by control that `origin/main`'s lock carries two real
vulnerabilities and this branch's carries none. Every Rust gate reproduces —
fmt, Clippy, 47 tests, deny, audit, unsafe — and `tests/toolchain_policy.rs`
carries eleven of its own mutation controls. The problem is the "Honest
remaining red gate" section. Its 9-failed figure reproduces exactly, but only
when `quire` and `quoin` are absent from `PATH`; with the pinned CLIs present
the suite is 1 failed / 20 passed, and the stated cause — a `0.21.0` floor that
the installed `quire 0.31.0` supposedly exceeds — does not exist anywhere in
the code. NFR-005 already names the real blocker in this same PR.

## Verdict

**FAIL** — one high. Not for the compiler change, which I would pass on its own
evidence, but because the PR's disclosed blocker is misattributed, and the
attribution routes a live problem to the wrong owner.

## Gates run at `a12ba79`

Exact Rust 1.98.1 (`rustc 1.98.1 (48a229cea 2026-09-01)`), `-j 2`, isolated
target directory.

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --locked --all-targets -- -D warnings` | pass |
| `cargo test --locked -- --include-ignored` | pass — **47 passed, 0 failed, 0 ignored** across 10 suites; matches the PR body exactly |
| `cargo deny check` | advisories, bans, licenses, sources ok — two non-failing configuration warnings, as stated |
| `cargo audit` | pass — 121 crate dependencies, **zero advisories** |
| `bash scripts/check_unsafe_comments.sh` | pass |
| `make assurance-inputs` | pass — conformance JSONL and `quire coverage --json` both produced |
| `make assurance-env` | pass |
| `python3 -m pytest` (fully provisioned) | **1 failed, 20 passed** — see FND-001 |

## Independently verified

| Claim | Check | Result |
| --- | --- | --- |
| The lock fixes real vulnerabilities | ran `cargo audit` against `origin/main`'s lock as a control | **2 vulnerabilities found** — `idna 0.4.0` RUSTSEC-2024-0421 and `time 0.3.36` RUSTSEC-2026-0009 (6.8 medium). At this head: zero |
| `idna` and `time` advance as stated | lock diff | `0.4.0` → `1.1.0`, `0.3.36` → `0.3.55`; both above the advisories' `Solution` floors (`>=1.0.0`, `>=0.3.47`) |
| The four Clippy 1.98.1 migrations are behaviour-preserving | read all four | `chunks_exact(2)` → `as_chunks::<2>().0.iter()` is equivalent under the `value.len() != 64` guard directly above it, which leaves no remainder and no `bytes[index]` overrun; the three `repeat(x).take(n)` → `repeat_n(x, n)` sites are exact |
| No allowance was added to pass | diff | no `#[allow]` anywhere; `lint` and `deny` were **strengthened** (`--locked`, `deny check licenses` → full `deny check`) |
| The AGPL exception is not novel | compared `deny.toml` with `agent-ix/quire-rs` | quire-rs carries the identical `ix-trace-rs` AGPL exception and the same `allow-git` entry; this follows the established ecosystem pattern rather than widening policy for this PR |
| `ci` gained real gates | Makefile diff | `msrv` → `supported-rust` + `qualification-rust`, plus `cargo-audit`, and every `cargo` invocation gained `--locked` |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | SR-035 FND-1512 and the PR body misattribute the red gate: the 9 failures are absent `quire`/`quoin` on `PATH`, not a version-floor comparison, and the named `0.21.0` floor does not exist | reviews/SR-035-rust-1.98.1-code-review.md:47, scripts/check_shared_pins.py:88 | correct-requirement-no-evidence |
| FND-002 | low | The workflow pins the toolchain action by full SHA and leaves three other actions on mutable tags, including the one that installs the audit tools | .github/workflows/ci.yml:18 | missing-requirement |
| FND-003 | low | `tc_036` asserts an exact count of two `toolchain: 1.98.1` occurrences, so adding a correctly pinned third Rust job fails the policy test | tests/toolchain_policy.rs:73 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the disclosed blocker is not the blocker

The PR body says:

> The full Python/shared-assurance suite is 12 passed / 9 failed because the
> pinned Quoin path cannot determine installed `quire 0.31.0` despite its
> stated `>=0.21.0` floor […] SR-035 records this as FND-1512.

SR-035 FND-1512 records it as **high, OPEN**, "external to this compiler
change", owned by Engineering Assurance #59 with consumer migration in
quire-research #60.

I reproduced the number and then the cause, and they are different things.

**The count is a property of the environment, not the branch.** Running the
same suite at the same head, changing only how much of the toolchain is
provisioned:

| Environment | Result |
| --- | --- |
| bare `pytest`, no producer inputs, no assurance venv | 12 failed, 9 passed |
| after `make assurance-inputs` | 4 failed, 17 passed |
| after `make assurance-env` as well — fully provisioned | **1 failed, 20 passed** |
| fully provisioned, but `quire`/`quoin` removed from `PATH` | **9 failed, 12 passed** |

The last row is the PR's figure, exactly. The suite is honest about why —
`test_shared_assurance.py` refuses to skip, with the comment "a gate that
stands down when its dependency is absent reports the same green as one that
ran" — so the 9 failures are that refusal firing, not a defect in a dependency.

**The stated cause does not exist.** `make pins` with the CLIs on `PATH`:

```
compatible   quire-cli   quire-cli 0.31.0 is the pinned version
compatible   quoin       quoin 0.23.1 is the pinned version
unknown      ix-flow     ix-flow 0.2.3 is not the pinned 0.0.4 and this matrix has never seen it
compatible   engineering-assurance   engineering-assurance 0.2.0 is the pinned version
```

and with them removed:

```
unknown      quire-cli   quire-cli was not observed; nothing was checked against the pin
unknown      quoin       quoin was not observed; nothing was checked against the pin
```

Three specifics, each measurable:

1. **There is no `0.21.0` floor.** `grep -rn "0.21.0"` across `scripts/`,
   `tests/` and the installed `engineering_assurance` distribution returns
   nothing. `compatibility-matrix.json` pins `quire-cli` at exact version
   `0.31.0` and classifies by equality, not by a floor.
2. **`quire 0.31.0` classifies `compatible`** — "is the pinned version" — the
   opposite of "cannot determine".
3. **The mechanism is observation, not comparison.**
   `check_shared_pins.py:88` `observe_quire()` shells out to `quire provenance`
   and returns `None` when `shutil.which("quire")` is `None` or the command
   exits non-zero. "Not observed; nothing was checked against the pin" is the
   matrix's `absent_is_not_pass` rule firing.

**The repository already says what the real blocker is.** This same PR's
`NFR-005` "Known non-compatibility findings" reads:

> Downstream shared-assurance matrix disagreement for `ix-flow` remains
> separate and shall not cause a compiler reversion.

That is correct and it matches my fully provisioned run: the one remaining
failure is `ix-flow 0.2.3` against a matrix pin of `0.0.4`, plus the matrix's
own `"state": "pending_human_acceptance"`, which makes `make pins` report
"toolchain gate: NOT satisfied" regardless of versions. So the spec and the
review disagree about the blocker, and the spec is right.

Why this is high rather than a wording correction: FND-1512 assigns ownership
on the strength of the stated cause. A quire-version classification defect
would belong to Engineering Assurance #59; an `ix-flow` version outside an
unaccepted matrix belongs to whoever owns the matrix entry and the human
acceptance. Filed as it stands, the real item is routed somewhere it will not
be found, and a `high` OPEN finding sits against this repository for a defect
that is not there.

The correction is small and makes the disclosure stronger, not weaker: state
that the suite is 1 failed / 20 passed with the pinned CLIs present, that the
single failure is the `ix-flow` matrix disagreement NFR-005 already names, and
that the gate is additionally held by `pending_human_acceptance` — which is a
human decision, exactly the kind of blocker that should be visible.

### FND-002 — one action pinned by SHA, three left on tags

The diff correctly replaces `dtolnay/rust-toolchain@stable` with a full commit
SHA plus an explicit `toolchain: 1.98.1`, in both jobs. In the same file:

```yaml
- uses: actions/checkout@v4
- uses: Swatinem/rust-cache@v2
- uses: taiki-e/install-action@v2
  with:
    tool: cargo-deny
```

`taiki-e/install-action@v2` is the step that installs `cargo-deny` and
`cargo-audit` — the two tools the renamed `supply-chain` job exists to run — and
`v2` is a mutable tag.

Pre-existing, and this PR strictly improves the file, which is why it is low.
Recorded because the sibling repository's drift audit rejects exactly this:
`agent-ix/quire-rs` `scripts/audits/check_tool_drift.sh` requires a full 40-hex
SHA on every workflow `uses:`, with a parametrized test controlling the `v4`
case. `tests/toolchain_policy.rs` asserts `!workflow.contains("rust-toolchain@stable")`
and could assert the same SHA rule in one more line.

### FND-003 — an exact-count assertion on a growing file

```rust
if self.workflow.matches("toolchain: 1.98.1").count() != 2
```

Two is the number of Rust jobs today. A third job, pinned correctly to 1.98.1,
fails `tc_036` — and the failure message would be "hosted compiler
declaration", which points at the wrong thing.

It fails closed, which is the safe direction, so this is low. `>= 2` plus the
existing `rust-toolchain@stable` refusal states the same policy without
counting jobs; better still, count `dtolnay/rust-toolchain` occurrences and
require each to be followed by the pinned version, which is what the audit is
actually about.

## What is correct

- **The supply-chain half is verified in both directions.** The control run
  against `origin/main`'s lock reports two vulnerabilities; this head reports
  zero. That is the strongest form of the claim and it holds.
- **`tests/toolchain_policy.rs` tests its own discrimination.** `tc_036`
  constructs eleven mutated policy surfaces — inherited 1.75 minimum, stale
  Clippy minimum, floating stable toolchain, inconsistent supported/qualification
  pair, floating hosted compiler, inconsistent assurance version, stale sealed
  assurance input, stale attestation identity, unaccepted decision, missing
  response window, unbounded hold — and asserts each is caught. That is the
  right pattern, and it covers surfaces a source-inspection test usually
  misses, including `assurance/change-assurance.json` and
  `scripts/assurance_chain.py`.
- **The gates were strengthened, not relaxed, to pass.** `make deny` went from
  `cargo deny check licenses` to the full `cargo deny check`; `make ci` gained
  `cargo-audit`, `supported-rust` and `qualification-rust`; every Cargo
  invocation gained `--locked`; the hosted `licenses` job became `supply-chain`
  and now runs both tools. `tc_037` asserts each of those Makefile operations by
  string, so the composite target cannot quietly lose one.
- **The `rustfmt.toml` deletion is honest cleanup.** `imports_granularity` and
  `group_imports` are nightly-only and were being ignored with a warning on
  every stable run — I saw the identical warnings in two sibling repositories
  that still carry them.
- **The refusal-not-skip discipline is right.** Both
  `assurance_interpreter()` and `chain_report()` raise rather than skip when
  their inputs are absent, each with a comment saying why. That is what made
  FND-001 measurable at all: an under-provisioned run is loud instead of green.

## One thing I could not verify

ADR-0055 moves from `proposed` to `accepted` with "**Accepted by the owner on
2026-09-08.**" and `owner: kreneskyp`. `tc_036` asserts the string
`status: accepted` is present, which tests the file and not the acceptance. That
is inherent to a governance record and not a defect — recorded only so it is
clear this review did not, and could not, confirm the acceptance itself.
