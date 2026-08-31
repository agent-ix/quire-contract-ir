---
id: REV-004
title: "Issue #8 expression implementation review"
type: Review
status: complete
---
# REV-004: Issue #8 expression implementation review

## Scope and independence boundary

This review covers the uncommitted issue #8 implementation against FR-013,
FR-014, FR-015, STD-001, and TC-016. The independent reviewer was restricted to
read-only static inspection and explicitly did not run tests or edit files.
Local test and evidence results are separate producer claims, not independent
review approval.

The first reviewer invocation returned no review and ended with an execution
error. It is not evidence. The successful retry returned six numbered findings
and an explicit `COUNT: 6`; the retained list below also contains exactly six
items.

## Findings

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-062 | High | Integer divide/remainder incorrectly required a checked-range proof under saturating overflow, although FR-015 adds that proof only for reject overflow. | fixed; independent narrow confirmation resolved |
| FND-063 | High | Record-literal field checking stopped at the first failed child instead of retaining diagnostics from independent sibling fields. | fixed; independent narrow confirmation resolved |
| FND-064 | Medium | Record-literal type resolution ran before duplicate-field detection, reversing the issue #8 duplicate-before-resolution precedence. | fixed; independent narrow confirmation resolved |
| FND-065 | Medium | TC-016 lacked excessive-collection and wrong-item-type fixtures. | fixed; independent narrow confirmation resolved |
| FND-066 | Medium | TC-016 lacked record-literal duplicate/missing/unknown-field and expression-level orphan fixtures. | fixed; independent narrow confirmation resolved |
| FND-067 | Medium | TC-016 never exercised saturating overflow behavior. | fixed; independent narrow confirmation resolved |

## Additional producer gap analysis

The implementation author separately identified the following gaps before and
during the independent pass. These are not attributed to the reviewer and will
receive distinct finding IDs when dispositions are recorded:

- rational singleton operations were not normalized exactly;
- compound-expression fact subjects could collide in the fallback structural
  key;
- recursive-type diagnostics selected a declaration rather than the
  participating field;
- successful typed rational literals retained the authored, unnormalized form;
- public constructor/accessor round trips omitted several accessors;
- checked negation did not retain its discharged range obligation;
- TC-016 claimed substantially more coverage than its fixtures demonstrated.

These producer findings are tracked as FND-068 through FND-074 in the order
listed above. They are fixed in the working tree; final disposition awaits the
closing review pass.

## Second independent gap scan

After the six original findings were fixed, a narrow confirmation reported all
six resolved with `COUNT OPEN: 0`. A separate remaining-gap scan returned five
numbered items and `COUNT: 5`:

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-075 | Medium | The required `obligation_kind` if-and-only-if invariant was manual rather than enforced at a public boundary. | fixed; serialization and deserialization reject both invalid shapes |
| FND-076 | Medium | A failed dedicated-stack thread spawn fell back to deep recursion on the caller stack and could abort. | fixed; failure now returns a structured fail-closed diagnostic |
| FND-077 | Low | TC-016 lacked the nested quantifier-local duplication path for `invalid_scope`. | fixed |
| FND-078 | Low | TC-016 lacked Boolean-negation, short-circuit-OR false-fact, and quantifier fact non-leakage fixtures. | fixed |
| FND-079 | Low | TC-016 did not demonstrate numeric bound propagation for a pure-call result. | fixed |

## Requirement-scoped closing reviews

The first FR-013 closing response was invalid: it emitted `COUNT: 2` and zero
items. It is retained here as a count failure and is not review evidence. The
replacement response emitted one item and `COUNT: 1`:

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-080 | Medium | Public expression node/depth limits used host-width `usize` despite FR-013 requiring wire-independent fixed-width semantic constants. | fixed; constants are now `u32` and comparisons convert internally |

The FR-014 closing review returned two items and `COUNT: 2`:

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-081 | Medium | `TypedExpression` exposed dependencies but did not implement the FR-012 `DependencySource` contract. | fixed |
| FND-082 | Low | TC-016 checked dependency presence but not mixed-kind structural ordering. | fixed |

The FR-015/STD-001 closing review returned one item and `COUNT: 1`:

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-083 | Medium | Guard extraction recognized literal and collection-length bounds only in one operand order and missed false-branch complements. | fixed; comparator reversal and symmetric index-bound extraction added with fixtures |

The producer's post-review reconciliation found one additional gap not listed
by the reviewer:

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-084 | Medium | Numeric-literal guard extraction and range refinement handled integers but omitted bounded rational literals. | fixed; rational non-zero/lower/upper facts now refine numerator interval sets with positive and negative fixtures |

## Closing confirmation and local gates

The first final independent read-only static confirmation reviewed the tree
after FND-083 against FR-013, FR-014, FR-015, and issue #8 STD-001 and returned
exactly `No actionable findings.` The producer then found and fixed FND-084. A
post-FND-084 independent static confirmation examined the rational interval,
normalization, guard, proof-selection, integer-interaction, and remainder paths
and again concluded `No actionable findings.` Neither confirmation executed
tests, and neither is evidence approval. All 23 findings FND-062 through
FND-084 are fixed.

The producer's first full local gate attempt stopped after formatting and
Clippy because system Python lacked the pinned RFC validators. After installing
the exact requirements in an isolated local environment, the second attempt
passed governance and expression tests but exposed that Rust governance tests
hard-code `python3` and therefore require the environment at the front of
`PATH`. The corrected third attempt passed formatting, Clippy, runtime pin
checks, the 13-case governance corpus, 7 mutation probes, 11 Python tests, all
27 Rust integration tests (including 8 TC-016 tests), doc tests, license policy,
and the unsafe-code audit. No GitHub Actions workflow was dispatched.
