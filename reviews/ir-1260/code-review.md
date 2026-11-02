---
id: SR-2026110101261
title: "IR-1260 code review of bounded Text literal reader"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir; IR-1260 PR #814; crates/quire-contract-model/src/checked_package/v2/equality_projection.rs, operations.rs; tests/conformance_qspec/main.rs, text_literals.rs"
review_set: subset
---

## Summary

Ticket: IR-1260. Reviewed the PR #814 Rust diff at dd8df98b against FR-038-AC-409 through AC-411 and QSpec FR-322-AC-304. The reader uses the existing Quire Exact Text profile and length semantics, and its direct public-reader test passes the published cases; the test oracle omits required vector and refusal metadata checks.

## Verdict

**CONDITIONAL** — one medium test coverage finding must be fixed before this PR can claim AC-411. No production reader defect, unsafe code, copied vector, or gate narrowing was found in this diff. The code lead owns the final full gate; this reviewer did not run Cargo builds or tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The published-vector test does not verify the required TX id set, recompute each row's length and normalization, or assert the containing graph-node locus on refusal; it can pass with missing or inconsistent AC-411 evidence. | tests/conformance_qspec/text_literals.rs:86 |

## Examined scope

```yaml
scope:
  - id: FR-038-AC-409
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: examined
    excerpt: |-
      planned until IR-1260 lands. A root or nested `text` literal typed by `Text[min, max; profile]` admits exactly when its retained `value` is already normalized under `nfc`, `nfd`, `nfkc`, or `nfkd` when applicable and its scalar-count length (UTF-8 byte count for `binary-utf8`) is within the inclusive bounds. The reader does not normalize or coerce the value. A plain `scalar_type`/`text` literal remains unbounded.
  - id: FR-038-AC-410
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: examined
    excerpt: |-
      planned until IR-1260 lands. A bounded Text literal below `min`, above `max`, or not in its declared normal form refuses `ill_typed`/`type-mismatch` at its exact `value` pointer, with the containing node as locus and no admitted package. A literal typed by a non-Text family refuses `ill_typed`/`operator-ineligible` at `type` first. No package refusal is reported as FR-141 runtime `text_length_out_of_domain`.
  - id: FR-038-AC-411
    path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
    role: examined
    excerpt: |-
      planned until IR-1260 lands. The conformance test reads QSpec `proposals/checked-package-v2/text-literal-vectors.json` at the selected checkout by reference and checks `TX-01`..`TX-14` plus `TX-13s`, including exact admission, refusal code/cause, primary RFC 6901 pointer and graph-node locus. It recomputes length and normalization with the reader's pinned Unicode 17.0.0 implementation and keeps no local vector, schema, table, or fixture copy.
  - id: FR-322-AC-304
    path: /home/peter/dev/quire-specification/spec/objects/interfaces/FR-322-checked-package-artifact.md
    role: context_only
    excerpt: |-
      A `text` literal typed by `Text[min, max; profile]` is admitted exactly when its `value` is already normalized under `profile` (`nfc`, `nfd`, `nfkc`, `nfkd`; the reader normalizes nothing) and its length (scalar count, or UTF-8 byte count under `binary-utf8`) lies in `[min, max]`, both bounds inclusive.
```

## Review methods and evidence

Rust review covered public API visibility, error precedence, malformed input, normalization semantics, resource bounds, test determinism, and trace tags. The diff changes no CI workflow or Cargo manifest. `quoin matrix` reports the three new local criteria as statically tagged; its evidence status is `no run evidence`. The code lead reported the named three-test conformance run green at dd8df98b. No applicable AssuranceProfile exists under `spec/`.
