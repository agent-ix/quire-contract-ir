---
id: SR-202611070951309524
title: IR-1404 PR969 final gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; PR969; changed checked_package Rust sources, tests
  and tools/read_path_proof
review_set: subset
relationships:
- target: ix://agent-ix/quire-contract-ir/FR-038
  type: reviews
---

## Summary

Ticket: IR-1404. Review completed after PR 969 had actually merged; it is not retroactive premerge approval. The lead reports the merged main tree equals the reviewed candidate. Independent final review of PR 969 frozen candidate against its first parent. The merge has exactly the expected main and qualified-F2 parents and 40 changed files. Relative to the qualified F2 tree only intake import order and equivalent borrowed operand matching / typed empty-row fixture differ. The excluded provider, static and diagnostic followups are not ancestors. No spec, plan, Cargo, lockfile or CI workflow change is present.

No applicable AssuranceProfile was found. Repository AGENTS.md/CLAUDE.md and the Rust baseline, API, implementation, safety, wire and testing references were applied. No source edits or branch mutations were made.

## Verdict

CONDITIONAL — no new scoped gap found; repository-wide assurance remains incomplete with known baseline gaps.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reconciliation: quire matrix (CLI 0.38.1 / engine 0.51.0), no execution store evidence read. Whole-repository static result: tagged 1030, untagged 96, tagged-by-ignored-test 1, method-without-symbol 26. These baseline gaps preclude a repository assurance PASS; they are not introduced acceptance criteria in this code-only merge. In the IR-1404 range AC-503, AC-507 and AC-517 remain untagged; AC-513 is method-without-symbol. These existing limitations are recorded without converting the owner's deferred work into a new merge blocker. AC-511's many tags establish presence only, not physical accounting completion.

Reverse mapping of the changed behaviors is FR-038 strict V2 reading, FR-035 contract-package compatibility and FR-358 caller-budget accounting. Inspected changed graph/operation/body logic is implemented rather than stubbed; boundary tests assert actual charged outcomes and loci. Optional standalone semantic gap review was skipped; ordinary code/test faithfulness was checked in the code-review lane. A full reverse audit of unrelated public APIs was not repeated for this frozen merge review.

Plan completion: not assessed

The supplied selected-toolchain workspace/all-target compile log finishes successfully, with warnings retained. This reviewer did not repeat compilation. The Python census unit suite passed 45 tests. No aggregate gate, Kani or physical proof was run. The earlier aggregate on main remains red (format/string-edge and coverage); the planner assigned the marker followup after this PR. No aggregate-pass claim is made. Whole-read AC-511 proof (132/660), AC-513 static closure and remaining F1 body-graph migration remain explicitly deferred under the owner's scope cut, not merge blockers.

## Reviewed units

