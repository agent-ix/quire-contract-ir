# Contributing

## Before you open a pull request: sign the CLA

Every contribution requires a signed Contributor License Agreement. Read it
at [`CLA.md`](CLA.md) in this repo before you start writing code.

When you open your first pull request, the CLA Assistant bot will comment
with instructions to sign electronically. Sign once and it's recorded for
future contributions across the org.

Contributions are welcome from people using any development method, including
agent-assisted workflows. The standard is the same for every contribution:

- requirements and acceptance criteria are updated before implementation;
- the repository's specification, review, test, and assurance gates pass;
- source and third-party provenance remain truthful and reviewable;
- domain producers retain their native structured result semantics, while
  Quire supplies static definition references and Quoin consumes explicit
  results without invoking producers; and
- the CODEOWNER, `@kreneskyp`, reviews the pull request and exclusively owns
  release decisions.

Do not push directly to `main`. Generated artifacts must retain their declared
derivation metadata and licensing. Do not copy material from repositories or
documents whose license does not permit reuse.

An agent may prepare a candidate and its evidence, but may not approve its own
pull request, decide that evidence is sufficient, create a source-release tag,
or claim validation, accreditation, or certification for a consuming project.
The exact ownership and compatibility rules are normative in
[`PGM-01`](spec/program/PGM-01-governance.md). Quire and Quoin remain non-executing development-time
boundaries, not runtime dependencies or a shared producer runner.
