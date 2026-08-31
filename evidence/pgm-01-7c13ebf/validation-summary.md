# PGM-01 retained validation summary

Subject revision: `7c13ebf87127cab0dbc7f276253875de19e85ebe`

This is the only evidence record in the candidate tree. Earlier records were
removed before merge because their claims exceeded what their verifier proved.
A squash merge excludes those intermediate commits from `main` ancestry.

The replacement verifier independently enumerates the committed non-evidence
tree and the non-ignored worktree set, requires the manifest to name the exact
66-file set, authenticates every evidence output to `HEAD`, validates the
manifest against its published Draft 7 schema, and accepts exactly one matching
record. Integration tests cover missing source ancestry, committed and
untracked added inputs, stale/zero/multiple record selection, and real Git and
filesystem behavior.

Local verification on 2026-08-30 (America/Los_Angeles):

- `make ci`: pass—13/13 corpus cases, 7/7 mutation probes, seven Python tests,
  14 Rust tests, formatting, lint, licenses, and unsafe audit.
- `make spec`: pass—21/21 documents grammar-clean and 28/28 traceability rows
  backed.
- Code review: three ultrareview passes and three focused passes. Every reported
  finding was remediated before this record.
- Gap analysis: pass—4/4 planned tasks done and no in-scope reverse gap.
- Squash simulation: pass with the subject explicitly not an ancestor of the
  simulated `main`; evidence verification remained successful.
- Branch-protection API: required CODEOWNER review and conversation resolution
  are enabled; required `Rust Checks` and `License Check` contexts remain
  configured; enforce-admins reports disabled. The named human authority's
  bounded PR #12 admin-merge exception is retained separately.
- GitHub Actions: intentionally disabled by operator policy. No workflow was
  dispatched, and no remote-check success is claimed.

The evidence is self-attesting: the committed checksum graph proves byte
consistency, not independent execution of commands or truth of narrative
outcomes. The pull-request subject may become unreachable after branch deletion;
the squash-safe verifier deliberately depends on the byte-identical current
non-evidence tree instead. No source tag is created because source release is a
separate human decision.

This record authorizes no source release or project qualification decision. It
records readiness for the bounded operator-authorized admin squash merge of PR
#12 only.
