---
id: SR-6509
title: spec-review/risk-complexity of IR-738 candidate application identity
type: SpecReview
analysis: risk-complexity
scope: agent-ix/quire-contract-ir@431f218fad539f01764fba4807a712ae16e21010; spec/checked_package/functional/FR-038-consume-checked-package-v2.md, spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md, spec/model/functional/FR-019-rust-library-interface.md, spec/model/functional/FR-046-derive-candidate-application-node-identity.md, spec/model/matrix/TC-058-model-crate-public-interface.md
review_set: subset
---

## Summary

Ticket: IR-738. Frozen five-document spec diff only; no runtime acceptance is claimed. Installed quoin 0.28.3 / quire 0.36.2 (engine 0.50.2). Focused validation passed (5/5 grammar-clean). GitHub CLI metadata queries failed HTTP 401; independently verified GitHub connector metadata confirms public repository 1351621876, PR 343 open/draft and the exact frozen head/base. No secret or configuration repair. Model ID is unavailable to this harness; no alternate-model claim. No applicable AssuranceProfile found. No Cargo, Kani, replay, build, lockfile, full gate, or product edit.

FR046 technical risk High / volatility Low: canonical identity and trust-boundary sharing require the independent preimage, actual-reader stale-key and group-substitution controls already planned. FR019 and FR038 changed cross-reference portions are Low technical / Low volatility, since ownership and accounting are unchanged. Top hazards are group-order drift, outside-group re-derivation omission and preimage-owner duplication; planned AC3/AC6 controls mitigate them. Deep-input handling remains the explicit failure-domain finding, not a claimed measured budget failure.

## Verdict

**PASS** — No method-specific defects found in the examined diff.

## Resource consistency

Resource consistency: N(N-1)/2 uniqueness comparisons and O(1) borrowed context storage are explicit. No repository NFR imposes a new helper CPU or heap ceiling. O(B) grouped body copy, O(TG) member search / up to N-node borrowed scans, and encoder/walk heap costs are disclosed separately from canonical output bytes. Reader work remains one charge before preimage/hash and does not acquire public-context validation. FR-019/FR-023 v1 limits do not apply to this V2 surface (FR-038 explicitly separates them). FR-038 AC214 memory measurement applies to actual reader calls, not helper drafts; its text and IR742 AC222/223 remain unchanged. No invented cap, alternate API, or reader-bound reduction is required.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Reviewed scope