```yaml
scope:
- id: FR-038-AC-379
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Implemented; evidence: `node_index::imported_index_resolves_in_the_importing_graph`
    and `checked_package_v2_refinement::refinement_external_abstract_resolves_only_the_directly_selected_package_root`.  Every
    index segment in a package document resolves against that document''s graph, even
    when the id''s `package` names an imported node. NI-14''s imported `Int[0, 9]`
    rebuilds against the importing graph and admits; resolving it against its owner''s
    library graph would create a self-cycle and fails the oracle. An external FR-375
    `{package, node}` key resolves against the selected dependency named by '
- id: FR-038-AC-499
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: WireDecode stage. For a genuine admitted package, a read-storage limit
    one below the cumulative read storage through the decoder's last request refuses
    at it with locus `PackageRead(WireDecode, i)`, where `i` and `form` follow the
    stage table. Separate one-below cases deny a request for a node vector (`"semantic_graph.nodes"`),
    an owned string, a structural node-id path, a `CheckedJson` handle, a `lock.dependency_selections`
    item (index = its position) and an `unknown_member` refusal's diagnostic; each
    charged amount equals the requested block's size. Every owned container is reserved
    at its e
- id: FR-038-AC-500
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: WireChecks stage. For a genuine admitted package, a read-storage limit
    one below the cumulative read storage through the last request of the header,
    owner-schema, flat-wire or interval-bound checks refuses at it with locus `PackageRead(WireChecks,
    i)` and that check's form; the decoded kind table is the denied request in one
    case, and the stage's exact limit admits the same kind table as an unlimited budget.
- id: FR-038-AC-501
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: NodeIndex stage. For a genuine package with index segments and for the
    100000-node chain, a read-storage limit one below the cumulative read storage
    through the last node-index request refuses at it with locus `PackageRead(NodeIndex,
    i)`; the shared structural key of node `i`, the uniqueness table and the cycle
    worklist are each the denied request in one case. A duplicate node id and an index
    cycle keep their refusal codes and pointers at any budget at least their requirement.
- id: FR-038-AC-502
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Lock stage. For a genuine package with dependency selections, required
    features, unit nodes and selected models, a read-storage limit one below the cumulative
    read storage through the lock stage's last request refuses at it with locus `PackageRead(Lock,
    i)` and the stage table's form; the dependency-identity table, the unit-reference
    table with its formatted names, the dependency admission worklist and the field
    tables are each the denied request in one case. A work limit one below the lock
    stage's cumulative work refuses at the existing lock work charge point.
- id: FR-038-AC-504
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Graph stage. For genuine packages that together reach every node family
    validator (frame, temporal, protocol, timed model, hybrid, refinement, operation,
    extension, state, claim and value bodies) and every sidecar row check, a read-storage
    limit one below the cumulative read storage through the last request the graph
    stage makes for node `i` refuses at it with locus `PackageRead(Graph, i)` and
    the node's `node_tag` as form (or the sidecar member and row index for a sidecar
    row); the typed `CheckedTerm` of node `i`, the declaration index, the lowering
    facts, one family validator's temporary tab
- id: FR-038-AC-505
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: SourceMap, Equivalence, Extension and Capabilities stages. For a genuine
    package with source-map entries, equivalence occurrences, extension nodes and
    capability rows, a read-storage limit one below the cumulative read storage through
    each stage's last request refuses at it with that stage's locus, index and form
    from the stage table; each stage's admitted outcome at its exact limit equals
    the unlimited outcome.
- id: FR-038-AC-506
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Diagnostics, OperationEffects and CheckedTermData stages. For a genuine
    package with diagnostics, operation-effect rows and checked-term data, a read-storage
    limit one below the cumulative read storage through each stage's last request
    refuses at it with that stage's locus, row index and form, and a work limit one
    below each stage's cumulative work refuses at its existing work charge point.
- id: FR-038-AC-509
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: Strict grammar under the shared reader. Package bytes with invalid UTF-8,
    a byte order mark, an unescaped control character, an invalid escape, a lone surrogate
    escape, a number outside RFC 8259 grammar, a number with no finite double, or
    trailing content each refuse `malformed_wire` with no pointer; a repeated member
    refuses `duplicate_member` at the later member's pointer, including a name holding
    `~` or `/`; a canonical-grammar package whose bytes differ from its canonical
    encoding refuses `noncanonical_wire` with no pointer; an unknown member refuses
    `unknown_member` at it; an explicit `nu
