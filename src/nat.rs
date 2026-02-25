//! Natural number operations with big integer support.

use crate::bignat::*;
use crate::object::LeanObject;
use crate::r#box::*;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

#[inline]
pub fn lean_unsigned_to_nat(n: usize) -> *mut LeanObject {
    if n <= LEAN_MAX_SMALL_NAT {
        lean_box(n)
    } else {
        unsafe { lean_alloc_bignat(BigUint::from(n)) }
    }
}

#[inline]
pub fn lean_usize_to_nat(n: usize) -> *mut LeanObject {
    lean_unsigned_to_nat(n)
}

#[inline]
pub fn lean_uint64_to_nat(n: u64) -> *mut LeanObject {
    let v = n as usize;
    if v as u64 == n && v <= LEAN_MAX_SMALL_NAT {
        lean_box(v)
    } else {
        unsafe { lean_alloc_bignat(BigUint::from(n)) }
    }
}

pub fn lean_big_usize_to_nat(n: usize) -> *mut LeanObject {
    lean_unsigned_to_nat(n)
}

pub fn lean_big_uint64_to_nat(n: u64) -> *mut LeanObject {
    lean_uint64_to_nat(n)
}

#[inline]
pub unsafe fn lean_nat_add(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        match va.checked_add(vb) {
            Some(r) if r <= LEAN_MAX_SMALL_NAT => lean_box(r),
            _ => {
                let ba = BigUint::from(va);
                let bb = BigUint::from(vb);
                lean_bignat_to_nat(ba + bb)
            }
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        lean_bignat_to_nat(ba + bb)
    }
}

#[inline]
pub unsafe fn lean_nat_sub(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        lean_box(va.saturating_sub(vb))
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        if ba >= bb {
            lean_bignat_to_nat(ba - bb)
        } else {
            lean_box(0)
        }
    }
}

#[inline]
pub unsafe fn lean_nat_mul(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        match va.checked_mul(vb) {
            Some(r) if r <= LEAN_MAX_SMALL_NAT => lean_box(r),
            _ => {
                let ba = BigUint::from(va);
                let bb = BigUint::from(vb);
                lean_bignat_to_nat(ba * bb)
            }
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        lean_bignat_to_nat(ba * bb)
    }
}

#[inline]
pub unsafe fn lean_nat_div(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        if vb == 0 {
            lean_box(0)
        } else {
            lean_box(va / vb)
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        if bb.is_zero() {
            lean_box(0)
        } else {
            lean_bignat_to_nat(ba / bb)
        }
    }
}

#[inline]
pub unsafe fn lean_nat_mod(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        if vb == 0 {
            lean_box(0)
        } else {
            lean_box(va % vb)
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        if bb.is_zero() {
            lean_box(0)
        } else {
            lean_bignat_to_nat(ba % bb)
        }
    }
}

#[inline]
pub unsafe fn lean_nat_eq(a: *mut LeanObject, b: *mut LeanObject) -> bool {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        a == b
    } else if lean_is_scalar(a) || lean_is_scalar(b) {
        // One scalar, one big => not equal
        false
    } else {
        lean_bignat_value(a) == lean_bignat_value(b)
    }
}

#[inline]
pub unsafe fn lean_nat_dec_eq(a: *mut LeanObject, b: *mut LeanObject) -> u8 {
    lean_nat_eq(a, b) as u8
}

#[inline]
pub unsafe fn lean_nat_dec_lt(a: *mut LeanObject, b: *mut LeanObject) -> u8 {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        (lean_unbox(a) < lean_unbox(b)) as u8
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        (ba < bb) as u8
    }
}

#[inline]
pub unsafe fn lean_nat_dec_le(a: *mut LeanObject, b: *mut LeanObject) -> u8 {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        (lean_unbox(a) <= lean_unbox(b)) as u8
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        (ba <= bb) as u8
    }
}

