// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Canonical reservation source controls. Kernel controls bind call paths;
//! whole-reader controls prove the actual IR budget and typed parser locus.
//! Class credit additionally requires the compiler-source binding check.

use super::{Counter, ThreadCounter};
use quire_contract_model::{
    with_native_budget_observer, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    NativeBudgetEvent, NativeBudgetResource, NativeJoinBudget, NativeJoinLocusSource,
    PackageReadResource, PackageReadStage,
};
use std::cell::RefCell;
use std::io::Write;

#[derive(Clone, Copy)]
struct Provider {
    name: &'static str,
    input: &'static [u8],
    target: usize,
}
const PROVIDERS: &[Provider] = &[
    Provider {
        name: "arena",
        input: b"null",
        target: 0,
    },
    Provider {
        name: "text",
        input: br#""x""#,
        target: 0,
    },
    Provider {
        name: "stack",
        input: b"[]",
        target: 0,
    },
    // Source path: value/stack -> node/slots -> complete/scratch_items -> close/items.
    Provider {
        name: "children",
        input: b"[null]",
        target: 3,
    },
    // Empty names reserve no text; the first two members fit MIN_CAPACITY.
    // Source path: stack -> scratch_members -> slots -> check_unique_names/order.
    Provider {
        name: "duplicate_sort",
        input: br#"{"":null,"":null}"#,
        target: 3,
    },
];
fn provider(name: &str) -> Provider {
    *PROVIDERS
        .iter()
        .find(|p| p.name == name)
        .expect("declared source provider")
}
fn log(mut row: serde_json::Value) {
    row["test"] = std::env::var("IR1404_ORIGINAL_TEST_NAME")
        .unwrap_or_else(|_| {
            std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_owned()
        })
        .into();
    if let Some(path) = std::env::var_os("IR1404_READ_PROOF_LOG") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("class facts log");
        writeln!(file, "{row}").expect("class facts written");
    }
}
struct KernelReservation {
    deny: Option<usize>,
    calls: usize,
    granted: u64,
    amounts: [usize; 32],
}
impl quire_canonical::Reservation for KernelReservation {
    fn reserve(&mut self, bytes: usize) -> Result<(), quire_canonical::ReservationDenied> {
        assert!(
            self.calls < self.amounts.len(),
            "source control callback capacity"
        );
        self.amounts[self.calls] = bytes;
        let at = self.calls;
        self.calls += 1;
        if self.deny == Some(at) {
            return Err(quire_canonical::ReservationDenied);
        }
        self.granted = self
            .granted
            .checked_add(u64::try_from(bytes).expect("request fits"))
            .expect("small control total");
        Ok(())
    }
}
fn kernel(input: &[u8], name: &str, deny: Option<usize>) {
    let mut reservation = KernelReservation {
        deny,
        calls: 0,
        granted: 0,
        amounts: [0; 32],
    };
    let mut result = None;
    let requested = ThreadCounter::measure(|| {
        result = Some(quire_canonical::read_with_reservation(
            input,
            u64::MAX,
            &mut reservation,
        ))
    });
    assert_eq!(
        requested, reservation.granted,
        "kernel reserves before every actual allocation"
    );
    let result = result.expect("actual canonical read");
    let outcome = if let Some(index) = deny {
        assert_eq!(
            reservation.calls,
            index + 1,
            "the designated refusal stops subsequent reservations"
        );
        let Err(quire_canonical::ReadError::ReservationDenied { requested }) = &result else {
            panic!("actual kernel reservation denial required: {result:?}")
        };
        assert_eq!(*requested, reservation.amounts[index]);
        "reservation-denied"
    } else if name == "empty" {
        assert!(result.is_err());
        "malformed"
    } else if name == "duplicate_sort" {
        assert!(matches!(
            result,
            Err(quire_canonical::ReadError::DuplicateName { .. })
        ));
        "duplicate-name"
    } else {
        assert!(result.is_ok());
        "parsed"
    };
    log(
        serde_json::json!({"kind":"canonical-kernel","provider":name,"input":std::str::from_utf8(input).unwrap(),"deny":deny,
        "calls":reservation.calls,"callbacks":&reservation.amounts[..reservation.calls],
        "granted_callbacks":deny.unwrap_or(reservation.calls),"denied_request":deny.map(|index|reservation.amounts[index]),
        "granted":reservation.granted,"requested":requested,"outcome":outcome}),
    );
}
#[derive(Default)]
enum Events {
    #[default]
    Idle,
    Count(usize),
    Record {
        events: Vec<NativeBudgetEvent>,
        overflow: bool,
    },
}
thread_local! {static EVENTS:RefCell<Events>=const{RefCell::new(Events::Idle)};}
fn event(event: NativeBudgetEvent) {
    EVENTS.with(|state| match &mut *state.borrow_mut() {
        Events::Idle => panic!("event outside class capture"),
        Events::Count(count) => *count = count.checked_add(1).expect("bounded event count"),
        Events::Record { events, overflow } => {
            if events.len() == events.capacity() {
                *overflow = true
            } else {
                events.push(event)
            }
        }
    });
}
fn call(
    input: &[u8],
    limit: Option<u64>,
) -> (CheckedPackageV2BudgetedReadResult, NativeJoinBudget, u64) {
    let mut budget = limit.map_or_else(NativeJoinBudget::unlimited, |limit| {
        NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit)
    });
    let evidence = CheckedPackageEvidence::new();
    let mut result = None;
    let requested = ThreadCounter::measure(|| {
        result = Some(CheckedPackageV2::read_with_budget(
            input,
            CheckedPackageReadLimits::bounded(),
            &evidence,
            &mut budget,
        ))
    });
    assert_eq!(
        budget.consumed().storage,
        0,
        "no logical storage for canonical read"
    );
    assert_eq!(
        requested,
        budget.read_storage_consumed(),
        "whole public read pays all actual requests"
    );
    (result.expect("actual public read"), budget, requested)
}
fn observe(
    input: &[u8],
    limit: Option<u64>,
) -> (
    CheckedPackageV2BudgetedReadResult,
    NativeJoinBudget,
    u64,
    Vec<NativeBudgetEvent>,
) {
    // First real read is observer-OFF; callback machinery is initialized only
    // afterward, outside every measured reader interval.
    let off = call(input, limit);
    EVENTS.with(|state| *state.borrow_mut() = Events::Count(0));
    let counted = with_native_budget_observer(event, || call(input, limit));
    let Events::Count(count) = EVENTS.with(|state| state.replace(Events::Idle)) else {
        unreachable!()
    };
    let events = Vec::with_capacity(count);
    EVENTS.with(|state| {
        *state.borrow_mut() = Events::Record {
            events,
            overflow: false,
        }
    });
    let recorded = with_native_budget_observer(event, || call(input, limit));
    assert_eq!(
        counted, recorded,
        "passive source capture preserves full outcome/budget/requests"
    );
    assert_eq!(
        off, recorded,
        "observer-OFF and recorded reads have the same full outcome/budget/requests"
    );
    let Events::Record { events, overflow } = EVENTS.with(|state| state.replace(Events::Idle))
    else {
        unreachable!()
    };
    assert!(!overflow, "preallocated event buffer overflow");
    assert_eq!(events.len(), count);
    (recorded.0, recorded.1, recorded.2, events)
}
fn package_events(events: &[NativeBudgetEvent]) -> Vec<&NativeBudgetEvent> {
    events
        .iter()
        .filter(|event| {
            event.resource == NativeBudgetResource::ReadStorage
                && event.amount > 0
                && event.locus.is_some_and(|locus| {
                    locus.source
                        == NativeJoinLocusSource::PackageRead(PackageReadStage::Envelope, 0)
                        && locus.form == "package_bytes"
                })
        })
        .collect()
}
fn whole(name: &str, phase: &str) {
    let input = if name == "empty" {
        b"".as_slice()
    } else {
        provider(name).input
    };
    let reference = if phase == "allow" {
        None
    } else {
        let path = std::env::var_os("IR1404_CANONICAL_REFERENCE_LOG");
        let facts = if let Some(path) = path {
            let text = std::fs::read_to_string(path).expect("same frozen provider reference");
            let row: serde_json::Value =
                serde_json::from_str(text.lines().next().expect("reference row")).unwrap();
            assert_eq!(row["provider"], name);
            assert_eq!(row["input"], std::str::from_utf8(input).unwrap());
            row
        } else {
            whole_reference(name, input)
        };
        Some(facts)
    };
    let limit = reference.as_ref().map(|facts| {
        let before = facts["target_before"].as_u64().expect("actual prefix");
        let amount = facts["target_amount"].as_u64().expect("actual reservation");
        let exact = before.checked_add(amount).expect("small provider bound");
        if phase == "deny" {
            exact - 1
        } else {
            exact
        }
    });
    let (result, budget, requested, events) = observe(input, limit);
    let package = package_events(&events);
    if let Some(reference) = &reference {
        let target = provider(name).target;
        let observed = package
            .get(target)
            .expect("target package parser callback reached");
        assert_eq!(
            observed.amount,
            u128::from(reference["target_amount"].as_u64().unwrap())
        );
        assert_eq!(
            observed.consumed_before,
            reference["target_before"].as_u64().unwrap()
        );
        assert_eq!(
            observed.admitted,
            phase == "exact",
            "exact or one-below target admission"
        );
        if phase == "deny" {
            assert_eq!(budget.first_unpaid_read_storage(), Some(observed.amount));
            assert_eq!(budget.first_unpaid_locus(), observed.locus);
            let bound = limit.expect("finite denial control");
            match &result {
                CheckedPackageV2BudgetedReadResult::BudgetIncomplete(stop) => {
                    assert_eq!(stop.denied.resource, PackageReadResource::ReadStorage);
                    assert_eq!(stop.denied.amount, observed.amount);
                    assert_eq!(stop.limit, bound);
                    assert_eq!(
                        stop.consumed,
                        observed.consumed_before + u64::try_from(observed.amount).unwrap()
                    );
                    assert!(
                        stop.path.is_none(),
                        "Envelope reservation denial is pointer-free"
                    );
                }
                CheckedPackageV2BudgetedReadResult::Exhausted(stop) => {
                    assert_eq!(stop.denied.resource, PackageReadResource::ReadStorage);
                    assert_eq!(stop.denied.amount, observed.amount);
                    assert_eq!(stop.configured_bound, bound);
                    assert_eq!(Some(stop.locus), observed.locus);
                    assert!(
                        stop.pending.is_none(),
                        "no semantic refusal began before this reservation denial"
                    );
                }
                other => panic!(
                    "denied parser reservation cannot admit or return semantic refusal: {other:?}"
                ),
            }
        }
    } else {
        assert!(
            matches!(result, CheckedPackageV2BudgetedReadResult::Refused(_)),
            "minimal scalar/container is not a positive V2 package"
        );
        if name == "duplicate_sort" {
            assert!(
                matches!(&result,CheckedPackageV2BudgetedReadResult::Refused(refusal) if refusal.code==CheckedPackageRefusalCode::DuplicateMember)
            );
        }
    }
    let mut row = serde_json::json!({"kind":"canonical-whole","provider":name,"phase":phase,"input":std::str::from_utf8(input).unwrap(),"requested":requested,"charged":budget.read_storage_consumed(),"reader_calls":3,"target":provider_index(name)});
    if name != "empty" {
        let target = package[provider(name).target];
        row["target_before"] = target.consumed_before.into();
        row["target_amount"] = u64::try_from(target.amount).unwrap().into();
        row["typed_locus"] =
            serde_json::json!({"stage":"Envelope","index":0,"form":"package_bytes"});
    }
    log(row);
}
fn provider_index(name: &str) -> Option<usize> {
    if name == "empty" {
        None
    } else {
        Some(provider(name).target)
    }
}
fn whole_reference(name: &str, input: &[u8]) -> serde_json::Value {
    let (_, _, _, events) = observe(input, None);
    let target = package_events(&events)[provider(name).target];
    serde_json::json!({"provider":name,"input":std::str::from_utf8(input).unwrap(),"target_before":target.consumed_before,"target_amount":u64::try_from(target.amount).unwrap()})
}
#[test]
fn kernel_empty() {
    kernel(b"", "empty", None);
}
#[test]
fn whole_empty() {
    whole("empty", "allow");
}
macro_rules! controls {
    ($allow:ident,$deny:ident,$whole:ident,$short:ident,$exact:ident,$name:literal) => {
        #[test]
        fn $allow() {
            let p = provider($name);
            kernel(p.input, p.name, None);
        }
        #[test]
        fn $deny() {
            let p = provider($name);
            kernel(p.input, p.name, Some(p.target));
        }
        #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
        #[test]
        fn $whole() {
            whole($name, "allow");
        }
        #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
        #[test]
        fn $short() {
            whole($name, "deny");
        }
        #[ix_trace_rs::trace("TC-472", "FR-038-AC-511")]
        #[test]
        fn $exact() {
            whole($name, "exact");
        }
    };
}
controls!(
    kernel_arena_allow,
    kernel_arena_deny,
    whole_arena_allow,
    whole_arena_deny,
    whole_arena_exact,
    "arena"
);
controls!(
    kernel_text_allow,
    kernel_text_deny,
    whole_text_allow,
    whole_text_deny,
    whole_text_exact,
    "text"
);
controls!(
    kernel_stack_allow,
    kernel_stack_deny,
    whole_stack_allow,
    whole_stack_deny,
    whole_stack_exact,
    "stack"
);
controls!(
    kernel_children_allow,
    kernel_children_deny,
    whole_children_allow,
    whole_children_deny,
    whole_children_exact,
    "children"
);
controls!(
    kernel_duplicate_sort_allow,
    kernel_duplicate_sort_deny,
    whole_duplicate_sort_allow,
    whole_duplicate_sort_deny,
    whole_duplicate_sort_exact,
    "duplicate_sort"
);

// Their real kernel denial oracles pass; source binding MUST reject these
// candidates as the named class because its genuine predecessor path is absent.
#[test]
fn negative_empty_string_is_not_text_growth() {
    kernel(br#""""#, "negative_text_empty", Some(0));
}
#[test]
fn negative_empty_array_is_not_retained_children() {
    kernel(b"[]", "negative_children_empty", Some(1));
}
#[test]
fn negative_scratch_child_is_not_retained_children() {
    kernel(b"[null]", "negative_children_scratch", Some(2));
}
