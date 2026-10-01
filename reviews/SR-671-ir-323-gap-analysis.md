---
id: SR-671
title: "PR 241 seam ADs gap analysis: measured claims against IR, QSL and codegen source"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@43967a9b8410c5825ffef77df48ff45bc265f23b; spec/assurance/AD-005-qsl-consumption-seam.md, spec/assurance/AD-006-codegen-consumption-seam.md against IR origin/main src/, crates/quire-contract-model/src/, tests/it/cycle_free_model.rs, deny.toml, Makefile, and QSL and codegen origin/main (read-only)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-005
    type: reviews
  - target: ix://agent-ix/quire-contract-ir/AD-006
    type: reviews
---
# SR-671: PR 241 seam ADs gap analysis

## Summary

Ticket: IR-323. Plan completion: not assessed. This is a spec-only PR, so the gap analysis
re-measures every claim the two ADs make about code. IR was read at origin/main 0a889f9, QSL at
d81193f9, codegen at 2fad745 and runtime at 215e443, the same revisions the ADs name.

These claims were re-verified and hold:
- `src/lib.rs:12` is a glob, and the crate doc calls itself a "compatibility bridge".
- `crates/quire-contract-model/src/lib.rs:31-47` has seven globs.
- `src/kani/mod.rs:19` and `:23` export the three lowerings and `KaniProvider*`, and the lowerings
  are 392 lines across three files.
- `tc_041_bridge_reexports...` is at `tests/it/cycle_free_model.rs:98-107`. The forbidden list
  names neither codegen nor runtime, and the root check is by `qsl-` prefix and git source.
- TC-055 has a matrix document but no test.
- `KaniOutcome` has four `pub` fields and there is no `KaniOutcomeError`. `non_success` rewrites
  `Proved` as `Refused`/`kani_outcome_kind_invalid`.
- There are zero `#[non_exhaustive]`, and the `PROFILE` constant is at `mod.rs:29`.
- `deny.toml` `[bans]` has no `deny` list, `make ci` includes `deny`, and this PR leaves
  `deny.toml` untouched.
- QSL `Cargo.toml:75` and `qsl-package/Cargo.toml:36`: 57 non-test files name the model crate,
  and `ir::ValueType` appears 212 times.
- QSL `checked_v2.rs:94-98` and `for_ir` at `:223`. TC-440 is `cfg(test)`, with one ignored case.
- QSL's `arch-lint-direction` is not in its `ci` target, and `emit.rs:961` says "pinned IR
  revision".
- Codegen `Cargo.toml:17`. No codegen source names `quire_contract_model` or `KaniProvider*`. The
  three one-line wrappers are at `:18`/`:19`, the string comparison at `kani_execution.rs:690`,
  and the stale `deny.toml:30` comment. Codegen's lock holds both IR packages at one commit and
  no `quire-spec-language`.
- IR-343, IR-274 and IR-347 exist in Linear.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AD-005 says no terminal-record type comes from IR, and states D-3 as an existing statement without saying it is untrue today. AD-006:61-62 says the same. Measured: the root crate exports `KaniProviderRecord`, whose doc reads "One FR-331 terminal record for a Kani run", and `KaniProviderResult`, the "QSpec FR-331 results values". Both are at `src/kani/outcome.rs:56-89` and are re-exported at `src/kani/mod.rs:23`. FR-039 "Items QSL owns" lists both. This is the FR-331 terminal record that the IR-323 owner ruling assigns to QSL, so the QSL seam AD states the target as the current state. AD-004 (PR 239) and AD-006's own divergence 2 record the types as still exported | spec/assurance/AD-005-qsl-consumption-seam.md:59-61,136-137 |
| FND-002 | medium | R3-Q1 says the dependency key `quire-contract-ir` with `package = "quire-contract-model"` "names the root crate, which QSL must not depend on". Cargo resolves `package =` to the model package. QSL's `Cargo.lock` holds `quire-contract-model` and no `quire-contract-ir` package, so QSL does not depend on the root crate. The AD's own edge table (line 91) says correctly that the key only "reads as" the root crate. The rename is a fair readability request, but the stated reason is false | spec/assurance/AD-005-qsl-consumption-seam.md:195 |
| FND-003 | low | AD-006 lists "IR to QSL: held by `tc_041` and the cargo-deny `bans` entries", gap "none". No `bans` deny entry exists: `deny.toml` `[bans]` has only `multiple-versions` and `wildcards`. AD-005 Decision D is written in the present tense ("`deny.toml` `[bans]` lists those crates under `deny`"), which also reads as current | spec/assurance/AD-006-codegen-consumption-seam.md:101; spec/assurance/AD-005-qsl-consumption-seam.md:119-122 |
| FND-004 | low | The AD calls the model crate "layer 4's one external edge", citing QSL ADR-011 §6.1. ADR-011's X-7 row lists `qsl-package`'s dependencies as `qsl-semantics`, `qsl-foundation`, `quire-exact`, `quire-contract-model`, `quire-canonical` and `thiserror`, so `quire-canonical` is a second external first-party edge | spec/assurance/AD-005-qsl-consumption-seam.md:90 |
| FND-005 | low | Two citations are imprecise. `package_id` is recomputed at `crates/quire-contract-model/src/checked_package/v2/mod.rs:759-760`, as `serde_json::to_value` of the typed preimage struct followed by `digest_json`, not "of a parsed value". QSL `checked_v2.rs:53` is a doc-comment line, not the definition of `V2ReadIncomplete::Limit` | spec/assurance/AD-005-qsl-consumption-seam.md:74-76,101 |

## Verdict

Changes needed. Almost every measured claim holds, including the hard ones: the bridge, the
globs, the lowerings, the `KaniOutcome` gaps, the missing deny guard and the encoder pair.
FND-001 is the one high finding: AD-005 shows the owner-ruled FR-331 terminal record as already
absent from IR. FND-002 gives a false reason for a routed need.