```yaml
clean: true
scope:
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: '[FR-046](../../model/functional/FR-046-derive-candidate-application-node-identity.md)'
  lines: '3273'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: owns the PLANNED/UNRUN public pre-read candidate-key helper. The reader and helper
  lines: '3274'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: shall share the authoritative application preimage construction and canonical
  lines: '3275'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: encoder. This sharing retains the reader's existing authenticated all-node
  lines: '3276'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: graph-order context, refusal translation, and one work-unit charge before
  lines: '3277'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: preimage construction and hashing; it does not run public draft-context
  lines: '3278'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: validation in the reader or add its refusals or charges. A helper candidate is
  lines: '3279'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: not admission evidence, and a changed body retaining that key is still subject
  lines: '3280'
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: examined
  excerpt: to the reader's stale-key check.
  lines: '3281'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: '  - target: ix://agent-ix/quire-contract-ir/FR-046'
  lines: '8'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: '    type: verifies'
  lines: '9'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-738).
  lines: '1645'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: Verify [FR-046](../../model/functional/FR-046-derive-candidate-application-node-identity.md), AC-1 through AC-6, through the public model API and actual `CheckedPackageV2::read`; no test replaces the production hash/helper/reader.
  lines: '1647'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 1. Before any package read, derive an ordinary application candidate and compare with an independently built canonical preimage. Change node tag, semantic form, semantic type, declaration absence/presence/content and body separately; compare actual digests. Alter only the provisional self selector and confirm unchanged ungrouped key.
  lines: '1649'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 2. Build a reader-admissible group containing application and nonapplication nodes in deliberately observable graph order. Include self and internal references through application arguments, aggregate members and binding values, plus outside-group and dependency references. Independently enumerate size/ordinals and compare the helper digest. Replace provisional keys and matching internal references with final candidate keys without reordering or relabelling; refresh dependencies, projection and package identity under existing contracts. Read the actual final wire and assert admission and exact
  lines: '1650'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: ' application keys. A changed outside target changes the helper digest.'
  lines: '1650'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 3. Missing grouped context, an absent target, repeated keys at distinct graph positions, a target with different fields, context from another draft, and a nonapplication draft each assert the precise typed variant and payload and absence of a candidate. Multiple repeats assert the earliest repeated position and earliest equal predecessor. Inspection checks constant-size borrowed context, checked position/count conversions and no hidden graph/group-vector clone or new cap.
  lines: '1651'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 4. Independently encode the successful preimage and measure C bytes. Limits C and C+1 return the same key; C-1 and zero return the typed canonical byte-limit error. A number that canonical encoding refuses returns its typed encoder error, rather than a byte-limit error or candidate. Do not count this as a memory/work bound over the graph or copied body.
  lines: '1652'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 5. Read reader-admissible ordinary and grouped final assemblies containing the helper candidates. Change a body's semantic content while retaining the candidate key and refresh only the existing package identity as required to reach node-key validation. Assert stale-node-key and the distinct expected node key. Use the existing reader work meter with an exhausted application charge and a later failing hash; assert the work refusal wins. Source inspection verifies a single authoritative application preimage/encoder, unchanged charge-before-preimage/hash, and no public-context validation in reade
  lines: '1653'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: r admission.
  lines: '1653'
- id: TC-048
  path: spec/checked_package/matrix/TC-048-checked-package-v2-strict-reader.md
  role: examined
  excerpt: 'These controls are PLANNED/UNRUN; this spec-only change supplies no runtime result. Bind eventual tests by `Trace: FR-046-AC-N`, never by a fabricated matrix row.'
  lines: '1655'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: The pre-read application identity items in
  lines: '142'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '[FR-046](./FR-046-derive-candidate-application-node-identity.md) are'
  lines: '143'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: PLANNED/UNRUN. They borrow a draft node and its ordered graph context, and return
  lines: '144'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: only a candidate key. A provisional self selector is not a verified semantic
  lines: '145'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: identity. The explicit canonical byte ceiling does not bound graph-context
  lines: '146'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: validation or body-copy memory. Reader admission remains required after
  lines: '147'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: final-key/reference substitution. The new row below is the required interface
  lines: '148'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: upon implementation, not a claim that these exports exist today.
  lines: '149'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: examined
  excerpt: '| `checked_package` (V2 candidate application identity; PLANNED/UNRUN) | function `derive_application_node_id`; types `ApplicationNodeDraft`, `ApplicationIdentityContext`, `CandidateApplicationNodeId`, `ApplicationIdentityError` | [FR-046](./FR-046-derive-candidate-application-node-identity.md) |'
  lines: '171'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-738). The API and its acceptance controls below are required
  lines: '17'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: upon implementation; this specification does not claim present exports or
  lines: '18'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: executed runtime evidence.
  lines: '19'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: When a caller supplies a draft application before checked-package reader
  lines: '21'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: admission, `quire-contract-model` SHALL derive its candidate canonical node
  lines: '22'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: identity through the reader's authoritative application preimage and encoder.
  lines: '23'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: A borrowed draft `CheckedSemanticNodeV2`, an optional immutable context built
  lines: '27'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: from the caller's complete ordered draft graph, and an explicit canonical
  lines: '28'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`byte_limit: u64`. A draft graph is pre-read input, not an admitted package.'
  lines: '29'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`Result<CandidateApplicationNodeId, ApplicationIdentityError>`. A candidate'
  lines: '33'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: contains the canonical `CheckedNodeId` for assembling the package wire; it
  lines: '34'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: does not certify admission, operation laws, reference resolution or group
  lines: '35'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: validity of that package.
  lines: '36'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The model crate SHALL export `ApplicationNodeDraft`,
  lines: '42'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`ApplicationIdentityContext`, `CandidateApplicationNodeId`,'
  lines: '43'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`ApplicationIdentityError` and `derive_application_node_id` by name.'
  lines: '44'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: These items have private storage and no unchecked context constructor.
  lines: '45'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`ApplicationNodeDraft::new(&CheckedSemanticNodeV2)` borrows the supplied node.'
  lines: '47'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'Its `node_id` is a provisional self/reference selector, explicitly unverified:'
  lines: '48'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: the final canonical ID need not be known when the draft is created. The wrapper
  lines: '49'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: does not validate or mutate that node. `ApplicationIdentityContext::try_new`
  lines: '50'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: takes `&[CheckedSemanticNodeV2]` and the draft; it returns a context or the
  lines: '51'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: typed error below. The context borrows this complete ordered snapshot and
  lines: '52'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: retains private validated target/group indices, not an owned graph or an
  lines: '53'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: independently mutable group list.
  lines: '54'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`derive_application_node_id(draft, context, byte_limit)` accepts a borrowed'
  lines: '56'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: draft, `Option<&ApplicationIdentityContext>` and `u64`, and returns the typed
  lines: '57'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: result above. `CandidateApplicationNodeId::candidate_node_id()` returns an
  lines: '58'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: immutable `&CheckedNodeId`; its name and documentation identify it as a
  lines: '59'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: candidate. No helper result converts to `CheckedPackageV2` or asserts that a
  lines: '60'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: provisional selector is an admitted semantic identity.
  lines: '61'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: When constructing context, the constructor SHALL find exactly one graph node
  lines: '65'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: with the draft's supplied key and require that node's complete fields to equal
  lines: '66'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: the borrowed draft. All supplied graph node keys are unique. The group is every
  lines: '67'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: node carrying the target's recursion label, filtered from the complete graph
  lines: '68'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: in its existing order, including nonapplication nodes. Its size and target
  lines: '69'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: ordinal are derived from that order; an arbitrary caller ordinal or a list
  lines: '70'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: containing only applications is not an accepted substitute. A context for an
  lines: '71'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: ungrouped draft has no group. A grouped draft requires context; an ungrouped
  lines: '72'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: draft accepts no context or its matching ungrouped context.
  lines: '73'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: If a context or draft check fails, the helper SHALL return the corresponding
  lines: '75'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`ApplicationIdentityError` variant with no candidate:'
  lines: '76'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| Variant | Condition and typed payload |'
  lines: '78'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `NonApplication` | The draft''s body root is not `application`; carries its supplied `CheckedNodeId`. |'
  lines: '80'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `MissingContext` | A grouped draft has no context; carries its supplied `CheckedNodeId`. |'
  lines: '81'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `MissingTarget` | The complete snapshot has no node with the draft''s supplied key; carries that key. |'
  lines: '82'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `RepeatedKey` | A graph key is repeated; carries the key and first/repeated graph positions as `u64`. |'
  lines: '83'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `ContextMismatch` | The snapshot target differs from the draft, or a derivation uses a context constructed for a different draft; carries the supplied draft key. |'
  lines: '84'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `ContextIndexOutOfRange` | A graph position, group size or ordinal cannot be represented as `u64`; carries the supplied draft key. |'
  lines: '85'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| `CanonicalEncoding` | The canonical encoder refuses the preimage, including the byte ceiling; retains the typed `quire_canonical::Error`. |'
  lines: '86'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: Construction scans repeated keys in graph order and selects the earliest
  lines: '88'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: repeated position, then the earliest equal predecessor; this refusal precedes
  lines: '89'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: missing-target and target mismatch. Derivation checks application root, then
  lines: '90'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: required/matching context, then canonical encoding. Group context never falls
  lines: '91'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: back to an ungrouped hash on failure. The constructor verifies the supplied
  lines: '92'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'snapshot, not its completeness relative to an unavailable future package:'
  lines: '93'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: supplying the complete intended graph remains the caller's obligation.
  lines: '94'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The helper and reader SHALL use one application preimage construction and one
  lines: '98'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'canonical encoder for `{version: "quire.application-node/v1", node_tag,'
  lines: '99'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: semantic_form, semantic_type, declaration, recursion, body}`. Missing
  lines: '100'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: declaration and ungrouped recursion encode as `null`. Grouped recursion encodes
  lines: '101'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`{size, ordinal}`. Body `reference` terms targeting group members encode as'
  lines: '102'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`{term: "group_reference", ordinal}` through application arguments, aggregate'
  lines: '103'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: members and binding values. Outside-group references and dependency references
  lines: '104'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: retain their supplied content. Semantic type and declaration retain their
  lines: '105'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: original preimage rules; they are not rewritten as body references. No
  lines: '106'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: provisional key is added as an extra preimage member.
  lines: '107'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The helper SHALL hash that preimage through `quire-canonical` with
  lines: '109'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`Limits::new(byte_limit)` and return domain `quire.checked-semantic-node/v1`.'
  lines: '110'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: Its byte ceiling is exactly the reader's application-hash ceiling, including
  lines: '111'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: success when the canonical preimage byte count equals the limit and refusal
  lines: '112'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: when it exceeds it. No alternate encoder, fallback hash or new ceiling exists.
  lines: '113'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The public context constructor SHALL borrow the graph without cloning nodes,
  lines: '117'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: bodies or a group-member vector. With N supplied nodes it performs at most
  lines: '118'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: N(N-1)/2 key comparisons for uniqueness and linear scans for target/group
  lines: '119'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: selection, plus one target-node equality comparison. Auxiliary context storage
  lines: '120'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'is constant in N: borrowed snapshot, target position, group size and ordinal.'
  lines: '121'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: Key comparison and node equality costs include the supplied key/field sizes.
  lines: '122'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: Position/count conversions are checked. No new graph-size cap, work meter,
  lines: '123'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: global cache or hidden owned graph is introduced.
  lines: '124'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The shared grouped-body rewrite currently needs one copied body, visits each
  lines: '126'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: term once and searches the group for each reference. With B body storage,
  lines: '127'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: T visited terms and G group nodes, its body copy is O(B) and reference selection
  lines: '128'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: is O(TG); a borrowed group scan can inspect up to N nodes per reference.
  lines: '129'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: Traversal and canonical encoding retain their existing heap-stack costs.
  lines: '130'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: 'These costs are bounded by the actual supplied input, not by `byte_limit`:'
  lines: '131'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: that limit constrains canonical output and does not bound context scans,
  lines: '132'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: the draft graph, body cloning or traversal allocations. This contract makes
  lines: '133'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: no new recoverable-allocation guarantee for the existing body clone or encoder;
  lines: '134'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: an allocator exhaustion outside their typed error surface is not a fabricated
  lines: '135'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '`CanonicalEncoding` refusal. Context construction itself needs no heap allocation.'
  lines: '136'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The reader SHALL retain its existing application ordering, authenticated graph
  lines: '138'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: context, refusal translation and one reader work-unit charge before preimage
  lines: '139'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: construction and hashing. Sharing the preimage/encoder does not call the public
  lines: '140'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: context constructor in the reader or add its checks, refusals or work charges
  lines: '141'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: to reader admission.
  lines: '142'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The caller SHALL replace every provisional key and matching reference in the
  lines: '146'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: final wire with the corresponding candidate/final node key while preserving
  lines: '147'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: the intended graph order and recursion labels. For group-internal body
  lines: '148'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: references, this substitution preserves the preimage's ordinals, including
  lines: '149'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: self references. Outside-group references must already name their intended
  lines: '150'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: final targets when deriving the referencing candidate; changing one later
  lines: '151'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: requires re-derivation. The same applies to semantic type and declaration
  lines: '152'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: members that bind identities. The helper does not compute another node's key
  lines: '153'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: or perform substitutions for the caller. The caller refreshes affected
  lines: '154'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: dependencies, identity projection and package identity under their existing
  lines: '155'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: contracts, then invokes `CheckedPackageV2::read` for final admission. A retained
  lines: '156'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: candidate after a body change remains subject to the reader's stale-key refusal.
  lines: '157'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '| ID | Criteria | Verification |'
  lines: '161'
- id: FR-046-AC-1
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: An external model-crate consumer derives a candidate from a typed draft before any package read, obtains its candidate key and matches typed errors; no helper output is an admission verdict or admitted-package constructor. The named public items are exported only by the model crate.
  lines: '163'
- id: FR-046-AC-2
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: An ordinary application produces the independently constructed canonical application-preimage digest. Changing node tag, semantic form, semantic type, declaration presence/content or body independently changes the bound digest. Changing only its provisional selector does not change an ungrouped candidate.
  lines: '164'
- id: FR-046-AC-3
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: A group containing applications and nonapplication nodes uses every group node in graph order. Self and internal references through application arguments, aggregate members and binding values produce the expected ordinals; provisional-to-final key substitution preserves the candidate. Outside-group and dependency references remain unchanged, and changing an outside target changes the candidate.
  lines: '165'
- id: FR-046-AC-4
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: Missing grouped context, absent target, repeated graph key, mismatching target fields or context from another draft return their exact typed variants/payloads and no hash candidate. Repetition order is deterministic; nonapplication input refuses. Inspection confirms borrowed immutable constant-size context, checked counters and the stated scan bounds, without a new cap or allocator-dependent context failure.
  lines: '166'
- id: FR-046-AC-5
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: For an independently encoded preimage of C canonical bytes, byte limits C and C+1 produce the same candidate; C-1 and zero refuse CanonicalEncoding with the canonical byte-limit error. A canonical nonencodable-number control retains its typed encoder error. The byte ceiling is not asserted to bound graph/context/body-copy memory or work.
  lines: '167'
- id: FR-046-AC-6
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: After correct final-key/reference substitution, reader-admissible ordinary and grouped controls read successfully with the helper candidate. Changing the body while retaining that key produces the reader's stale-node-key outcome and distinct expected key. Inspection finds one authoritative preimage/encoder and unchanged reader charge-before-hash ordering; an exhausted existing work charge wins before an otherwise failing application hash. Public context validation adds no reader refusal or work charge. All runtime procedures remain planned/unrun until implementation.
  lines: '168'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: '[FR-019](./FR-019-rust-library-interface.md) owns the explicit model-crate'
  lines: '172'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: public inventory. [FR-028](./FR-028-separate-cycle-free-contract-model.md)
  lines: '173'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: owns the cycle-free substrate boundary. [FR-038](../../checked_package/functional/FR-038-consume-checked-package-v2.md)
  lines: '174'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: owns reader admission, application-key preimage semantics and stale-key refusal.
  lines: '175'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: The existing [TC-048](../../checked_package/matrix/TC-048-checked-package-v2-strict-reader.md)
  lines: '176'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: and [TC-058](../matrix/TC-058-model-crate-public-interface.md) are the intended
  lines: '177'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: procedure artifacts; the new runtime controls are PLANNED/UNRUN. This slice
  lines: '178'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: adds no compatibility layer, schema, downstream engine dependency, operation
  lines: '179'
- id: FR-046
  path: spec/model/functional/FR-046-derive-candidate-application-node-identity.md
  role: examined
  excerpt: law, package identity helper or reader gate.
  lines: '180'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: '  - target: ix://agent-ix/quire-contract-ir/FR-046'
  lines: '8'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: '    type: verifies'
  lines: '9'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: PLANNED/UNRUN (IR-738). Verify
  lines: '52'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: '[FR-046](../functional/FR-046-derive-candidate-application-node-identity.md),'
  lines: '53'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: AC-1, and the corresponding planned row in
  lines: '54'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: '[FR-019](../functional/FR-019-rust-library-interface.md).'
  lines: '55'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: Add the five named FR-046 items to the inventory comparison upon implementation.
  lines: '56'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: Compile a consumer probe that creates a borrowed `ApplicationNodeDraft`,
  lines: '57'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: constructs `ApplicationIdentityContext`, calls `derive_application_node_id`,
  lines: '58'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: obtains `CandidateApplicationNodeId::candidate_node_id()` and exhaustively
  lines: '59'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: matches `ApplicationIdentityError`. The consumer assembles a candidate before
  lines: '60'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: reading any package; no helper result constructs or substitutes for an admitted
  lines: '61'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: '`CheckedPackageV2`. Preserve the existing model-only/root-crate path checks.'
  lines: '62'
- id: TC-058
  path: spec/model/matrix/TC-058-model-crate-public-interface.md
  role: examined
  excerpt: This additional procedure supplies no executed evidence until implementation.
  lines: '63'
- id: FR-019
  path: spec/model/functional/FR-019-rust-library-interface.md
  role: context_only
  excerpt: 'wire-independent `MAX_SEMANTIC_NODES: u32 = 25000`,

    `MAX_SEMANTIC_DEPTH: u32 = 256`, and

    `MAX_SEMANTIC_COLLECTION_ITEMS: u32 = 10000`, plus the parser guard

    `MAX_WIRE_JSON_DEPTH: u32 = 576`. A complete operation input may

    contain at most that many decoded semantic nodes; nested value-type or other

    recursive structure may be at most that deep; and every declaration,

    requirement, clause, field, variant, parameter, trace, item, or other semantic

    collection may contain at most that many entries. Preflight is iterative and

    occurs before recursive validation, canonicalization, or coverage.

    The first'
  lines: 128-141
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: '`quire-canonical` crate, which this repository depends on at `branch = "main"`

    as a git dependency, with its source allow-listed in `deny.toml`. The crate

    owns the encoder; this repository holds no copy of it, adds no wrapper type

    around it and writes no `DEPTH` literal. Every identity digest this repository

    computes is computed through `quire-canonical`, and no encoder of this

    repository''s own remains: the `package_id`, a nominal preimage''s digest, an

    application and a structural node key, a lowered node''s `ir_id`, a lowered

    package''s bytes and `package_id`, and a selected model document''s di'
  lines: 394-425
- id: FR-038
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: '### The flat wire


    This reader enforces the closed body grammar of merged QSpec FR-322

    "Body grammar" (FR-322-AC-39 through FR-322-AC-41; FR-341-AC-6; FR-370-AC-5 and

    FR-370-AC-12; QSpec TC-233, TC-303 and TC-427 BG-02), and this section states what

    that requires of this reader without copying the grammar. Where the reader and

    that text differ, the text is this reader''s requirement; a reading of a nested

    application in "Operation identity", "Application node keys" and the temporal and

    state-clause sections is superseded by it. The

    grammar covers every node `body` and every diagnostic `details`'
  lines: 642-704
- id: FR-038-AC-214
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: '| FR-038-AC-214 | CODE-GATED (IR-575; isolated 2026-10-10 measurement passed, final repo gates pending). The production `CheckedPackageV2::read` path directly decodes the closed wire into the typed package without ever constructing a full-document `serde_json::Value` tree or allocating a second full-document canonical byte buffer; `Value` terms inside the typed wire remain permitted. Source inspection and the real-reader adverse corpus confirm unchanged duplicate-member, noncanonical-byte, unknown-member and first-refusal code/pointer outcomes, unchanged identity and source-map content, and th'
  lines: 4091-4091
- id: FR-038-AC-222
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: '| FR-038-AC-222 | PLANNED/UNRUN (IR-742). Through `CheckedPackageV2::read`, a `float32_bits` literal declared at a Float32 type and a `float64_bits` literal declared at a Float64 type each admit in otherwise valid packages. Retaining each value kind and changing only its declared `type` to the opposite IEEE width, while re-deriving the affected node and package identities, refuses `ill_typed`/`operator-ineligible` at that literal''s `type`, including in an operand position whose catalogued family would otherwise accept the declared type. Neither mutation is admitted or refused merely at an encl'
  lines: 4089-4089
- id: FR-038-AC-223
  path: spec/checked_package/functional/FR-038-consume-checked-package-v2.md
  role: context_only
  excerpt: '| FR-038-AC-223 | PLANNED/UNRUN (IR-742). Through `CheckedPackageV2::read`, valid `quire.op.ieee.to_float32` over Float64 and `quire.op.ieee.to_float64` over Float32 admit when each `type_argument` and `result_type` name the same correctly typed destination node. For each identity, changing `result_type` and the enclosing application''s root-node `semantic_type` together to the source-width node while retaining the valid destination `type_argument`, and re-deriving all affected identities, refuses `ill_typed`/`operator-ineligible` at the application''s `result_type` by the `member` result form; '
  lines: 4090-4090
- id: FR-028
  path: spec/model/functional/FR-028-separate-cycle-free-contract-model.md
  role: context_only
  excerpt: "\nThe Contract IR repository SHALL expose its stable semantic substrate only from a dependency-free `quire-contract-model` package, which the `quire-contract-ir` package consumes without re-exporting it, so the production Cargo graph remains acyclic, the root package depends on no crate from the agent-ix/quire-spec-language repository, and every model item has one import path.\n\n## Package architecture\n\nThe repository becomes one Cargo workspace with:\n\n- `quire-contract-model`: the existing package/requirement/clause, type,\n  expression, definedness, canonicalization, identity, binding, limit,"
  lines: 16-24
- id: StR-001
  path: spec/core/stakeholder/StR-001-authoritative-contract.md
  role: context_only
  excerpt: 'Requirement authors need one versioned semantic contract whose identity,

    expressions, definedness conditions, and execution anchors survive every

    downstream lowering without hand-authored semantic copies.


    ## Rationale'
  lines: 10-14
- id: NFR-001
  path: spec/core/non-functional/NFR-001-determinism.md
  role: context_only
  excerpt: 'canonical bytes, digests, dependency sets, and coverage classes across supported

    operating systems and process runs when input bytes, supported profiles, and

    dependency versions are fixed.'
  lines: 15-17
- id: NFR-002
  path: spec/core/non-functional/NFR-002-portability-independence.md
  role: context_only
  excerpt: 'width, operating-system paths, solver APIs, runtime APIs, and architecture-model

    vocabulary.

    '
  lines: 15-17
- id: crates/quire-contract-model/src/checked_package/v2/operations.rs
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: context_only
  excerpt: "pub(super) fn validate_application_keys(\n    nodes: &[CheckedSemanticNodeV2],\n    index: &BTreeMap<&CheckedNodeId, usize>,\n    meter: &mut WorkMeter,\n    bytes: u64,\n) -> Result<(), ValidationFailure> {\n    // Each recursion group's members, in graph order.\n    let mut groups: BTreeMap<&str, Vec<&CheckedNodeId>> = BTreeMap::new();\n    for node in nodes {\n        if let Some(label) = node.recursion_group.as_deref() {\n            groups.entry(label).or_default().push(&node.node_id);\n        }\n    }\n    for (&node_id, &position) in index {\n        let node = &nodes[position];\n        if !is_appli"
  lines: 132-181
- id: crates/quire-contract-model/src/checked_package/v2/operations.rs
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: context_only
  excerpt: "fn group_references<'a>(term: &'a Value, group: &[&CheckedNodeId]) -> Cow<'a, Value> {\n    if group.is_empty() {\n        return Cow::Borrowed(term);\n    }\n    // One copy of the body, rewritten in place by one visit of each of its\n    // terms: no enclosing term is copied again at any level.\n    let mut rewritten = term.clone();\n    let ControlFlow::Continue(()) = walk(&mut GroupRewrite { group }, &mut rewritten);\n    Cow::Owned(rewritten)\n}\n\n/// The walk of [`group_references`], over the terms of a copy of the body: each\n/// `reference` to a member of the recursion group becomes"
  lines: 224-236
- id: crates/quire-contract-model/src/checked_package/v2/mod.rs
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: context_only
  excerpt: "/// A checked V2 semantic graph node.\n#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct CheckedSemanticNodeV2 {\n    /// Stable checked node identity.\n    pub node_id: CheckedNodeId,\n    /// Must be `quire.checked-semantic-graph/v2`.\n    pub schema_version: Box<str>,\n    /// Closed semantic family (see [`CheckedNodeTag`]).\n    pub node_tag: Box<str>,\n    /// Closed semantic form for the family.\n    pub semantic_form: Box<str>,\n    /// Checked semantic type identity.\n    pub semantic_type: CheckedNodeId,\n    /// Identity-based dependencies.\n "
  lines: 118-150
- id: crates/quire-contract-model/src/checked_package/shared.rs
  path: crates/quire-contract-model/src/checked_package/shared.rs
  role: context_only
  excerpt: "\n/// A checked semantic graph node identity.\n#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, FixedShape)]\n#[serde(deny_unknown_fields)]\npub struct CheckedNodeId {\n    /// Must be `quire.checked-semantic-node/v1`.\n    pub domain: Box<str>,\n    /// Lowercase SHA-256 digest.\n    pub digest: Box<str>,"
  lines: 424-432
findings: []
bindings: []
```

## Limitations and skipped methods

Production gap-analysis, runtime Rust review gates, correctness/fuzz/matrix authoring are inapplicable to a spec-only diff. Architecture/security/app reviews add no new system architecture, compliance standard or UI scope. Calibrated criterion-strength is unavailable: installed schema includes its enum, but no Jev client/tool is exposed and the installed skill explicitly says not to run without the client. No calibrated verdict or fabricated artifact is emitted. Manual AC falsifiability belongs to the base/integrity review, not a substitute for Jev.
