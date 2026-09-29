# Schemas

## Domain schemas

| File | Describes |
| --- | --- |
| `contract-conformance-fixture-v1.schema.json` | The conformance corpus fixture inputs and expectations for each operation. |
| `contract-package-reference-v1.schema.json` | The serialized contract package wire form. |

These describe this repository's own domain artifacts and evolve with the
contract model.

The runner reads both from this directory (`--schemas schemas`).
