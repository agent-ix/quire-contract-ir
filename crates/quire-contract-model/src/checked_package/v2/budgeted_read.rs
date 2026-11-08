// SPDX-License-Identifier: AGPL-3.0-or-later
//! The budget-charged strict V2 read: its entries, outcomes and the package
//! envelope's read and canonical comparison under the caller's budget.
//!
//! Implements: FR-038 "Budget-charged strict read", FR-358.

use super::native_join_budget::{NativeJoinBudget, NativeJoinLocus, NativeJoinLocusSource};
use super::{
    CatalogReadError, CheckedPackageEvidence, CheckedPackageV2, CheckedPackageV2ReadResult,
    PackageReadStage, ValidationFailure,
};
use crate::checked_package::common::{exceeds, refuse_bytes_exceeded};
use crate::checked_package::shared::{
    CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageReadLimits, CheckedPackageRefusal,
    CheckedPackageRefusalCode, JsonPointer,
};
use ix_trace_rs::trace;
use quire_canonical::{Document, ReadError, Reservation, ReservationDenied};

/// The budget counter a strict read ran out of.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageReadResource {
    /// Validation work units.
    Work,
    /// Bytes of heap requested by the read.
    ReadStorage,
}

/// A charge the budget could not admit: its resource and full attempted amount.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageReadDeniedCharge {
    /// The resource that ran out.
    pub resource: PackageReadResource,
    /// The full amount that was attempted.
    pub amount: u128,
}

/// A budgeted read that stopped at its first unpaid charge, with the owned
/// diagnostic affordable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedPackageBudgetIncomplete {
    /// The first unpaid charge.
    pub denied: PackageReadDeniedCharge,
    /// The budget's configured limit for the denied resource.
    pub limit: u64,
    /// Cumulative admitted consumption of the resource plus the denied amount.
    pub consumed: u64,
    /// The RFC 6901 pointer of the value whose charge failed, when a package
    /// value was being decoded or validated.
    pub path: Option<JsonPointer>,
}

/// The fixed-size class of the outcome a read was building when its budget ran
/// out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckedPackagePendingOutcome {
    /// A refusal with this code.
    Refused(CheckedPackageRefusalCode),
    /// A structural `incomplete` of this kind.
    Incomplete(CheckedPackageLimit),
}

/// A budgeted read whose budget could not pay for the diagnostic of its own
/// outcome. It owns no pointer, locus list or string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedPackageReadExhausted {
    /// The original first unpaid charge.
    pub denied: PackageReadDeniedCharge,
    /// The configured limit of the denied resource, including zero.
    pub configured_bound: u64,
    /// Where the denied charge was made.
    pub locus: NativeJoinLocus,
    /// The outcome being built, if any.
    pub pending: Option<CheckedPackagePendingOutcome>,
}

/// The closed result of a budgeted strict V2 read.
// The inline typed refusal makes this larger than its other cases; boxing it
// would change the unbudgeted result's shape this one mirrors.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageV2BudgetedReadResult {
    /// A complete V2 package is safe for lowering.
    Admitted(Box<CheckedPackageV2>),
    /// The input is invalid for V2.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a structural cap or the byte limit.
    Incomplete(CheckedPackageIncomplete),
    /// The budget ran out; the diagnostic was affordable.
    BudgetIncomplete(CheckedPackageBudgetIncomplete),
    /// The budget ran out and could not pay for the diagnostic.
    Exhausted(CheckedPackageReadExhausted),
    /// The reader's operation catalog is unreadable; no input was judged.
    CatalogUnavailable(CatalogReadError),
}

/// The closed result of dispatching bytes through the budgeted read.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedPackageBudgetedDispatchResult {
    /// A `quire.checked-package/v2` package admitted by the V2 reader.
    AdmittedV2(Box<CheckedPackageV2>),
    /// The input is invalid for its selected version, or selects none.
    Refused(CheckedPackageRefusal),
    /// The input exceeded a structural cap or the byte limit.
    Incomplete(CheckedPackageIncomplete),
    /// The budget ran out; the diagnostic was affordable.
    BudgetIncomplete(CheckedPackageBudgetIncomplete),
    /// The budget ran out and could not pay for the diagnostic.
    Exhausted(CheckedPackageReadExhausted),
    /// The reader's operation catalog is unreadable; no input was judged.
    CatalogUnavailable(CatalogReadError),
}

