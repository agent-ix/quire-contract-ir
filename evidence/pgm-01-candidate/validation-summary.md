# PGM-01 retained validation summary

Collected: 2026-08-30

Subject implementation revision:
`6e45964cf5efdc472ebe33cdb43b2ae1d4bda370`

## Tool identities

| Tool | Identity |
|---|---|
| Quire | 0.31.0; CLI `4f6ed024`; engine `0.46.0@ca7362d4` |
| Rust | rustc 1.94.1 (`e408947bf`); cargo 1.94.1 (`29ea6fb6a`); declared MSRV 1.75 |
| cargo-deny | 0.19.8 |
| PGM validator | `scripts/validate_governance.py` at the subject revision |
| JSON Schema cross-check | Python 3.10.12; jsonschema 3.2.0; Draft 7 validator |

## Results

| Command or check | Result |
|---|---|
| `python3 scripts/validate_governance.py` | pass; 9/9 manifest cases matched |
| Independent Draft 7 schema validation | pass; 2/2 positive fixtures accepted and 7/7 negative fixtures rejected |
| `quire validate --scope . 'spec/**/*.md' --summary` | pass; 3/3 documents grammar-clean, zero grammar findings |
| `CARGO_TARGET_DIR=target make ci` | pass; format, clippy, governance, 14 Rust tests, licenses, and unsafe audit |
| Git diff whitespace check | pass |

The negative corpus retained `MISSING_PRODUCER`, `MISSING_INPUTS`,
`MISSING_SCHEMA_IDENTITY`, `MISSING_BACKEND`, `MISSING_OUTPUTS`,
`INVALID_DIGEST`, and `UNSUPPORTED_SCHEMA` as explicit outcomes. The solver
fixture retained an `inconclusive` result and did not convert it into successful
analysis evidence.

The cargo-deny license gate reported only pre-existing unmatched allow-list
warnings and ended with `licenses ok`. Stable rustfmt reported the repository's
pre-existing warnings for nightly-only import-grouping settings; formatting
still passed.

## Limitations and open workflow gates

- GitHub Actions is intentionally disabled at repository level by operator
  policy; no remote workflow was dispatched and PR #12 reports no checks. Local
  results are the retained automated evidence but do not silently satisfy the
  protected branch's configured `Rust Checks` and `License Check` contexts.
- The branch-protection snapshot records configuration, not a review approval.
- The human CODEOWNER must decide how the configured required-check gate is
  resolved while Actions remains disabled. CODEOWNER review, merge, project
  Done, issue closure, and any source-release decision remain human/external
  workflow actions.
