# Contract IR v0.1 conformance corpus

Run the corpus without linking the Rust library:

```text
quire-contract-conformance run --corpus corpus/contract-v0.1 --schemas schemas
```

The corpus is this directory. Each `inputs/<id>.json` is one fixture, its
operation is the `<id>` prefix before the first `-`, and its expectation is
`expectations/<id>.json`. Canonical byte files under `canonical/` intentionally
have no terminal newline.

The eight integer members of the type system (an integer type's `minimum` and
`maximum`, a rational type's `numerator_minimum`, `numerator_maximum` and
`maximum_denominator`, an integer literal's `value` and a rational literal's
`numerator` and `denominator`) are decimal strings in every input and every
canonical file. An input of the form `{"document_json": "<text>"}` (the package
and expression operations accept it) hands the decoder that exact text, so a
fixture can supply a value the schema refuses, such as a JSON number in one of
the eight members or a revision above 2^53, and expect the decoder's own code;
the same value in an ordinary input fails the run as `invalid_corpus`.

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
