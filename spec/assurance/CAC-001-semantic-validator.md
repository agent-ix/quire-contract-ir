---
id: CAC-001
title: "Semantic validator and canonicalizer assurance contract"
type: ComponentAssuranceContract
status: proposed
owner: kreneskyp
kind: deterministic
responsibility: reject invalid contracts and produce deterministic identities for valid v0.1 packages
inputs: [UTF-8 JSON package bytes, supported schema profile, canonicalization profile, artifact trace identities]
outputs: [validated semantic package or ordered diagnostics, dependency sets, canonical bytes, SHA-256 identities, coverage classifications]
invariants: [no validated value bypasses schema and semantic checks, every executable clause has an anchor, every partial operation has a proven definedness condition, orphaned artifacts never contribute covered status]
failure_behaviors: [reject unsupported versions, reject malformed or ill-typed input, report potential undefined operations, report stale or missing references, fail the runner on expectation mismatch]
version_pins:
  rust: "1.75 minimum; stable toolchain recorded per candidate"
  serde: "exact Cargo.lock resolution"
  serde_json: "exact Cargo.lock resolution"
  sha2: "exact Cargo.lock resolution"
  json_schema: "Draft 7 checked-in schema digest"
controls:
  surfaces: [typed Result API, stable diagnostic registry, JSON Lines runner output, corpus manifest, golden digest fixtures]
  fallback: preserve input and diagnostics; emit no canonical identity for invalid input
  abstention: classify unsupported or inconclusive conditions explicitly
  escalation: block merge or source release and assign the finding to the named owner
isolation: no downstream generator, runtime, solver, monitor, Quoin, or Quire code is linked into the semantic library
replacement: a replacement demonstrates byte-identical canonical corpus output and diagnostic/dependency equivalence before activation
relationships:
  - target: ix://agent-ix/quire-contract-ir/AP-001
    type: references
---
# Semantic validator and canonicalizer assurance contract

## Component Boundary

The component begins at untrusted JSON or programmatically constructed wire
values and ends at validated semantic values, diagnostics, dependencies,
canonical bytes, digests, and trace classifications. External lowerings and
human decisions remain outside.

## Required Behavior

Every accepted package conforms to the supported schema and semantic rules.
Every executable clause is anchored. Every partial operation is proven defined.
Every identity is deterministic for the declared profile. Every rejected input
has an ordered structured diagnostic.

## Failure Handling

Malformed, unsupported, ill-typed, potentially undefined, stale, and orphaned
inputs fail closed. No invalid input receives canonical success bytes, a valid
digest, or covered status. Operational runner failures use exit class 2.

## Controls

Callers receive typed results and stable diagnostics. Reviewers can reproduce
the corpus, compare golden bytes, inspect exact dependency sets, and identify
the tool/profile revision. The fallback retains evidence without asserting
success.

## Replacement

A replacement runs the complete pinned corpus on supported platforms, matches
canonical bytes and digests, preserves diagnostic/dependency semantics, and
receives independent review before adoption.
