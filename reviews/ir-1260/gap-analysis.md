---
id: SR-2026110101262
title: "IR-1260 gap analysis of bounded Text literal reader"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; IR-1260 PR #814; FR-038-AC-409 through AC-411; crates/quire-contract-model/src/checked_package/v2/operations.rs; tests/conformance_qspec/text_literals.rs"
review_set: subset
relationships:
  - type: references
    target: ix://agent-ix/quire-contract-ir/FR-038
---

## Summary

Ticket: IR-1260. The computed matrix statically tags AC-409, AC-410, and AC-411, and the reader implementation has an owning requirement. The new test tags misassign part of AC-410 and AC-411 coverage.

## Verdict

**CONDITIONAL** — correct the new test trace tags and the AC-411 oracle before relying on the matrix's static coverage claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The QSpec vector test lacks the AC-411 trace tag, so the computed matrix does not credit the only test that reads the required published vectors to AC-411. | tests/conformance_qspec/text_literals.rs:80 |
| FND-002 | medium | The compatibility and family-precedence control claims AC-411 without reading published vectors; its family-precedence assertion belongs to AC-410. | tests/conformance_qspec/text_literals.rs:117 |
| FND-003 | medium | The scalar and nested literal control claims AC-411 without reading published vectors; its scalar and nested admission assertions belong to AC-409. | tests/conformance_qspec/text_literals.rs:148 |

## Coverage

Plan completion: not assessed.

Computed matrix (`quoin matrix --repo ... --json`): AC-409 is tagged by the vector and compatibility tests; AC-410 only by the vector test; AC-411 only by the compatibility and scalar/nested controls. All three report `no run evidence` in Quoin, which is not treated as a new PR defect. The code lead separately reported three named public-reader tests passing at dd8df98b. Reverse gap check found FR-038-AC-409/410 as the owner of the new reader branch; no new source stub, vendored vector, or unowned behavior was found. Optional full semantic review was not selected.

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
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At a95d2752, `qspec_text_literal_vectors_match_reader` carries `FR-038-AC-411` alongside AC-409 and AC-410. |
| FND-002 | fixed | The compatibility and family-precedence control no longer claims AC-411 and now carries AC-410 with AC-409. |
| FND-003 | fixed | The scalar and nested control no longer claims AC-411 and now carries AC-409 and AC-410. |

## Disposition verdict

**PASS (round 1, source review)** — The original trace findings are fixed at a95d2752. The computed matrix was not rerun in this disposition; the focused test and final full gate remain with the code lead.
