//! Arbitrary-precision natural numbers sufficient to decide whether a
//! canonical decimal rational is reduced, with every limb operation charged as
//! work so the metered cost bounds the real cost.

use super::WorkMeter;
use crate::checked_package::common::ValidationFailure;
use std::cmp::Ordering;

/// Little-endian base-2^32 limbs with no most-significant zero limbs.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Natural(Vec<u32>);

impl Natural {
    /// Parses ASCII decimal digits already validated by the caller. Each digit
    /// touches every limb accumulated so far, and is charged that many units.
    fn parse(digits: &str, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let mut limbs = Vec::new();
        for byte in digits.bytes() {
            meter.charge(limb_work(limbs.len()))?;
            let mut carry = u64::from(byte.wrapping_sub(b'0'));
            for limb in &mut limbs {
                let product = u64::from(*limb) * 10 + carry;
                *limb = low32(product);
                carry = product >> 32;
            }
            if carry != 0 {
                limbs.push(low32(carry));
            }
        }
        Ok(Self(limbs))
    }

    /// The work of one pass over this number's limbs (at least one unit).
    fn work(&self) -> u64 {
        limb_work(self.0.len())
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    fn is_even(&self) -> bool {
        self.0.first().is_none_or(|limb| limb & 1 == 0)
    }

    fn is_one(&self) -> bool {
        self.0.as_slice() == [1]
    }

    fn halve(&mut self) {
        let mut carry = 0_u32;
        for limb in self.0.iter_mut().rev() {
            let next = *limb & 1;
            *limb = (*limb >> 1) | (carry << 31);
            carry = next;
        }
        self.normalize();
    }

    /// Replaces `self` with `self - smaller`; requires `self >= smaller`.
    fn subtract(&mut self, smaller: &Self) {
        let mut borrow = 0_i64;
        for (index, limb) in self.0.iter_mut().enumerate() {
            let right = i64::from(smaller.0.get(index).copied().unwrap_or(0));
            let mut difference = i64::from(*limb) - right - borrow;
            if difference < 0 {
                difference += 1 << 32;
                borrow = 1;
            } else {
                borrow = 0;
            }
            *limb = low32(u64::try_from(difference).unwrap_or(0));
        }
        self.normalize();
    }

    fn normalize(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }
}

/// One unit per limb touched, and at least one unit per operation.
fn limb_work(limbs: usize) -> u64 {
    u64::try_from(limbs).unwrap_or(u64::MAX).max(1)
}

/// The low 32 bits of a limb computation, without a lossy cast.
fn low32(value: u64) -> u32 {
    u32::try_from(value & u64::from(u32::MAX)).unwrap_or(u32::MAX)
}

impl Ord for Natural {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl PartialOrd for Natural {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Returns whether `gcd(numerator, denominator) == 1` for canonical decimal
/// magnitudes (no sign). Binary GCD; each halving costs the limbs it shifts and
/// each compare-and-subtract step costs the limbs of the larger operand.
pub(super) fn coprime(
    numerator: &str,
    denominator: &str,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let mut left = Natural::parse(numerator, meter)?;
    let mut right = Natural::parse(denominator, meter)?;
    if left.is_zero() {
        return Ok(right.is_one());
    }
    if right.is_zero() {
        return Ok(left.is_one());
    }
    if left.is_even() && right.is_even() {
        return Ok(false);
    }
    loop {
        while left.is_even() {
            meter.charge(left.work())?;
            left.halve();
        }
        while right.is_even() {
            meter.charge(right.work())?;
            right.halve();
        }
        meter.charge(left.work().max(right.work()))?;
        match left.cmp(&right) {
            Ordering::Equal => return Ok(left.is_one()),
            Ordering::Less => right.subtract(&left),
            Ordering::Greater => left.subtract(&right),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_package::shared::CheckedPackageLimit;

    fn check(numerator: &str, denominator: &str) -> bool {
        let mut meter = WorkMeter::new(u64::MAX);
        coprime(numerator, denominator, &mut meter).expect("unbounded work")
    }

    /// Tracing: TC-048, FR-038-AC-5
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
        coprime(numerator, denominator, &mut meter).expect("unbounded work");
        meter.consumed()
    }

    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_charges_work_and_stops_at_the_limit() {
        let mut meter = WorkMeter::new(3);
        assert!(coprime("12345", "7", &mut meter).is_err());

        // Exact and one-over budgets for a multi-limb decision, computed by
        // hand for 2^64 + 1 over 1:
        // - parse "18446744073709551617": digit 1 costs 1; digits 2..=11 see
        //   one limb (10); digits 12..=20 see two limbs (9 * 2 = 18) => 29.
        // - parse "1" => 1.
        // - step 2^64 + 1 vs 1 (three limbs) => 3; left becomes 2^64.
        // - halve 2^64 (three limbs) => 3; 2^63..=2^32 (two limbs) => 32 * 2;
        //   2^31..=2^1 (one limb) => 31 => 98.
        // - step 1 vs 1 => 1.
        // Total 29 + 1 + 3 + 98 + 1 = 132; one unit per operation would be 87.
        let numerator = "18446744073709551617";
        let denominator = "1";
        let exact = 132;
        assert_eq!(work_of(numerator, denominator), exact);
        let mut meter = WorkMeter::new(exact);
        assert_eq!(coprime(numerator, denominator, &mut meter), Ok(true));
        let mut meter = WorkMeter::new(exact - 1);
        assert_eq!(
            coprime(numerator, denominator, &mut meter),
            Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Work,
                exact - 1,
                exact
            ))
        );
    }

    /// Tracing: TC-048, FR-038-AC-3
    #[test]
    fn tc_048_parse_work_grows_with_limbs_not_digits() {
        // Every digit after the first limb touches every accumulated limb, so
        // a long magnitude costs superlinearly more than its digit count.
        let digits = "9".repeat(2_000);
        let mut meter = WorkMeter::new(u64::MAX);
        let limbs = Natural::parse(&digits, &mut meter)
            .expect("unbounded work")
            .0
            .len();
        assert!(limbs > 200, "{limbs} limbs");
        assert!(meter.consumed() > 100 * 2_000, "{}", meter.consumed());

        // A default-sized budget refuses a million-digit scale during parsing.
        let huge = "1".repeat(1_000_000);
        let budget = 1_000_000;
        let mut meter = WorkMeter::new(budget);
        assert!(matches!(
            coprime(&huge, "1", &mut meter),
            Err(ValidationFailure::Incomplete(
                CheckedPackageLimit::Work,
                1_000_000,
                _
            ))
        ));
    }
}
