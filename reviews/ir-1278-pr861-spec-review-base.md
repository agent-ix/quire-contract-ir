---
id: SR-20261103150650
title: "Base spec review of IR-1278 temporal clause names PR 861"
type: SpecReview
analysis: base
scope: "agent-ix/quire-contract-ir; PR 861 head cfebbee717f0ab9294e0fe163d5ac36f0dc23482; FR-038 name prose and AC-442..447; FR-019 inventory and AC-25"
review_set: base
---

## Summary

Reviewed the two-file PR #861 diff against merged QSpec FR-322 temporal clause names, AC-342..358, published CN name vectors and BO occurrence positives. This is a spec-only preimplementation review.

## Verdict

PASS. The spec defines one closed name row per claim occurrence; retains equal or distinct names for a shared node without changing node identity; keeps name and clock-binding carriers independent; assigns schema and semantic refusals to the published codes, causes and pointers with row-wise precedence; bounds row visits; and uses QSpec vectors by reference. FR-019 public type, borrowed read and optional typed encoder input are marked planned. Every new acceptance criterion starts with its planned-until status. No new hash, pin, copied artifact or compatibility path appears.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

QSpec origin/main FR-322 temporal clause names prose and FR-322-AC-342..358; QSpec temporal-clause-name-vectors.json CN-P1..P3, CN-S1..S8, CN-R1..R11, CN-G1; temporal-clock-occurrence-vectors.json BO-P1/P2. The only PR changes are FR-038 and FR-019 at head cfebbee717f0ab9294e0fe163d5ac36f0dc23482. Targeted Quire validation of both changed files and `git diff --check` pass; Quire reports the pre-existing AC-364 grammar warning. No applicable AssuranceProfile file was found in this repository.

## Scope

- **FR-038** (examined): The V2 reader SHALL admit QSpec FR-322's separate, closed `temporal_clause_names` carrier. It is absent exactly when the semantic graph has no `temporal`/`temporal_clause` node; otherwise it is a non-empty array with exactly one row for each `claim` occurrence of every such node.
- **FR-038-AC-442** (examined): planned until IR-1278 lands. The strict reader admits the separate closed `temporal_clause_names` rows `{declaration, occurrence, name}` only for local temporal-clause `claim` occurrences and exposes the exact admitted pair and identifier through borrowed typed rows. The canonical writer accepts optional typed borrowed rows, emits each member in the closed shape and supplied order, omits the member for `None`, and preserves both names across read/encode/read-back. It does not derive a name from a node path, source span, source map, formula, clock binding or requirement label.
- **FR-038-AC-443** (examined): planned until IR-1278 lands. CN-P1 admits one named clause; CN-P2 admits one shared structural clause node with two `claim` occurrences and two distinct name rows; CN-P3 changes a name without changing its node or `node_id`. Two rows for one node may carry equal names as well, without reader uniqueness refusal. The member is absent exactly when no temporal-clause node exists; a clause node's own `declaration` member (CN-G1) refuses `invalid_semantic_graph` at `/semantic_graph/nodes/{N}/declaration` before name-row checks. Name rows do not replace the independent per-occurrence `temporal_bindin
- **FR-038-AC-444** (examined): planned until IR-1278 lands. CN-S1..S4 refuse a missing `name`, `occurrence` or `declaration`, or an extra row member, as `malformed_wire` at `/temporal_clause_names/{i}`. CN-S5..S7 refuse a non-string, empty or qualified name, and any other value outside the single identifier grammar, as `malformed_wire` at `/temporal_clause_names/{i}/name`; CN-S8 and a noninteger occurrence refuse `malformed_wire` at `/temporal_clause_names/{i}/occurrence`. No default, alias or declaration-only row is admitted.
- **FR-038-AC-445** (examined): planned until IR-1278 lands. After temporal-binding checks and before FR-370 clause-body joins, each name row completes declaration join, occurrence join, repeated-pair check and pair-order check before the next. CN-R3/R4 refuse `invalid_package`/`invalid-value` at `/temporal_clause_names/{i}/declaration` for an absent or wrong-kind node; CN-R5 refuses that pair at `/temporal_clause_names/{i}/occurrence` for an ordinal absent or not a `claim` of that node. CN-R6 refuses `invalid_package`/`conflicting-definition` at the later whole row for a repeated `(NodeRef, occurrence)` key; CN-R7 refuses `
- **FR-038-AC-446** (examined): planned until IR-1278 lands. After all name-row joins, duplicates and order checks, the reader requires one row for every temporal-clause `claim` occurrence. CN-R1's missing carrier and CN-R2's uncovered second occurrence refuse `invalid_package`/`invalid-value` at `/temporal_clause_names`; CN-R8's declaration-join failure and CN-R11's row descent take precedence over that missing coverage. A graph without a temporal clause admits absence, and the reader never manufactures a name or partial admitted view.
- **FR-038-AC-447** (examined): planned until IR-1278 lands. TC-048 reads QSpec `proposals/checked-package-v2/temporal-clause-name-vectors.json` by reference from the selected checkout and checks the exact CN-P1..P3, CN-S1..S8, CN-R1..R11 and CN-G1 case sets for admission or exact code, cause, primary RFC 6901 pointer and structural node locus. It also reads the temporal-clock occurrence positives whose name rows cover distinct claim occurrences. Each name row consumes one validation-work visit before inspection, within the caller's finite work budget; an exhausted budget returns `incomplete` at the first uncharged row with 
- **FR-019 inventory row** (examined): type `TemporalClauseName`; associated borrowed `CheckedPackageV2::temporal_clause_names` read and optional borrowed `CheckedPackageSidecarsRef::temporal_clause_names` encoder input. The row is constructible as typed writer input and readable only as a borrowed row of a strictly admitted package.
- **FR-019-AC-25** (examined): planned until IR-1278 lands. A default-feature external producer constructs `TemporalClauseName` rows with `CheckedNodeId` declaration, non-negative occurrence ordinal and identifier name, passes them as optional borrowed `CheckedPackageSidecarsRef::temporal_clause_names` input to `encode_checked_package_v2`, and emits no raw JSON. A default-feature consumer borrows `Option<&[TemporalClauseName]>` through `CheckedPackageV2::temporal_clause_names()` after strict admission and observes two separate named occurrences of one shared clause node; it passes those same rows to the encoder and strictly
