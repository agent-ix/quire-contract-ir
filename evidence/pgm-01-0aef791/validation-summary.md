# PGM-01 retained validation summary

Subject revision: `0aef791c20029752d3f75cca1a21f8056ee06fb9`

This is the only evidence record in the candidate tree. It replaces all prior
records, which were removed because their claims exceeded what their verifier
could prove. A squash merge is required so those intermediate commits do not
become ancestors of `main`.

Verified locally on 2026-08-30:

- `make ci`: pass (13/13 corpus cases, 7/7 mutation probes, 4 Python tests,
  14 Rust tests, formatting, lint, licenses, and unsafe audit).
- `make spec`: pass (21/21 documents grammar-clean and 28/28 traceability rows
  backed).
- Code review: three ultrareview passes plus one focused pass; every verified
  finding was fixed.
- Gap analysis: pass; 4/4 planned tasks done and no in-scope reverse gap.
- GitHub Actions: intentionally disabled by operator policy; no workflow was
  dispatched and no remote-check success is claimed.

The release-only verifier independently enumerates all 64 non-evidence files in
the subject Git tree, requires an exact manifest key set, compares every digest
to both the subject blob and current file, and requires this manifest, summary,
and sibling checksum file to be committed byte-for-byte in `HEAD`.

This record does not authorize a source release. It records readiness for the
operator-authorized admin squash merge of PR #12 only.
