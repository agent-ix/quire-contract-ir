# Schemas

## Domain schemas

| File | Describes |
| --- | --- |
| `contract-conformance-fixture-v1.schema.json` | The conformance corpus fixture inputs and expectations for each operation. |
| `contract-package-reference-v1.schema.json` | The serialized contract package wire form. |
| `contract-executable-projection-v1.schema.json` | The executable projection binding (FR-023), compiled into the model crate. |
| `conformance-trace-map-v1.json` | The registry mapping each operation's coverage tokens to acceptance criteria (FR-018), compiled into the model crate. |

These describe this repository's own domain artifacts and evolve with the
contract model.

The runner reads the fixture and package schemas from this directory
(`--schemas schemas`).
