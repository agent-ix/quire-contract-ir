// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Selected-rustc metadata feasibility exporter, not a reachability proof.
//!
//! Build with the selected compiler's matching rustc-dev component. The caller
//! supplies ordinary rustc arguments and authentic upstream metadata. This
//! inspects direct calls in the input crate and their available upstream MIR;
//! it never claims a transitive closure or substitutes a missing upstream body.
//! Successful observation continues the normal compiler pipeline and requested
//! emissions; only an explicit evidence bound or error stops compilation.
//! Compiler-private interfaces deliberately follow the selected installation.

#![feature(rustc_private)]
#![forbid(unsafe_code)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::mir::{AggregateKind, Body, Operand, Rvalue, StatementKind, TerminatorKind};
use rustc_middle::ty::{self, Instance, TyCtxt, TypeVisitableExt};
use rustc_span::def_id::LocalDefId;
use std::fmt::Write as _;
use std::io::Write as _;

const MAX_OWNERS: usize = 128;
const MAX_BLOCKS: usize = 512;
const MAX_STATEMENTS: usize = 4096;
const MAX_OUTPUT: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
enum Stop {
    InvalidTargetBound,
    InvalidRoots,
    RootNotFound,
    OwnerBound,
    TargetBound,
    BlockBound,
    StatementBound,
    OutputBound,
    InstanceResolution,
    OutputIo,
    AnalysisNotObserved,
}

struct Exporter {
    output: String,
    targets: usize,
    target_bound: usize,
    failure: Option<Stop>,
    analysis_observed: bool,
    roots: Vec<String>,
    selected_owners: Vec<LocalDefId>,
}

/// JSON strings are encoded directly to avoid a second, unrelated crate's
/// metadata or wire dependencies inside the compiler driver.
fn quoted(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => {
                let _ = write!(output, "\\u{:04x}", u32::from(character));
            }
            other => output.push(other),
        }
    }
    output.push('"');
}

impl Exporter {
    fn record(&mut self, kind: &str, fields: &[(&str, String)]) -> Result<(), Stop> {
        let before = self.output.len();
        self.output.push_str("{\"kind\":");
        quoted(&mut self.output, kind);
        for (key, value) in fields {
            self.output.push(',');
            quoted(&mut self.output, key);
            self.output.push(':');
            quoted(&mut self.output, value);
        }
        self.output.push_str("}\n");
        if self.output.len() > MAX_OUTPUT {
            self.output.truncate(before);
            return Err(Stop::OutputBound);
        }
        Ok(())
    }