#[inline]
pub unsafe fn lean_nat_shiftr(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        if vb >= usize::BITS as usize {
            lean_box(0)
        } else {
            lean_box(va >> vb)
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        let vb = if lean_is_scalar(b) {
            lean_unbox(b)
        } else {
            // Shifting by a huge amount always yields 0
            if !lean_is_scalar(a) {
                crate::lean_dec(a);
            }
            crate::lean_dec(b);
            return lean_box(0);
        };
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        lean_bignat_to_nat(ba >> vb)
    }
}

/// Nat exponentiation: a^b
pub unsafe fn lean_nat_pow(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        if vb == 0 {
            return lean_box(1);
        }
        if va <= 1 {
            return lean_box(va);
        }
        match va.checked_pow(vb as u32) {
            Some(r) if r <= LEAN_MAX_SMALL_NAT => lean_box(r),
            _ => {
                let ba = BigUint::from(va);
                let bb = BigUint::from(vb);
                lean_bignat_to_nat(num_traits::pow(ba, bb.to_usize().unwrap_or(usize::MAX)))
            }
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        lean_bignat_to_nat(num_traits::pow(ba, bb.to_usize().unwrap_or(usize::MAX)))
    }
}

/// Nat predecessor: n - 1 (saturating at 0).
pub unsafe fn lean_nat_pred(n: *mut LeanObject) -> *mut LeanObject {
    lean_nat_sub(n, lean_box(1))
}

/// Boxed wrapper for lean_nat_dec_eq. Used when Nat.decEq is passed as a
/// first-class value (e.g., stored in a closure for typeclass dictionaries).
/// Takes owned arguments and returns a boxed Bool.
pub unsafe fn l_Nat_decEq___boxed(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    let r = lean_nat_dec_eq(a, b);
    crate::rc::lean_dec(a);
    crate::rc::lean_dec(b);
    lean_box(r as usize)
}

/// Nat land (bitwise AND).
pub unsafe fn lean_nat_land(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        lean_box(va & vb)
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        lean_bignat_to_nat(ba & bb)
    }
}

/// Nat lor (bitwise OR).
pub unsafe fn lean_nat_lor(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        lean_box(va | vb)
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        lean_bignat_to_nat(ba | bb)
    }
}

/// Nat lxor (bitwise XOR).
pub unsafe fn lean_nat_lxor(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) && lean_is_scalar(b) {
        let va = lean_unbox(a);
        let vb = lean_unbox(b);
        lean_box(va ^ vb)
    } else {
        let ba = lean_nat_to_biguint(a);
        let bb = lean_nat_to_biguint(b);
        if !lean_is_scalar(a) {
            crate::lean_dec(a);
        }
        if !lean_is_scalar(b) {
            crate::lean_dec(b);
        }
        lean_bignat_to_nat(ba ^ bb)
    }
}

/// Nat shiftl: a << b
pub unsafe fn lean_nat_shiftl(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    let ba = lean_nat_to_biguint(a);
    let vb = if lean_is_scalar(b) {
        lean_unbox(b)
    } else {
        // Big nat shift: try to extract as usize, otherwise cap
        let bb = lean_nat_to_biguint(b);
        crate::lean_dec(b);
        match bb.to_usize() {
            Some(shift) => {
                let result = ba << shift;
                if !lean_is_scalar(a) {
                    crate::lean_dec(a);
                }
                return lean_bignat_to_nat(result);
            }
            None => {
                // Exponent too large for memory — panic
                if !lean_is_scalar(a) {
                    crate::lean_dec(a);
                }
                panic!("lean_nat_shiftl: shift amount exceeds usize");
            }
        }
    };
    if !lean_is_scalar(a) {
        crate::lean_dec(a);
    }
    lean_bignat_to_nat(ba << vb)
}

