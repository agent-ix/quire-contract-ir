---
id: SR-2026111219210500000002
title: "gap-analysis of IR-1528 operation_bodies sidecar"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-contract-ir; PR #1018 at 306ded5e; FR-038 Operation-body carrier, FR-038-AC-541, FR-019-AC-28; crates/quire-contract-model/src/checked_package/v2/operation_bodies.rs; tests/conformance_qspec/operation_bodies.rs"
review_set: subset
---

## Summary

Gap analysis for quire-contract-ir PR #1018. Ticket: IR-1528. Plan completion: not assessed. Each ticket acceptance item was measured against the code and the QSpec f268d62f vectors, which are read by path through `QUIRE_SPECIFICATION_DIR`:

1. The encoder writes the member from a typed `&[OperationBodyRow]` through `CheckedPackageSidecarsRef::operation_bodies`. The reader parses the member and applies FR-322's joins and refusals. Met.
2. OBC-01..06 are admitted and round-trip through the typed encoder. OBS-01..13 and OBR-01..20 refuse with the recorded code, cause and locus. OBR-21 does not: running it gives `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/{n}/node_id`, not the recorded `invalid_package`/`invalid-value` at `/operation_bodies/2/body/node`. Measured, test passes.
3. A package without the member admits and both reads return `None`. Met.

On OBR-21, the owner join (owner.rs:703-733) runs over every graph node before any sidecar. It refuses a function node whose `ModelOwner` identity (acme/billing) declares no operation `ix://acme/orders/Order/settle`. FR-322 places that join before carrier checks, so no conforming check order can reach OBR-21's recorded code. Deferring the owner join for carrier-referenced nodes would break FR-322's stage order. This is a QSpec vector defect, not an IR deviation to fix here. The same node sits in OBC-04's published `graph_nodes`, so OBC-04 as published would itself be refused. The IR harness drops unreferenced foreign-owner nodes (operation_bodies.rs:298), which is a sound adaptation.

On has_body, FR-154 domain documents carry no body marker; `has_body` exists only in the vector sketches. The only evidence available to the IR is the graph's body clause function node, owned by the operation member through the owner join. FR-038 states this derivation explicitly. It is sound for every vector. Its limit is inherent: a producer that drops both the node and the `function` row is undetectable from the package.

The matrix (`quire matrix --format tsv`) shows FR-019-AC-28 as `untagged`, as are the neighbouring FR-019-AC-25..27, even though the tests carry `#[trace(..., "FR-019-AC-28")]`. That is a repo-wide matrix pattern, not this PR's. FR-038-AC-541 does not appear in the matrix at all: see the spec-review SR, FND-001.

## Examined scope

- Ticket acceptance (1) encoder plus reader (examined): met.
- Ticket acceptance (2) vectors (examined): 6 positive, 13 schema and 20 of 21 reader vectors met; OBR-21 unreachable as recorded (FND-001).
- Ticket acceptance (3) absent member (examined): met.
- FR-038-AC-541 (examined): tests bind it; it is not minted, see the spec-review SR.
- FR-019-AC-28 (examined): bound by the positive round-trip and the absent-member tests through the public API from an external test crate.
- QSpec FR-322-AC-397 (context_only).

## Verdict

The code covers the ticket's acceptance except OBR-21, which is a QSpec vector defect: it needs a QSpec ticket, and OBR-20 should go on the same ticket. The polymorphic dispatch candidate case has no test (FND-002), which is why the code-review high finding went unseen.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | OBR-21 is unreachable as recorded under FR-322's own stage order. The owner join refuses the foreign-owner node with `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/{n}/node_id`. The test hard-codes that result, and FR-038-AC-541 records the exception, but no QSpec ticket is cited to fix the vector. The same applies to OBC-04 carrying that node, and to OBR-20, whose candidate body is absent from the graph (two defects in a case that should hold one). | tests/conformance_qspec/operation_bodies.rs:415-425 |
| FND-002 | medium | No test covers a dispatch table with more than one candidate (QSpec DT-01's shape) next to an `operation_bodies` member. The only candidate cases (OBC-05, OBR-12, OBR-20) have one candidate each, so the code-review high finding (valid polymorphic dispatch refused) has no failing test. | tests/conformance_qspec/operation_bodies.rs:138-224 |

## New findings (disposition pass 1)

Measured at 8e4d0cf4. OBR-20 passes the generic branch of `tc_987_published_refusal_vectors_refuse_with_the_recorded_code_and_locus` (recorded code, cause and locus `/operation_bodies/1/body`); only OBR-21 is special-cased. OBC-04 as published, with its foreign-owner node kept (probe in a throwaway worktree), is refused `missing_declaration`/`missing-selection` at `/semantic_graph/nodes/12/node_id`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The matrix row, the PR body and the 8e4d0cf4 commit message say OBR-20 "cannot reach" its recorded code or locus. It does, and the test asserts it. Its defect is different: it holds two defects (its candidate body node is absent from the graph) and meets its locus only because the carrier runs before `checked_term_data` (code-review FND-002). The OBR-21 claim is accurate. The OBC-04 claim is accurate in substance, but OBC-04 is a positive case with no recorded code or locus: it is refused as published and admitted only because the harness drops the unreferenced foreign-owner node. Also, FR-038-AC-541 names only OBR-21 as the exception, so the AC and the matrix row disagree. Make the three statements and the AC say the same accurate thing. | spec/checked_package/matrix/tests.md:160 |

## New findings (disposition pass 2)

Measured at 9f4b2267; see the code-review SR for the probes.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The FR-038-AC-541 matrix row still describes the old join ("a dispatch candidate joins the row of the declaration its selection key names"). The FR-038 statement, the code and the PR body now describe the effective-declaration join. Update the phrase. | spec/checked_package/matrix/tests.md:160 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | STD-661 (QSpec, Backlog) now tracks OBR-21, OBC-04 and the FR-322 candidate wording, and the OBR-21 branch of the test cites it. |
| FND-002 | still-open | The new multi-candidate test does not cover DT-01's shape: its second candidate has no row and is never joined (code-review FND-006). No test has two joined candidates with distinct bodies. Probe A shows the code admits that shape, so a test for it is a small addition. |
| FND-002 | fixed | Two joined candidates with distinct bodies are now tested (`a_redefining_receivers_candidate_is_joined_to_its_own_row`, plus its refusal twin), along with the inheriting receiver. Probes over a three-level chain agree. |
| FND-003 | fixed | The matrix row, the PR body and the 9f4b2267 commit message now say that OBR-20 reaches its recorded result (two defects, carrier first), that OBR-21 cannot, and that OBC-04 is admitted only because the harness drops its foreign-owner node. FR-038-AC-541 now names OBC-04 next to OBR-21. |
