//! Arbitrary-precision natural numbers sufficient to decide whether a
//! canonical decimal rational is reduced, with every step charged as work.

use super::WorkMeter;
use crate::checked_package::common::ValidationFailure;
use std::cmp::Ordering;

/// Little-endian base-2^32 limbs with no most-significant zero limbs.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Natural(Vec<u32>);

impl Natural {
    /// Parses ASCII decimal digits already validated by the caller.
    fn parse(digits: &str, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let mut limbs = Vec::new();
        for byte in digits.bytes() {
            meter.charge(1)?;
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
/// magnitudes (no sign). Binary GCD; each shift or subtraction costs one work.
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
            meter.charge(1)?;
            left.halve();
        }
        while right.is_even() {
            meter.charge(1)?;
            right.halve();
        }
        meter.charge(1)?;
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

    fn check(numerator: &str, denominator: &str) -> bool {
        let mut meter = WorkMeter::new(u64::MAX);
        coprime(numerator, denominator, &mut meter).expect("unbounded work")
    }

    #[test]
    fn decides_reduction_beyond_u128() {
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

    #[test]
    fn charges_work_and_stops_at_the_limit() {
        let mut meter = WorkMeter::new(3);
        assert!(coprime("12345", "7", &mut meter).is_err());
    }
}
