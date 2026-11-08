//! Exact nonnegative arithmetic through quire-exact's caller reservations.
//! Parse and GCD work use conservative bounds in the former 32-bit-limb units;
//! multiplication keeps its original operand-limb product charge.

use super::stage_storage::{Position, Site};
use super::WorkMeter;
use crate::checked_package::common::ValidationFailure;
use crate::checked_package::shared::JsonPointer;
use quire_exact::Integer;
use std::cmp::Ordering;

/// A borrowed arithmetic position at the caller's current graph site.
pub(super) fn site<'a>(meter: &WorkMeter, at: impl Into<Position<'a>>) -> Site<'a> {
    let (index, form) = meter
        .site()
        .map_or((0, "graph"), |site| (site.index, site.form));
    Site::new(super::PackageReadStage::Graph, index, form, at)
}

/// Charges work at the caller's original active locus, preserving its amount
/// and denial accounting. Only the denied outcome's borrowed pointer is paid
/// and materialized here; an admitted charge allocates nothing.
pub(super) fn work_at(
    meter: &mut WorkMeter,
    site: Site<'_>,
    amount: u64,
) -> Result<(), ValidationFailure> {
    if meter.budget_mut().is_none() {
        return meter.charge(amount, || site.at.build());
    }
    // The original charge records its first-unpaid locus before any diagnostic
    // reservation. Its temporary empty pointer owns no heap.
    match meter.charge(amount, JsonPointer::root) {
        Err(ValidationFailure::Budget(mut stop)) => {
            let paid = meter
                .budget_mut()
                .is_some_and(|budget| budget.prepay_diagnostic(site.at.pointer_len() as u128));
            stop.path = paid.then(|| site.at.build());
            Err(ValidationFailure::Budget(stop))
        }
        outcome => outcome,
    }
}

/// The decimal parse cannot touch more than this many 32-bit limbs per digit:
/// n digits need at most ceil(10n/3) bits. Saturation becomes a work denial.
fn parse_work(digits: usize) -> u64 {
    let digits = u64::try_from(digits).unwrap_or(u64::MAX);
    digits.saturating_mul(digits.saturating_mul(10).div_ceil(3).div_ceil(32).max(1))
}

fn limbs(value: &Integer) -> u64 {
    value.magnitude_bits().div_ceil(32).max(1)
}

/// Each old halving decreases the sum S of operand bit lengths; every unequal
/// odd comparison/subtraction is followed by at least one such halving. There
/// are at most S halvings and S+1 comparisons, each touching at most L limbs.
fn gcd_work(left: &Integer, right: &Integer) -> u64 {
    left.magnitude_bits()
        .saturating_add(right.magnitude_bits())
        .saturating_mul(2)
        .saturating_add(1)
        .saturating_mul(limbs(left).max(limbs(right)))
}

fn parse(digits: &str, meter: &mut WorkMeter, at: Site<'_>) -> Result<Integer, ValidationFailure> {
    work_at(meter, at, parse_work(digits.len()))?;
    meter.exact_integer(digits)?.ok_or_else(|| {
        super::stage_storage::refuse(
            meter,
            &at,
            super::stage_storage::Refusal::new(
                super::CheckedPackageRefusalCode::MalformedWire,
                at.at,
            ),
        )
    })
}

/// Decide reduction from validated canonical decimal magnitudes. Zero and the
/// both-even shortcut do not run a GCD and therefore charge no GCD bound.
pub(super) fn coprime(
    numerator: &str,
    denominator: &str,
    meter: &mut WorkMeter,
    at: Site<'_>,
) -> Result<bool, ValidationFailure> {
    let previous = meter.site();
    meter.at_node(at.index, at.form);
    let result = (|| {
        let left = parse(numerator, meter, at)?;
        let right = parse(denominator, meter, at)?;
        if left.is_zero() {
            return Ok(right == Integer::one());
        }
        if right.is_zero() {
            return Ok(left == Integer::one());
        }
        if left.is_even() && right.is_even() {
            return Ok(false);
        }
        work_at(meter, at, gcd_work(&left, &right))?;
        Ok(meter.exact_gcd(&left, &right)? == Integer::one())
    })();
    meter.restore_site(previous);
    result
}

