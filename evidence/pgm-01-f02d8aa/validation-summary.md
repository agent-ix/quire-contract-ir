# PGM-01 retained validation summary

Subject revision: `f02d8aa3aad386e1d68b65df3db33ee6e57b4cfa`

This is the only evidence record in the candidate tree. Earlier records were
removed before merge because their claims exceeded what their verifier proved.
A squash merge excludes those intermediate commits from `main` ancestry.

The replacement verifier is squash-safe: it independently enumerates the
current committed non-evidence tree, requires the manifest to name that exact
set, checks all 66 digests against both `HEAD` and the worktree, authenticates
every evidence output to `HEAD`, validates the manifest against its published
Draft 7 schema, and rejects multiple matching records. A real temporary-Git
integration test proves that verification does not depend on source-revision
ancestry and that a newly committed, uncovered input is rejected.

Local verification on 2026-08-30 (America/Los_Angeles):

- `make ci`: pass—13/13 corpus cases, 7/7 mutation probes, six Python tests,
  14 Rust tests, formatting, lint, licenses, and unsafe audit.
- `make spec`: pass—21/21 documents grammar-clean and 28/28 traceability rows
  backed.
- Code review: three ultrareview passes and two focused passes. The final
  focused pass produced nine findings; all nine were remediated in the subject.
- Gap analysis: pass—4/4 planned tasks done and no in-scope reverse gap.
- Branch-protection API: required CODEOWNER review and conversation resolution
  are enabled; required `Rust Checks` and `License Check` contexts remain
  configured; enforce-admins currently reports disabled.
- GitHub Actions: intentionally disabled by operator policy. No workflow was
  dispatched, and no remote-check success is claimed.

The manifest records exact commands, environment/tool identities and digests,
individual outcomes, limitations, the complete input checksum set, and the
retained output set. The sibling checksum file covers the manifest, this
summary, and the branch-protection snapshot without self-reference.

This record does not authorize a source release. It records readiness for the
operator-authorized admin squash merge of PR #12 only.
