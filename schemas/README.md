# Schemas

Five files, three classes, and the difference between them is what the shared
assurance migration turned on.

## Domain output — live, written against, owned here

| File | Describes |
| --- | --- |
| `contract-conformance-manifest-v1.schema.json` | The conformance corpus manifest: which fixtures exist, what each one covers, and the digests that identify them. |
| `contract-package-reference-v1.schema.json` | The serialized contract package wire form. |

These describe *this repository's own domain artifacts*. A schema that
describes a contract package or a conformance corpus is not a generic evidence
family, and the migration contract says so in as many words. They stay, they are
validated against, and they evolve with the contract model.

## Domain derivation record — live, historical interoperability

| File | Describes |
| --- | --- |
| `derivation-evidence-envelope-v1.schema.json` | The v0.1 domain-derivation record: which producer, which inputs, which backend, which outputs, and a typed result that keeps `inconclusive`, `unsupported`, `rejected`, `timed-out`, `pending`, and `error` apart from success. |

`scripts/validate_governance.py` and `corpus/governance/` are its gate, and both
are `KEEP` in the accepted migration decision table. FR-008 classifies the
record as producer-owned structured output and the schema as a historical
compatibility surface — not an evidence store, not a universal runner, and not a
parallel result family. Quoin may retain and audit such a record; it does not
own its shape, because the shape is a domain producer's own.

## Frozen historical — read, never written

| File | Describes |
| --- | --- |
| `pgm01-evidence-v1.schema.json` | The retained PGM-01 evidence manifest, as ten immutable records in `evidence/` were validated against it. |
| `evidence-correction-v1.schema.json` | The append-only correction that supersedes a claim in one of those records. |

**Nothing validates against these any more.** The verifier that did was the
repository-local retention and integrity authority the migration removes, and it
is gone. What replaced it is not another local reader: `evidence/` is read
through Engineering Assurance's read-only compatibility mapping, driven by
`scripts/pgm01_compatibility_view.py`.

They are not deleted, and the reason is specific rather than sentimental. Every
one of the ten retained manifests carries

```json
"schemaIdentity": {
  "path": "schemas/pgm01-evidence-v1.schema.json",
  "sha256": "..."
}
```

— an immutable record naming this file, by path and by digest, as the shape it
was written to. Deleting the file would not remove a generic evidence family
from this repository; the family is the *verifier*, and that is what went. It
would instead break a reference inside bytes the migration is required to leave
untouched and readable, so that a later reader could no longer resolve what
those records claim about themselves. "Preserve legacy history read-only" and
"delete the local generic evidence schema" point in opposite directions here,
and readability of the immutable record wins.

The freeze is enforced rather than described. `TC-024` locks all three schema
files by digest alongside every byte under `evidence/`, and asserts that no
script in this repository references either frozen schema. A gate that started
validating against one again would turn red.
