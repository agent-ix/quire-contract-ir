---
id: SR-617
title: "spec review of PR 225 (FR-028, FR-039, AD-001 after IR drops QSL)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-contract-ir@dbc07f9fe556f25fec0e3b3851628dd8ca459385; spec/contract/FR-028, spec/interface/FR-039, spec/assurance/AD-001, spec/contract-test-matrix.md, with FR-031, FR-037, TC-055, TC-223, AD-002 as context"
review_set: subset
---
# SR-617: spec review of PR 225

## Summary

Ticket: IR-358. This is a spec-integrity review of the edits to FR-028, FR-039, AD-001 and the matrix. The FR-028 statement and dependency diagram, and the FR-039 "Items QSL owns" list, now agree with the code. The PR did not follow the new rule, that the root package depends on no QSL crate, into the requirements that still need the root crate to name `qsl_replay::TerminalValue`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-039-AC-3 and the FR-039 Public items `kani` row require the root crate to expose a public function returning `qsl_replay::TerminalValue`. The new FR-028 statement forbids any QSL crate dependency, and this PR's own FR-039 paragraph says the map's host "must depend on both sides". FR-039 contradicts itself, and FR-031-AC-5 (TC-223), TC-055's outcome-map probe and AD-002 still give that map to Contract IR, where it can no longer be written | spec/interface/FR-039-root-crate-public-interface.md:105 |
| FND-002 | medium | The AD-001 owner table still says "the root crate depends on QSL, names QSL's replay types", which contradicts FR-028 and the AD-001 Kani-boundary paragraph edited in this same PR | spec/assurance/AD-001-contract-ir-architecture.md:66 |
| FND-003 | low | FR-028 "Dependency and admission rules" still says the workspace Cargo graph SHALL have the direction `quire-canonical + quire-exact + quire-contract-model -> quire-spec-language`. QSL is no longer in this workspace's graph | spec/contract/FR-028-separate-cycle-free-contract-model.md:47 |
| FND-004 | low | FR-028 names the prohibited dependencies two ways. The statement says "no `quire-spec-language` or `qsl-*` crate" (crate names), while the package bullet says "nothing from `quire-spec-language`" (the repository, which also ships `quire-exact`). Two implementers would test different sets | spec/contract/FR-028-separate-cycle-free-contract-model.md:17 |

## Finding Detail

- FND-001: This needs an owner decision, and the plan does not cover it. Adding only `qsl-replay` would bring back the path/git model split, because qsl-replay -> qsl-eval -> qsl-package -> quire-contract-model, so the map has to leave this repo. Name its host (codegen is the obvious consumer) and move the text: FR-031-AC-5 and its map table, the FR-039 `kani` table row and AC-3's second clause, TC-055's outcome-map probe, TC-223, and the AD-002 boundary sentence. Until then, mark them retargeted rather than planned-here.
- FND-002: Change the relation cell to "QSL depends on `quire-contract-model`; the root crate depends on no QSL crate (FR-028)".
- FND-003: Move the QSL line out of the SHALL block into context prose, or drop it.
- FND-004: Choose the repository form ("no crate from agent-ix/quire-spec-language") and use it in the statement, the bullet and AC-3.

## Scope

- `FR-028`, examined: The Contract IR repository SHALL expose its stable semantic substrate only from a dependency-free `quire-contract-model` package, which the `quire-contract-ir` package consumes without re-exporting it, so the production Cargo graph remains acyclic, the root package depends on no `quire-spec-language` or `qsl-*` crate, and every model item has one import path. (FND-003, FND-004)
- `FR-028-AC-3`, examined: coherent with the rewritten tc_041 for its manifest clause. The untested QSL-graph clause is SR-616 FND-001.
- `FR-039-AC-3`, examined: Code naming any item the "Items QSL owns" or "Items codegen owns" section lists through `quire_contract_ir` fails to compile, and the `kani` outcome map returns a `qsl_replay::TerminalValue` for every `KaniOutcome`. (FND-001)
- `FR-039` "Items QSL owns", examined: the `Native*` types and `replay_with_native_runtime` were removed from the list and from the code in step. This is clean.
- `AD-001` owner table and Kani boundary, examined. (FND-002)
- `FR-031-AC-5`, `TC-055`, `TC-223`, `AD-002`, context_only: each still assigns the TerminalValue map to Contract IR.
- `FR-037-AC-6`, context_only: the root has no `replay` module. This is pre-existing planned work: `src/kani/replay.rs` remains, and this PR only narrows it.