/// A budget denial inside the read, before its outcome is built.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::checked_package) struct BudgetStop {
    pub(in crate::checked_package) resource: PackageReadResource,
    pub(in crate::checked_package) amount: u128,
    /// The pointer of the value whose charge failed, when there is one.
    pub(in crate::checked_package) path: Option<JsonPointer>,
}

impl BudgetStop {
    /// The structural `incomplete` a reader that holds no caller budget (the
    /// FR-035 contract-package reader, lowering) reports for this stop. Those
    /// readers meter against their own `work` limit and never build one, so
    /// this only keeps their matches over the read outcomes closed.
    pub(in crate::checked_package) fn without_budget(self) -> CheckedPackageIncomplete {
        CheckedPackageIncomplete {
            limit_kind: CheckedPackageLimit::Work,
            limit: 0,
            consumed: u64::try_from(self.amount).unwrap_or(u64::MAX),
            path: self.path,
        }
    }
}

/// The locus form of an owned refusal or `incomplete` diagnostic.
const DIAGNOSTIC_FORM: &str = "diagnostic";

impl CheckedPackageV2 {
    /// Reads one canonical V2 value under the caller's budget.
    ///
    /// This is the one read implementation: every work step and every heap
    /// request is charged to `budget` before it is made, and `limits.work` is
    /// not consulted. A refused, incomplete or exhausted read keeps its
    /// admitted charges in the budget's cumulative consumption.
    #[trace("FR-038-AC-495", "FR-038-AC-496")]
    pub fn read_with_budget(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
        budget: &mut NativeJoinBudget,
    ) -> CheckedPackageV2BudgetedReadResult {
        Self::read_with_budget_against(
            super::operation_catalog::production_catalog(),
            bytes,
            limits,
            evidence,
            budget,
        )
    }

    /// [`Self::read_with_budget`] against the given result of reading the
    /// operation catalog: an unreadable catalog returns before any charge.
    pub(super) fn read_with_budget_against(
        catalog: &'static Result<super::operation_catalog::OperationCatalog, CatalogReadError>,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
        budget: &mut NativeJoinBudget,
    ) -> CheckedPackageV2BudgetedReadResult {
        let catalog = match super::operation_catalog::available(catalog) {
            Ok(catalog) => catalog,
            Err(error) => return CheckedPackageV2BudgetedReadResult::CatalogUnavailable(error),
        };
        let work_before = budget.consumed();
        let storage_before = budget.read_storage_mark();
        budget.resume();
        match Self::admit_bytes(catalog, bytes, limits, evidence, budget) {
            Ok(package) => {
                budget.finish_read(work_before, storage_before);
                CheckedPackageV2BudgetedReadResult::Admitted(Box::new(package))
            }
            Err(stop) => {
                budget.abort();
                budgeted_outcome(stop, budget)
            }
        }
    }

