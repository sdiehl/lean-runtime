//! Signed integer (Int) operations with big integer support.
//!
//! Int in Lean is a signed integer that can be arbitrarily large.
//! Small integers (fitting in i32 range) are stored as scalars via `lean_box`.
//! Large non-negative integers reuse the BigUint (tag 250) representation.
//! Large negative integers are stored as BigInt objects at tag 251.
//!
//! Scalar encoding for Int: `lean_box(n as u32 as usize)` where `n: i32`.
//! Decoding: `lean_unbox(o) as u32 as i32 as i64`.

use crate::bignat::*;
use crate::object::*;
use crate::r#box::*;
use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};
use std::mem;

/// Maximum value that fits in a scalar Int (i32::MAX on 64-bit systems).
pub const LEAN_MAX_SMALL_INT: i64 = i32::MAX as i64;

/// Minimum value that fits in a scalar Int (i32::MIN on 64-bit systems).
pub const LEAN_MIN_SMALL_INT: i64 = i32::MIN as i64;

/// Size of a big int object in bytes.
const LEAN_BIGINT_OBJECT_SIZE: usize = LEAN_OBJECT_SIZE + mem::size_of::<BigInt>();

// ---------------------------------------------------------------------------
// BigInt heap allocation (tag 251, for negative big integers)
// ---------------------------------------------------------------------------

/// Allocate a new big int object from a `BigInt`.
/// The returned object has rc=1 and tag=LEAN_BIGINT_TAG (251).
pub unsafe fn lean_alloc_bigint(val: BigInt) -> *mut LeanObject {
    let obj = lean_alloc_object(LEAN_BIGINT_OBJECT_SIZE);
    (*obj).rc = 1;
    (*obj).tag = LEAN_BIGINT_TAG;
    (*obj).other = 0;
    (*obj).cs_sz = 0;
    let data_ptr = (obj as *mut u8).add(LEAN_OBJECT_SIZE) as *mut BigInt;
    std::ptr::write(data_ptr, val);
    obj
}

/// Get a reference to the BigInt stored in a big int object (tag 251).
#[inline]
pub unsafe fn lean_bigint_value(o: *mut LeanObject) -> &'static BigInt {
    debug_assert_eq!((*o).tag, LEAN_BIGINT_TAG);
    let data_ptr = (o as *mut u8).add(LEAN_OBJECT_SIZE) as *const BigInt;
    &*data_ptr
}

/// Drop the BigInt inside a big int object (must be called before freeing).
pub unsafe fn lean_bigint_drop(o: *mut LeanObject) {
    debug_assert_eq!((*o).tag, LEAN_BIGINT_TAG);
    let data_ptr = (o as *mut u8).add(LEAN_OBJECT_SIZE) as *mut BigInt;
    std::ptr::drop_in_place(data_ptr);
}

/// Free a big int object: drop the BigInt, then free the memory.
pub unsafe fn lean_free_bigint(o: *mut LeanObject) {
    lean_bigint_drop(o);
    lean_free_object(o, LEAN_BIGINT_OBJECT_SIZE);
}

/// Size of a big int object (constant).
pub const fn lean_bigint_object_size() -> usize {
    LEAN_BIGINT_OBJECT_SIZE
}

// ---------------------------------------------------------------------------
// Scalar conversion helpers
// ---------------------------------------------------------------------------

/// Convert a scalar Int object to i64.
/// Uses the C runtime cast chain: unbox as usize, truncate to u32, sign-extend via i32 to i64.
#[inline(always)]
pub fn lean_scalar_to_int64(a: *mut LeanObject) -> i64 {
    debug_assert!(lean_is_scalar(a));
    lean_unbox(a) as u32 as i32 as i64
}

/// Convert an i64 to a Lean Int object.
/// If the value fits in i32 range, box as scalar; otherwise allocate a big int.
#[inline]
pub fn lean_int64_to_int(n: i64) -> *mut LeanObject {
    if (LEAN_MIN_SMALL_INT..=LEAN_MAX_SMALL_INT).contains(&n) {
        lean_box(n as u32 as usize)
    } else if n >= 0 {
        // Large non-negative: store as BigUint at tag 250
        unsafe { lean_alloc_bignat(BigUint::from(n as u64)) }
    } else {
        // Large negative: store as BigInt at tag 251
        unsafe { lean_alloc_bigint(BigInt::from(n)) }
    }
}

/// Convert an i32 to a Lean Int object.
#[inline]
pub fn lean_int_to_int(n: i32) -> *mut LeanObject {
    lean_int64_to_int(n as i64)
}

// ---------------------------------------------------------------------------
// Object to BigInt conversion
// ---------------------------------------------------------------------------

/// Convert any Lean Int/Nat object to a BigInt.
/// Handles scalars (interpret as signed via cast chain), BigUint (tag 250),
/// and BigInt (tag 251).
unsafe fn lean_obj_to_bigint(a: *mut LeanObject) -> BigInt {
    if lean_is_scalar(a) {
        BigInt::from(lean_scalar_to_int64(a))
    } else if (*a).tag == LEAN_MPZ_TAG {
        BigInt::from(lean_bignat_value(a).clone())
    } else {
        debug_assert_eq!((*a).tag, LEAN_BIGINT_TAG);
        lean_bigint_value(a).clone()
    }
}

