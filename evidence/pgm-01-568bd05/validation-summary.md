# PGM-01 retained validation summary

Subject revision: `568bd05508c0a18095538788b980f301966e8639`

This is the only evidence record in the candidate tree. Superseded records were
removed before merge because their claims exceeded what their verifier proved;
the required squash excludes those intermediate commits from `main` ancestry.

The verifier independently enumerates the committed non-evidence tree and the
non-ignored worktree set, requires the manifest to name the exact 66-file set,
authenticates all five evidence outputs to `HEAD`, validates the manifest against
its published Draft 7 schema, and accepts exactly one matching record.

Local verification on 2026-08-30 (America/Los_Angeles): `make release-check`
passes with 13/13 corpus cases, 7/7 mutation probes, seven Python tests, 14 Rust
tests, 21/21 grammar-clean documents, and 28/28 backed rows. Real squash
simulation passes while the subject is not an ancestor. Three ultrareview and
four focused review passes concluded with no actionable finding.

GitHub Actions is manual-dispatch only (`workflow_dispatch` is the sole trigger)
and no workflow was dispatched. The live protection snapshot reports required
CODEOWNER review and conversation resolution, configured Rust/license contexts,
and disabled admin enforcement; the named authority's bounded PR #12 admin
squash exception is retained separately.

This evidence is self-attesting and does not authorize a source release, source
tag, project qualification, or accreditation decision.