    /// Reads one canonical V2 value without exposing a partial package.
    ///
    /// Equivalent to [`Self::read_with_budget`] under a budget whose work
    /// limit is `limits.work` and whose read storage is unbounded.
    pub fn read(
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageV2ReadResult {
        Self::read_against(
            super::operation_catalog::production_catalog(),
            bytes,
            limits,
            evidence,
        )
    }

    /// [`Self::read`] against the given result of reading the operation
    /// catalog.
    pub(super) fn read_against(
        catalog: &'static Result<super::operation_catalog::OperationCatalog, CatalogReadError>,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageV2ReadResult {
        let mut budget =
            NativeJoinBudget::new(limits.work, u64::MAX).with_read_storage_limit(u64::MAX);
        match Self::read_with_budget_against(catalog, bytes, limits, evidence, &mut budget) {
            CheckedPackageV2BudgetedReadResult::Admitted(package) => {
                CheckedPackageV2ReadResult::Admitted(package)
            }
            CheckedPackageV2BudgetedReadResult::Refused(refusal) => {
                CheckedPackageV2ReadResult::Refused(refusal)
            }
            CheckedPackageV2BudgetedReadResult::Incomplete(incomplete) => {
                CheckedPackageV2ReadResult::Incomplete(incomplete)
            }
            CheckedPackageV2BudgetedReadResult::CatalogUnavailable(error) => {
                CheckedPackageV2ReadResult::CatalogUnavailable(error)
            }
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(incomplete) => {
                CheckedPackageV2ReadResult::Incomplete(CheckedPackageIncomplete {
                    limit_kind: CheckedPackageLimit::Work,
                    limit: limits.work,
                    consumed: incomplete.consumed,
                    path: incomplete.path,
                })
            }
            // The read-storage limit of `u64::MAX` exceeds every possible total
            // of requested bytes, so the unbudgeted read is never exhausted;
            // the work counter's own account stands in if it ever were.
            CheckedPackageV2BudgetedReadResult::Exhausted(_) => {
                CheckedPackageV2ReadResult::Incomplete(CheckedPackageIncomplete {
                    limit_kind: CheckedPackageLimit::Work,
                    limit: limits.work,
                    consumed: budget.consumed().work,
                    path: None,
                })
            }
        }
    }
}

/// Builds the budgeted outcome of a stopped read, charging its owned
/// diagnostic to the budget first.
fn budgeted_outcome(
    stop: ValidationFailure,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    match stop {
        ValidationFailure::Refused(refusal) => {
            let bytes = refusal
                .loci
                .iter()
                .filter_map(|location| location.path.as_ref())
                .chain(refusal.document_pointer.as_ref())
                .map(|pointer| pointer.as_str().len() as u128)
                .sum();
            match charge_outcome_diagnostic(
                bytes,
                CheckedPackagePendingOutcome::Refused(refusal.code),
                budget,
            ) {
                None => CheckedPackageV2BudgetedReadResult::Refused(*refusal),
                Some(exhausted) => exhausted,
            }
        }
        ValidationFailure::Incomplete(incomplete) => {
            let bytes = incomplete
                .path
                .as_ref()
                .map_or(0, |pointer| pointer.as_str().len() as u128);
            match charge_outcome_diagnostic(
                bytes,
                CheckedPackagePendingOutcome::Incomplete(incomplete.limit_kind),
                budget,
            ) {
                None => CheckedPackageV2BudgetedReadResult::Incomplete(incomplete),
                Some(exhausted) => exhausted,
            }
        }
        ValidationFailure::Budget(stop) => budget_denial(stop, budget),
    }
}

/// Charges the owned diagnostic of a refusal or `incomplete` outcome before it
/// is returned; `Some` is the `Exhausted` outcome when the budget cannot pay.
fn charge_outcome_diagnostic(
    bytes: u128,
    pending: CheckedPackagePendingOutcome,
    budget: &mut NativeJoinBudget,
) -> Option<CheckedPackageV2BudgetedReadResult> {
    let locus = NativeJoinLocus {
        source: NativeJoinLocusSource::Package,
        form: DIAGNOSTIC_FORM,
    };
    budget.set_active_locus(Some(locus));
    let paid = budget.charge_read_storage(bytes);
    budget.set_active_locus(None);
    if paid {
        return None;
    }
    Some(CheckedPackageV2BudgetedReadResult::Exhausted(
        CheckedPackageReadExhausted {
            denied: PackageReadDeniedCharge {
                resource: PackageReadResource::ReadStorage,
                amount: bytes,
            },
            configured_bound: budget.read_storage_limit(),
            locus: budget.first_unpaid_locus().unwrap_or(locus),
            pending: Some(pending),
        },
    ))
}

fn budget_denial(
    stop: BudgetStop,
    budget: &mut NativeJoinBudget,
) -> CheckedPackageV2BudgetedReadResult {
    let denied = PackageReadDeniedCharge {
        resource: stop.resource,
        amount: stop.amount,
    };
    let (limit, consumed) = match stop.resource {
        PackageReadResource::Work => (budget.work_limit(), budget.consumed().work),
        PackageReadResource::ReadStorage => {
            (budget.read_storage_limit(), budget.read_storage_consumed())
        }
    };
    let original_locus = budget.first_unpaid_locus();
    let diagnostic_bytes = stop
        .path
        .as_ref()
        .map_or(0, |path| path.as_str().len() as u128);
    if let Some(locus) = original_locus {
        budget.set_active_locus(Some(NativeJoinLocus {
            source: locus.source,
            form: DIAGNOSTIC_FORM,
        }));
    }
    let affordable = budget.charge_read_storage(diagnostic_bytes);
    budget.set_active_locus(None);
    match (affordable, original_locus) {
        (true, _) => {
            let consumed =
                u64::try_from(u128::from(consumed).saturating_add(stop.amount)).unwrap_or(u64::MAX);
            CheckedPackageV2BudgetedReadResult::BudgetIncomplete(CheckedPackageBudgetIncomplete {
                denied,
                limit,
                consumed,
                path: stop.path,
            })
        }
        (false, Some(locus)) => {
            CheckedPackageV2BudgetedReadResult::Exhausted(CheckedPackageReadExhausted {
                denied,
                configured_bound: limit,
                locus,
                pending: None,
            })
        }
        // A budget denial always records its locus; with none recorded the
        // denial still reports its fixed-size facts at the package level.
        (false, None) => {
            CheckedPackageV2BudgetedReadResult::Exhausted(CheckedPackageReadExhausted {
                denied,
                configured_bound: limit,
                locus: NativeJoinLocus {
                    source: NativeJoinLocusSource::Package,
                    form: DIAGNOSTIC_FORM,
                },
                pending: None,
            })
        }
    }
}

/// The shared reader's and writer's reservation: charges read storage on the
/// caller's budget at its active locus.
struct ReadStorageReservation<'b>(&'b mut NativeJoinBudget);

