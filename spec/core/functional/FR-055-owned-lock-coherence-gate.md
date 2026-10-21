---
id: FR-055
title: "Consume the owned lock coherence gate"
type: FR
relationships:
  - target: ix://agent-ix/quire-cli/FR-027
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-007
    type: references
---
# FR-055: Consume the owned lock coherence gate

## Description

When a developer runs `make deny`, quire-contract-ir shall execute the released
`quire check-lock Cargo.lock` command and propagate its failure to Make.
[CLI FR-027](ix://agent-ix/quire-cli/FR-027) owns the algorithm, diagnostics and
installation contract. This requirement owns only the consumer wiring and removal
of the copied checker under IR-641. It introduces no product API or dependency edge.

## Behavior

The repository shall remove `scripts/check_one_copy.awk` without retaining a copied
checker or compatibility route. The repository shall retain its other dependency-policy
checks. The adoption change shall leave the committed dependency manifests and lock
unchanged; an existing incoherent lock remains a failure until its owner resolves it
in separate work. No repository-specific revision allowance is introduced.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-055-AC-1 | The normal `make deny` recipe invokes the released `quire check-lock Cargo.lock` command on this repository's committed lock. On a coherent scratch lock the real command succeeds and the recipe completes when its other policy checks pass; an invocation record identifies the executable and argument without substituting checker logic. | Test |
| FR-055-AC-2 | Through the same normal `make deny` recipe, a scratch lock with duplicate first-party Git package names and a scratch lock with distinct crate names at split resolved revisions of one repository each cause a non-zero Make exit and the real command's coherence diagnostic. The aligned multi-crate control passes the command. Every agent-ix Git source in these coherence controls has a valid full resolved fragment admitted by CLI FR-027; a malformed-input diagnostic cannot satisfy a duplicate or split-revision control. These controls exercise CLI FR-027; they do not define a second normalization algorithm. | Test |
| FR-055-AC-3 | A missing checker executable, malformed scratch lock, or checker refusal cannot produce successful `make deny`. The recipe propagates failure without an ignored exit, success fallback, or skipped check. | Test |
| FR-055-AC-4 | The committed `scripts/check_one_copy.awk` file is absent, the normal recipe contains no reference to it, and no replacement local implementation, executable copy, compatibility shim, or revision allowance bypasses the owned command. | Inspection |
| FR-055-AC-5 | Adoption leaves the committed Cargo manifests and lock byte-for-byte unchanged and adds no runtime dependency on developer tooling. Qualification runs the real command on the committed lock and reports any existing incoherence as a failing gate with an owning blocker; a scratch passing control or an unrelated gate failure cannot qualify that lock. | Test |

## Qualification Status

PLANNED: these criteria precede the consumer implementation. This document neither
removes the current script nor claims an installed-host receipt or passing consumer
gate. Qualification must exercise the normal Make recipe and released binary, not
only inspect recipe text or run the command outside the recipe.

## Dependencies

- [CLI FR-027](ix://agent-ix/quire-cli/FR-027) owns the released developer command.
- IR-641 owns consumer implementation and qualification; existing policy gates retain their ownership.

## Consumer Verification Plan

PLANNED: repository-owned integration tests bind each new criterion on actual
`#[test]` functions. The tests run the normal `make deny` recipe in a scratch
consumer tree with the actual released Quire executable. They may isolate the
unrelated Cargo policy stage to make the checker observation decisive, but may
not substitute the checker, copy its algorithm, or treat a malformed-input refusal
as coherence evidence. Final qualification separately runs the repository's
normal unchanged gates on its committed lock.
