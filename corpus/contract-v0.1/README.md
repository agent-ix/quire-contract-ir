# Contract IR v0.1 conformance corpus

Run the corpus without linking the Rust library:

```text
quire-contract-conformance run --corpus corpus/contract-v0.1 --schemas schemas
```

The corpus is this directory. Each `inputs/<id>.json` is one fixture, its
operation is the `<id>` prefix before the first `-`, and its expectation is
`expectations/<id>.json`. Canonical byte files under `canonical/` intentionally
have no terminal newline.

The fixtures target constructs, diagnostics, obligations, operations and exact
boundaries. The runner derives each fixture's coverage tokens from its input
and actual result, maps them to acceptance-criterion targets through
`schemas/conformance-trace-map-v1.json`, and reports both in its structured
row. The union of observed tokens must equal the published inventory. These
are relevant observations, not a claim that the corpus alone proves each entire
criterion. Large exact-edge fixtures are reproducibly authored by
`scripts/generate_conformance_corpus.py`, which freezes runner output as
expectations and rejects a declared token the runner does not observe. For this
crate, a full match proves deterministic regression stability against those
frozen outputs, not independent semantic correctness.