/// Cross multiplication through reserved integers, retaining the former
/// max(1, left32limbs) * max(1, right32limbs) work before each product.
pub(super) fn compare_rationals(
    left: (&str, &str),
    right: (&str, &str),
    meter: &mut WorkMeter,
    at: Site<'_>,
) -> Result<Ordering, ValidationFailure> {
    let previous = meter.site();
    meter.at_node(at.index, at.form);
    let result = (|| {
        let left_numerator = parse(left.0, meter, at)?;
        let left_denominator = parse(left.1, meter, at)?;
        let right_numerator = parse(right.0, meter, at)?;
        let right_denominator = parse(right.1, meter, at)?;
        work_at(
            meter,
            at,
            limbs(&left_numerator).saturating_mul(limbs(&right_denominator)),
        )?;
        let scaled_left = meter.exact_mul(&left_numerator, &right_denominator)?;
        work_at(
            meter,
            at,
            limbs(&right_numerator).saturating_mul(limbs(&left_denominator)),
        )?;
        let scaled_right = meter.exact_mul(&right_numerator, &left_denominator)?;
        Ok(scaled_left.cmp(&scaled_right))
    })();
    meter.restore_site(previous);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::shared::CheckedPackageLimit;
    use ix_trace_rs::trace;

    #[trace("TC-472", "FR-038-AC-508", "FR-038-AC-516")]
    #[test]
    fn reserved_gcd_work_denials_preserve_the_active_locus_and_pay_the_pointer() {
        use super::super::native_join_budget::{
            NativeJoinLocus, NativeJoinLocusSource, PackageReadStage,
        };
        let path = super::super::stage_storage::Path::root().key("scale");
        let at = Site::new(PackageReadStage::Graph, 45, "timed_model", path);
        let original = NativeJoinLocus {
            source: NativeJoinLocusSource::PackageRead(PackageReadStage::Graph, 4),
            form: "scalar_type",
        };
        for allowance in [0, path.pointer_len() as u64 - 1, path.pointer_len() as u64] {
            let mut budget =
                super::super::NativeJoinBudget::new(1, 0).with_read_storage_limit(allowance);
            budget.set_active_locus(Some(original));
            let mut outcome = None;
            let allocated = allocation_counter::measure(|| {
                outcome = Some(coprime("1", "1", &mut WorkMeter::over(&mut budget), at));
            });
            let Err(ValidationFailure::Budget(stop)) = outcome.unwrap() else {
                panic!("work denial")
            };
            assert_eq!(stop.resource, super::super::PackageReadResource::Work);
            assert_eq!(stop.amount, 1);
            assert_eq!(budget.first_unpaid_locus(), Some(original));
            assert_eq!(allocated.bytes_total, budget.read_storage_consumed());
            assert_eq!(
                stop.path.as_ref().map(JsonPointer::as_str),
                (allowance == path.pointer_len() as u64).then_some("/scale")
            );
        }
    }

    #[trace("TC-472", "FR-038-AC-516", "FR-038-AC-511")]
    #[test]
    fn reserved_multilimb_gcd_pays_every_parse_and_reduction_request() {
        let numerator = "340282366920938463463374607431768211457";
        let denominator = "340282366920938463463374607431768211459";
        let run = |limit| {
            let mut budget =
                super::super::NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let allocated = allocation_counter::measure(|| {
                result = Some(coprime(
                    numerator,
                    denominator,
                    &mut WorkMeter::over(&mut budget),
                    at(),
                ));
            });
            (result.unwrap(), allocated.bytes_total, budget)
        };
        let (result, allocated, budget) = run(u64::MAX);
        assert_eq!(result, Ok(true));
        assert_eq!(allocated, budget.read_storage_consumed());
        // Both parses and both GCD copies request six 32-bit limbs each.
        assert_eq!(allocated, 4 * 6 * 4);
        assert_eq!(run(allocated).0, Ok(true));
        for limit in [23, 47, 71, 95] {
            let (result, allocated, budget) = run(limit);
            assert!(matches!(result, Err(ValidationFailure::Budget(_))));
            assert_eq!(allocated, budget.read_storage_consumed());
            assert_eq!(budget.first_unpaid_read_storage(), Some(24));
            let locus = budget.first_unpaid_locus().unwrap();
            assert_eq!(locus.form, "exact_number");
            assert_eq!(
                locus.source,
                super::super::NativeJoinLocusSource::PackageRead(
                    super::super::PackageReadStage::Graph,
                    0
                )
            );
        }
        assert_eq!(
            coprime(
                "680564733841876926926749214863536422914",
                numerator,
                &mut WorkMeter::new(u64::MAX),
                at()
            ),
            Ok(false)
        );
    }

    /// The rational a charge is reported at in these tests.
    fn at() -> Site<'static> {
        Site::new(
            super::super::PackageReadStage::Graph,
            0,
            "timed_model",
            super::super::stage_storage::Path::root().key("scale"),
        )
    }

    fn check(numerator: &str, denominator: &str) -> bool {
        let mut meter = WorkMeter::new(u64::MAX);
        coprime(numerator, denominator, &mut meter, at()).expect("unbounded work")
    }

    #[trace("TC-048", "FR-038-AC-121")]
    #[test]
    fn tc_048_decides_reduction_beyond_u128() {
        assert!(check("340282366920938463463374607431768211457", "1"));
        assert!(check("5463", "20"));
        assert!(!check("27315", "100"));
        assert!(check("0", "1"));
        assert!(!check("0", "2"));
        assert!(!check(
            "680564733841876926926749214863536422914",
            "340282366920938463463374607431768211457"
        ));
        assert!(check(
            "340282366920938463463374607431768211456",
            "340282366920938463463374607431768211457"
        ));
    }

    fn work_of(numerator: &str, denominator: &str) -> u64 {
        let mut meter = WorkMeter::new(u64::MAX);
        coprime(numerator, denominator, &mut meter, at()).expect("unbounded work");
        meter.consumed()
    }

    #[trace("TC-048", "FR-038-AC-3")]
    #[test]
    fn tc_048_charges_work_and_stops_at_the_limit() {
        let mut meter = WorkMeter::new(3);
        assert!(coprime("12345", "7", &mut meter, at()).is_err());

        // The old digit/limb binary implementation cost 132 here. The new
        // operand-size bound is 60 + 1 parse units plus 399 GCD units.
        let numerator = "18446744073709551617";
        let denominator = "1";
        let exact = 460;
        assert!(exact >= 132);
        assert_eq!(work_of(numerator, denominator), exact);
        let mut meter = WorkMeter::new(exact);
        assert_eq!(coprime(numerator, denominator, &mut meter, at()), Ok(true));
        let mut meter = WorkMeter::new(exact - 1);
        assert_eq!(
            coprime(numerator, denominator, &mut meter, at()),
            Err(ValidationFailure::incomplete(
                CheckedPackageLimit::Work,
                exact - 1,
                exact,
                Some(at().at.build())
            ))
        );
    }

    #[trace("TC-048", "FR-038-AC-3")]
    #[test]
    fn tc_048_parse_work_grows_with_limbs_not_digits() {
        // Every digit after the first limb touches every accumulated limb, so
        // a long magnitude costs superlinearly more than its digit count.
        let digits = "9".repeat(2_000);
        let mut meter = WorkMeter::new(u64::MAX);
        let limbs = parse(&digits, &mut meter, at())
            .expect("unbounded work")
            .magnitude_bits()
            .div_ceil(32);
        assert!(limbs > 200, "{limbs} limbs");
        assert!(meter.consumed() > 100 * 2_000, "{}", meter.consumed());

        // A default-sized budget refuses a million-digit scale during parsing.
        let huge = "1".repeat(1_000_000);
        let budget = 1_000_000;
        let mut meter = WorkMeter::new(budget);
        assert!(matches!(
            coprime(&huge, "1", &mut meter, at()),
            Err(ValidationFailure::Incomplete(incomplete))
                if incomplete.limit_kind == CheckedPackageLimit::Work
                    && incomplete.limit == 1_000_000
                    && incomplete.path == Some(at().at.build())
        ));
    }

    fn compare(left: (&str, &str), right: (&str, &str)) -> Ordering {
        let mut meter = WorkMeter::new(u64::MAX);
        compare_rationals(left, right, &mut meter, at()).expect("unbounded work")
    }

    /// The cross-multiplied comparison is exact beyond 64 bits and orders
    /// rationals, equal ones included, as their values do.
    #[trace("TC-048", "FR-038-AC-121")]
    #[test]
    fn tc_048_compares_rationals_exactly_beyond_2_pow_64() {
        let above = ("18446744073709551617", "3");
        let below = ("18446744073709551616", "3");
        assert_eq!(compare(above, below), Ordering::Greater);
        assert_eq!(compare(below, above), Ordering::Less);
        assert_eq!(compare(above, above), Ordering::Equal);
        assert_eq!(compare(("1", "2"), ("2", "3")), Ordering::Less);
        assert_eq!(compare(("2", "4"), ("1", "2")), Ordering::Equal);
        assert_eq!(compare(("0", "1"), ("0", "7")), Ordering::Equal);
        assert_eq!(compare(("0", "1"), ("1", "99")), Ordering::Less);
        // 2^32 * 2^32 carries across limbs: (2^32 / 1) against (2^64 / 2^32).
        assert_eq!(
            compare(("4294967296", "1"), ("18446744073709551616", "4294967296")),
            Ordering::Equal
        );
    }

    /// The comparison charges its parses and products to the meter and stops
    /// at the limit.
    #[trace("TC-048", "FR-038-AC-3")]
    #[test]
    fn tc_048_comparison_work_is_charged_and_stops_at_the_limit() {
        let left = ("18446744073709551617", "3");
        let right = ("18446744073709551616", "3");
        let mut meter = WorkMeter::new(u64::MAX);
        assert_eq!(
            compare_rationals(left, right, &mut meter, at()),
            Ok(Ordering::Greater)
        );
        let exact = meter.consumed();
        assert!(exact > 0);
        let mut meter = WorkMeter::new(exact - 1);
        assert!(matches!(
            compare_rationals(left, right, &mut meter, at()),
            Err(ValidationFailure::Incomplete(incomplete))
                if incomplete.limit_kind == CheckedPackageLimit::Work
                    && incomplete.path == Some(at().at.build())
        ));
    }
}
