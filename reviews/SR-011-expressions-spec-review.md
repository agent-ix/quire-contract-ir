---
id: SR-011
title: "Issue 8 type, expression, and definedness specification review"
type: SpecReview
analysis: architecture-evaluation
scope: "FR-013, FR-014, FR-015, STD-001, NFR-002, TM-002, TC-016, TASK-007"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-contract-ir/issues/8
    type: reviews
---
# Issue 8 type, expression, and definedness specification review

## Summary

Issue #8 extends the validated identity foundation without taking ownership of
canonical bytes, migration, the external conformance runner, or source-release
decisions. The type environment is closed and declaration-backed; expression
checking is source-located and deterministic; definedness is a static proof
obligation rather than permissive runtime behavior.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-023 | low | Every public type and declaration needs closed validation rules and a registered failure class. | FR-013, STD-001 |
| FND-024 | low | Current/pre/post observations and quantifier locals must be scoped without architecture-language assumptions. | FR-014 |
| FND-025 | low | Short-circuit and total Boolean operators must remain structurally distinct and semantically testable. | FR-014, FR-015, TC-016 |
| FND-026 | low | Every partial operation must require a declaration or dominating guard proof. | FR-015 |
| FND-027 | low | Deterministic equality/dependencies must remain distinct from FR-016 canonical bytes. | FR-014, FR-016 |
| FND-028 | low | Issue #8 criteria need requirement-tagged evidence without claiming issue #9/#10 work. | TM-002, TC-016 |

## Independent Spec Review Disposition

The first independent read-only pass labeled its response "11 findings" but
contained twelve actionable bullets, then identified a bounded-rational risk as
an additional thirteenth item. All thirteen are retained below with fixed
dispositions. A clean closing review is still required before Implement.

The follow-up verified those dispositions and reported twelve additional
implementation gaps, FND-042 through FND-053. Those are also fixed below; no
finding is waived.

The closing pass verified FND-029 through FND-053, then labeled its response
"six new blockers" while containing seven actionable bullets. All seven are
retained as FND-054 through FND-060 and fixed below.

The final gate found one cross-sentence range-refinement contradiction,
FND-061. It is fixed, and the narrow independent confirmation returned no
actionable findings.