/// Convert a BigInt result back to a Lean Int object.
/// If the value fits in i32 range, returns a scalar.
/// If non-negative and large, stores as BigUint (tag 250).
/// If negative and large, stores as BigInt (tag 251).
unsafe fn lean_bigint_to_int(val: BigInt) -> *mut LeanObject {
    if let Some(n) = val.to_i64() {
        if (LEAN_MIN_SMALL_INT..=LEAN_MAX_SMALL_INT).contains(&n) {
            return lean_box(n as u32 as usize);
        }
    }
    if val.sign() != Sign::Minus {
        // Non-negative large value: store as BigUint
        let (_, digits) = val.to_u32_digits();
        let biguint = BigUint::new(digits);
        lean_alloc_bignat(biguint)
    } else {
        // Negative large value: store as BigInt
        lean_alloc_bigint(val)
    }
}

/// Decrement the reference count of a non-scalar Int object.
#[inline]
unsafe fn lean_dec_non_scalar(a: *mut LeanObject) {
    if !lean_is_scalar(a) {
        crate::lean_dec(a);
    }
}

// ---------------------------------------------------------------------------
// Nat to Int conversion
// ---------------------------------------------------------------------------

/// Convert a Nat to an Int. If the Nat is a small scalar, return as-is.
/// If the Nat is a big nat, return as-is (it is already a valid non-negative Int).
#[inline]
pub unsafe fn lean_nat_to_int(a: *mut LeanObject) -> *mut LeanObject {
    // All Nat representations (scalar or BigUint tag 250) are valid non-negative Ints.
    // However, if a scalar Nat value exceeds LEAN_MAX_SMALL_INT (i32::MAX = 2147483647),
    // it would be misinterpreted as negative by the Int scalar cast chain.
    // In that case we need to promote to a BigUint.
    if lean_is_scalar(a) {
        let v = lean_unbox(a);
        if v <= LEAN_MAX_SMALL_INT as usize {
            a
        } else {
            lean_alloc_bignat(BigUint::from(v))
        }
    } else {
        // Big nat object (tag 250): already valid as a non-negative Int.
        // We take ownership, so just pass through.
        a
    }
}

/// Compute `Int.negSucc n = -(n + 1)` from a Nat `n`.
pub unsafe fn lean_int_neg_succ_of_nat(a: *mut LeanObject) -> *mut LeanObject {
    // Compute succ = a + 1
    let succ = lean_nat_succ(a);
    // Convert succ to Int
    let int_val = lean_nat_to_int(succ);
    // Negate
    lean_int_neg(int_val)
}

// ---------------------------------------------------------------------------
// Nat successor (used by negSucc)
// ---------------------------------------------------------------------------

/// Add 1 to a Nat object. Returns a new owned object.
pub fn lean_nat_succ(a: *mut LeanObject) -> *mut LeanObject {
    unsafe { crate::nat::lean_nat_add(a, lean_box(1)) }
}

// ---------------------------------------------------------------------------
// Negation
// ---------------------------------------------------------------------------

/// Negate an Int: return `-a`.
pub unsafe fn lean_int_neg(a: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a) {
        let v = lean_scalar_to_int64(a);
        lean_int64_to_int(-v)
    } else {
        let big = lean_obj_to_bigint(a);
        lean_dec_non_scalar(a);
        lean_bigint_to_int(-big)
    }
}

// ---------------------------------------------------------------------------
// Arithmetic operations
// ---------------------------------------------------------------------------

/// Int addition: `a1 + a2`.
pub unsafe fn lean_int_add(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let v1 = lean_scalar_to_int64(a1);
        let v2 = lean_scalar_to_int64(a2);
        lean_int64_to_int(v1 + v2)
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        lean_dec_non_scalar(a1);
        lean_dec_non_scalar(a2);
        lean_bigint_to_int(b1 + b2)
    }
}

/// Int subtraction: `a1 - a2`.
pub unsafe fn lean_int_sub(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let v1 = lean_scalar_to_int64(a1);
        let v2 = lean_scalar_to_int64(a2);
        lean_int64_to_int(v1 - v2)
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        lean_dec_non_scalar(a1);
        lean_dec_non_scalar(a2);
        lean_bigint_to_int(b1 - b2)
    }
}

/// Int multiplication: `a1 * a2`.
pub unsafe fn lean_int_mul(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let v1 = lean_scalar_to_int64(a1);
        let v2 = lean_scalar_to_int64(a2);
        // i32 * i32 can overflow i64 range? No: i32::MAX^2 = ~4.6e18 < i64::MAX = ~9.2e18.
        // But it can overflow i32 range, so use i64 multiply and convert.
        lean_int64_to_int(v1 * v2)
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        lean_dec_non_scalar(a1);
        lean_dec_non_scalar(a2);
        lean_bigint_to_int(b1 * b2)
    }
}

/// Int truncated division: `a1 / a2`, truncated toward zero. Returns 0 if `a2 == 0`.
pub unsafe fn lean_int_div(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let v1 = lean_scalar_to_int64(a1);
        let v2 = lean_scalar_to_int64(a2);
        if v2 == 0 {
            lean_box(0u32 as usize)
        } else {
            lean_int64_to_int(v1 / v2)
        }
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        lean_dec_non_scalar(a1);
        lean_dec_non_scalar(a2);
        if b2.is_zero() {
            lean_box(0u32 as usize)
        } else {
            lean_bigint_to_int(b1 / b2)
        }
    }
}

/// Int truncated division (exact). Same as lean_int_div.
pub unsafe fn lean_int_div_exact(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    lean_int_div(a1, a2)
}

