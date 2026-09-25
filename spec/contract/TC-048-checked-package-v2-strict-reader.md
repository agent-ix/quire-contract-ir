---
id: TC-048
title: "CheckedPackage V2 strict reader re-derives package and nominal identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
---
# TC-048: CheckedPackage V2 strict reader re-derives package and nominal identities

## Description

Verify FR-038-AC-1 through FR-038-AC-5 (QSpec FR-322-AC-4, FR-322-AC-8,
FR-322-AC-10) against the vendored V2 fixtures and node-identity vectors, and
FR-038-AC-27 through FR-038-AC-30 (QSpec FR-322-AC-29 through FR-322-AC-34
"Model-owned members") against a self-built domain package document, and
FR-038-AC-31 through FR-038-AC-33 (QSpec FR-322-AC-35) against the
`DependencySelection` entries of a self-built package, and FR-038-AC-34
against the reader's and the bounded-Kani modules' own source, and
FR-038-AC-35 through FR-038-AC-38 (QSpec FR-322-AC-36 and FR-322-AC-37)
against a self-built dependency package and a package that calls it. QSpec's
own TC-280, TC-281, `dependency-selection-vectors.json` and `function-call`
node-identity vectors run through the same reader under
`make qspec-vectors`, which reads them from the checkout `QSPEC_DIR` names.

## Test Procedure

Dispatch an unknown, empty, absent and malformed `contract_version`, and
independently a malformed document, a duplicate top-level member and
noncanonical bytes, before any version is selected. Admit both vendored V2
positive fixtures. Apply every vendored adverse
structural mutation and compare the outcome. Independently inject malformed
JSON, a duplicate member, an unknown member, noncanonical bytes, an absent or
mismatched context digest, an unreported or unsupported required feature, a
dangling reference and an incomplete source map. Admit at exact byte, depth, node,
edge, occurrence, diagnostic and work limits, then lower each limit by one.
Recompute every fixture's package id, edit excluded and included preimage
members, and re-read. Re-derive each node-identity vector digest; build one
package holding every vector node; apply every vendored invalid mutation with
the retained digest, remove or swap a preimage, and change a retained
preimage's enum case, `semantic_type`, dependency and unit target while
mirroring the identity projection and package id. Re-own the nominal enum
declaration by a model owner (`identity`, `node`) over a locked `sha256-jcs`
domain package and re-read it; then name an unselected package, select none,
empty the node, restore a compiled-model owner or lock shape carrying
`authority`, `revision` or `export`, change the selection's digest domain,
version or evidence domain, and set a model node to `model_export`.

## Expected Results

An unknown, empty or absent `contract_version` refuses as
`unknown_contract_version`; a malformed `contract_version`, document, or a
duplicate or noncanonical top-level member refuses as `malformed_wire`,
`duplicate_member` or `noncanonical_wire` before any version-specific
decoding. Positive fixtures admit with their recorded package ids. Every adverse and
injected case returns its exact refusal code or incomplete accounting with no
package. Excluded edits keep the id and included edits change it. Every vector
digest matches; every nominal mutation and contradictory cross-field join
refuses as `invalid_semantic_graph`. The model-owned package validates
against the vendored schema and admits; each model join mismatch refuses as
`invalid_semantic_graph`, a compiled-model owner or lock shape carrying
`authority`, `revision` or `export` as `unknown_member`, a foreign digest
domain as `digest_domain_mismatch`, an absent or raw-only domain package
digest as `stale_dependency`, and `model_export` as `invalid_semantic_graph`.

## Parameter and compound-unit nodes (FR-038-AC-22, FR-038-AC-23)

