#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""Compiler evidence for the opaque production-reader root.

This reads LLVM/MIR produced by the selected installed rustc, never a Rust AST.
Unknown calls, missing bodies, ambiguous source/type evidence, and unsupported
pointer flows remain explicit blockers. Merely producing a graph is not proof.
"""

from __future__ import annotations

import argparse
from collections import defaultdict
from dataclasses import dataclass, field
import fcntl
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time


class EvidenceError(ValueError):
    """The compiler evidence cannot support a closed claim."""


IDENT = r'(?:"(?:\\[0-9A-Fa-f]{2}|[^"])*"|[-a-zA-Z$._0-9]+)'
TOKEN = re.compile(r"[@%]" + IDENT + r"|\b(?:null|undef|poison|zeroinitializer)\b")
SYMBOL = re.compile(r"@(" + IDENT + r")")
SSA = re.compile(r"%(" + IDENT + r")")
SAFE_ENV = (
    "HOME", "USER", "LOGNAME", "TMPDIR", "CARGO_HOME", "RUSTUP_HOME",
    "SSL_CERT_FILE", "SSL_CERT_DIR", "SYSTEMROOT",
)


def ir_string(value: str) -> str:
    if not value.startswith('"'):
        return value
    data = re.sub(
        r"\\([0-9A-Fa-f]{2})",
        lambda match: chr(int(match.group(1), 16)),
        value[1:-1],
    )
    return data.encode("latin1").decode("utf8")


def split_fields(text: str) -> list[str]:
    """Split an LLVM list, retaining nested types and quoted identifiers."""
    result, start, stack = [], 0, []
    quoted, escaped = False, False
    for index, char in enumerate(text):
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char in "([{<":
            stack.append(char)
        elif char in ")]}>":
            if not stack:
                raise EvidenceError("unbalanced compiler list")
            stack.pop()
        elif char == "," and not stack:
            result.append(text[start:index].strip())
            start = index + 1
    if stack or quoted:
        raise EvidenceError("unfinished compiler list")
    result.append(text[start:].strip())
    return result


def balanced_end(text: str, start: int) -> int:
    opening = text[start]
    closing = {"(": ")", "[": "]", "{": "}", "<": ">"}[opening]
    depth, quoted, escaped = 0, False, False
    for index in range(start, len(text)):
        char = text[index]
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char == opening:
            depth += 1
        elif char == closing:
            depth -= 1
            if depth == 0:
                return index
    raise EvidenceError("unfinished compiler expression")


def take_type(text: str) -> tuple[str, str]:
    text = text.strip()
    if text.startswith(("[", "{", "<")):
        end = balanced_end(text, 0) + 1
        return text[:end], text[end:].strip()
    match = re.match(r"(?:%" + IDENT + r"|ptr(?:\s+addrspace\(\d+\))?|i\d+|half|float|double|void)(?=\s|,|$)", text)
    if not match:
        raise EvidenceError(f"unsupported LLVM type: {text[:100]}")
    return match.group(), text[match.end():].strip()


@dataclass(frozen=True)
class Atom:
    kind: str
    name: str
    offset: int = 0
    origin: str = ""


Value = dict[tuple[int, ...], set[Atom]]


def scalar(*atoms: Atom) -> Value:
    return {(): set(atoms)} if atoms else {}


def join_into(destination: Value, source: Value) -> bool:
    changed = False
    for path, atoms in source.items():
        before = len(destination.setdefault(path, set()))
        destination[path].update(atoms)
        changed |= before != len(destination[path])
    return changed


def joined(*values: Value) -> Value:
    result: Value = {}
    for value in values:
        join_into(result, value)
    return result


@dataclass
class Instruction:
    text: str
    block: str
    ir_line: int
    result: str | None = None
    location: int | None = None


@dataclass
class Function:
    symbol: str
    module: "Module"
    header: str
    parameters: list[str]
    instructions: list[Instruction]
    debug: int | None
    declaration: bool = False


class Module:
    def __init__(self, path: Path):
        self.path = path.resolve()
        self.text = path.read_text()
        self.metadata: dict[int, str] = {}
        self.types: dict[str, str] = {}
        self.attributes: dict[int, str] = {}
        self.functions: dict[str, Function] = {}
        self.global_lines: dict[str, str] = {}
        self.global_slots: dict[tuple[str, int], Value] = {}
        self.aliases: dict[str, str] = {}
        self.vtables: set[str] = set()
        self.parse()

    def parse(self) -> None:
        lines = self.text.splitlines()
        for line in lines:
            if match := re.match(r"!(\d+) = (.*)", line):
                self.metadata[int(match[1])] = match[2]
            elif match := re.match(r"attributes #(\d+) = (.*)", line):
                self.attributes[int(match[1])] = match[2]
            elif match := re.match(r"(%"+IDENT+r") = type (.*)", line):
                self.types[match[1]] = match[2]
            elif match := re.match(r"@("+IDENT+r") = (.*)", line):
                self.global_lines[ir_string(match[1])] = match[2]
        index = 0
        while index < len(lines):
            header = lines[index]
            if not header.startswith(("define ", "declare ")):
                index += 1
                continue
            match = SYMBOL.search(header)
            if not match:
                raise EvidenceError("compiler function has no linkage identity")
            name = ir_string(match[1])
            start = header.find("(", match.end())
            end = balanced_end(header, start)
            parameters = []
            for argument in split_fields(header[start + 1:end]):
                names = SSA.findall(argument)
                if names:
                    parameters.append("%" + names[-1])
            debug = re.search(r"!dbg !(\d+)", header)
            body: list[Instruction] = []
            declaration = header.startswith("declare ")
            index += 1
            block = "entry"
            while not declaration and index < len(lines) and lines[index] != "}":
                raw = lines[index].strip()
                line_number = index + 1
                if label := re.match(r"([-a-zA-Z$._0-9]+):", raw):
                    block = label[1]
                elif raw and not raw.startswith((";", "#dbg")):
                    while index + 1 < len(lines) and (
                        lines[index + 1].lstrip().startswith(("to label ", "unwind label "))
                    ):
                        index += 1
                        raw += " " + lines[index].strip()
                    result = re.match(r"(%"+IDENT+r") = ", raw)
                    location = re.search(r"!dbg !(\d+)", raw)
                    body.append(Instruction(
                        raw, block, line_number,
                        result[1] if result else None,
                        int(location[1]) if location else None,
                    ))
                index += 1
            self.functions[name] = Function(
                name, self, header, parameters, body,
                int(debug[1]) if debug else None, declaration,
            )

    def meta(self, number: int | None) -> str:
        return self.metadata.get(number, "") if number is not None else ""

    def subprogram(self, number: int | None) -> int | None:
        visited = set()
        while number is not None and number not in visited:
            visited.add(number)
            text = self.meta(number)
            if "!DISubprogram(" in text:
                return number
            parent = re.search(r"scope: !(\d+)", text)
            number = int(parent[1]) if parent else None
        return None

    def span(self, number: int | None) -> dict | None:
        text = self.meta(number)
        program = self.subprogram(number)
        definition = self.meta(program)
        file_match = re.search(r"file: !(\d+)", definition)
        if not file_match:
            return None
        file_info = self.meta(int(file_match[1]))
        file_name = re.search(r'filename: ("(?:\\.|[^"])*")', file_info)
        directory = re.search(r'directory: ("(?:\\.|[^"])*")', file_info)
        if not file_name:
            return None
        path = Path(ir_string(file_name[1]))
        if not path.is_absolute() and directory:
            path = Path(ir_string(directory[1])) / path
        line = re.search(r"\bline: (\d+)", text)
        column = re.search(r"\bcolumn: (\d+)", text)
        return {
            "path": str(path),
            "line": int(line[1]) if line else None,
            "column": int(column[1]) if column else None,
            "end_line": None,
            "end_column": None,
            "origin": "llvm.DILocation" if "!DILocation(" in text else "llvm.DISubprogram",
        }

    def linkage(self, number: int | None) -> str | None:
        text = self.meta(self.subprogram(number))
        match = re.search(r'linkageName: ("(?:\\.|[^"])*")', text)
        return ir_string(match[1]) if match else None

    def size_align(self, ty: str) -> tuple[int, int]:
        ty = ty.strip()
        if ty in self.types:
            return self.size_align(self.types[ty])
        if ty.startswith("ptr"):
            return 8, 8
        if match := re.fullmatch(r"i(\d+)", ty):
            size = (int(match[1]) + 7) // 8
            return size, min(16, 1 << max(0, size - 1).bit_length())
        if ty in ("half", "float", "double"):
            size = {"half": 2, "float": 4, "double": 8}[ty]
            return size, size
        if match := re.fullmatch(r"\[(\d+) x (.*)]", ty):
            size, alignment = self.size_align(match[2])
            return int(match[1]) * size, alignment
        if ty.startswith(("{", "<{")):
            fields, packed = self.structure(ty)
            size, maximum = 0, 1
            for item in fields:
                length, alignment = self.size_align(item)
                alignment = 1 if packed else alignment
                size = (size + alignment - 1) // alignment * alignment
                size += length
                maximum = max(maximum, alignment)
            return (size + maximum - 1) // maximum * maximum, maximum
        raise EvidenceError(f"unknown compiler layout {ty}")

    @staticmethod
    def structure(ty: str) -> tuple[list[str], bool]:
        packed = ty.startswith("<{")
        return split_fields(ty[2:-2] if packed else ty[1:-1]), packed

    def field_offset(self, ty: str, index: int) -> tuple[int, str]:
        ty = self.types.get(ty, ty)
        if match := re.fullmatch(r"\[(\d+) x (.*)]", ty):
            if not 0 <= index < int(match[1]):
                raise EvidenceError("array probe outside compiler layout")
            return index * self.size_align(match[2])[0], match[2]
        fields, packed = self.structure(ty)
        if not 0 <= index < len(fields):
            raise EvidenceError("structure probe outside compiler layout")
        offset = 0
        for slot, field_type in enumerate(fields):
            length, alignment = self.size_align(field_type)
            alignment = 1 if packed else alignment
            offset = (offset + alignment - 1) // alignment * alignment
            if slot == index:
                return offset, field_type
            offset += length
        raise EvidenceError("missing compiler field")

    def initialize_globals(self, functions: set[str]) -> None:
        def constant(ty: str, value: str, symbol: str, offset: int) -> None:
            if ty.startswith("ptr"):
                tokens = TOKEN.findall(value)
                if tokens:
                    token = tokens[-1]
                    if token.startswith("@"):
                        target = ir_string(token[1:])
                        atom = Atom("function", target, origin="vtable" if symbol in self.vtables else "global") if target in functions else Atom("global", target)
                        self.global_slots[(symbol, offset)] = scalar(atom)
                    elif token == "null":
                        self.global_slots[(symbol, offset)] = scalar(Atom("null", "null"))
                return
            resolved = self.types.get(ty, ty)
            if resolved.startswith(("{", "<{")) and value.startswith(("{", "<{")):
                fields, packed = self.structure(resolved)
                values = split_fields(value[2:-2] if packed else value[1:-1])
                if len(fields) != len(values):
                    raise EvidenceError("compiler aggregate initializer arity")
                for index, item in enumerate(values):
                    _, expression = take_type(item)
                    displacement, field_type = self.field_offset(resolved, index)
                    constant(field_type, expression, symbol, offset + displacement)
            elif match := re.fullmatch(r"\[(\d+) x (.*)]", resolved):
                if value.startswith("["):
                    for index, item in enumerate(split_fields(value[1:-1])):
                        _, expression = take_type(item)
                        constant(match[2], expression, symbol, offset + index * self.size_align(match[2])[0])

        for symbol, line in self.global_lines.items():
            if " alias " in " " + line:
                names = SYMBOL.findall(line)
                if names:
                    self.aliases[symbol] = ir_string(names[-1])
                continue
            # This classification uses the compiler's DIGlobalVariable, not a
            # blanket collection of all vtables as possible call targets.
            debug = re.search(r"!dbg !(\d+)", line)
            if debug:
                info = self.meta(int(debug[1]))
                global_id = re.search(r"var: !(\d+)", info)
                if global_id and "{vtable}" in self.meta(int(global_id[1])):
                    self.vtables.add(symbol)
            match = re.search(r"\b(?:constant|global)\s+(.*)", line)
            if not match:
                continue
            try:
                ty, expression = take_type(match[1])
                expression = split_fields(expression)[0]
                constant(ty, expression, symbol, 0)
            except EvidenceError:
                # A later load from an unrepresented slot is unresolved.
                continue


@dataclass
class State:
    function: Function
    values: dict[str, Value] = field(default_factory=dict)
    returns: Value = field(default_factory=dict)


def kind_for(instance: str, virtual: bool = False) -> str:
    if virtual:
        return "vtable"
    if "drop_glue" in instance or "drop_in_place" in instance or " as core::ops::drop::Drop>" in instance:
        return "drop"
    if "{closure" in instance or "::call_once" in instance or "::call_mut" in instance:
        return "closure"
    if " as core::ops::" in instance or " as std::ops::" in instance:
        return "operator"
    return "call"


def forbidden_instance(instance: str) -> str | None:
    # A type merely appearing in a monomorphic argument is not an invoked API.
    # In particular drop_glue::<BTreeMap<...>> is not BTreeMap::new, and an
    # unstable-sort GapGuard in a Drop argument is not a stable sort call.
    offset = 0
    while (start := instance.find("::<", offset)) >= 0:
        depth, end = 1, start + 3
        while depth and end < len(instance):
            depth += (instance[end] == "<") - (instance[end] == ">")
            end += 1
        if depth:
            return None
        instance = instance[:start] + instance[end:]
        offset = start
    if re.search(r"(^|[<(, ])serde_json::(?:de::|from_(?:str|slice|reader)|raw::)", instance):
        return "serde_json parser/deserializer/RawValue"
    if re.search(r"(^|[<(, ])serde_json::value::Value", instance) and re.search(r"::(?:clone|from|deserialize|default)\b", instance):
        return "serde_json::Value construction"
    if re.search(r"(^|[<(, ])num_bigint::", instance) and re.search(r"::(?:parse|from_str|add|sub|mul|div|rem|pow|gcd|shl|shr)", instance):
        return "num-bigint parse/arithmetic"
    map_type = re.search(r"(^|[<(, ])(?:alloc::collections::btree::(?:map::BTreeMap|set::BTreeSet)|std::collections::hash::map::HashMap|hashbrown::map::HashMap|serde_json::map::Map)", instance)
    if map_type and re.search(r"::(?:new|default|insert|entry|from_iter|collect|clone|reserve)\b", instance):
        return "tree/hash/serde map construction"
    # The prescribed unstable API reaches core's implementation function named
    # `sort`; this is algorithm classification, not an external/body allowlist.
    unstable_implementation = instance.startswith("core::slice::sort::unstable::")
    if (not unstable_implementation and re.search(r"::(?:sort|sort_by|sort_by_key|sort_by_cached_key)(?:::<|<|$)", instance)) or "::sort::stable::" in instance:
        return "sort other than sort_unstable"
    return None


def thir_aggregate_evidence(path: Path) -> list[dict]:
    """Read actual THIR Adt nodes and their owning compiler DefIds.

    Rendered `ty: Value` is never resolved by a name table. The AdtDef's own
    compiler `did` identifies the constructed type, including aliased types and
    local modules which happen to have an external crate's name.
    """
    owner, span = None, None
    pending, records = [], []
    span_pattern = r"span: (.+?):(\d+):(\d+): (\d+):(\d+) \(#\d+\)"
    with path.open() as source:
        for line in source:
            text = line.rstrip("\n")
            if text.startswith("DefId(") and text.endswith("):"):
                if pending:
                    raise EvidenceError("unterminated compiler THIR Adt node")
                owner, span = text[:-1], None
            if match := re.search(span_pattern, text):
                span = {"path": match[1], "line": int(match[2]), "column": int(match[3]),
                        "end_line": int(match[4]), "end_column": int(match[5]), "origin": "rustc.THIR.Expr"}
            indent = len(text) - len(text.lstrip())
            if re.fullmatch(r"\s+Adt \{", text):
                pending.append({"indent": indent, "owner_def_id": owner, "source_span": span,
                                "artifact": str(path)})
            if pending and "adt_def_id" not in pending[-1]:
                if match := re.match(r"\s*did: (DefId\(.*\))$", text):
                    pending[-1]["adt_def_id"] = match[1]
            if pending and text.strip() == "}" and indent == pending[-1]["indent"]:
                record = pending.pop()
                did = record.get("adt_def_id", "")
                actual = re.search(r" ~ (.*)\)$", did)
                if not record["owner_def_id"] or not actual or record["source_span"] is None:
                    raise EvidenceError("compiler THIR aggregate owner/type/source correspondence incomplete")
                qualified = re.sub(r"^([^\[]+)\[[^]]+\]", r"\1", actual[1])
                reason = None
                if qualified == "serde_json::value::Value":
                    reason = "serde_json::Value construction"
                elif qualified in {
                    "alloc::collections::btree::map::BTreeMap", "alloc::collections::btree::set::BTreeSet",
                    "std::collections::hash::map::HashMap", "hashbrown::map::HashMap", "serde_json::map::Map",
                }:
                    reason = "tree/hash/serde map construction"
                record.update({"qualified_adt": qualified, "violation": reason})
                del record["indent"]
                records.append(record)
    if pending:
        raise EvidenceError("unterminated compiler THIR Adt node")
    return records


class Graph:
    """Conservative, context-insensitive pointer flow over compiler SSA.

    A finite set is never inferred from all compatible module functions or
    vtables. Only actual argument, store/load, aggregate, phi/select, and
    compiler-global flows can contribute targets. Missing flows remain red.
    """

    def __init__(self, modules: list[Module], names: dict[str, str], mir_files: list[Path] = ()):
        self.modules = modules
        self.names = names
        self.functions: dict[str, Function] = {}
        self.duplicates: set[str] = set()
        self.memory: dict[Atom, Value] = {}
        self.states: dict[str, State] = {}
        self.edges: dict[tuple, dict] = {}
        self.blockers: dict[tuple, dict] = {}
        self.boundaries: dict[str, dict] = {}
        self.inline_functions: dict[str, tuple[Module, int]] = {}
        self.mir_functions = self.read_mir(mir_files)
        self.mir_by_name: dict[str, list[dict]] = defaultdict(list)
        for body in self.mir_functions:
            header = body["header"].split("(", 1)[0].removeprefix("fn ").strip()
            short = re.sub(r"::<[^<>]*>", "", header).split("::")[-1].split("<", 1)[0]
            self.mir_by_name[short].append(body)
        for module in modules:
            for symbol, function in module.functions.items():
                prior = self.functions.get(symbol)
                if prior and not prior.declaration and not function.declaration:
                    self.duplicates.add(symbol)
                if prior is None or prior.declaration:
                    self.functions[symbol] = function
        for module in modules:
            module.initialize_globals(set(self.functions))
            for (symbol, offset), value in module.global_slots.items():
                join_into(self.memory.setdefault(Atom("global", symbol, offset), {}), value)

    @staticmethod
    def read_mir(paths: list[Path]) -> list[dict]:
        functions = []
        for path in paths:
            current = None
            for line in path.read_text().splitlines():
                if line.startswith("fn "):
                    current = {"header": line, "lines": [], "spans": [], "artifact": str(path)}
                    functions.append(current)
                if current is not None:
                    current["lines"].append(line)
                    for match in re.finditer(r"(?:at |span: )(.+?):(\d+):(\d+): (\d+):(\d+)", line):
                        current["spans"].append({
                            "path": match[1], "line": int(match[2]), "column": int(match[3]),
                            "end_line": int(match[4]), "end_column": int(match[5]),
                            "origin": "rustc.MIR",
                        })
                if line == "}":
                    current = None
        return functions

    def name(self, symbol: str) -> str:
        return self.names.get(symbol, symbol)

    def identify(self, symbol: str) -> dict:
        function = self.functions.get(symbol)
        if function:
            module, debug = function.module, function.debug
            span = module.span(debug)
            signature = function.header.split(" !dbg", 1)[0]
        elif symbol in self.inline_functions:
            module, debug = self.inline_functions[symbol]
            span, signature = module.span(debug), module.meta(debug)
        else:
            span, signature = None, None
        return {
            "def_path": self.name(symbol),
            "instance": symbol,
            "source_span": span,
            "compiler_signature": signature,
        }

    def block(self, caller: str, instruction: Instruction | None, reason: str, module: Module | None = None) -> None:
        function = self.functions.get(caller)
        module = module or (function.module if function else None)
        span = module.span(instruction.location) if module and instruction else self.identify(caller)["source_span"]
        key = caller, json.dumps(span, sort_keys=True), reason
        self.blockers[key] = {"caller_instance": caller, "source_span": span, "reason": reason}

    def edge(self, caller: str, target: str, instruction: Instruction, module: Module, virtual: bool = False, evidence: str = "llvm.direct_call") -> None:
        span = module.span(instruction.location)
        key = caller, target, instruction.ir_line, str(module.path), evidence
        self.edges[key] = {
            "caller_instance": caller, "callee_instance": target,
            "kind": kind_for(self.name(target), virtual), "source_span": span,
            "evidence": {"kind": evidence, "artifact": str(module.path), "ir_line": instruction.ir_line},
        }

    def token_value(self, token: str, state: State) -> Value:
        if token.startswith("%"):
            return state.values.get(token, {})
        if token.startswith("@"):
            symbol = ir_string(token[1:])
            aliases = state.function.module.aliases
            seen = set()
            while symbol in aliases and symbol not in seen:
                seen.add(symbol)
                symbol = aliases[symbol]
            return scalar(Atom("function", symbol)) if symbol in self.functions else scalar(Atom("global", symbol))
        if token in ("null", "zeroinitializer"):
            return scalar(Atom("null", token))
        return scalar(Atom("unknown", token))

    def operand(self, expression: str, state: State) -> Value:
        if "getelementptr " in expression:
            return self.gep(expression[expression.index("getelementptr "):], state)
        tokens = TOKEN.findall(expression)
        return self.token_value(tokens[-1], state) if tokens else {}

    def gep(self, expression: str, state: State) -> Value:
        text = re.sub(r"^getelementptr(?:\s+(?:inbounds|nuw|nusw))*\s*", "", expression)
        if text.startswith("("):
            text = text[1:balanced_end(text, 0)]
        fields = split_fields(text)
        if len(fields) < 3:
            return scalar(Atom("unknown", "incomplete compiler GEP"))
        ty = fields[0]
        base = self.operand(fields[1], state).get((), set())
        offsets = {0}
        try:
            for index, field_value in enumerate(fields[2:]):
                if field_value.startswith("!"):
                    break
                match = re.search(r"\b-?\d+\s*$", field_value)
                if not match:
                    raise EvidenceError("dynamic GEP requires a bounded compiler index proof")
                slot = int(match.group())
                if index == 0:
                    length = state.function.module.size_align(ty)[0]
                    offsets = {offset + slot * length for offset in offsets}
                else:
                    displacement, ty = state.function.module.field_offset(ty, slot)
                    offsets = {offset + displacement for offset in offsets}
        except EvidenceError as error:
            return scalar(Atom("unknown", str(error)))
        return scalar(*(
            Atom(atom.kind, atom.name, atom.offset + offset, atom.origin)
            if atom.kind in ("stack", "heap", "global") else
            Atom("unknown", f"GEP base has {atom.kind} provenance")
            for atom in base for offset in offsets
        ))

    def load(self, address: Value) -> Value:
        result: Value = {}
        for atom in address.get((), set()):
            if atom.kind in ("stack", "heap", "global"):
                join_into(result, self.memory.get(atom, {}))
            else:
                join_into(result, scalar(Atom("unknown", f"load through {atom.kind}: {atom.name}")))
        return result

    def store(self, address: Value, value: Value) -> bool:
        changed = False
        for atom in address.get((), set()):
            if atom.kind in ("stack", "heap", "global"):
                changed |= join_into(self.memory.setdefault(atom, {}), value)
        return changed

    def logical_caller(self, state: State, instruction: Instruction) -> str:
        module = state.function.module
        location = instruction.location
        caller = state.function.symbol
        chain = []
        visited = set()
        while location is not None and location not in visited:
            visited.add(location)
            text = module.meta(location)
            program = module.subprogram(location)
            symbol = module.linkage(program)
            if symbol:
                chain.append((symbol, program, location))
            inline = re.search(r"inlinedAt: !(\d+)", text)
            location = int(inline[1]) if inline else None
        for symbol, program, origin in reversed(chain):
            if symbol != caller:
                self.inline_functions[symbol] = module, program
                self.edge(caller, symbol, instruction, module, evidence="llvm.inlinedAt")
                caller = symbol
        return caller

    @staticmethod
    def call(instruction: Instruction) -> tuple[str, list[str]] | None:
        text = instruction.text
        if not re.search(r"\b(?:call|invoke|callbr)\b", text):
            return None
        if re.search(r"\basm\b", text):
            return "inline-assembly", []
        match = re.search(r"\b(?:call|invoke|callbr)\b.*?("+r"[@%]"+IDENT+r")\s*\(", text)
        if not match:
            return "unparsed-call", []
        start = text.find("(", match.start(1) + len(match[1]))
        end = balanced_end(text, start)
        return match[1], split_fields(text[start + 1:end]) if end > start + 1 else []

    def intrinsic(self, function: Function) -> bool:
        if not function.symbol.startswith("llvm."):
            return False
        groups = re.findall(r"#(\d+)", function.header)
        attributes = " ".join(function.module.attributes.get(int(group), "") for group in groups)
        if "nocallback" not in attributes:
            return False
        self.boundaries[function.symbol] = {
            "typed_instance": function.symbol,
            "compiler_reason": "LLVM intrinsic with compiler-emitted nocallback attribute",
            "callback_exclusion_evidence": {
                "declaration": function.header, "attributes": attributes,
                "artifact": str(function.module.path),
            },
        }

        return True

    def process(self, state: State) -> bool:
        changed = False
        function, module = state.function, state.function.module
        for instruction in function.instructions:
            text = instruction.text
            rhs = text.split(" = ", 1)[1] if instruction.result else text
            value: Value = {}
            caller = self.logical_caller(state, instruction)
            if " alloca " in " " + rhs:
                value = scalar(Atom("stack", function.symbol + ":" + str(instruction.result)))
            elif rhs.startswith("getelementptr "):
                value = self.gep(rhs, state)
            elif rhs.startswith("store "):
                fields = split_fields(rhs[6:])
                if len(fields) < 2:
                    self.block(caller, instruction, "unparsed compiler store", module)
                else:
                    address = self.operand(fields[1], state)
                    stored = self.operand(fields[0], state)
                    try:
                        _, literal = take_type(re.sub(r"^(?:atomic |volatile )+", "", fields[0]))
                        if literal.startswith(("{", "[", "<")):
                            self.block(caller, instruction, "compound constant store has unmodeled field/pointer effects", module)
                            stored = scalar(Atom("unknown", "compound constant store field provenance"))
                    except EvidenceError:
                        self.block(caller, instruction, "store type/byte effects are not represented", module)
                    if any(atom.kind not in ("stack", "heap", "global") for atom in address.get((), set())):
                        self.block(caller, instruction, "unknown-address store may alter callable memory; finite prior targets are not complete", module)
                    changed |= self.store(address, stored)
            elif rhs.startswith("load "):
                fields = split_fields(rhs[5:])
                value = self.load(self.operand(fields[1], state)) if len(fields) > 1 else scalar(Atom("unknown", "unparsed load"))
            elif rhs.startswith("select "):
                fields = split_fields(rhs[7:])
                value = joined(*(self.operand(field, state) for field in fields[1:3]))
            elif rhs.startswith("phi "):
                value = joined(*(self.operand(item.split(",", 1)[0], state)
                                 for item in re.findall(r"\[([^]]+)]", rhs)))
            elif rhs.startswith(("bitcast ", "addrspacecast ", "freeze ", "ptrtoint ", "inttoptr ")):
                value = self.operand(rhs.split(" to ", 1)[0], state)
                if not value and rhs.startswith("inttoptr "):
                    value = scalar(Atom("unknown", "integer-derived callable address"))
            elif rhs.startswith("extractvalue "):
                fields = split_fields(rhs[len("extractvalue "):])
                source = self.operand(fields[0], state)
                path = tuple(int(field) for field in fields[1:] if field.isdecimal())
                value = {key[len(path):]: atoms for key, atoms in source.items() if key[:len(path)] == path}
            elif rhs.startswith("insertvalue "):
                fields = split_fields(rhs[len("insertvalue "):])
                value = joined(self.operand(fields[0], state))
                path = tuple(int(field) for field in fields[2:] if field.isdecimal())
                for key, atoms in self.operand(fields[1], state).items():
                    value.setdefault(path + key, set()).update(atoms)
            elif rhs.startswith("ret "):
                changed |= join_into(state.returns, self.operand(rhs[4:], state))
            if call := self.call(instruction):
                target_expression, arguments = call
                if target_expression in ("inline-assembly", "unparsed-call"):
                    self.block(caller, instruction, target_expression, module)
                    value = scalar(Atom("unknown", target_expression))
                else:
                    targets = self.operand(target_expression, state).get((), set())
                    for atom in targets:
                        if atom.kind != "function":
                            continue
                        self.edge(caller, atom.name, instruction, module,
                                  atom.origin == "vtable",
                                  "llvm.direct_call" if target_expression.startswith("@") else "llvm.pointer_flow")
                        callee = self.functions.get(atom.name)
                        if not callee:
                            self.block(caller, instruction, "compiler target has no available declaration/body: " + atom.name, module)
                            continue
                        if callee.declaration:
                            if atom.name.startswith(("llvm.memcpy", "llvm.memmove", "llvm.memset")):
                                self.block(caller, instruction, "memory intrinsic effects not represented by pointer-flow model: " + atom.name, module)
                            if not self.intrinsic(callee):
                                self.block(caller, instruction, f"external body unavailable: {self.name(atom.name)}", module)
                                join_into(value, scalar(Atom("unknown", f"return from {atom.name}")))
                        else:
                            if atom.name not in self.states:
                                self.states[atom.name] = State(callee)
                                changed = True
                            child = self.states[atom.name]
                            if len(callee.parameters) != len(arguments):
                                self.block(caller, instruction, f"ABI argument mismatch for {self.name(atom.name)}", module)
                            for formal, argument in zip(callee.parameters, arguments):
                                changed |= join_into(child.values.setdefault(formal, {}), self.operand(argument, state))
                            join_into(value, child.returns)
                    # Empty flows are deferred until the fixed point. Unknown
                    # flows never become known merely because candidates exist.
                    if any(atom.kind != "function" for atom in targets):
                        provenance = "; ".join(sorted(atom.name for atom in targets if atom.kind != "function"))
                        self.block(caller, instruction, "indirect call retains unknown/non-callable provenance: " + provenance, module)
            if instruction.result:
                if not value and "ptr" in rhs and not self.call(instruction) and not rhs.startswith(("load ", "getelementptr ", "select ", "phi ")):
                    value = scalar(Atom("unknown", f"unsupported pointer instruction: {rhs[:120]}"))
                changed |= join_into(state.values.setdefault(instruction.result, {}), value)
        return changed

    def mir_evidence(self, record: dict) -> tuple[dict | None, list[dict]]:
        span = record["source_span"]
        if not span:
            return None, []
        # LLVM owns the monomorphic identity. MIR contributes only compiler
        # body/source evidence; ambiguous short names cannot close the claim.
        qualified = re.sub(r"::<[^<>]*>", "", record["def_path"])
        name = qualified.split("::")[-1].split("<", 1)[0]
        candidates = []
        for body in self.mir_by_name.get(name, []):
            header = body["header"].split("(", 1)[0]
            if not re.search(r"(?:^fn |::)" + re.escape(name) + r"(?:<|$)", header):
                continue
            if any(item["path"] == span["path"] for item in body["spans"]):
                candidates.append(body)
        if len(candidates) != 1:
            return None, []
        body = candidates[0]
        risks = []
        for line in body["lines"]:
            if re.search(r"\b(?:Value|RawValue|BTreeMap|BTreeSet|HashMap|Map)::(?:[A-Z]\w*|new|default|from_iter|insert|entry)\b", line):
                risks.append({
                    "reason": "compiler MIR aggregate/API requires unambiguous forbidden-type qualification",
                    "artifact": body["artifact"], "compiler_statement": line.strip(),
                })
        return body, risks

    def analyze(self, root: str) -> dict:
        if root not in self.functions or self.functions[root].declaration:
            raise EvidenceError("opaque compiler root has no definition")
        initial = State(self.functions[root])
        for parameter in initial.function.parameters:
            initial.values[parameter] = scalar(Atom("unknown", f"opaque root argument {parameter}"))
        self.states[root] = initial
        converged = False
        deadline = time.monotonic() + 45
        for _ in range(512):
            changed = False
            for state in list(self.states.values()):
                if time.monotonic() >= deadline:
                    break
                changed |= self.process(state)
            if time.monotonic() >= deadline:
                self.block(root, None, "compiler pointer-flow analysis time bound reached")
                break
            if not changed:
                converged = True
                break
            if len(self.states) > 50_000:
                break
        if not converged:
            self.block(root, None, "compiler pointer-flow resource bound reached before fixed point")
        callable_cells = defaultdict(set)
        for address, value in self.memory.items():
            if any(atom.kind == "function" for atoms in value.values() for atom in atoms):
                callable_cells[(address.kind, address.name)].add(address.offset)
        for state in self.states.values():
            for instruction in state.function.instructions:
                call = self.call(instruction)
                if call and call[0].startswith("%"):
                    atoms = self.operand(call[0], state).get((), set())
                    if not atoms:
                        self.block(state.function.symbol, instruction, "indirect call has no compiler-proven finite targets")
                if instruction.text.startswith("store "):
                    fields = split_fields(instruction.text[6:])
                    if len(fields) < 2 or not self.operand(fields[1], state).get((), set()):
                        self.block(state.function.symbol, instruction, "store address has no compiler-proven memory provenance")
                    elif len(fields) >= 2:
                        stored = self.operand(fields[0], state)
                        try:
                            ty, _ = take_type(re.sub(r"^(?:atomic |volatile )+", "", fields[0]))
                            width = state.function.module.size_align(ty)[0]
                        except EvidenceError:
                            width = None
                        for address in self.operand(fields[1], state).get((), set()):
                            cells = callable_cells.get((address.kind, address.name), ())
                            overlaps = any(width is None or address.offset < cell + 8 and cell < address.offset + width for cell in cells)
                            if overlaps and (not stored or width != 8):
                                self.block(state.function.symbol, instruction, "unmodeled scalar/partial store overlaps callable memory; prior finite targets are not complete")
            if state.function.symbol in self.duplicates:
                self.block(state.function.symbol, None, "multiple definitions require compiler/linker resolution")
        reached = set(self.states) | set(self.inline_functions)
        reached.update(edge["callee_instance"] for edge in self.edges.values())
        violations = []
        records = []
        for symbol in sorted(reached):
            record = self.identify(symbol)
            body, risks = self.mir_evidence(record)
            record["mir_artifact"] = body["artifact"] if body else None
            if reason := forbidden_instance(record["def_path"]):
                violations.append({"typed_instance": symbol, "source_span": record["source_span"], "reason": reason})
            for risk in risks:
                self.block(symbol, None, risk["reason"] + ": " + risk["compiler_statement"])
            compiler_drop = re.match(r"(?:core|std)::ptr::drop_(?:glue|in_place)", record["def_path"])
            if body is None and not compiler_drop and symbol not in self.boundaries:
                self.block(symbol, None, "compiler MIR source/body correspondence unavailable or ambiguous")
            if record["source_span"] is None and symbol not in self.boundaries:
                self.block(symbol, None, "compiler source location unavailable")
            records.append(record)
        return {
            "root": self.identify(root),
            "functions": records,
            "edges": list(self.edges.values()),
            "unresolved": list(self.blockers.values()),
            "trusted_boundaries": list(self.boundaries.values()),
            "violations": violations,
            "result": {
                "static_ac513_proven": False,
                "scope": "compiler-derived closure of the named opaque root",
                "pointer_flow_fixed_point": converged,
                "source_bans_checked": False,
                "resolved_graph_without_reported_bans": converged and not self.blockers and not violations,
            },
        }


def safe_environment(toolchain: Path) -> dict[str, str]:
    env = {key: os.environ[key] for key in SAFE_ENV if key in os.environ}
    env.update({"PATH": str(toolchain) + ":/usr/bin:/bin", "CARGO_BUILD_JOBS": "2",
                "CARGO_INCREMENTAL": "0", "CARGO_PROFILE_DEV_DEBUG": "0",
                "CARGO_PROFILE_TEST_DEBUG": "0"})
    return env


def execute(arguments: list[str], env: dict[str, str], cwd: Path, log: Path) -> str:
    result = subprocess.run(arguments, cwd=cwd, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
    log.write_text(result.stdout)
    if result.returncode:
        raise EvidenceError(f"compiler command failed ({result.returncode}); see {log}")
    return result.stdout


def demangle(objects: list[Path], llvm_bin: Path, env: dict[str, str]) -> dict[str, str]:
    names = {}
    for obj in objects:
        command = [str(llvm_bin / "llvm-nm"), "--no-sort", "--format=posix", str(obj)]
        raw = subprocess.check_output(command, env=env, text=True).splitlines()
        decoded = subprocess.check_output(command[:1] + ["--demangle"] + command[1:], env=env, text=True).splitlines()
        if len(raw) != len(decoded):
            raise EvidenceError("llvm-nm demangling changed symbol cardinality")
        for before, after in zip(raw, decoded):
            # POSIX suffix is type/address/size; demangled Rust names may contain spaces.
            pattern = r"^(.*) ([A-Za-z?])(?: ([0-9a-f]+)(?: ([0-9a-f]+))?)?$"
            a, b = re.match(pattern, before), re.match(pattern, after)
            if not a or not b or a.groups()[1:] != b.groups()[1:]:
                raise EvidenceError("llvm-nm symbol order/evidence mismatch")
            names[a[1]] = b[1]
    return names


def analyze_artifacts(llvm_files: list[Path], mir_files: list[Path], objects: list[Path],
                      llvm_bin: Path, env: dict[str, str], root_suffix: str) -> dict:
    names = demangle(objects, llvm_bin, env)
    modules = [Module(path) for path in llvm_files]
    missing = set()
    for module in modules:
        missing.update(symbol for symbol in module.functions if symbol.startswith(("_R", "_ZN")) and symbol not in names)
        missing.update(ir_string(match) for match in re.findall(r'linkageName: ("[^"\n]*")', module.text)
                       if ir_string(match).startswith(("_R", "_ZN")) and ir_string(match) not in names)
    if missing:
        # This synthetic module is exclusively an llvm-nm demangling input.
        # It is NEVER included in modules or used as body/edge evidence.
        aid = llvm_files[0].parent / "demangle-only.ll"
        aid.write_text("\n".join('declare void @"' + name + '"()' for name in sorted(missing)) + "\n")
        bitcode = aid.with_suffix(".bc")
        execute([str(llvm_bin / "llvm-as"), str(aid), "-o", str(bitcode)], env, aid.parent,
                aid.with_suffix(".log"))
        names.update(demangle([bitcode], llvm_bin, env))
    # Layout implementation below is deliberately limited to the authenticated
    # 64-bit installed compiler target. Other layouts cannot silently close.
    for module in modules:
        if not re.search(r'target triple = "x86_64-', module.text):
            raise EvidenceError("unsupported compiler target layout; expected x86_64")
    graph = Graph(modules, names, mir_files)
    roots = [symbol for symbol, name in names.items() if name.endswith(root_suffix)
             and symbol in graph.functions and not graph.functions[symbol].declaration]
    if len(roots) != 1:
        raise EvidenceError(f"expected one compiler root {root_suffix}, got {len(roots)}")
    return graph.analyze(roots[0])


def recover_embedded_bitcode(archives: list[Path], llvm: Path, directory: Path,
                             env: dict[str, str]) -> tuple[list[Path], list[dict]]:
    """Recover real compiler modules; never infer bodies from library names."""
    directory.mkdir(parents=True, exist_ok=True)
    bitcode, evidence = [], []
    for archive_index, archive in enumerate(sorted(set(archives))):
        members = subprocess.check_output([str(llvm / "llvm-ar"), "t", str(archive)],
                                          env=env, text=True).splitlines()
        for member_index, member in enumerate(members):
            if not member.endswith(".o"):
                continue
            prefix = directory / f"archive-{archive_index}-member-{member_index}"
            obj, bc = prefix.with_suffix(".o"), prefix.with_suffix(".bc")
            obj.write_bytes(subprocess.check_output([str(llvm / "llvm-ar"), "p", str(archive), member], env=env))
            sections = subprocess.check_output([str(llvm / "llvm-objdump"), "--section-headers", str(obj)], env=env, text=True)
            record = {"archive": str(archive), "member": member, "section": ".llvmbc",
                      "section_headers": sections, "recovered": False}
            if re.search(r"\s\.llvmbc\s", sections):
                execute([str(llvm / "llvm-objcopy"), "--dump-section=.llvmbc=" + str(bc), str(obj)],
                        env, directory, prefix.with_suffix(".log"))
                if not bc.is_file() or not bc.stat().st_size:
                    raise EvidenceError("compiler bitcode section extraction produced no body")
                bitcode.append(bc)
                record.update({"recovered": True, "artifact": str(bc)})
            evidence.append(record)
            # Only our extracted transient copy is removed; installed archives
            # and all other lanes' targets are untouched.
            obj.unlink()
    return bitcode, evidence


def owned_target(requested: Path, source_root: Path, tool_root: Path) -> Path:
    resolved = requested.resolve()
    # Do not resolve the allowed `target` bases: a target symlink into another
    # lane must not turn that other lane into an allowed ownership boundary.
    bases = (source_root.resolve() / "target", tool_root.resolve() / "target")
    if not any(resolved.is_relative_to(base) for base in bases):
        raise EvidenceError("resolved compiler target escapes the owned source/tool target roots")
    return resolved


def reserve_report(requested: Path) -> Path:
    output = requested.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    return output


def lower_process_priority() -> None:
    if os.getpriority(os.PRIO_PROCESS, 0) < 10:
        os.setpriority(os.PRIO_PROCESS, 0, 10)
    subprocess.run(["/usr/bin/ionice", "-c", "3", "-p", str(os.getpid())],
                   stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, check=True)


def production_run(arguments: argparse.Namespace) -> dict:
    source = arguments.source_root.resolve(strict=True)
    output = arguments.out_dir.resolve()
    toolchain = arguments.toolchain_bin.resolve(strict=True)
    llvm = arguments.llvm_bin.resolve(strict=True)
    target = owned_target(arguments.target_dir, source, Path(__file__).resolve().parents[2])
    if not all((toolchain / name).is_file() for name in ("cargo", "rustc")):
        raise EvidenceError("absolute installed cargo/rustc are required")
    if not all((llvm / name).is_file() for name in ("llvm-as", "llvm-nm", "llvm-link", "opt", "llvm-ar", "llvm-objcopy", "llvm-objdump")):
        raise EvidenceError("matching absolute installed LLVM tools are required")
    if not output.is_dir() or any(output.iterdir()):
        raise EvidenceError("report directory must be newly and exclusively reserved")
    target.mkdir(parents=True, exist_ok=True)
    if shutil.disk_usage(target).free < 4 * 1024**3:
        raise EvidenceError("less than 4 GiB free on owned compiler-artifact filesystem")
    if shutil.disk_usage(output).free < 64 * 1024**2:
        raise EvidenceError("less than 64 MiB free on report filesystem")
    env = safe_environment(toolchain)
    env["RUSTC"] = str(toolchain / "rustc")
    env["RUSTFLAGS"] = "-Cembed-bitcode=yes -Cdebuginfo=1"
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=source, env=env, text=True).strip()
    dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=source, env=env, text=True)
    if dirty:
        raise EvidenceError("production evidence requires a clean explicit source snapshot")
    version = subprocess.check_output([env["RUSTC"], "-vV"], env=env, text=True)
    llvm_version = subprocess.check_output([str(llvm / "llvm-as"), "--version"], env=env, text=True)
    rust_llvm = re.search(r"LLVM version: (\S+)", version)
    selected_llvm = re.search(r"LLVM version (\d+\.\d+\.\d+)", llvm_version)
    if not rust_llvm or not selected_llvm or rust_llvm[1] != selected_llvm[1]:
        raise EvidenceError("installed rustc/LLVM tool versions do not match")
    cargo_version = subprocess.check_output([str(toolchain / "cargo"), "-V"], env=env, text=True).strip()
    compiler_flags = ["--emit=link,llvm-ir,mir", "-Csymbol-mangling-version=v0",
                      "-Cdebuginfo=1", "-Copt-level=0", "-Ccodegen-units=1"]
    if "nightly" in version:
        compiler_flags += ["-Zmir-include-spans=yes", "-Zprint-mono-items=yes"]
    command = [str(toolchain / "cargo"), "rustc", "--locked", "--offline",
               "-p", "quire-contract-model", "--lib", "--no-default-features",
               "--target-dir", str(target / "cargo"), "--message-format=json-render-diagnostics",
               "--"] + compiler_flags
    root_source = Path(__file__).resolve().with_name("static_reader_root.rs")
    root_output = target / "root"
    root_output.mkdir(exist_ok=True)
    provenance = {"source_root": str(source), "source_head": head, "source_clean": True,
                  "tool_source": str(Path(__file__).resolve()), "rustc": version,
                  "cargo": cargo_version, "llvm": llvm_version, "production_features": [],
                  "command": command, "compiler_flags": compiler_flags,
                  "target_dir": str(target), "environment_keys": sorted(env),
                  "lock_requested": time.time(), "purpose": "prototype; final folded head must be verified again"}
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2))
    with open("/tmp/titan-cargo.lock", "a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        provenance["lock_acquired"] = time.time()
        # Nice/ionice cover cargo and all its compiler children.
        cargo_output = execute(["nice", "-n", "10", "ionice", "-c", "3"] + command,
                               env, source, target / "cargo.log")
        artifacts = []
        metadata = []
        dependency_dirs = set()
        runtime_archives = []
        for line in cargo_output.splitlines():
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            if record.get("reason") == "compiler-artifact":
                dependency_dirs.update(str(Path(name).parent) for name in record.get("filenames", [])
                                       if name.endswith((".rlib", ".rmeta", ".so", ".dylib")))
                if record.get("target", {}).get("name") != "quire_contract_model":
                    runtime_archives.extend(Path(name) for name in record.get("filenames", []) if name.endswith(".rlib"))
            if record.get("reason") == "compiler-artifact" and record.get("target", {}).get("name") == "quire_contract_model":
                artifacts.extend(Path(name) for name in record.get("filenames", []) if name.endswith(".rlib"))
                metadata.extend(Path(name) for name in record.get("filenames", []) if name.endswith(".rmeta"))
        if len(artifacts) != 1:
            raise EvidenceError("expected exactly one Cargo-owned model rlib artifact")
        root_command = [env["RUSTC"], str(root_source), "--crate-name", "static_reader_adapter",
                        "--crate-type", "lib", "--edition=2024", "--extern",
                        "quire_contract_model=" + str(metadata[0] if len(metadata) == 1 else artifacts[0]),
                        "--out-dir", str(root_output)]
        for directory in sorted(dependency_dirs):
            root_command += ["-L", "dependency=" + directory]
        root_command += compiler_flags
        provenance["root_command"] = root_command
        execute(["nice", "-n", "10", "ionice", "-c", "3"] + root_command,
                env, source, target / "root.log")
        if len(metadata) != 1:
            raise EvidenceError("one full Cargo model metadata artifact is required for precise compiler output selection")
        model_stem = metadata[0].stem.removeprefix("lib")
        model_ir = metadata[0].with_name(model_stem + ".ll")
        model_mir = metadata[0].with_name(model_stem + ".mir")
        llvm_files = [model_ir, root_output / "static_reader_adapter.ll"]
        if not all(path.is_file() for path in llvm_files + [model_mir]):
            raise EvidenceError("exact Cargo model artifact's LLVM/MIR outputs are unavailable")
        root_module = Module(next(root_output.glob("*.ll")))
        root_symbols = [symbol for symbol in root_module.functions
                        if "static_reader_root" in symbol and not root_module.functions[symbol].declaration]
        if len(root_symbols) != 1:
            raise EvidenceError("compiler adapter root symbol is ambiguous")
        linked = target / "reader-linked.bc"
        pruned = target / "reader-root-closure.ll"
        target_libdir = Path(subprocess.check_output([env["RUSTC"], "--print", "target-libdir"], env=env, text=True).strip())
        runtime_archives.extend(target_libdir.glob("*.rlib"))
        recovered, archive_evidence = recover_embedded_bitcode(runtime_archives, llvm, target / "runtime-bitcode", env)
        provenance["runtime_archive_recovery"] = archive_evidence
        provenance["explicit_rustflags"] = env["RUSTFLAGS"]
        root_cfg = subprocess.check_output([env["RUSTC"], "--print=cfg"], env=env, text=True)
        strategy = re.search(r'^panic="(abort|unwind)"$', root_cfg, re.MULTILINE)
        if not strategy:
            raise EvidenceError("compiler root panic strategy is unavailable")
        provenance["root_compiler_cfg"] = root_cfg
        alternative = "libpanic_abort-" if strategy[1] == "unwind" else "libpanic_unwind-"
        unselected = {Path(item["artifact"]) for item in archive_evidence
                      if item.get("recovered") and Path(item["archive"]).name.startswith(alternative)}
        recovered = [path for path in recovered if path not in unselected]
        provenance["unselected_runtime_alternative"] = {
            "archive_prefix": alternative, "reason": "mutually exclusive panic runtime; actual opaque root compiler cfg selects " + strategy[1],
            "artifacts": list(map(str, sorted(unselected))),
        }
        link_command = [str(llvm / "llvm-link"), *map(str, llvm_files + recovered), "-o", str(linked)]
        prune_command = [str(llvm / "opt"), "-passes=internalize,globaldce",
                         "-internalize-public-api-list=" + root_symbols[0],
                         str(linked), "-S", "-o", str(pruned)]
        provenance["compiler_root_pruning"] = [link_command, prune_command]
        for name, invocation in (("link", link_command), ("prune", prune_command)):
            execute(["nice", "-n", "10", "ionice", "-c", "3"] + invocation,
                    env, source, target / (name + ".log"))
    provenance["lock_released"] = time.time()
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2))
    llvm_files = [pruned]
    mir_files = [model_mir, root_output / "static_reader_adapter.mir"]
    # Cargo rlibs are real symbol-table sources, never substituted body evidence.
    objects = []
    provenance["analysis_lock_requested"] = time.time()
    with open("/tmp/titan-cargo.lock", "a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        provenance["analysis_lock_acquired"] = time.time()
        report = analyze_artifacts(llvm_files, mir_files, objects, llvm, env, "::static_reader_root")
    provenance["analysis_lock_released"] = time.time()
    (output / "provenance.json").write_text(json.dumps(provenance, indent=2))
    report["provenance"] = provenance
    report["limitations"] = [
        "Prototype emits the selected model library and opaque adapter; missing runtime dependency bodies remain unresolved.",
        "LLVM pointer-flow supports finite local flows only; unknown memory, indirect targets and external bodies cannot close proof.",
        "Compiler MIR short type names require unambiguous qualification before aggregate construction can be cleared.",
    ]
    # Until the aggregate/type qualification pass and all dependency body capture
    # are implemented, this prototype must never advertise a closed AC513 proof.
    report["unresolved"].append({"caller_instance": report["root"]["instance"],
        "source_span": report["root"]["source_span"],
        "reason": "compiler evidence completeness not established: dependency bodies and fully qualified aggregate construction pass"})
    report["result"]["static_ac513_proven"] = False
    report["result"]["source_bans_checked"] = False
    final_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=source, env=env, text=True).strip()
    final_dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=source, env=env, text=True)
    if final_head != head or final_dirty:
        raise EvidenceError("source changed during compiler evidence extraction")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", required=True, type=Path)
    parser.add_argument("--out-dir", required=True, type=Path)
    parser.add_argument("--toolchain-bin", required=True, type=Path)
    parser.add_argument("--llvm-bin", required=True, type=Path)
    parser.add_argument("--target-dir", type=Path,
                        default=Path(__file__).resolve().parents[2] / "target/proof-static/reader")
    arguments = parser.parse_args()
    for key, value in vars(arguments).items():
        if not value.is_absolute():
            parser.error("--" + key.replace("_", "-") + " must be absolute")
    try:
        arguments.out_dir = reserve_report(arguments.out_dir)
    except OSError as error:
        print(json.dumps({"result": {"static_ac513_proven": False}, "error": str(error),
                          "report_written": False}), file=sys.stderr)
        return 2
    try:
        lower_process_priority()
        report = production_run(arguments)
    except (EvidenceError, OSError, subprocess.SubprocessError) as error:
        report = {"root": None, "functions": [], "edges": [], "trusted_boundaries": [],
                  "result": {"static_ac513_proven": False, "source_bans_checked": False},
                  "unresolved": [{"caller_instance": None, "source_span": None, "reason": str(error)}]}
    report_file = arguments.out_dir / "compiler-reachability.json"
    with report_file.open("x") as output:
        output.write(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"report": str(report_file), "result": report["result"]}))
    return 0 if report["result"]["static_ac513_proven"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
