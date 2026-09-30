---
id: SR-622
title: "code review of PR 227 (IR use-local/use-remote aligned to codegen)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@c2b266dac1a19ee8c3a0920054615864128bf96c; Makefile, scripts/check_one_copy.awk, .gitignore, CLAUDE.md"
review_set: subset
---
# SR-622: code review of PR 227

## Summary

Ticket: IR-358. This PR aligns the use-local/use-remote block and the one-copy awk gate with agent-ix/quire-contract-codegen main (b61c328). One change is IR-only: the Cargo.lock snapshot is taken after LOCAL_PATCHES validation. The diff touches no Rust source, so the rust-review lane does not apply.

Diffed against CG main, the block is identical apart from LOCAL_PATCHES and the snapshot line moving from before the validation loop to after it. `scripts/check_one_copy.awk` is byte-identical to CG's, and the `.gitignore` line matches CG's.

Every behaviour was checked by running it in the worktree:

- entries for one repo given in any order go into one `[patch]` table (tested with `CARGO=true` and an interleaved A, B, A list)
- `CARGO=false` fails, restores the lock and leaves no files
- a patch cargo reports as "was not used" fails (quire-contract-codegen, which IR does not depend on)
- a malformed entry, a missing sibling, or a bad entry after a good one fails and leaves no files under .cargo
- use-local then use-remote with quire-verification-contracts leaves `git status --porcelain --ignored` empty
- the awk works without -F and fails on a duplicated agent-ix entry in a scratch copy of the lock
- `make deny` passes
- SIBLINGS resolves to the main checkout's parent directory from the linked worktree

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | After a successful use-local, re-running it with a malformed or missing-sibling entry deletes .cargo/config.toml but leaves the patched Cargo.lock and the snapshot in place. The metadata-failure paths restore the lock; these two paths do not. The tree is left with no patch, a patched lock (`M Cargo.lock`), and `--locked` switched back on | Makefile:119-129 |
| FND-002 | low | The PR description does not mention its one deliberate divergence from CG (c2b266d, the snapshot moved after validation). CG main still takes the snapshot before validation, so on CG a bad entry leaves an orphaned .cargo/Cargo.lock.pre-local. The two repos stay unaligned until CG gets the same fix | Makefile:131 |

## Finding Detail

- FND-001: Reproduced by running `make use-local LOCAL_PATCHES=quire-verification-contracts:quire-verification-contracts:.` and then `make use-local LOCAL_PATCHES='quire-verification-contracts:quire-verification-contracts:. bogus'`. Afterwards .cargo holds only Cargo.lock.pre-local and `git status` shows ` M Cargo.lock`. `make use-remote` recovers. Fix: move `: > .cargo/config.toml` (line 120) below the validation loop, so the validation paths never touch the existing config. They then need no `rm -f`, a bad entry on a re-run leaves the earlier working patch in place, and a bad entry on a fresh tree still writes no file. The alternative is for both validation branches to restore the lock from the snapshot, as lines 146 and 151 do.
- FND-002: Name c2b266d in the PR body as the intentional divergence, and open a CG ticket to port it so the blocks match again.

## Verdict

Approve once FND-001 is fixed, since it is a one-line move. FND-002 is housekeeping. Everything else matches CG and behaves as described.
