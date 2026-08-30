# PGM-01 retained validation summary

Collected: 2026-08-30

Subject implementation revision:
`63bdf31758a11f6e0d596e505b3e01f7069558d0`

This immutable record supersedes `pgm-01-7d8c769` after three multi-agent code
review passes and a current-format gap analysis. All verified review findings
are fixed: command documentation matches the executable gates, Draft 7 format
checking fails closed with exact RFC validator dependencies, and the evidence
gate authenticates every retained output plus every recorded input against both
the subject Git blob and current candidate. The sibling
`evidence/pgm-01-63bdf31.sha256` covers this manifest and both retained outputs
without self-reference.

## Tool identities

| Tool | Identity |
|---|---|
| Quire | 0.31.0; CLI `4f6ed024`; engine `0.46.0@ca7362d4` |
| Rust | rustc 1.94.1 (`e408947bf`); cargo 1.94.1 (`29ea6fb6a`); declared MSRV 1.75 |
| cargo-deny | 0.19.8 |
| PGM validator | `scripts/validate_governance.py` at the subject revision |
| Evidence verifier | `scripts/verify_evidence.py` at the subject revision |
| JSON Schema validator | Python 3.10.12; jsonschema 3.2.0; rfc3339-validator 0.1.4; rfc3986-validator 0.1.1 |
| Code review | Claude ultrareview sessions `017Fqx3…`, `014k9uag…`, and `014B5Uj…` |

## Results

| Command or check | Result |
|---|---|
| Three multi-agent code-review passes | pass after remediation; three verified findings fixed, six candidates refuted, no finding left open |
| Gap analysis (`reviews/2026-08-30-plan-001-pgm01.md`) | PASS; 4/4 tasks done, 28/28 rows backed, zero status lies, zero untracked symbols, zero in-scope reverse gaps |
| Isolated format-checker reproduction | pass; malformed date-time, URI, and URI-reference values are rejected and all required checkers are registered |
| Stale-record negative check | pass; `make evidence-verify` rejected `pgm-01-7d8c769` after current inputs changed |
| `PATH=<isolated-python-3.10.12-venv>/bin:$PATH CARGO_TARGET_DIR=/tmp/quire-contract-ir-wave0-target make ci` | pass; format, clippy, 13/13 Draft 7 corpus, 7/7 schema/format probes, one Python verifier test, 14 Rust tests, licenses, and unsafe audit |
| `PATH=<isolated-python-3.10.12-venv>/bin:$PATH make spec` | pass; 21/21 documents grammar-clean; strict coverage 28/28 backed; Python 1/1 and Rust 12/12 test symbols bound; zero status lies and zero unbacked rows |
| `PATH=<isolated-python-3.10.12-venv>/bin:$PATH python3 scripts/validate_governance.py --mutation-probes --json` | pass; four schema weakenings and three malformed identity formats detected, 7/7 |
| `git diff --check origin/main...63bdf31` | pass; no output |
| `PATH=<isolated-python-3.10.12-venv>/bin:$PATH make evidence-verify` | pass; unique current record selected; 3/3 retained outputs and 48/48 subject/current inputs matched |

The published Draft 7 schema remains the sole conformance engine. The runtime
gate now refuses to run if `date-time`, `uri`, or `uri-reference` validation is
unavailable, eliminating environment-dependent silent format acceptance.

The installed Quire module still has its documented status-column contradiction:
the TestMatrix archetype accepts only `Coverage Status`, while coverage is
configured for `Status`. The archetype-valid form is retained. Strict coverage
nevertheless reports complete backing with zero status lies.

## Limitations and workflow disposition

- GitHub Actions is intentionally disabled at repository level by operator
  policy; no remote workflow was dispatched and absent PR checks are
  intentional. Local results do not silently satisfy `Rust Checks` and
  `License Check`.
- The operator explicitly authorized `--admin` merge after clean code review,
  gap analysis, remediation, and final local verification.
- PR merge and issue/project transitions are workflow actions, not a source tag
  or project-specific qualification decision.
- Every source-release decision remains separate and human-controlled.
- The five program assurance artifacts remain an explicit issue #5 obligation.