Build a package in QSL FR-092's shapes for `function both(a: Boolean, b:
Boolean): Boolean { a and b }` plus the compound unit `Metre^2`, compare its
recomputed keys with QSL's FR-092 vectors, read it and lower every node.
Then mutate one parameter, the compound unit or the application node at a
time and re-read. The package admits and every node lowers; each mutation
refuses as `invalid_semantic_graph` at the member it breaks, located at the
mutated node, and the dimensionless compound unit admits.

## Model-owned members (FR-038-AC-27 through FR-038-AC-30)

Build a Semantic IR 2.0.0 document with an object type `Order` declaring the
operation `total(Integer): Integer`, select it in a package's lock and add a
`dispatch_call` on `Order.total` over a `Reference<Order>` receiver. Read it:
it admits. Rename the called operation to one the document does not declare
and read it: it refuses `ill_typed`/`operator-ineligible` at the member's
`name`. Give `Order` a supertype the document does not declare and read it:
the refusal is the document's, `missing_declaration`/`missing-name`, at
`/lock/model_selections/0`. Read the document directly: supply none, one
under another digest, forged bytes, another version, and the matching one;
give a node an invalid object id and refer to it as a supertype and a
`typeRef` from nodes on both sides of it; give one field a dangling `typeRef`
and a multiplicity of `lower > upper` in both member orders; name a
relationship as a `typeRef` from a node read before and one read after its
owner; give two nodes no identity. Compare the whole refusal and its pointer
with the expected one. Read with the `work` limit at zero, and with each work
limit one below what the direct read and a member resolution used, and check
the pointer of the `incomplete` outcome resolves in the lock.

## Dependency selections (FR-038-AC-31 through FR-038-AC-33)

Set the lock's and identity preimage's `dependency_selections` to two
`DependencySelection` entries in ascending identity order, re-derive the
package id and read: it admits, and both members read back as supplied.
Change one entry's `package_id` and check the package id changes. Mutate one
entry at a time to a wrong-domain, empty-identity, short-digest, bare-digest,
`version`-less or `Selection`-shaped entry and read. Repeat an identity,
adjacent and not, and reverse two entries, and read. Order two identities
whose UTF-8 and UTF-16 orders differ and read. Compare the whole refusal code,
cause and pointer with the expected one. Run QSpec's valid package, six entry
mutations and three order vectors through the same reader under
`make qspec-vectors`. With no dependency package supplied, QSpec's valid
package and its admitted order vectors refuse `missing_import`/`missing-selection`
at the first entry, after every array check passes.

## Dependency references (FR-038-AC-35 through FR-038-AC-38)

Declare the fixture's `function` node as a dependency function over a
bounded integer returning Boolean, read it, and lock its `package_id` as a
`dependency_selections` entry of a package whose function call has a
`dependency_reference` callee with that entry's `package` and the function as
`node`, its application key re-derived and no `dependencies` entry for the
target. Read it with the dependency supplied and check it admits and the term
is in the body verbatim. List the target in `dependencies`. Change only the
term's `package`, then only its `node`, keeping the key, and check each is a
stale key; re-derive each and check the three ids differ. Mutate the term to a
bare-digest, other-domain and short-digest `package`, another-domain `node`,
a missing `node` and an extra member. Place the well-formed term as a second
argument, as the argument of another operation, as an aggregate member and as a
node body root. Supply no package, another identity, another version and a
package of another `package_id`. Name a `package` no entry carries, a `node`
that is absent, one with no `declaration` and a declared record. Give the
dependency function a declared-record parameter and result, a `Set` of a
declared record, a tuple holding one and a `Reference` to a `model` node, and a
`Set` of a bounded integer. Compare the whole refusal code, cause and pointer
with the expected one. Place QSpec's `function-call` operation vector's
preimage with the dependency callee, compare the id the reader derives with the
id computed from the vector's own shape, and read the four term mutations
through the reader under `make qspec-vectors`.

## String edge (FR-038-AC-34)

Scan the non-test source of `src/kani/` and `checked_package/` for string
reads and compare the unmarked, unlisted ones with the empty set. Scan a
synthetic source for each kind of read (literal and constant comparison,
`as_ref` comparison, match arm, or-pattern, `matches!`, prefix test, downstream
`from_wire`, `Some("..")` comparison) and require a hit, and for enum
comparisons, comments, a `#[cfg(test)]` module and a marked edge and require
none. Reintroduce one string match in a function that is not an edge and
confirm the gate names it.

## Refusal and limit locations (FR-038-AC-24 through FR-038-AC-26)

For every refusal above, compare the whole refusal, including its pointer,
with the expected one. Check that each pointer resolves in the mutated
document. Add an unknown member whose name holds `~` and `/` at the top
level, on a node and on a locked source, and repeat a top-level member whose
name holds both. Dispatch the unknown versions again and read the version
each refusal carries. For each one-over limit, compare the pointer the
`incomplete` outcome carries and check that it resolves.

Each refusal points at exactly the value its mutation changed, with `~`
escaped as `~0`, `/` as `~1`, and array elements by index. A missing member
is located at the object that lacks it. Malformed JSON and non-canonical
bytes carry no pointer. Every refusal code is unchanged. Each
`unknown_contract_version` refusal carries the exact string read, including
the empty one. Each one-over limit other than the byte limit names the value
whose charge failed, and that pointer resolves; the byte limit names none.
