---
id: SR-15312
title: "IR-641 IR consumer specification base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; IR-641; spec/, src/, crates/, tests/, Makefile, AGENTS.md, CLAUDE.md"
review_set: subset
---

## Summary

All three changed spec files were examined. FR-055 AC1–5 are falsifiable, delegate the algorithm to the existing single CLI owner, cover coherent/duplicate/split/malformed/missing-binary cases, and preserve other policy checks and committed dependencies. No implementation or installed-host evidence is claimed.

Ticket: IR-641. Reviewed PR #499, exact frozen candidate and fresh main range verified outside this artifact. The change is specification-only.

## Verdict

**PASS** — No scoped defect found in the planned consumer acceptance contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

No applicable AssuranceProfile with required review selection was found in the spec. No active exception was granted. CLI FR-027 was read from its published source; no copying of its checker logic is proposed. User-authorized dependency preservation and script-removal inspection are explicit task constraints, not a new hash/pin catalog.

All five new acceptance criteria and AD-007 O-4 were examined. Test coverage is pending consumer implementation and is reported without waiver by the separate full gap analysis. No source or Cargo diff exists; Rust execution lanes are not applicable to this stage.

## Examined Scope

```yaml
scope:
- id: FR-055
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: When a developer runs `make deny`, quire-contract-ir shall execute the
    released `quire check-lock Cargo.lock` command and propagate its failure to Make.
- id: FR-055-AC-1
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: The normal `make deny` recipe invokes the released `quire check-lock Cargo.lock`
    command on this repository's committed lock. On a coherent scratch lock the real
    command succeeds and the recipe completes when its other policy checks pass; an
    invocation record identifies the executable and argument without substituting
    checker logic.
- id: FR-055-AC-2
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: Through the same normal `make deny` recipe, a scratch lock with duplicate
    first-party Git package names and a scratch lock with distinct crate names at
    split resolved revisions of one repository each cause a non-zero Make exit and
    the real command's coherence diagnostic. The aligned multi-crate control passes
    the command. Every agent-ix Git source in these coherence controls has a valid
    full resolved fragment admitted by CLI FR-027; a malformed-input diagnostic cannot
    satisfy a duplicate or split-revision control. These controls exercise CLI FR-027;
    they do not define a second normalization al
- id: FR-055-AC-3
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: A missing checker executable, malformed scratch lock, or checker refusal
    cannot produce successful `make deny`. The recipe propagates failure without an
    ignored exit, success fallback, or skipped check.
- id: FR-055-AC-4
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: The committed `scripts/check_one_copy.awk` file is absent, the normal recipe
    contains no reference to it, and no replacement local implementation, executable
    copy, compatibility shim, or revision allowance bypasses the owned command.
- id: FR-055-AC-5
  path: spec/core/functional/FR-055-owned-lock-coherence-gate.md
  role: examined
  excerpt: Adoption leaves the committed Cargo manifests and lock byte-for-byte unchanged
    and adds no runtime dependency on developer tooling. Qualification runs the real
    command on the committed lock and reports any existing incoherence as a failing
    gate with an owning blocker; a scratch passing control or an unrelated gate failure
    cannot qualify that lock.
- id: AD-007-O-4
  path: spec/assurance/AD-007-cross-repo-dependency-graph.md
  role: examined
  excerpt: O-4. Selected owner; consumer adoption PLANNED. IR-736 selected quire-cli's
    released `quire check-lock` command as the sole developer-tool owner;
- id: spec.md-developer-gate-adoption
  path: spec/spec.md
  role: examined
  excerpt: FR-055 defines planned consumption of the single owned CLI lock coherence
    gate under IR-641. It does not claim consumer implementation or qualification.
```
