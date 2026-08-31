# Contract IR v0.1 conformance corpus

Run the corpus without linking the Rust library:

```text
quire-contract-conformance run --manifest corpus/contract-v0.1/manifest.json
```

A downstream evidence record pins both the repository commit and the content
digests it actually executed. For example, record `git rev-parse HEAD`, copy the
relevant `.sha256` files, and verify them with `sha256sum -c` before running the
tool. A copied pin is not evidence that this corpus ran downstream.

The package schema, conformance schema, inventory, every authored input and
expectation, and every canonical byte file have adjacent SHA-256 sidecars.
Canonical byte files intentionally have no terminal newline. The checked-in
manifest does not try to contain its own commit identity.

The manifest contains targeted construct, diagnostic, obligation, operation,
and exact-boundary fixtures. The runner derives observable coverage from each
fixture's declarative input and actual result and rejects an unobserved
`covers` token before comparing expectations. Large exact-edge fixtures are
reproducibly authored by `scripts/generate_conformance_corpus.py`; the script
freezes runner output as expectations but cannot bypass the observation check.

Automatic CI triggers and crate publication remain disabled. A later human
release decision owns both changes.
