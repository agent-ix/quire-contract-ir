---
id: SR-202611070951309534
title: PR983 checked JSON constructor code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-contract-ir; IR-1404; checked_json.rs constructor and scalar
  callers
review_set: subset
---

## Summary

Ticket: IR-1404. Independent final-source-head review of PR983 at 31b20804 against main d671d43b. The public constructor now encodes its first exact-text Document through the existing CheckedJson encoder before canonical readback, preserving the established typed error instead of silently rounding an out-of-range integer. The private scalar constructor retains its documented read-as-written behavior, so public literal_integer construction remains valid and later encoding rejects the original integer. Checked the original-text i128 guard, signed/unsigned boundaries, nested values, caller behavior, ownership of the flat Document, error propagation, iterative encode/drop, and retained evidence. No public API, schema, dependency, workflow, gate narrowing, numeric algorithm copy, or new seam is introduced. No applicable AssuranceProfile was found under spec/. Source remains unchanged.

## Verdict

PASS for this one-file constructor fix: no introduced findings. Whole original envelope completion, AC511 whole-corpus accounting and AC513 proof remain outside this PASS.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Three genuine native controls PASS (selected 1.98.1 workspace/lib compile 21.99s; test <0.01s): signed/unsigned/nested rejection, safe integer boundaries/fraction/canonical readback, and actual public literal_integer original-text preservation followed by typed encoder rejection. Final workspace all-targets Clippy with conformance-cli,test-support and -D warnings PASS in 16.78s; scoped formatting and diff checks PASS. Retained original exact envelope test advances past its strict integer assertion and remains RED on a separate existing /name allocation-accounting failure (296 unreserved bytes); no whole-test PASS or whole-read qualification is claimed. No Cargo command was run by this reviewer, following the user's bounded checks and post-merge aggregate rule. Selected Quire 0.38.1 matrix lists all three new tests under AC79/AC514. These are partial numeric/construction bindings; they do not establish every clause of either criterion. Existing global coverage gaps are not repaired or credited here. AC511 whole-corpus and AC513 remain UNPROVEN.

Plan completion: not assessed

Full reviewed scope:

```yaml
scope:
- id: CheckedJson::from_serializable
  path: crates/quire-contract-model/src/checked_package/v2/checked_json.rs
  role: examined
  excerpt: Encodes `value` canonically and reads it back.
- id: CheckedJson::scalar
  path: crates/quire-contract-model/src/checked_package/v2/checked_json.rs
  role: examined
  excerpt: 'A handle for a scalar literal payload, read as written: a number the canonical
    encoder cannot write (an integer past 2^53) is held and is the encoder''s refusal
    when the package is written, as it always was.'
- id: FR-038-AC-79
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A package document holding 9007199254740993, -9007199254740993 or 2.0 in
    a node body, or an object with UTF-8-out-of-order member names, refuses noncanonical_wire
    before graph and body checks.
- id: FR-038-AC-514
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: A `literal` term's `value` and each diagnostic detail of an admitted package
    are `CheckedJson` handles into the package's one retained document. Cloning one
    shares the retained document without allocating a tree; equality holds exactly
    when canonical encodings are equal, regardless of document or node identity; `Debug`
    prints the canonical JSON text; `node()` exposes every JSON kind with number source
    text; `from_canonical_bytes` refuses non-canonical bytes; `from_value` and `to_value`
    round-trip; a `CheckedTerm` and a `CheckedDiagnosticV2` serialize to and deserialize
    from the same JSON as be
```