| ID | Severity | Finding | Disposition |
|---|---|---|---|
| FND-029 | high | Saturating arithmetic wording could suppress division/remainder zero obligations. | FR-015 limits saturation totality to add/subtract/multiply/negate; every divide/remainder retains non-zero obligations. |
| FND-030 | high | The expression node/operator set was not closed and omitted nodes required by definedness. | FR-014 enumerates every literal, reference, access, call, numeric, comparison, Boolean, and quantifier node. |
| FND-031 | high | Guard facts could discharge obligations on a different observation or expression. | FR-015 requires exact resolved structural subject equality, including observation and nested nodes. |
| FND-032 | high | State-observation permissions were delegated to an undefined context. | FR-014 now gives a closed execution-point/observation table; STD-001 cites that table. |
| FND-033 | high | Diagnostic precedence covered issue #6 only. | STD-001 now orders all issue #8 declaration, resolution, typing, root, and definedness classes. |
| FND-034 | medium | Collection length/index types were undefined. | FR-014 derives one unsigned bounded index type from the collection maximum and requires exact index typing. |
| FND-035 | medium | Numeric guards could not contribute checked-range facts. | FR-015 recognizes numeric ordering against literal bounds. |
| FND-036 | medium | Potential-undefined failures did not identify their obligation class. | STD-001 requires the closed `obligation_kind` field and FR-015-AC-5 verifies every value. |
| FND-037 | medium | Issue #8 JSON round-trip language created an unowned public schema. | FR-013-AC-4 now covers constructor/accessor structural equality; FR-018 retains schema ownership. |
| FND-038 | medium | Element-only quantifiers made general guarded indexing impractical. | FR-014 adds element and index domains; index domains contribute exact local/collection bounds facts. |
| FND-039 | medium | Typed expressions did not explicitly implement FR-012 dependency derivation and TC-016 omitted FR-012. | FR-014 states the dependency-source contract and TM-002 traces TC-016 to FR-012 through FR-015. |
| FND-040 | low | Task status did not distinguish Spec Review from Implement. | TASK-007 declares `phase: spec_review` and states implementation has not begun. |
| FND-041 | medium | Unbounded rational magnitude/denominator created a resource and determinism risk for later canonicalization. | FR-013 bounds numerator and normalized denominator; rational arithmetic uses checked range obligations. |
| FND-042 | high | Numeric and aggregate literals had no explicit type-acquisition rule. | FR-014 requires every literal to name its complete value/derived index type and validates values against it. |
| FND-043 | high | Numeric compatibility, result types, and governing overflow policy were ambiguous. | FR-014 requires complete type equality and returns that type; remainder/negate domains are closed. |
| FND-044 | high | Checked-range fact refinement and interval composition were unspecified. | FR-015 defines bottom-up mathematical intervals, exact-subject refinement, checked `i128` implementation bounds, and AC-6. |
| FND-045 | high | Nested-local and value-declaration shadowing rules contradicted each other. | FR-014 permits value shadowing with local precedence, forbids enclosing-local reuse, and confines locals to predicates. |
| FND-046 | high | Unbounded expression depth/node count allowed stack/resource failure without a diagnostic. | FR-013/FR-014 fix limits at 10000 nodes and depth 256, require explicit-stack preflight, register `expression_too_large`, and add AC-5. |
| FND-047 | medium | Declaration provenance spans were criterion-only rather than normative behavior. | FR-013 now requires spans on every declaration, variant, field, parameter, input, and state. |
| FND-048 | medium | Enum, record, and collection literal well-formedness lacked exact rules/codes. | FR-014 requires variant existence, exact record fields, typed bounded items, and names each primary code. |
| FND-049 | medium | Rational normalization was undefined. | FR-013 defines positive-denominator gcd normalization, canonical zero, and normalize-before-bounds behavior. |
| FND-050 | medium | Diagnostic set order, recovery, and observation precedence were unspecified. | STD-001 defines declaration-first/authored-preorder order, one primary per node, parent suppression, sibling continuation, and observation placement. |
| FND-051 | medium | Dependency identity granularity and order were untestable. | FR-014 defines observation-qualified state/input and owner-qualified field/variant/function identities, structural order, and AC-6. |
| FND-052 | low | The diagnostic public-field list omitted mandatory obligation kind and closed severity. | STD-001 now declares severity `error` and if-and-only-if obligation field semantics. |
| FND-053 | low | FR-014 and STD-001 frontmatter omitted normative FR-012/FR-013/FR-014/FR-015 edges. | The trace graph now carries each dependency/reference explicitly. |
| FND-054 | high | Numeric declaration bounds were wider than the range assumed by interval analysis. | FR-013 caps integer/rational numerator bounds at signed 64-bit values and denominators at the positive signed maximum. |
| FND-055 | high | Field, unwrap, index, and pure-call results had no range rule. | FR-015 assigns each the declared numeric result bounds and AC-6 requires fixtures. |
| FND-056 | high | Rational denominator propagation had no executable rule or positive oracle. | FR-015 specifies normalized singleton and unreduced worst-case numerator/denominator formulas plus AC-7. |
| FND-057 | medium | A non-zero guard discharged division but did not refine an interval containing zero. | FR-015 uses finite interval sets split around zero and fails checked range if zero remains. |
| FND-058 | medium | Multiple proof facts made the retained discharge span nondeterministic. | FR-015 selects the innermost contributing guard, authored-preorder tie break, then declaration fallback; AC-5 verifies it. |
| FND-059 | low | Text remained an unbounded one-node resource. | FR-013 caps text at 1048576 Unicode scalar values and STD-001 registers `text_bound_exceeded`. |
| FND-060 | low | Dependency kind/observation/path order was implementation-defined. | FR-014 defines exact kind and observation orders plus scalar-value path ordering. |
| FND-061 | high | Exact-subject guards narrowed references but not numeric fields, unwraps, indices, lengths, or calls. | FR-015 starts each such node at declared bounds and applies the same exact-subject fact intersection rule. |

## Dependencies

- **Upstream**: TASK-006 / issue #6 validated identities and dependency-source contract.
- **Downstream**: issue #9 canonicalization and issue #10 conformance consume only validated typed expressions.
