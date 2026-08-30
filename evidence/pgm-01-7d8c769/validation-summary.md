# PGM-01 retained validation summary

Collected: 2026-08-30

Subject implementation revision:
`7d8c769245fc2630cae461fd67a14543599ca757`

This immutable record supersedes `pgm-01-772806a` after an operator-authorized
code-review and gap-analysis pass. The gap analysis found that the four Task
artifacts lacked explicit completion state; the multi-agent code review found
that `CLAUDE.md` no longer described the executable Make targets accurately.
Both findings are fixed in the subject revision. The sibling
`evidence/pgm-01-7d8c769.sha256` covers this manifest and both retained outputs
without self-reference.

## Tool identities

| Tool | Identity |
|---|---|
| Quire | 0.31.0; CLI `4f6ed024`; engine `0.46.0@ca7362d4` |
| Rust | rustc 1.94.1 (`e408947bf`); cargo 1.94.1 (`29ea6fb6a`); declared MSRV 1.75 |
| cargo-deny | 0.19.8 |
| PGM validator | `scripts/validate_governance.py` at the subject revision |
| JSON Schema validator | Python 3.10.12; jsonschema 3.2.0; Draft 7 validator |
| Code review | Claude ultrareview `session_017Fqx3ci4rXBMr2rBZKGG7S` |

## Results

| Command or check | Result |
|---|---|
| Multi-agent code review | pass after remediation; one verified documentation-drift finding fixed, two candidates refuted |
| Gap analysis (`reviews/2026-08-30-plan-001-pgm01.md`) | PASS; 4/4 tasks done, 26/26 rows backed, zero status lies, zero untracked symbols, zero in-scope reverse gaps |
| `env CARGO_TARGET_DIR=/tmp/quire-contract-ir-wave0-target make ci` | pass; format, clippy, 13/13 Draft 7 corpus, 4/4 schema mutation probes, 14 Rust tests, licenses, and unsafe audit |
| `make spec` | pass; 21/21 documents grammar-clean; strict coverage 26/26 backed; 12/12 test symbols bound; zero status lies and zero unbacked rows |
| `python3 scripts/validate_governance.py --mutation-probes --json` | pass; weakening producer, backend, output, or provenance required fields was detected in 4/4 probes |
| `git diff --check origin/main...7d8c769` | pass; no output |
| `make evidence-verify` | pass; manifest and both retained outputs matched the external checksum file |

The published Draft 7 schema remains the sole conformance engine. The corpus
retains targeted outcomes for missing producer/tool, inputs, nested schema
identity, backend, and outputs, plus malformed digests and unsupported versions.
The external-engine fixture remains semantically `inconclusive`.

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
