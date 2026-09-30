---
id: SR-623
title: "gap analysis of PR 227 (IR use-local/use-remote aligned to codegen)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir@c2b266dac1a19ee8c3a0920054615864128bf96c; Makefile, scripts/check_one_copy.awk, .gitignore, CLAUDE.md, .github/workflows/ci.yml (context)"
review_set: subset
---
# SR-623: gap analysis of PR 227

## Summary

Ticket: IR-358. This is a planless gap analysis, proportional to a build-tooling PR, so plan completion was not assessed. No FR or AC in `spec/` governs the use-local/use-remote targets or the one-copy gate. `deny.toml:21` is the only other place that mentions the gate, and it still describes it correctly. The CLAUDE.md command lines agree with the implemented behaviour: the snapshot, the restore, the two failure conditions, and "no snapshot: lock untouched".

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | CI never runs the one-copy gate. The Supply-chain job runs `cargo deny check`, not `make deny`, so `scripts/check_one_copy.awk`, IR-358's guard, runs only locally through `make deny` and `make ci`. This predates the PR: ci.yml is unchanged | .github/workflows/ci.yml:53-54 |

## Finding Detail

- FND-001: Change the step to `run: make deny`, or add a step `awk -f scripts/check_one_copy.awk Cargo.lock`. The PR does not touch ci.yml, so this can be deferred to a follow-up ticket.

## Scope

- `Makefile use-local/use-remote`, examined: behaviour matches the CLAUDE.md command docs.
- `scripts/check_one_copy.awk`, examined: sets FS itself, and has both clean and duplicate controls.
- `CLAUDE.md` use-local/use-remote lines, examined: accurate.
- `deny.toml:21` gate comment, context_only.
- `.github/workflows/ci.yml` supply-chain job, examined (FND-001).

## Verdict

No gap is introduced by this PR. The one pre-existing gap is that CI does not run the gate this PR edits.
