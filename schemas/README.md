# Schemas

Four contracts, one class. What is here is what this repository's own domain
artifacts are shaped like; every generic evidence schema the campaign once
carried is deleted.

## Domain output — live, written against, owned here

| File | Describes |
| --- | --- |
| `contract-conformance-fixture-v1.schema.json` | The conformance corpus fixture inputs and expectations for each operation. |
| `contract-package-reference-v1.schema.json` | The serialized contract package wire form. |
| `temporal-ecosystem-manifest-v1.schema.json` | The nine-repository campaign's contract selections, semantic nodes, typed edges, and unresolved gaps admitted by FR-027. |
| `temporal-ecosystem-model-v1.schema.json` | The deterministic bounded non-authoritative model exported and strict-read by FR-027. |

These describe *this repository's own domain artifacts*. A schema that
describes a contract package or a conformance corpus is not a generic evidence
family, and the migration contract says so in as many words. They stay, they are
validated against, and they evolve with the contract model.

## Deleted — the evidence schemas

`pgm01-evidence-v1.schema.json` and `evidence-correction-v1.schema.json` were
frozen rather than deleted by the shared-assurance migration, because every one
of the ten retained manifests under `evidence/` named one of them by path and
digest as the shape it was written to. Deleting a file while immutable records
had to resolve a reference to it would have broken bytes the migration was
required to leave untouched.

`derivation-evidence-envelope-v1.schema.json` outlived that round. It was live
at the time — `scripts/validate_governance.py` loaded it and validated
`corpus/governance/` against it on every `make governance` — so it was kept, and
correctly so on the evidence then available. It was still the same deprecated
pre-stable evidence format, and the rule that required it, PGM-01-R08, is now
withdrawn on the same ground and under the same decision as the retained records
themselves
([engineering-assurance#7](https://github.com/agent-ix/engineering-assurance/issues/7)).
The schema, its validator, its fourteen fixtures, the `make governance` target
and the pinned Draft 7 Python lane are deleted together, because a validator
with nothing left to validate is not a gate.

All three are gone. Nothing replaces them: no successor envelope, no second
result family, no local validator.

## Why the two above stayed

Each was checked by grepping for its filename and `$id` across `src/`,
`scripts/`, `tests/`, `corpus/` and `spec/`, and then by mutation — editing the
file and watching a gate go red — rather than by reading its name:

- `contract-conformance-fixture-v1.schema.json` — the runner names its `$id`
  and validates every fixture input and expectation against it;
  `schemas/contract-executable-projection-v1.schema.json` references its
  definitions. Live.
- `contract-package-reference-v1.schema.json` — the runner names its `$id` and
  validates every successful package fixture against it. Live.

The runner reads both from this directory (`--schemas schemas`).
