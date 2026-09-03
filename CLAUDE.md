# quire-contract-ir

Versioned semantic contract model and canonical representation for assurance tooling.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make spec           # validate and cover all Quire artifacts
make assurance-env  # create the pinned shared-assurance interpreter
make assurance-inputs # run the native producers the shared path consumes
make pins           # classify the shared toolchain against the accepted matrix
make assurance-chain # seal, retain, receipt, and re-verify through Quoin
make assurance      # pins + assurance-chain
make assurance-record # transcribe a conformance run into the Quoin evidence store
make release-check  # run all local release gates
make test           # Python suite + cargo test
make build          # release build
make clean          # cargo clean and drop the assurance workspace
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # all local release gates, including spec, MSRV, and shared assurance
```

The test target requires the Python declared by `.python-version`.

The assurance targets run in a second interpreter, `.venv-assurance`, built from
`requirements-assurance.txt`, so its pinned `engineering-assurance` distribution
cannot collide with anything installed system-wide.

## Assurance boundaries

Producers run natively, in `make assurance-inputs` and nowhere else. Quire
exports static facts and Quoin transcribes, retains, audits, and reports bytes
it is handed; neither runs anything. No gate reads a verdict from stdout or
stderr — a verdict recovered from console text is a verdict the producer never
made. This repository retains no `evidence/` tree; the owner released the
preservation constraint for the pre-stable phase on 2026-09-02
(engineering-assurance#7) and it was deleted rather than carried forward. Only
`@kreneskyp` records a decision, so a receipt that reads `incomplete` for
`decision_missing` is correct rather than broken.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
tests/integration.rs   # end-to-end tests
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
