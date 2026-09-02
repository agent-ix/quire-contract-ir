# assurance/

What this repository *states* about a change, in the shapes the shared
components require. Nothing here is evidence, and nothing here is retained.

| File | What it is |
| --- | --- |
| `pins.json` | The Engineering Assurance release this repository adopts, and the digests of the artifacts it actually reads from it. |
| `change-assurance.json` | The declaration for issue #39: its requirements, preservation constraints, proof obligations, and open unknowns. |

## Why there is no evidence in here

Retention is Quoin's. `scripts/assurance_chain.py` writes into a Quoin store
under `target/`, which is not tracked, and `make assurance-record` writes a
transcribed run into the Quoin evidence store the tool itself lays out. This
directory holds only inputs a human or agent authored, so that a reviewer can
read what was claimed separately from what was measured.

Neither store is committed, and that is a decision rather than an oversight. A
record naming a revision, stored in a commit that changes the revision, is stale
the moment it lands — which is precisely what left the deleted verifier red on
`main`, where no retained record's subject tree still matched `HEAD`. Where a
store is retained is a deployment decision. What is proven on every invocation
is the path, not a snapshot of one run of it.

`evidence/` is the other thing that is not in here: ten immutable PGM-01 records
and one append-only correction, frozen, read through Engineering Assurance's
compatibility mapping and never written to again.

## What derives what

Nothing in `change-assurance.json` is inferred except the digests its own
`derived_fields` list names — the bytes at a declared path, and the candidate
revision the caller states. A field the declaration does not state is a
validation failure at seal time rather than a blank, which is what makes the
sealed record a statement somebody made rather than a shape a tool filled in.

## The chain

```bash
make assurance-inputs   # the native producers run here, and only here
make assurance          # pins, legacy compatibility, and the Quoin chain
```

The order matters and the split is the point. `assurance-inputs` runs the
contract conformance runner and `quire coverage`. Everything after it consumes
files that already exist: Quire exports static facts and executes nothing, Quoin
transcribes, retains, audits, and reports bytes it is handed and executes
nothing. If a step in the chain ever needs to run a producer to answer its
question, the boundary has moved and the answer is not trustworthy.

## The decision that is not here

There is no approval in this directory and there will not be one. The
verification receipt for this candidate reads `incomplete` with reason
`decision_missing`, because only `@kreneskyp` can record an ix-flow decision
event and none exists. That is the honest state of the candidate, not a gap in
the tooling, and a receipt that read `valid` without one would be the single
worst thing this migration could produce.