impl Reservation for ReadStorageReservation<'_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), ReservationDenied> {
        if self.0.charge_read_storage(bytes as u128) {
            Ok(())
        } else {
            Err(ReservationDenied)
        }
    }
}

/// The denial the budget just recorded, as the stop that ends the read.
fn denial(budget: &NativeJoinBudget) -> ValidationFailure {
    let (resource, amount) = budget
        .first_unpaid_resource()
        .unwrap_or((PackageReadResource::ReadStorage, 0));
    ValidationFailure::Budget(BudgetStop {
        resource,
        amount,
        path: None,
    })
}

pub(super) fn at_stage(budget: &mut NativeJoinBudget, stage: PackageReadStage, form: &'static str) {
    budget.set_active_locus(Some(NativeJoinLocus {
        source: NativeJoinLocusSource::PackageRead(stage, 0),
        form,
    }));
}

/// A sink comparing the canonical encoding with the package bytes as it is
/// written; it asks the writer for no memory of its own.
struct ComparingSink<'a> {
    original: &'a [u8],
    at: usize,
    agrees: bool,
}

impl quire_canonical::Sink for ComparingSink<'_> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), quire_canonical::Error> {
        let end = self.at.checked_add(bytes.len());
        if end.and_then(|end| self.original.get(self.at..end)) != Some(bytes) {
            self.agrees = false;
        }
        self.at = end.unwrap_or(usize::MAX);
        Ok(())
    }
}

/// The package document as the canonical writer encodes it, refusing a whole
/// number past 2^53 that is not a 64-bit integer: such a spelling
/// (`100000000000000000000`) is its own RFC 8785 text as a double, yet it is
/// an integer outside I-JSON, so its bytes are not canonical. The walk keeps
/// its task list on the writer's reservation.
struct PackageDocument<'d> {
    document: &'d Document,
}

impl quire_canonical::Encode for PackageDocument<'_> {
    // The one string comparison is a closed-vocabulary guard on a serde_json
    // private member name, not an open key.
    #[ix_trace_rs::string_edge]
    fn encode_into<S: quire_canonical::Sink + ?Sized>(
        &self,
        writer: &mut quire_canonical::Writer<'_, S>,
    ) -> Result<(), quire_canonical::Error> {
        use quire_canonical::Node;
        const MAXIMUM_INTEGER: f64 = 9_007_199_254_740_992.0;
        enum Task<'d> {
            Value(quire_canonical::NodeRef<'d>),
            Name(&'d str),
            EndArray,
            EndObject,
        }
        let mut tasks = Vec::new();
        writer.reserve_list(&mut tasks, 1)?;
        tasks.push(Task::Value(self.document.root()));
        while let Some(task) = tasks.pop() {
            match task {
                Task::Value(value) => match value.node() {
                    Node::Null => writer.null()?,
                    Node::Bool(flag) => writer.bool(flag)?,
                    Node::Number(number) => {
                        let float = number.value();
                        if number.text().parse::<i64>().is_err()
                            && number.text().parse::<u64>().is_err()
                            && float.fract() == 0.0
                            && float.abs() > MAXIMUM_INTEGER
                        {
                            return Err(quire_canonical::Error::Internal {
                                invariant: "an integer past 2^53 is not canonical",
                            });
                        }
                        writer.number(float)?;
                    }
                    Node::String(text) => writer.string(text)?,
                    Node::Array(items) => {
                        writer.begin_array()?;
                        let start = tasks.len();
                        writer.reserve_list(&mut tasks, items.len().saturating_add(1))?;
                        tasks.push(Task::EndArray);
                        tasks.extend(items.map(Task::Value));
                        tasks.get_mut(start + 1..).unwrap_or_default().reverse();
                    }
                    Node::Object(members) => {
                        // serde_json reads an object whose first member is this
                        // private name as a number, so two byte strings would
                        // decode to one package: the spelling is not canonical.
                        let mut members = members.peekable();
                        if members
                            .peek()
                            .is_some_and(|(name, _)| *name == SERDE_JSON_NUMBER_TOKEN)
                        {
                            return Err(quire_canonical::Error::Internal {
                                invariant: "serde_json's number token is not canonical",
                            });
                        }
                        writer.begin_object()?;
                        let start = tasks.len();
                        writer.reserve_list(
                            &mut tasks,
                            members.len().saturating_mul(2).saturating_add(1),
                        )?;
                        tasks.push(Task::EndObject);
                        for (name, member) in members {
                            tasks.push(Task::Name(name));
                            tasks.push(Task::Value(member));
                        }
                        tasks.get_mut(start + 1..).unwrap_or_default().reverse();
                    }
                },
                Task::Name(name) => writer.name(name)?,
                Task::EndArray => writer.end_array()?,
                Task::EndObject => writer.end_object()?,
            }
        }
        Ok(())
    }
}