/// Int truncated modulus (C-style `%`): sign matches dividend.
/// Returns `a1` if `a2 == 0`.
pub unsafe fn lean_int_mod(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let v1 = lean_scalar_to_int64(a1);
        let v2 = lean_scalar_to_int64(a2);
        if v2 == 0 {
            a1
        } else {
            lean_int64_to_int(v1 % v2)
        }
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        if b2.is_zero() {
            lean_dec_non_scalar(a2);
            // Return a1 as-is (take ownership)
            if lean_is_scalar(a1) {
                a1
            } else {
                // Need to return an owned copy; a1 is consumed so return it directly.
                a1
            }
        } else {
            lean_dec_non_scalar(a1);
            lean_dec_non_scalar(a2);
            lean_bigint_to_int(b1 % b2)
        }
    }
}

/// Int Euclidean division (Daan Leijen's algorithm).
/// `ediv(n, d)`:
///   - if d == 0, return 0
///   - q = trunc(n / d), r = n % d
///   - if r < 0: q = (d > 0) ? q - 1 : q + 1
pub unsafe fn lean_int_ediv(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let n = lean_scalar_to_int64(a1);
        let d = lean_scalar_to_int64(a2);
        if d == 0 {
            return lean_box(0u32 as usize);
        }
        let q = n / d;
        let r = n % d;
        if r < 0 {
            lean_int64_to_int(if d > 0 { q - 1 } else { q + 1 })
        } else {
            lean_int64_to_int(q)
        }
    } else {
        let n = lean_obj_to_bigint(a1);
        let d = lean_obj_to_bigint(a2);
        lean_dec_non_scalar(a1);
        lean_dec_non_scalar(a2);
        if d.is_zero() {
            return lean_box(0u32 as usize);
        }
        let (q, r) = n.div_rem(&d);
        if r.is_negative() {
            lean_bigint_to_int(if d.is_positive() { q - 1 } else { q + 1 })
        } else {
            lean_bigint_to_int(q)
        }
    }
}

/// Int Euclidean modulus. Result is always >= 0.
/// `emod(n, d)`:
///   - if d == 0, return n
///   - r = n % d
///   - if r < 0: r = (d > 0) ? r + d : r - d
pub unsafe fn lean_int_emod(a1: *mut LeanObject, a2: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        let n = lean_scalar_to_int64(a1);
        let d = lean_scalar_to_int64(a2);
        if d == 0 {
            return a1;
        }
        let r = n % d;
        if r < 0 {
            lean_int64_to_int(if d > 0 { r + d } else { r - d })
        } else {
            lean_int64_to_int(r)
        }
    } else {
        let n = lean_obj_to_bigint(a1);
        let d = lean_obj_to_bigint(a2);
        if d.is_zero() {
            lean_dec_non_scalar(a2);
            return a1;
        }
        lean_dec_non_scalar(a1);
        lean_dec_non_scalar(a2);
        let r = &n % &d;
        if r.is_negative() {
            lean_bigint_to_int(if d.is_positive() { r + d } else { r - d })
        } else {
            lean_bigint_to_int(r)
        }
    }
}

// ---------------------------------------------------------------------------
// Comparison operations
// ---------------------------------------------------------------------------

/// Int equality: `a1 == a2`.
pub unsafe fn lean_int_eq(a1: *mut LeanObject, a2: *mut LeanObject) -> bool {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        a1 == a2
    } else if lean_is_scalar(a1) || lean_is_scalar(a2) {
        // One scalar, one big: if both represent the same value it would fit in scalar.
        // So they can't be equal.
        false
    } else if (*a1).tag == (*a2).tag {
        if (*a1).tag == LEAN_MPZ_TAG {
            lean_bignat_value(a1) == lean_bignat_value(a2)
        } else {
            lean_bigint_value(a1) == lean_bigint_value(a2)
        }
    } else {
        // Different tags (one is BigUint, one is BigInt) means different signs.
        false
    }
}

/// Int inequality: `a1 != a2`.
pub unsafe fn lean_int_ne(a1: *mut LeanObject, a2: *mut LeanObject) -> bool {
    !lean_int_eq(a1, a2)
}

/// Int less-than-or-equal: `a1 <= a2`.
pub unsafe fn lean_int_le(a1: *mut LeanObject, a2: *mut LeanObject) -> bool {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        lean_scalar_to_int64(a1) <= lean_scalar_to_int64(a2)
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        b1 <= b2
    }
}

/// Int less-than: `a1 < a2`.
pub unsafe fn lean_int_lt(a1: *mut LeanObject, a2: *mut LeanObject) -> bool {
    if lean_is_scalar(a1) && lean_is_scalar(a2) {
        lean_scalar_to_int64(a1) < lean_scalar_to_int64(a2)
    } else {
        let b1 = lean_obj_to_bigint(a1);
        let b2 = lean_obj_to_bigint(a2);
        b1 < b2
    }
}

/// Decidable Int equality: returns 1 if equal, 0 otherwise.
pub unsafe fn lean_int_dec_eq(a1: *mut LeanObject, a2: *mut LeanObject) -> u8 {
    lean_int_eq(a1, a2) as u8
}

/// Decidable Int less-than-or-equal: returns 1 if a1 <= a2, 0 otherwise.
pub unsafe fn lean_int_dec_le(a1: *mut LeanObject, a2: *mut LeanObject) -> u8 {
    lean_int_le(a1, a2) as u8
}