/// Nat log2: floor(log2(n)), 0 for n=0.
pub unsafe fn lean_nat_log2(a: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) {
        let va = lean_unbox(a);
        if va == 0 {
            lean_box(0)
        } else {
            lean_box((usize::BITS - 1 - va.leading_zeros()) as usize)
        }
    } else {
        let ba = lean_nat_to_biguint(a);
        crate::lean_dec(a);
        let bits = ba.bits();
        if bits == 0 {
            lean_box(0)
        } else {
            lean_unsigned_to_nat((bits - 1) as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn add() {
        unsafe {
            let r = lean_nat_add(lean_box(3), lean_box(4));
            assert_eq!(lean_unbox(r), 7);
        }
    }

    #[test]
    fn sub() {
        unsafe {
            let r = lean_nat_sub(lean_box(10), lean_box(3));
            assert_eq!(lean_unbox(r), 7);
        }
    }

    #[test]
    fn sub_underflow() {
        unsafe {
            let r = lean_nat_sub(lean_box(2), lean_box(5));
            assert_eq!(lean_unbox(r), 0);
        }
    }

    #[test]
    fn mul() {
        unsafe {
            let r = lean_nat_mul(lean_box(6), lean_box(7));
            assert_eq!(lean_unbox(r), 42);
        }
    }

    #[test]
    fn div() {
        unsafe {
            let r = lean_nat_div(lean_box(10), lean_box(3));
            assert_eq!(lean_unbox(r), 3);
        }
    }

    #[test]
    fn div_by_zero() {
        unsafe {
            let r = lean_nat_div(lean_box(10), lean_box(0));
            assert_eq!(lean_unbox(r), 0);
        }
    }

    #[test]
    fn modulo() {
        unsafe {
            let r = lean_nat_mod(lean_box(10), lean_box(3));
            assert_eq!(lean_unbox(r), 1);
        }
    }

    #[test]
    fn mod_by_zero() {
        unsafe {
            let r = lean_nat_mod(lean_box(10), lean_box(0));
            assert_eq!(lean_unbox(r), 0);
        }
    }

    #[test]
    fn dec_eq() {
        unsafe {
            assert_eq!(lean_nat_dec_eq(lean_box(5), lean_box(5)), 1);
            assert_eq!(lean_nat_dec_eq(lean_box(5), lean_box(6)), 0);
        }
    }

    #[test]
    fn dec_lt() {
        unsafe {
            assert_eq!(lean_nat_dec_lt(lean_box(3), lean_box(5)), 1);
            assert_eq!(lean_nat_dec_lt(lean_box(5), lean_box(3)), 0);
            assert_eq!(lean_nat_dec_lt(lean_box(3), lean_box(3)), 0);
        }
    }

    #[test]
    fn dec_le() {
        unsafe {
            assert_eq!(lean_nat_dec_le(lean_box(3), lean_box(5)), 1);
            assert_eq!(lean_nat_dec_le(lean_box(3), lean_box(3)), 1);
            assert_eq!(lean_nat_dec_le(lean_box(5), lean_box(3)), 0);
        }
    }

    #[test]
    fn add_identity() {
        unsafe {
            for n in [0, 1, 100, 999_999] {
                let r1 = lean_nat_add(lean_box(n), lean_box(0));
                assert_eq!(lean_unbox(r1), n);
                let r2 = lean_nat_add(lean_box(0), lean_box(n));
                assert_eq!(lean_unbox(r2), n);
            }
        }
    }

    #[test]
    fn mul_identity_and_zero() {
        unsafe {
            for n in [0, 1, 42, 1_000_000] {
                let r1 = lean_nat_mul(lean_box(n), lean_box(1));
                assert_eq!(lean_unbox(r1), n);
                let r2 = lean_nat_mul(lean_box(n), lean_box(0));
                assert_eq!(lean_unbox(r2), 0);
            }
        }
    }

    #[test]
    fn chained_ops() {
        unsafe {
            let a = lean_nat_add(lean_box(5), lean_box(3));
            let b = lean_nat_mul(a, lean_box(2));
            let c = lean_nat_sub(b, lean_box(6));
            assert_eq!(lean_unbox(c), 10);
        }
    }

    #[test]
    fn div_mod_relationship() {
        unsafe {
            for (a, b) in [(17, 5), (100, 7), (255, 16), (1, 1), (0, 3)] {
                let q = lean_unbox(lean_nat_div(lean_box(a), lean_box(b)));
                let r = lean_unbox(lean_nat_mod(lean_box(a), lean_box(b)));
                assert_eq!(q * b + r, a, "failed for a={a}, b={b}");
            }
        }
    }

    #[test]
    fn sub_saturation_many() {
        unsafe {
            for (a, b) in [(0, 0), (0, 1), (0, 100), (5, 10), (1, usize::MAX >> 1)] {
                let r = lean_nat_sub(lean_box(a), lean_box(b));
                assert_eq!(lean_unbox(r), a.saturating_sub(b));
            }
        }
    }

    #[test]
    fn comparison_with_zero() {
        unsafe {
            assert_eq!(lean_nat_dec_eq(lean_box(0), lean_box(0)), 1);
            assert_eq!(lean_nat_dec_lt(lean_box(0), lean_box(0)), 0);
            assert_eq!(lean_nat_dec_le(lean_box(0), lean_box(0)), 1);
            assert_eq!(lean_nat_dec_lt(lean_box(0), lean_box(1)), 1);
            assert_eq!(lean_nat_dec_le(lean_box(0), lean_box(1)), 1);
        }
    }

    #[test]
    fn unsigned_to_nat_roundtrip() {
        for v in [0, 1, 255, 65535, 1_000_000] {
            let n = lean_unsigned_to_nat(v);
            assert_eq!(lean_unbox(n), v);
        }
    }

    #[test]
    fn large_scalar_arithmetic() {
        unsafe {
            let big = (1usize << 30) - 1;
            let r = lean_nat_add(lean_box(big), lean_box(1));
            assert_eq!(lean_unbox(r), big + 1);
        }
    }

    // Big nat tests

    #[test]
    fn add_overflow_to_bignat() {
        unsafe {
            let max = LEAN_MAX_SMALL_NAT;
            let a = lean_box(max);
            let b = lean_box(1);
            let r = lean_nat_add(a, b);
            // Result should be a big nat
            assert!(!lean_is_scalar(r));
            let val = lean_nat_to_biguint(r);
            assert_eq!(val, BigUint::from(max) + BigUint::from(1u64));
            crate::lean_dec(r);
        }
    }

    #[test]
    fn mul_overflow_to_bignat() {
        unsafe {
            let big = LEAN_MAX_SMALL_NAT;
            let a = lean_box(big);
            let b = lean_box(2);
            let r = lean_nat_mul(a, b);
            assert!(!lean_is_scalar(r));
            let val = lean_nat_to_biguint(r);
            assert_eq!(val, BigUint::from(big) * BigUint::from(2u64));
            crate::lean_dec(r);
        }
    }

    #[test]
    fn big_add_big() {
        unsafe {
            let a = lean_alloc_bignat(BigUint::from(u128::MAX));
            let b = lean_alloc_bignat(BigUint::from(1u64));
            let r = lean_nat_add(a, b);
            let val = lean_nat_to_biguint(r);
            assert_eq!(val, BigUint::from(u128::MAX) + BigUint::from(1u64));
            crate::lean_dec(r);
        }
    }

    #[test]
    fn big_sub_to_zero() {
        unsafe {
            let a = lean_alloc_bignat(BigUint::from(100u64));
            let b = lean_alloc_bignat(BigUint::from(200u64));
            let r = lean_nat_sub(a, b);
            assert!(lean_is_scalar(r));
            assert_eq!(lean_unbox(r), 0);
        }
    }

    #[test]
    fn big_sub_demotes_to_scalar() {
        unsafe {
            let a = lean_alloc_bignat(BigUint::from(u128::MAX));
            let b = lean_alloc_bignat(BigUint::from(u128::MAX) - BigUint::from(42u64));
            let r = lean_nat_sub(a, b);
            assert!(lean_is_scalar(r));
            assert_eq!(lean_unbox(r), 42);
        }
    }

    #[test]
    fn big_div_mod() {
        unsafe {
            let a = lean_alloc_bignat(BigUint::from(1000000000000u64));
            let b = lean_alloc_bignat(BigUint::from(7u64));
            let q = lean_nat_div(a, b);
            let a2 = lean_alloc_bignat(BigUint::from(1000000000000u64));
            let b2 = lean_alloc_bignat(BigUint::from(7u64));
            let r = lean_nat_mod(a2, b2);
            let qv = lean_nat_to_biguint(q);
            let rv = lean_nat_to_biguint(r);
            assert_eq!(
                qv * BigUint::from(7u64) + rv,
                BigUint::from(1000000000000u64)
            );
            crate::lean_dec(q);
            crate::lean_dec(r);
        }
    }

    #[test]
    fn big_eq() {
        unsafe {
            let a = lean_alloc_bignat(BigUint::from(u128::MAX));
            let b = lean_alloc_bignat(BigUint::from(u128::MAX));
            assert_eq!(lean_nat_dec_eq(a, b), 1);
            let c = lean_alloc_bignat(BigUint::from(u128::MAX) - BigUint::from(1u64));
            assert_eq!(lean_nat_dec_eq(a, c), 0);
            // scalar vs big: not equal
            assert_eq!(lean_nat_dec_eq(lean_box(42), a), 0);
            crate::lean_dec(a);
            crate::lean_dec(b);
            crate::lean_dec(c);
        }
    }

    #[test]
    fn big_lt_le() {
        unsafe {
            let a = lean_alloc_bignat(BigUint::from(u128::MAX));
            let b = lean_alloc_bignat(BigUint::from(u128::MAX) + BigUint::from(1u64));
            assert_eq!(lean_nat_dec_lt(a, b), 1);
            assert_eq!(lean_nat_dec_le(a, b), 1);
            assert_eq!(lean_nat_dec_lt(b, a), 0);
            // scalar < big
            assert_eq!(lean_nat_dec_lt(lean_box(0), a), 1);
            assert_eq!(lean_nat_dec_lt(a, lean_box(0)), 0);
            crate::lean_dec(a);
            crate::lean_dec(b);
        }
    }

    #[test]
    fn pow_small() {
        unsafe {
            let r = lean_nat_pow(lean_box(2), lean_box(10));
            assert!(lean_is_scalar(r));
            assert_eq!(lean_unbox(r), 1024);
        }
    }

    #[test]
    fn pow_zero_exp() {
        unsafe {
            let r = lean_nat_pow(lean_box(999), lean_box(0));
            assert_eq!(lean_unbox(r), 1);
        }
    }

    #[test]
    fn pow_overflow() {
        unsafe {
            let r = lean_nat_pow(lean_box(2), lean_box(64));
            assert!(!lean_is_scalar(r));
            let val = lean_nat_to_biguint(r);
            assert_eq!(val, BigUint::from(1u64) << 64);
            crate::lean_dec(r);
        }
    }

    #[test]
    fn pred() {
        unsafe {
            assert_eq!(lean_unbox(lean_nat_pred(lean_box(5))), 4);
            assert_eq!(lean_unbox(lean_nat_pred(lean_box(0))), 0);
            assert_eq!(lean_unbox(lean_nat_pred(lean_box(1))), 0);
        }
    }

    #[test]
    fn log2_basic() {
        unsafe {
            assert_eq!(lean_unbox(lean_nat_log2(lean_box(0))), 0);
            assert_eq!(lean_unbox(lean_nat_log2(lean_box(1))), 0);
            assert_eq!(lean_unbox(lean_nat_log2(lean_box(2))), 1);
            assert_eq!(lean_unbox(lean_nat_log2(lean_box(7))), 2);
            assert_eq!(lean_unbox(lean_nat_log2(lean_box(8))), 3);
            assert_eq!(lean_unbox(lean_nat_log2(lean_box(1024))), 10);
        }
    }
}
