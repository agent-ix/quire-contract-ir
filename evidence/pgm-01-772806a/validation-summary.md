# PGM-01 retained validation summary

Collected: 2026-08-30

Subject implementation revision:
`772806a7240dedf1d8fd285b577415b679623b0d`

This immutable record supersedes `pgm-01-0208e6f` after PR-range validation
found trailing blank lines in newly added Markdown files. The correction removes
those lines, and the exact base-to-subject command
`git diff --check origin/main...772806a` now exits zero with no output. The
sibling `evidence/pgm-01-772806a.sha256` covers this manifest and both retained
outputs without self-reference.

## Tool identities

| Tool | Identity |
|---|---|
| Quire | 0.31.0; CLI `4f6ed024`; engine `0.46.0@ca7362d4` |
| Rust | rustc 1.94.1 (`e408947bf`); cargo 1.94.1 (`29ea6fb6a`); declared MSRV 1.75 |
| cargo-deny | 0.19.8 |
| PGM validator | `scripts/validate_governance.py` at the subject revision |
| JSON Schema validator | Python 3.10.12; jsonschema 3.2.0; Draft 7 validator |

## Results

| Command or check | Result |
|---|---|
| `env CARGO_TARGET_DIR=/tmp/quire-contract-ir-wave0-target make ci` | pass; format, clippy, 13/13 Draft 7 corpus, 4/4 schema mutation probes, 14 Rust tests, licenses, and unsafe audit |
| `make spec` | pass; 20/20 documents grammar-clean; strict coverage 26/26 backed; 12/12 test symbols bound; zero status lies and zero unbacked rows |
| `python3 scripts/validate_governance.py --mutation-probes --json` | pass; weakening producer, backend, output, or provenance required fields was detected in 4/4 probes |
| `git diff --check origin/main...772806a` | pass; no output |
| `make evidence-verify` | pass; manifest and both retained outputs matched the external checksum file |

The published Draft 7 schema remains the sole conformance engine. The corpus
retains targeted outcomes for missing producer/tool, inputs, nested schema
identity, backend, and outputs, plus malformed digests and unsupported versions.
The external-engine fixture remains semantically `inconclusive`.

The installed Quire module still has its documented status-column contradiction:
the TestMatrix archetype accepts only `Coverage Status`, while coverage is
configured for `Status`. The archetype-valid form is retained. Strict coverage
nevertheless reports complete backing with zero status lies.

## Limitations and open workflow gates

- GitHub Actions is intentionally disabled at repository level by operator
  policy; no remote workflow was dispatched and absent PR checks are
  intentional. Local results do not silently satisfy `Rust Checks` and
  `License Check`.
- `@kreneskyp` must supply a non-stale CODEOWNER approval and decide how the
  configured check gate is resolved while Actions remains disabled.
- PR merge, project Done, issue #3 closure, and every source-release decision
  remain human/external workflow actions.
- The five program assurance artifacts remain an explicit issue #5 obligation.