const SERDE_JSON_NUMBER_TOKEN: &str = "$serde_json::private::Number";
const SERDE_JSON_RECURSION_LIMIT: usize = 128;

/// The deepest container nesting of bytes the shared reader accepted, counted
/// in one pass that holds no memory.
fn nesting_depth(bytes: &[u8]) -> usize {
    let (mut depth, mut deepest) = (0_usize, 0_usize);
    let (mut in_string, mut escaped) = (false, false);
    for byte in bytes {
        if in_string {
            match (escaped, byte) {
                (true, _) => escaped = false,
                (false, b'\\') => escaped = true,
                (false, b'"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'[' | b'{' => {
                depth = depth.saturating_add(1);
                deepest = deepest.max(depth);
            }
            b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// Reads the package bytes with the shared reader and requires them to be
/// their own canonical encoding, charging every reservation of both to the
/// budget's read storage. The byte limit is checked before any request.
///
/// A byte-stream fault refuses before `noncanonical_wire`, as FR-038-AC-509
/// orders them.
pub(in crate::checked_package) fn read_envelope(
    bytes: &[u8],
    limits: CheckedPackageReadLimits,
    budget: &mut NativeJoinBudget,
) -> Result<Document, ValidationFailure> {
    if exceeds(bytes.len(), limits.bytes) {
        return Err(refuse_bytes_exceeded(limits.bytes, bytes.len()));
    }
    at_stage(budget, PackageReadStage::Envelope, "package_bytes");
    let document = match quire_canonical::read_with_reservation(
        bytes,
        limits.bytes,
        &mut ReadStorageReservation(budget),
    ) {
        Ok(document) => document,
        Err(ReadError::Limit(_)) => return Err(refuse_bytes_exceeded(limits.bytes, bytes.len())),
        Err(ReadError::DuplicateName { pointer, .. }) => {
            return Err(ValidationFailure::refused(
                CheckedPackageRefusalCode::DuplicateMember,
                JsonPointer::from_escaped(pointer),
            ));
        }
        Err(ReadError::ReservationDenied { .. }) => return Err(denial(budget)),
        Err(_) => {
            return Err(ValidationFailure::refused_bytes(
                CheckedPackageRefusalCode::MalformedWire,
            ))
        }
    };
    // The typed decode below reads term bodies with serde_json, which this
    // read keeps to its 128-level recursion bound.
    if nesting_depth(bytes) >= SERDE_JSON_RECURSION_LIMIT {
        return Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::MalformedWire,
        ));
    }
    at_stage(budget, PackageReadStage::Canonical, "canonical_bytes");
    let mut sink = ComparingSink {
        original: bytes,
        at: 0,
        agrees: true,
    };
    let encoded = quire_canonical::encode_with_reservation(
        &mut sink,
        &PackageDocument {
            document: &document,
        },
        quire_canonical::Limits::new(limits.bytes),
        &mut ReadStorageReservation(budget),
    );
    match encoded {
        Err(quire_canonical::Error::ReservationDenied { .. }) => Err(denial(budget)),
        Ok(_) if sink.agrees && sink.at == bytes.len() => {
            budget.set_active_locus(None);
            Ok(document)
        }
        _ => Err(ValidationFailure::refused_bytes(
            CheckedPackageRefusalCode::NoncanonicalWire,
        )),
    }
}