- id: FR-038-AC-510
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: On a budgeted entry, a work denial whose diagnostic is affordable returns
    `BudgetIncomplete` with `denied` `Work` and its amount, the budget's configured
    work limit, cumulative work plus the denied amount, and the value's pointer; a
    read-storage denial likewise returns `BudgetIncomplete` with `denied` `ReadStorage`.
    When the diagnostic's own request is denied, or the denied request was a refusal
    diagnostic, the read returns `Exhausted` holding the original denied resource
    and amount, its configured bound (including zero), its `PackageRead` locus and
    the pending outcome class (refusal code or s
- id: FR-038-AC-512
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: 'Over the TC-048 corpus, every budget whose limits are at least the read''s
    measured requirement returns the same outcome as the unlimited budget: an equal
    admitted package (equal graph, terms, models, selected documents, source map,
    diagnostics and lowering of every node) or an equal refusal or structural `incomplete`
    (code, cause, loci, `document_pointer`). Repeating a read with equal bytes, evidence
    and limits charges equal work and read storage and stops at the same first unpaid
    charge and locus.'
- id: FR-038-AC-516
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: The read's term walks call quire-walk's caller-reservation entry, and every
    integer or rational the read parses, reduces or compares while validating is built
    and combined through quire-exact's caller-reservation entries. For a package whose
    canonical rational literals, dimension exponents, timed-model magnitudes and schedulability
    bounds hold multi-limb values, and for a term walk whose stack grows several times,
    each walker growth and each limb request is charged at its full requested size,
    and a one-below limit at each refuses with form `"walk"` or `"exact_number"` and
    no partial result. Ca
- id: FR-038-AC-503
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: ModelDocument stage. For a package with two selected models, a read-storage
    limit one below the cumulative read storage through the second document's last
    reader reservation refuses with locus `PackageRead(ModelDocument, 1)` and form
    `"model_document"`, and no model admitted; the declaration tables decoded from
    the document (form `"declarations"`) and its retained `Arc<Document>` are each
    the denied request in one case. The document is read through the shared reader's
    reservation entry, its declarations are read from the document without a `serde_json::Value`,
    and its work charge (one unit per
- id: FR-038-AC-507
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: Retained stage. For a genuine package with direct dependencies, a read-storage
    limit one below the read's full read-storage requirement refuses at the last retained
    request with locus `PackageRead(Retained, 0)` and no package; the direct-dependency
    vector, the closure index, the assembled typed semantic graph, the node index,
    each shared `Arc` of the admitted package, the admission binding and the result
    `Box` are each the denied request in one case, each reporting its stage-table
    form. A read-storage limit equal to the requirement admits.
- id: FR-038-AC-511
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: 'Whole-read accounting, including each read''s owned catalog. A test binary
    whose global allocator counts, on the reading thread during the call, the size
    of each allocation request and the new size of each reallocation request reads,
    under an unlimited budget, an exact budget and each one-below budget of AC-497
    through AC-507: every positive TC-048 fixture, the QSpec checked-package conformance
    packages this repository reads, and the adverse shapes — the 100000-node reference
    chain, a 10000-node flat graph, a node with 10000 dependencies, a 1 MiB string
    member, 10000 source-map entries, 10000 d'
- id: FR-038-AC-513
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: The V2 read path (`CheckedPackageV2::read` from entry through returning
    its result, including the owned catalog, and every function it reaches) calls
    no `serde_json` parser, deserializer or `RawValue`, constructs no `serde_json::Value`,
    calls no `num-bigint` parse or arithmetic, sorts only with `sort_unstable*`, and
    builds no `BTreeMap`, `BTreeSet`, `HashMap` or `serde_json::Map`; its tables are
    sorted exact-capacity slices and its containers are reserved at exact capacity
    before they are filled. Every read parses the catalog with the shared `quire_canonical`
    reader and charges its own catalog
- id: FR-038-AC-517
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: The existing public types `CheckedPackageV2ReadResult`, `CheckedPackageDispatchResult`,
    `CheckedPackageLimit`, `CheckedPackageIncomplete`, `NativeJoinChargeKind`, `NativeJoinCharge`
    and `NativeJoinDeniedCharge` keep exactly their variants and fields. The budgeted
    entries return `CheckedPackageV2BudgetedReadResult` and `CheckedPackageBudgetedDispatchResult`,
    each holding the unbudgeted result's variants plus `BudgetIncomplete(CheckedPackageBudgetIncomplete)`
    and `Exhausted(CheckedPackageReadExhausted)`, with `read_checked_package_with_budget`
    mapping each V2 variant to its dispatch variant; `Pa
```