    fn body<'tcx>(
        &mut self,
        tcx: TyCtxt<'tcx>,
        identity: String,
        body: &Body<'tcx>,
        phase: &str,
    ) -> Result<(), Stop> {
        if body.basic_blocks.len() > MAX_BLOCKS {
            return Err(Stop::BlockBound);
        }
        self.record(
            "body",
            &[
                ("def_id", identity.clone()),
                ("phase", phase.into()),
                ("blocks", body.basic_blocks.len().to_string()),
                ("locals", body.local_decls.len().to_string()),
                (
                    "source",
                    tcx.sess.source_map().span_to_diagnostic_string(body.span),
                ),
            ],
        )?;
        let mut statements = 0_usize;
        for block in body.basic_blocks.iter() {
            for statement in &block.statements {
                statements += 1;
                if statements > MAX_STATEMENTS {
                    return Err(Stop::StatementBound);
                }
                if let StatementKind::Assign(assignment) = &statement.kind {
                    // A unit enum construction can be a typed constant after
                    // MIR optimization, with no Aggregate statement remaining.
                    if let Rvalue::Use(Operand::Constant(constant), _) = &assignment.1 {
                        if let ty::Adt(adt, args) = constant.const_.ty().kind() {
                            self.record(
                                "adt_constant",
                                &[
                                    ("body_def_id", identity.clone()),
                                    ("phase", phase.into()),
                                    ("def_id", format!("{:?}", adt.did())),
                                    ("def_path", tcx.def_path_str(adt.did())),
                                    ("generic_args", format!("{args:?}")),
                                    ("compiler_constant", format!("{:?}", constant.const_)),
                                    (
                                        "source",
                                        tcx.sess
                                            .source_map()
                                            .span_to_diagnostic_string(statement.source_info.span),
                                    ),
                                ],
                            )?;
                        }
                    }
                    if let Rvalue::Aggregate(kind, _) = &assignment.1 {
                        if let AggregateKind::Adt(adt, variant, args, _, _) = &**kind {
                            self.record(
                                "adt",
                                &[
                                    ("body_def_id", identity.clone()),
                                    ("phase", phase.into()),
                                    ("def_id", format!("{adt:?}")),
                                    ("def_path", tcx.def_path_str(*adt)),
                                    ("variant", format!("{variant:?}")),
                                    ("generic_args", format!("{args:?}")),
                                    (
                                        "source",
                                        tcx.sess
                                            .source_map()
                                            .span_to_diagnostic_string(statement.source_info.span),
                                    ),
                                ],
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn built_roots<'tcx>(&mut self, tcx: TyCtxt<'tcx>) -> Result<(), Stop> {
        // Match complete compiler definition paths, never a leaf or source-span
        // heuristic. Unselected V1 or legacy bodies are not inspected.
        let owners: Vec<_> = tcx.hir_body_owners().collect();
        for &owner in &owners {
            if self.roots.contains(&tcx.def_path_str(owner.to_def_id())) {
                self.selected_owners.push(owner);
            }
        }
        if self.selected_owners.len() != self.roots.len() {
            return Err(Stop::RootNotFound);
        }
        let mut position = 0;
        while position < self.selected_owners.len() {
            if self.selected_owners.len() > MAX_OWNERS {
                return Err(Stop::OwnerBound);
            }
            let owner = self.selected_owners[position];
            position += 1;
            let built = tcx.mir_built(owner);
            let body = built.borrow();
            self.body(
                tcx,
                format!("{:?}", owner.to_def_id()),
                &body,
                "built_local",
            )?;
            for block in body.basic_blocks.iter() {
                if let TerminatorKind::Call { func, .. } = &block.terminator().kind {
                    if let ty::FnDef(target, _) = *func.ty(&body.local_decls, tcx).kind() {
                        if let Some(local) = target.as_local() {
                            if owners.contains(&local) && !self.selected_owners.contains(&local) {
                                self.selected_owners.push(local);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn collect<'tcx>(&mut self, tcx: TyCtxt<'tcx>) -> Result<(), Stop> {
        for owner in self.selected_owners.clone() {
            let owner_id = owner.to_def_id();
            let body = tcx.optimized_mir(owner_id);
            self.body(tcx, format!("{owner_id:?}"), body, "optimized_local")?;
            for block in body.basic_blocks.iter() {
                let TerminatorKind::Call { func, .. } = &block.terminator().kind else {
                    continue;
                };
                let ty::FnDef(declaration, args) = *func.ty(&body.local_decls, tcx).kind() else {
                    self.record(
                        "unresolved_call",
                        &[
                            ("caller_def_id", format!("{owner_id:?}")),
                            (
                                "reason",
                                "indirect call; no finite-target proof in metadata exporter".into(),
                            ),
                        ],
                    )?;
                    continue;
                };
                if self.targets >= self.target_bound {
                    return Err(Stop::TargetBound);
                }
                self.targets += 1;
                // A generic owner's substitutions are not a monomorphic
                // environment. Preserve that gap instead of guessing an Instance.
                if args.has_param() {
                    self.record(
                        "unresolved_call",
                        &[
                            ("caller_def_id", format!("{owner_id:?}")),
                            ("declaration_def_id", format!("{declaration:?}")),
                            (
                                "reason",
                                "generic local call requires an actual monomorphic caller Instance"
                                    .into(),
                            ),
                        ],
                    )?;
                    continue;
                }
                let instance = Instance::try_resolve(
                    tcx,
                    ty::TypingEnv::fully_monomorphized(),
                    declaration,
                    args,
                )
                .map_err(|_| Stop::InstanceResolution)?;
                let Some(instance) = instance else {
                    self.record(
                        "unresolved_call",
                        &[
                            ("declaration_def_id", format!("{declaration:?}")),
                            ("reason", "selected compiler returned no Instance".into()),
                        ],
                    )?;
                    continue;
                };
                let resolved = instance.def_id();
                let item_body = matches!(instance.def, ty::InstanceKind::Item(_));
                let compiler_available = tcx.is_mir_available(resolved);
                let available = item_body && compiler_available;
                self.record(
                    "call",
                    &[
                        ("caller_def_id", format!("{owner_id:?}")),
                        ("caller_def_path", tcx.def_path_str(owner_id)),
                        (
                            "call_source",
                            tcx.sess
                                .source_map()
                                .span_to_diagnostic_string(block.terminator().source_info.span),
                        ),
                        ("declaration_def_id", format!("{declaration:?}")),
                        ("declaration_def_path", tcx.def_path_str(declaration)),
                        ("resolved_def_id", format!("{resolved:?}")),
                        ("resolved_def_path", tcx.def_path_str(resolved)),
                        ("generic_args", format!("{:?}", instance.args)),
                        ("instance", format!("{instance:?}")),
                        ("symbol", tcx.symbol_name(instance).name.to_owned()),
                        ("is_mir_available", compiler_available.to_string()),
                        ("body_exported", available.to_string()),
                        (
                            "mir_status",
                            if !item_body {
                                "compiler-generated Instance needs instance-MIR export"
                            } else if available {
                                "available"
                            } else {
                                "absent from actual supplied compiler metadata"
                            }
                            .into(),
                        ),
                        (
                            "source",
                            tcx.sess
                                .source_map()
                                .span_to_diagnostic_string(tcx.def_span(resolved)),
                        ),
                    ],
                )?;
                if available {
                    self.body(
                        tcx,
                        format!("{resolved:?}"),
                        tcx.optimized_mir(resolved),
                        "optimized_target",
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl Callbacks for Exporter {
    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        self.failure = self.built_roots(tcx).err();
        if self.failure.is_some() {
            Compilation::Stop
        } else {
            Compilation::Continue
        }
    }

    fn after_analysis<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        self.analysis_observed = true;
        self.failure = self.collect(tcx).err();
        if self.failure.is_some() {
            Compilation::Stop
        } else {
            Compilation::Continue
        }
    }
}

fn main() {
    let target_bound = match std::env::var("AC513_METADATA_MAX_TARGETS") {
        Ok(value) => value.parse::<usize>().ok().filter(|bound| *bound <= 4096),
        Err(std::env::VarError::NotPresent) => Some(32),
        Err(std::env::VarError::NotUnicode(_)) => None,
    };
    let roots = std::env::var("AC513_METADATA_ROOTS").ok().map(|value| {
        value
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    });
    let valid_roots = roots.as_ref().is_some_and(|values| {
        !values.is_empty()
            && values.len() <= MAX_OWNERS
            && values
                .iter()
                .enumerate()
                .all(|(at, value)| !values[..at].contains(value))
    });
    let mut exporter = Exporter {
        output: String::new(),
        targets: 0,
        target_bound: target_bound.unwrap_or(0),
        failure: target_bound.is_none().then_some(Stop::InvalidTargetBound),
        analysis_observed: false,
        roots: roots.unwrap_or_default(),
        selected_owners: Vec::new(),
    };
    if !valid_roots {
        exporter.failure = Some(Stop::InvalidRoots);
    }
    if exporter.failure.is_none() {
        rustc_driver::run_compiler(&std::env::args().collect::<Vec<_>>(), &mut exporter);
    }
    if exporter.failure.is_none() && !exporter.analysis_observed {
        exporter.failure = Some(Stop::AnalysisNotObserved);
    }
    let complete = exporter.failure.is_none();
    // A bounded emergency summary remains distinct from all body records.
    let summary = format!("{{\"kind\":\"summary\",\"complete\":{complete},\"targets\":{},\"closure_proven\":false,\"scope\":\"selected exact roots/local callees; built local and available optimized target MIR only\",\"failure\":\"{:?}\"}}\n", exporter.targets, exporter.failure);
    exporter.output.push_str(&summary);
    if std::io::stdout()
        .lock()
        .write_all(exporter.output.as_bytes())
        .is_err()
    {
        exporter.failure = Some(Stop::OutputIo);
    }
    if exporter.failure.is_some() {
        std::process::exit(2);
    }
}