/// Decidable Int less-than: returns 1 if a1 < a2, 0 otherwise.
pub unsafe fn lean_int_dec_lt(a1: *mut LeanObject, a2: *mut LeanObject) -> u8 {
    lean_int_lt(a1, a2) as u8
}

/// Decidable Int non-negative: returns 1 if a >= 0, 0 otherwise.
pub unsafe fn lean_int_dec_nonneg(a: *mut LeanObject) -> u8 {
    if lean_is_scalar(a) {
        (lean_scalar_to_int64(a) >= 0) as u8
    } else if (*a).tag == LEAN_MPZ_TAG {
        // BigUint is always non-negative
        1
    } else {
        // BigInt (tag 251): check sign
        (!lean_bigint_value(a).is_negative()) as u8
    }
}

// ---------------------------------------------------------------------------
// Conversion operations
// ---------------------------------------------------------------------------

/// Convert a non-negative Int to a Nat. Asserts that `a >= 0`.
/// If scalar, return as-is (the bit pattern is the same for non-negative values).
/// If big (tag 250 BigUint), return as-is.
/// If big (tag 251 BigInt), convert to BigUint (should be non-negative).
pub unsafe fn lean_int_to_nat(a: *mut LeanObject) -> *mut LeanObject {
    if lean_int_dec_nonneg(a) != 1 {
        panic!("lean_int_to_nat: value is negative");
    }
    if lean_is_scalar(a) {
        // For non-negative scalar Int, the value is already valid as Nat.
        a
    } else if (*a).tag == LEAN_MPZ_TAG {
        // Already a BigUint, valid as Nat.
        a
    } else {
        // BigInt (tag 251), must be non-negative.
        let val = lean_bigint_value(a).clone();
        crate::lean_dec(a);
        let (_, digits) = val.to_u32_digits();
        lean_alloc_bignat(BigUint::new(digits))
    }
}

/// Absolute value of an Int, returned as a Nat.
/// `nat_abs(i)`:
///   - if i >= 0: inc ref and convert to nat
///   - if i < 0: negate then convert to nat
pub unsafe fn lean_nat_abs(i: *mut LeanObject) -> *mut LeanObject {
    if lean_is_scalar(i) {
        let v = lean_scalar_to_int64(i);
        if v >= 0 {
            // Already non-negative, return as nat
            // The scalar encoding for non-negative i32 is the same as for nat.
            i
        } else {
            // Negate and convert: -v is positive, fits in i64.
            let abs_val = (-v) as u64;
            if abs_val <= LEAN_MAX_SMALL_NAT as u64 {
                lean_box(abs_val as usize)
            } else {
                lean_alloc_bignat(BigUint::from(abs_val))
            }
        }
    } else {
        let big = lean_obj_to_bigint(i);
        crate::lean_dec(i);
        let abs_val = big.magnitude().clone();
        lean_bignat_to_nat(abs_val)
    }
}

/// Parse a string into an Int.
pub fn lean_cstr_to_int(s: &str) -> *mut LeanObject {
    match s.parse::<BigInt>() {
        Ok(val) => unsafe { lean_bigint_to_int(val) },
        Err(_) => lean_box(0u32 as usize),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: create a scalar Int from an i32 value.
    fn mk_int(n: i32) -> *mut LeanObject {
        lean_int_to_int(n)
    }

    // Helper: extract i64 from a Lean Int object.
    unsafe fn int_to_i64(o: *mut LeanObject) -> i64 {
        if lean_is_scalar(o) {
            lean_scalar_to_int64(o)
        } else {
            let big = lean_obj_to_bigint(o);
            big.to_i64().expect("value should fit in i64 for test")
        }
    }

    // Helper: extract BigInt from a Lean Int object.
    unsafe fn int_to_bigint(o: *mut LeanObject) -> BigInt {
        lean_obj_to_bigint(o)
    }

    // Helper: free a non-scalar Int object.
    unsafe fn dec(o: *mut LeanObject) {
        crate::lean_dec(o);
    }

    // -----------------------------------------------------------------------
    // Scalar conversion tests
    // -----------------------------------------------------------------------

    #[test]
    fn scalar_roundtrip_positive() {
        let obj = mk_int(42);
        assert!(lean_is_scalar(obj));
        assert_eq!(lean_scalar_to_int64(obj), 42);
    }

    #[test]
    fn scalar_roundtrip_negative() {
        let obj = mk_int(-42);
        assert!(lean_is_scalar(obj));
        assert_eq!(lean_scalar_to_int64(obj), -42);
    }

    #[test]
    fn scalar_roundtrip_zero() {
        let obj = mk_int(0);
        assert!(lean_is_scalar(obj));
        assert_eq!(lean_scalar_to_int64(obj), 0);
    }

    #[test]
    fn scalar_roundtrip_min() {
        let obj = mk_int(i32::MIN);
        assert!(lean_is_scalar(obj));
        assert_eq!(lean_scalar_to_int64(obj), i32::MIN as i64);
    }

    #[test]
    fn scalar_roundtrip_max() {
        let obj = mk_int(i32::MAX);
        assert!(lean_is_scalar(obj));
        assert_eq!(lean_scalar_to_int64(obj), i32::MAX as i64);
    }

    #[test]
    fn int64_to_int_large_positive() {
        let obj = lean_int64_to_int(i64::MAX);
        assert!(!lean_is_scalar(obj));
        unsafe {
            let val = int_to_bigint(obj);
            assert_eq!(val, BigInt::from(i64::MAX));
            dec(obj);
        }
    }

    #[test]
    fn int64_to_int_large_negative() {
        let obj = lean_int64_to_int(i64::MIN);
        assert!(!lean_is_scalar(obj));
        unsafe {
            let val = int_to_bigint(obj);
            assert_eq!(val, BigInt::from(i64::MIN));
            dec(obj);
        }
    }

    // -----------------------------------------------------------------------
    // Negation tests
    // -----------------------------------------------------------------------

    #[test]
    fn neg_positive() {
        unsafe {
            let a = mk_int(10);
            let r = lean_int_neg(a);
            assert_eq!(int_to_i64(r), -10);
        }
    }

    #[test]
    fn neg_negative() {
        unsafe {
            let a = mk_int(-7);
            let r = lean_int_neg(a);
            assert_eq!(int_to_i64(r), 7);
        }
    }

    #[test]
    fn neg_zero() {
        unsafe {
            let a = mk_int(0);
            let r = lean_int_neg(a);
            assert_eq!(int_to_i64(r), 0);
        }
    }

    #[test]
    fn neg_min_int() {
        unsafe {
            // Negating i32::MIN gives i32::MAX + 1 which still fits in i64 but not i32.
            let a = mk_int(i32::MIN);
            let r = lean_int_neg(a);
            assert_eq!(int_to_i64(r), -(i32::MIN as i64));
        }
    }

    // -----------------------------------------------------------------------
    // Addition tests
    // -----------------------------------------------------------------------

    #[test]
    fn add_positive() {
        unsafe {
            let r = lean_int_add(mk_int(3), mk_int(4));
            assert_eq!(int_to_i64(r), 7);
        }
    }

    #[test]
    fn add_negative() {
        unsafe {
            let r = lean_int_add(mk_int(-3), mk_int(-4));
            assert_eq!(int_to_i64(r), -7);
        }
    }

    #[test]
    fn add_mixed() {
        unsafe {
            let r = lean_int_add(mk_int(-10), mk_int(3));
            assert_eq!(int_to_i64(r), -7);
        }
    }

    #[test]
    fn add_to_zero() {
        unsafe {
            let r = lean_int_add(mk_int(5), mk_int(-5));
            assert_eq!(int_to_i64(r), 0);
        }
    }

    #[test]
    fn add_overflow_to_big() {
        unsafe {
            // i32::MAX + 1 overflows scalar range
            let r = lean_int_add(mk_int(i32::MAX), mk_int(1));
            assert!(!lean_is_scalar(r));
            assert_eq!(int_to_i64(r), i32::MAX as i64 + 1);
            dec(r);
        }
    }

    #[test]
    fn add_underflow_to_big() {
        unsafe {
            // i32::MIN + (-1) underflows scalar range
            let r = lean_int_add(mk_int(i32::MIN), mk_int(-1));
            assert!(!lean_is_scalar(r));
            assert_eq!(int_to_i64(r), i32::MIN as i64 - 1);
            dec(r);
        }
    }

    // -----------------------------------------------------------------------
    // Subtraction tests
    // -----------------------------------------------------------------------

    #[test]
    fn sub_positive() {
        unsafe {
            let r = lean_int_sub(mk_int(10), mk_int(3));
            assert_eq!(int_to_i64(r), 7);
        }
    }

    #[test]
    fn sub_to_negative() {
        unsafe {
            let r = lean_int_sub(mk_int(3), mk_int(10));
            assert_eq!(int_to_i64(r), -7);
        }
    }

    #[test]
    fn sub_double_negative() {
        unsafe {
            let r = lean_int_sub(mk_int(-3), mk_int(-10));
            assert_eq!(int_to_i64(r), 7);
        }
    }

    // -----------------------------------------------------------------------
    // Multiplication tests
    // -----------------------------------------------------------------------

    #[test]
    fn mul_positive() {
        unsafe {
            let r = lean_int_mul(mk_int(6), mk_int(7));
            assert_eq!(int_to_i64(r), 42);
        }
    }

    #[test]
    fn mul_negative() {
        unsafe {
            let r = lean_int_mul(mk_int(-6), mk_int(7));
            assert_eq!(int_to_i64(r), -42);
        }
    }

    #[test]
    fn mul_both_negative() {
        unsafe {
            let r = lean_int_mul(mk_int(-6), mk_int(-7));
            assert_eq!(int_to_i64(r), 42);
        }
    }

    #[test]
    fn mul_zero() {
        unsafe {
            let r = lean_int_mul(mk_int(999), mk_int(0));
            assert_eq!(int_to_i64(r), 0);
        }
    }

    #[test]
    fn mul_overflow() {
        unsafe {
            let r = lean_int_mul(mk_int(i32::MAX), mk_int(2));
            assert!(!lean_is_scalar(r));
            assert_eq!(int_to_i64(r), i32::MAX as i64 * 2);
            dec(r);
        }
    }

    // -----------------------------------------------------------------------
    // Division tests
    // -----------------------------------------------------------------------

    #[test]
    fn div_positive() {
        unsafe {
            let r = lean_int_div(mk_int(10), mk_int(3));
            assert_eq!(int_to_i64(r), 3);
        }
    }

    #[test]
    fn div_negative_dividend() {
        unsafe {
            // Truncated toward zero: -10 / 3 = -3
            let r = lean_int_div(mk_int(-10), mk_int(3));
            assert_eq!(int_to_i64(r), -3);
        }
    }

    #[test]
    fn div_negative_divisor() {
        unsafe {
            // 10 / (-3) = -3
            let r = lean_int_div(mk_int(10), mk_int(-3));
            assert_eq!(int_to_i64(r), -3);
        }
    }

    #[test]
    fn div_both_negative() {
        unsafe {
            // -10 / (-3) = 3
            let r = lean_int_div(mk_int(-10), mk_int(-3));
            assert_eq!(int_to_i64(r), 3);
        }
    }

    #[test]
    fn div_by_zero() {
        unsafe {
            let r = lean_int_div(mk_int(10), mk_int(0));
            assert_eq!(int_to_i64(r), 0);
        }
    }

    // -----------------------------------------------------------------------
    // Modulus tests
    // -----------------------------------------------------------------------

    #[test]
    fn mod_positive() {
        unsafe {
            let r = lean_int_mod(mk_int(10), mk_int(3));
            assert_eq!(int_to_i64(r), 1);
        }
    }

    #[test]
    fn mod_negative_dividend() {
        unsafe {
            // C-style: -10 % 3 = -1
            let r = lean_int_mod(mk_int(-10), mk_int(3));
            assert_eq!(int_to_i64(r), -1);
        }
    }

    #[test]
    fn mod_negative_divisor() {
        unsafe {
            // 10 % (-3) = 1
            let r = lean_int_mod(mk_int(10), mk_int(-3));
            assert_eq!(int_to_i64(r), 1);
        }
    }

    #[test]
    fn mod_by_zero() {
        unsafe {
            // Returns a1 when a2 == 0
            let a1 = mk_int(7);
            let r = lean_int_mod(a1, mk_int(0));
            assert_eq!(int_to_i64(r), 7);
        }
    }

    // -----------------------------------------------------------------------
    // Euclidean division tests
    // -----------------------------------------------------------------------

    #[test]
    fn ediv_positive() {
        unsafe {
            let r = lean_int_ediv(mk_int(10), mk_int(3));
            assert_eq!(int_to_i64(r), 3);
        }
    }

    #[test]
    fn ediv_negative_dividend_positive_divisor() {
        unsafe {
            // ediv(-10, 3): trunc = -3, rem = -1, rem < 0 and d > 0 => q = -3 - 1 = -4
            let r = lean_int_ediv(mk_int(-10), mk_int(3));
            assert_eq!(int_to_i64(r), -4);
        }
    }

    #[test]
    fn ediv_negative_dividend_negative_divisor() {
        unsafe {
            // ediv(-10, -3): trunc = 3, rem = -1, rem < 0 and d < 0 => q = 3 + 1 = 4
            let r = lean_int_ediv(mk_int(-10), mk_int(-3));
            assert_eq!(int_to_i64(r), 4);
        }
    }

    #[test]
    fn ediv_by_zero() {
        unsafe {
            let r = lean_int_ediv(mk_int(10), mk_int(0));
            assert_eq!(int_to_i64(r), 0);
        }
    }

    #[test]
    fn ediv_exact() {
        unsafe {
            // -6 / 3 = -2, rem = 0, no adjustment needed
            let r = lean_int_ediv(mk_int(-6), mk_int(3));
            assert_eq!(int_to_i64(r), -2);
        }
    }

    // -----------------------------------------------------------------------
    // Euclidean modulus tests
    // -----------------------------------------------------------------------

    #[test]
    fn emod_positive() {
        unsafe {
            let r = lean_int_emod(mk_int(10), mk_int(3));
            assert_eq!(int_to_i64(r), 1);
        }
    }

    #[test]
    fn emod_negative_dividend_positive_divisor() {
        unsafe {
            // emod(-10, 3): r = -1, r < 0 and d > 0 => r + d = -1 + 3 = 2
            let r = lean_int_emod(mk_int(-10), mk_int(3));
            assert_eq!(int_to_i64(r), 2);
        }
    }

    #[test]
    fn emod_negative_dividend_negative_divisor() {
        unsafe {
            // emod(-10, -3): r = -1, r < 0 and d < 0 => r - d = -1 - (-3) = 2
            let r = lean_int_emod(mk_int(-10), mk_int(-3));
            assert_eq!(int_to_i64(r), 2);
        }
    }

    #[test]
    fn emod_by_zero() {
        unsafe {
            let a1 = mk_int(7);
            let r = lean_int_emod(a1, mk_int(0));
            assert_eq!(int_to_i64(r), 7);
        }
    }

    #[test]
    fn emod_always_nonneg() {
        unsafe {
            for n in [-10, -7, -3, -1, 0, 1, 3, 7, 10] {
                for d in [-5, -3, -1, 1, 3, 5] {
                    let r = lean_int_emod(mk_int(n), mk_int(d));
                    let val = int_to_i64(r);
                    assert!(val >= 0, "emod({n}, {d}) = {val}, expected >= 0");
                }
            }
        }
    }

    #[test]
    fn ediv_emod_relationship() {
        unsafe {
            // For all n, d: n == ediv(n, d) * d + emod(n, d)
            for n in [-10, -7, -3, -1, 0, 1, 3, 7, 10] {
                for d in [-5, -3, -1, 1, 3, 5] {
                    let q = int_to_i64(lean_int_ediv(mk_int(n), mk_int(d)));
                    let r = int_to_i64(lean_int_emod(mk_int(n), mk_int(d)));
                    assert_eq!(
                        q * (d as i64) + r,
                        n as i64,
                        "ediv/emod relationship failed for n={n}, d={d}: q={q}, r={r}"
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Comparison tests
    // -----------------------------------------------------------------------

    #[test]
    fn eq_positive() {
        unsafe {
            assert!(lean_int_eq(mk_int(5), mk_int(5)));
            assert!(!lean_int_eq(mk_int(5), mk_int(6)));
        }
    }

    #[test]
    fn eq_negative() {
        unsafe {
            assert!(lean_int_eq(mk_int(-5), mk_int(-5)));
            assert!(!lean_int_eq(mk_int(-5), mk_int(5)));
        }
    }

    #[test]
    fn ne() {
        unsafe {
            assert!(lean_int_ne(mk_int(3), mk_int(4)));
            assert!(!lean_int_ne(mk_int(3), mk_int(3)));
        }
    }

    #[test]
    fn le() {
        unsafe {
            assert!(lean_int_le(mk_int(-5), mk_int(3)));
            assert!(lean_int_le(mk_int(3), mk_int(3)));
            assert!(!lean_int_le(mk_int(5), mk_int(3)));
        }
    }

    #[test]
    fn lt() {
        unsafe {
            assert!(lean_int_lt(mk_int(-5), mk_int(3)));
            assert!(!lean_int_lt(mk_int(3), mk_int(3)));
            assert!(!lean_int_lt(mk_int(5), mk_int(3)));
        }
    }

    #[test]
    fn dec_eq() {
        unsafe {
            assert_eq!(lean_int_dec_eq(mk_int(5), mk_int(5)), 1);
            assert_eq!(lean_int_dec_eq(mk_int(5), mk_int(-5)), 0);
        }
    }

    #[test]
    fn dec_le() {
        unsafe {
            assert_eq!(lean_int_dec_le(mk_int(-1), mk_int(0)), 1);
            assert_eq!(lean_int_dec_le(mk_int(0), mk_int(0)), 1);
            assert_eq!(lean_int_dec_le(mk_int(1), mk_int(0)), 0);
        }
    }

    #[test]
    fn dec_lt() {
        unsafe {
            assert_eq!(lean_int_dec_lt(mk_int(-1), mk_int(0)), 1);
            assert_eq!(lean_int_dec_lt(mk_int(0), mk_int(0)), 0);
            assert_eq!(lean_int_dec_lt(mk_int(1), mk_int(0)), 0);
        }
    }

    #[test]
    fn dec_nonneg() {
        unsafe {
            assert_eq!(lean_int_dec_nonneg(mk_int(0)), 1);
            assert_eq!(lean_int_dec_nonneg(mk_int(5)), 1);
            assert_eq!(lean_int_dec_nonneg(mk_int(-1)), 0);
            assert_eq!(lean_int_dec_nonneg(mk_int(i32::MIN)), 0);
            assert_eq!(lean_int_dec_nonneg(mk_int(i32::MAX)), 1);
        }
    }

    // -----------------------------------------------------------------------
    // Conversion tests
    // -----------------------------------------------------------------------

    #[test]
    fn nat_to_int_small() {
        unsafe {
            let nat = lean_box(42);
            let int = lean_nat_to_int(nat);
            assert!(lean_is_scalar(int));
            assert_eq!(lean_scalar_to_int64(int), 42);
        }
    }

    #[test]
    fn nat_to_int_large_scalar() {
        unsafe {
            // Nat value larger than i32::MAX but still a scalar (fits in usize >> 1).
            // This should promote to BigUint since it doesn't fit in scalar Int range.
            let big_nat = lean_box((LEAN_MAX_SMALL_INT as usize) + 1);
            let int = lean_nat_to_int(big_nat);
            assert!(!lean_is_scalar(int));
            let val = int_to_bigint(int);
            assert_eq!(val, BigInt::from(LEAN_MAX_SMALL_INT + 1));
            dec(int);
        }
    }

    #[test]
    fn int_to_nat_small() {
        unsafe {
            let int = mk_int(42);
            let nat = lean_int_to_nat(int);
            assert!(lean_is_scalar(nat));
            assert_eq!(lean_unbox(nat), 42);
        }
    }

    #[test]
    fn nat_abs_positive() {
        unsafe {
            let int = mk_int(42);
            let nat = lean_nat_abs(int);
            assert!(lean_is_scalar(nat));
            assert_eq!(lean_unbox(nat), 42);
        }
    }

    #[test]
    fn nat_abs_negative() {
        unsafe {
            let int = mk_int(-42);
            let nat = lean_nat_abs(int);
            assert!(lean_is_scalar(nat));
            assert_eq!(lean_unbox(nat), 42);
        }
    }

    #[test]
    fn nat_abs_zero() {
        unsafe {
            let int = mk_int(0);
            let nat = lean_nat_abs(int);
            assert!(lean_is_scalar(nat));
            assert_eq!(lean_unbox(nat), 0);
        }
    }

    #[test]
    fn nat_abs_min_int() {
        unsafe {
            // abs(i32::MIN) = 2147483648, which fits in a scalar nat
            let int = mk_int(i32::MIN);
            let nat = lean_nat_abs(int);
            assert!(lean_is_scalar(nat));
            assert_eq!(lean_unbox(nat), 2147483648);
        }
    }

    #[test]
    fn neg_succ_of_nat() {
        unsafe {
            // negSucc(0) = -(0+1) = -1
            let r = lean_int_neg_succ_of_nat(lean_box(0));
            assert_eq!(int_to_i64(r), -1);
        }
    }

    #[test]
    fn neg_succ_of_nat_5() {
        unsafe {
            // negSucc(5) = -(5+1) = -6
            let r = lean_int_neg_succ_of_nat(lean_box(5));
            assert_eq!(int_to_i64(r), -6);
        }
    }

    // -----------------------------------------------------------------------
    // String parsing tests
    // -----------------------------------------------------------------------

    #[test]
    fn cstr_to_int_positive() {
        let r = lean_cstr_to_int("42");
        assert!(lean_is_scalar(r));
        assert_eq!(lean_scalar_to_int64(r), 42);
    }

    #[test]
    fn cstr_to_int_negative() {
        let r = lean_cstr_to_int("-42");
        assert!(lean_is_scalar(r));
        assert_eq!(lean_scalar_to_int64(r), -42);
    }

    #[test]
    fn cstr_to_int_zero() {
        let r = lean_cstr_to_int("0");
        assert!(lean_is_scalar(r));
        assert_eq!(lean_scalar_to_int64(r), 0);
    }

    #[test]
    fn cstr_to_int_large() {
        unsafe {
            let r = lean_cstr_to_int("99999999999999999999");
            assert!(!lean_is_scalar(r));
            let val = int_to_bigint(r);
            assert_eq!(val, "99999999999999999999".parse::<BigInt>().unwrap());
            dec(r);
        }
    }

    #[test]
    fn cstr_to_int_large_negative() {
        unsafe {
            let r = lean_cstr_to_int("-99999999999999999999");
            assert!(!lean_is_scalar(r));
            let val = int_to_bigint(r);
            assert_eq!(val, "-99999999999999999999".parse::<BigInt>().unwrap());
            dec(r);
        }
    }

    #[test]
    fn cstr_to_int_invalid() {
        let r = lean_cstr_to_int("hello");
        assert!(lean_is_scalar(r));
        assert_eq!(lean_scalar_to_int64(r), 0);
    }

    // -----------------------------------------------------------------------
    // Big integer arithmetic tests
    // -----------------------------------------------------------------------

    #[test]
    fn big_add() {
        unsafe {
            // Create a big positive int and add to it
            let a = lean_int64_to_int(i64::MAX);
            let b = mk_int(1);
            let r = lean_int_add(a, b);
            let val = int_to_bigint(r);
            assert_eq!(val, BigInt::from(i64::MAX) + BigInt::from(1));
            dec(r);
        }
    }

    #[test]
    fn big_sub() {
        unsafe {
            let a = lean_int64_to_int(i64::MIN);
            let b = mk_int(1);
            let r = lean_int_sub(a, b);
            let val = int_to_bigint(r);
            assert_eq!(val, BigInt::from(i64::MIN) - BigInt::from(1));
            dec(r);
        }
    }

    #[test]
    fn big_mul() {
        unsafe {
            let a = lean_int64_to_int(i64::MAX);
            let b = lean_int64_to_int(i64::MAX);
            let r = lean_int_mul(a, b);
            let val = int_to_bigint(r);
            assert_eq!(val, BigInt::from(i64::MAX) * BigInt::from(i64::MAX));
            dec(r);
        }
    }

    #[test]
    fn big_neg() {
        unsafe {
            let a = lean_int64_to_int(i64::MAX);
            let r = lean_int_neg(a);
            let val = int_to_bigint(r);
            assert_eq!(val, -BigInt::from(i64::MAX));
            dec(r);
        }
    }

    #[test]
    fn big_eq() {
        unsafe {
            let a = lean_int64_to_int(i64::MAX);
            let b = lean_int64_to_int(i64::MAX);
            assert!(lean_int_eq(a, b));
            dec(a);
            dec(b);
        }
    }

    #[test]
    fn big_lt() {
        unsafe {
            let a = lean_int64_to_int(i64::MIN);
            let b = lean_int64_to_int(i64::MAX);
            assert!(lean_int_lt(a, b));
            dec(a);
            dec(b);
        }
    }

    #[test]
    fn big_nat_abs() {
        unsafe {
            let a = lean_int64_to_int(i64::MIN);
            let r = lean_nat_abs(a);
            let val = lean_nat_to_biguint(r);
            let expected = BigUint::from(i64::MIN.unsigned_abs());
            assert_eq!(val, expected);
            crate::lean_dec(r);
        }
    }

    #[test]
    fn big_dec_nonneg() {
        unsafe {
            let pos = lean_int64_to_int(i64::MAX);
            let neg = lean_int64_to_int(i64::MIN);
            assert_eq!(lean_int_dec_nonneg(pos), 1);
            assert_eq!(lean_int_dec_nonneg(neg), 0);
            dec(pos);
            dec(neg);
        }
    }

    #[test]
    fn big_ediv_emod() {
        unsafe {
            // Test with big values: ediv/emod relationship
            let big_val = BigInt::from(10i64).pow(20);
            let divisor = BigInt::from(7i64);
            let a = lean_bigint_to_int(big_val.clone());
            let b = lean_bigint_to_int(-divisor.clone());
            crate::lean_inc(a);
            crate::lean_inc(b);
            let q_obj = lean_int_ediv(a, b);
            crate::lean_inc(a);
            crate::lean_inc(b);
            let r_obj = lean_int_emod(a, b);
            let q = int_to_bigint(q_obj);
            let r = int_to_bigint(r_obj);
            assert!(r >= BigInt::zero(), "emod result must be >= 0");
            assert_eq!(
                q * BigInt::from(-7i64) + r,
                big_val,
                "ediv/emod relationship must hold"
            );
            dec(q_obj);
            dec(r_obj);
            dec(a);
            dec(b);
        }
    }
}
