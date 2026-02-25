//! Int8/16/32/64/ISize (signed integer) operations for the Lean runtime.
//!
//! In Lean's ABI, signed integers are stored as their unsigned counterpart type.
//! Int8 values are passed as `u8`, Int16 as `u16`, Int32 as `u32`, Int64 as `u64`,
//! ISize as `usize`. The signed interpretation is applied only during operations
//! (by casting to the signed type, operating, and casting back to unsigned).

use crate::object::LeanObject;
use crate::r#box::*;

// ============================================================================
// Int8 (stored as u8)
// ============================================================================

#[inline(always)]
pub fn lean_int8_neg(a: u8) -> u8 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_int8_add(a: u8, b: u8) -> u8 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_int8_sub(a: u8, b: u8) -> u8 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_int8_mul(a: u8, b: u8) -> u8 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_int8_div(a: u8, b: u8) -> u8 {
    let lhs = a as i8;
    let rhs = b as i8;
    if rhs == 0 {
        0
    } else {
        // Widen to i16 to avoid overflow trap on i8::MIN / -1
        ((lhs as i16) / (rhs as i16)) as u8
    }
}

#[inline(always)]
pub fn lean_int8_mod(a: u8, b: u8) -> u8 {
    let lhs = a as i8;
    let rhs = b as i8;
    if rhs == 0 {
        a
    } else {
        ((lhs as i16) % (rhs as i16)) as u8
    }
}

#[inline(always)]
pub fn lean_int8_land(a: u8, b: u8) -> u8 {
    a & b
}

#[inline(always)]
pub fn lean_int8_lor(a: u8, b: u8) -> u8 {
    a | b
}

#[inline(always)]
pub fn lean_int8_xor(a: u8, b: u8) -> u8 {
    a ^ b
}

#[inline(always)]
pub fn lean_int8_shift_right(a: u8, b: u8) -> u8 {
    let lhs = a as i8;
    // smod 8: Euclidean modulo to get shift amount in [0, 8)
    let rhs = (((b as i8) % 8) + 8) % 8;
    (lhs >> rhs) as u8
}

#[inline(always)]
pub fn lean_int8_shift_left(a: u8, b: u8) -> u8 {
    // smod 8: Euclidean modulo to get shift amount in [0, 8)
    let rhs = (((b as i8) % 8) + 8) % 8;
    a << (rhs as u8)
}

#[inline(always)]
pub fn lean_int8_complement(a: u8) -> u8 {
    !(a as i8) as u8
}

#[inline(always)]
pub fn lean_int8_abs(a: u8) -> u8 {
    if (a as i8) < 0 {
        a.wrapping_neg()
    } else {
        a
    }
}

#[inline(always)]
pub fn lean_int8_dec_eq(a: u8, b: u8) -> u8 {
    ((a as i8) == (b as i8)) as u8
}

#[inline(always)]
pub fn lean_int8_dec_lt(a: u8, b: u8) -> u8 {
    ((a as i8) < (b as i8)) as u8
}

#[inline(always)]
pub fn lean_int8_dec_le(a: u8, b: u8) -> u8 {
    ((a as i8) <= (b as i8)) as u8
}

// --- Int8 conversions ---

#[inline(always)]
pub fn lean_int8_to_int16(a: u8) -> u16 {
    (a as i8 as i16) as u16
}

#[inline(always)]
pub fn lean_int8_to_int32(a: u8) -> u32 {
    (a as i8 as i32) as u32
}

#[inline(always)]
pub fn lean_int8_to_int64(a: u8) -> u64 {
    (a as i8 as i64) as u64
}

#[inline(always)]
pub fn lean_int8_to_isize(a: u8) -> usize {
    (a as i8 as isize) as usize
}

// --- Int8 <-> Int/Nat ---

#[inline(always)]
pub unsafe fn lean_int8_of_int(a: *mut LeanObject) -> u8 {
    if lean_is_scalar(a) {
        (lean_unbox(a) as u32 as i32 as i8) as u8
    } else {
        use num_traits::ToPrimitive;
        if (*a).tag == crate::object::LEAN_MPZ_TAG {
            // Big nat (non-negative): truncate to u8
            let v = crate::bignat::lean_bignat_value(a);
            v.to_u8().unwrap_or(0)
        } else {
            // Big int (negative): truncate to i8
            let v = crate::int::lean_bigint_value(a);
            v.to_i8().unwrap_or(0) as u8
        }
    }
}

#[inline(always)]
pub unsafe fn lean_int8_of_nat(a: *mut LeanObject) -> u8 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u8
    } else {
        let v = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        v.to_u8().unwrap_or(0)
    }
}

/// Convert Int8 to a boxed Int (Lean integer object).
/// Int8 always fits in scalar range (i32 range), so we just box directly.
#[inline(always)]
pub fn lean_int8_to_int(a: u8) -> *mut LeanObject {
    let v = a as i8 as i64;
    // i8 range [-128, 127] always fits in scalar (i32 range)
    lean_box(v as u32 as usize)
}

// ============================================================================
// Int16 (stored as u16)
// ============================================================================

#[inline(always)]
pub fn lean_int16_neg(a: u16) -> u16 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_int16_add(a: u16, b: u16) -> u16 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_int16_sub(a: u16, b: u16) -> u16 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_int16_mul(a: u16, b: u16) -> u16 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_int16_div(a: u16, b: u16) -> u16 {
    let lhs = a as i16;
    let rhs = b as i16;
    if rhs == 0 {
        0
    } else {
        // Widen to i32 to avoid overflow trap on i16::MIN / -1
        ((lhs as i32) / (rhs as i32)) as u16
    }
}

#[inline(always)]
pub fn lean_int16_mod(a: u16, b: u16) -> u16 {
    let lhs = a as i16;
    let rhs = b as i16;
    if rhs == 0 {
        a
    } else {
        ((lhs as i32) % (rhs as i32)) as u16
    }
}

#[inline(always)]
pub fn lean_int16_land(a: u16, b: u16) -> u16 {
    a & b
}

#[inline(always)]
pub fn lean_int16_lor(a: u16, b: u16) -> u16 {
    a | b
}

#[inline(always)]
pub fn lean_int16_xor(a: u16, b: u16) -> u16 {
    a ^ b
}

#[inline(always)]
pub fn lean_int16_shift_right(a: u16, b: u16) -> u16 {
    let lhs = a as i16;
    // smod 16
    let rhs = (((b as i16) % 16) + 16) % 16;
    (lhs >> rhs) as u16
}

#[inline(always)]
pub fn lean_int16_shift_left(a: u16, b: u16) -> u16 {
    // smod 16
    let rhs = (((b as i16) % 16) + 16) % 16;
    a << (rhs as u16)
}

#[inline(always)]
pub fn lean_int16_complement(a: u16) -> u16 {
    !(a as i16) as u16
}

#[inline(always)]
pub fn lean_int16_abs(a: u16) -> u16 {
    if (a as i16) < 0 {
        a.wrapping_neg()
    } else {
        a
    }
}

#[inline(always)]
pub fn lean_int16_dec_eq(a: u16, b: u16) -> u8 {
    ((a as i16) == (b as i16)) as u8
}

#[inline(always)]
pub fn lean_int16_dec_lt(a: u16, b: u16) -> u8 {
    ((a as i16) < (b as i16)) as u8
}

#[inline(always)]
pub fn lean_int16_dec_le(a: u16, b: u16) -> u8 {
    ((a as i16) <= (b as i16)) as u8
}

// --- Int16 conversions ---

#[inline(always)]
pub fn lean_int16_to_int8(a: u16) -> u8 {
    (a as i16 as i8) as u8
}

#[inline(always)]
pub fn lean_int16_to_int32(a: u16) -> u32 {
    (a as i16 as i32) as u32
}

#[inline(always)]
pub fn lean_int16_to_int64(a: u16) -> u64 {
    (a as i16 as i64) as u64
}

#[inline(always)]
pub fn lean_int16_to_isize(a: u16) -> usize {
    (a as i16 as isize) as usize
}

// --- Int16 <-> Int/Nat ---

#[inline(always)]
pub unsafe fn lean_int16_of_int(a: *mut LeanObject) -> u16 {
    if lean_is_scalar(a) {
        (lean_unbox(a) as u32 as i32 as i16) as u16
    } else {
        use num_traits::ToPrimitive;
        if (*a).tag == crate::object::LEAN_MPZ_TAG {
            let v = crate::bignat::lean_bignat_value(a);
            v.to_u16().unwrap_or(0)
        } else {
            let v = crate::int::lean_bigint_value(a);
            v.to_i16().unwrap_or(0) as u16
        }
    }
}

#[inline(always)]
pub unsafe fn lean_int16_of_nat(a: *mut LeanObject) -> u16 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u16
    } else {
        let v = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        v.to_u16().unwrap_or(0)
    }
}

/// Convert Int16 to a boxed Int (Lean integer object).
/// Int16 always fits in scalar range (i32 range), so we just box directly.
#[inline(always)]
pub fn lean_int16_to_int(a: u16) -> *mut LeanObject {
    let v = a as i16 as i64;
    lean_box(v as u32 as usize)
}

// ============================================================================
// Int32 (stored as u32)
// ============================================================================

#[inline(always)]
pub fn lean_int32_neg(a: u32) -> u32 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_int32_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_int32_sub(a: u32, b: u32) -> u32 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_int32_mul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_int32_div(a: u32, b: u32) -> u32 {
    let lhs = a as i32;
    let rhs = b as i32;
    if rhs == 0 {
        0
    } else {
        // Widen to i64 to avoid overflow trap on i32::MIN / -1
        ((lhs as i64) / (rhs as i64)) as u32
    }
}

#[inline(always)]
pub fn lean_int32_mod(a: u32, b: u32) -> u32 {
    let lhs = a as i32;
    let rhs = b as i32;
    if rhs == 0 {
        a
    } else {
        ((lhs as i64) % (rhs as i64)) as u32
    }
}

#[inline(always)]
pub fn lean_int32_land(a: u32, b: u32) -> u32 {
    a & b
}

#[inline(always)]
pub fn lean_int32_lor(a: u32, b: u32) -> u32 {
    a | b
}

#[inline(always)]
pub fn lean_int32_xor(a: u32, b: u32) -> u32 {
    a ^ b
}

#[inline(always)]
pub fn lean_int32_shift_right(a: u32, b: u32) -> u32 {
    let lhs = a as i32;
    // smod 32
    let rhs = (((b as i32) % 32) + 32) % 32;
    (lhs >> rhs) as u32
}

#[inline(always)]
pub fn lean_int32_shift_left(a: u32, b: u32) -> u32 {
    // smod 32
    let rhs = (((b as i32) % 32) + 32) % 32;
    a << (rhs as u32)
}

#[inline(always)]
pub fn lean_int32_complement(a: u32) -> u32 {
    !(a as i32) as u32
}

#[inline(always)]
pub fn lean_int32_abs(a: u32) -> u32 {
    if (a as i32) < 0 {
        a.wrapping_neg()
    } else {
        a
    }
}

#[inline(always)]
pub fn lean_int32_dec_eq(a: u32, b: u32) -> u8 {
    ((a as i32) == (b as i32)) as u8
}

#[inline(always)]
pub fn lean_int32_dec_lt(a: u32, b: u32) -> u8 {
    ((a as i32) < (b as i32)) as u8
}

#[inline(always)]
pub fn lean_int32_dec_le(a: u32, b: u32) -> u8 {
    ((a as i32) <= (b as i32)) as u8
}

// --- Int32 conversions ---

#[inline(always)]
pub fn lean_int32_to_int8(a: u32) -> u8 {
    (a as i32 as i8) as u8
}

#[inline(always)]
pub fn lean_int32_to_int16(a: u32) -> u16 {
    (a as i32 as i16) as u16
}

#[inline(always)]
pub fn lean_int32_to_int64(a: u32) -> u64 {
    (a as i32 as i64) as u64
}

#[inline(always)]
pub fn lean_int32_to_isize(a: u32) -> usize {
    (a as i32 as isize) as usize
}

// --- Int32 <-> Int/Nat ---

#[inline(always)]
pub unsafe fn lean_int32_of_int(a: *mut LeanObject) -> u32 {
    if lean_is_scalar(a) {
        (lean_unbox(a) as u32 as i32) as u32
    } else {
        use num_traits::ToPrimitive;
        if (*a).tag == crate::object::LEAN_MPZ_TAG {
            let v = crate::bignat::lean_bignat_value(a);
            v.to_u32().unwrap_or(0)
        } else {
            let v = crate::int::lean_bigint_value(a);
            v.to_i32().unwrap_or(0) as u32
        }
    }
}

#[inline(always)]
pub unsafe fn lean_int32_of_nat(a: *mut LeanObject) -> u32 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u32
    } else {
        let v = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        v.to_u32().unwrap_or(0)
    }
}

/// Convert Int32 to a boxed Int (Lean integer object).
/// Int32 always fits in scalar range (i32 range), so we just box directly.
#[inline(always)]
pub fn lean_int32_to_int(a: u32) -> *mut LeanObject {
    let v = a as i32 as i64;
    lean_box(v as u32 as usize)
}

// ============================================================================
// Int64 (stored as u64)
// ============================================================================

#[inline(always)]
pub fn lean_int64_neg(a: u64) -> u64 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_int64_add(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_int64_sub(a: u64, b: u64) -> u64 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_int64_mul(a: u64, b: u64) -> u64 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_int64_div(a: u64, b: u64) -> u64 {
    let lhs = a as i64;
    let rhs = b as i64;
    if rhs == 0 {
        0
    } else {
        // Widen to i128 to avoid overflow trap on i64::MIN / -1
        ((lhs as i128) / (rhs as i128)) as u64
    }
}

#[inline(always)]
pub fn lean_int64_mod(a: u64, b: u64) -> u64 {
    let lhs = a as i64;
    let rhs = b as i64;
    if rhs == 0 {
        a
    } else {
        ((lhs as i128) % (rhs as i128)) as u64
    }
}

#[inline(always)]
pub fn lean_int64_land(a: u64, b: u64) -> u64 {
    a & b
}

#[inline(always)]
pub fn lean_int64_lor(a: u64, b: u64) -> u64 {
    a | b
}

#[inline(always)]
pub fn lean_int64_xor(a: u64, b: u64) -> u64 {
    a ^ b
}

#[inline(always)]
pub fn lean_int64_shift_right(a: u64, b: u64) -> u64 {
    let lhs = a as i64;
    // smod 64
    let rhs = (((b as i64) % 64) + 64) % 64;
    (lhs >> rhs) as u64
}

#[inline(always)]
pub fn lean_int64_shift_left(a: u64, b: u64) -> u64 {
    // smod 64
    let rhs = (((b as i64) % 64) + 64) % 64;
    a << (rhs as u64)
}

#[inline(always)]
pub fn lean_int64_complement(a: u64) -> u64 {
    !(a as i64) as u64
}

#[inline(always)]
pub fn lean_int64_abs(a: u64) -> u64 {
    if (a as i64) < 0 {
        a.wrapping_neg()
    } else {
        a
    }
}

#[inline(always)]
pub fn lean_int64_dec_eq(a: u64, b: u64) -> u8 {
    ((a as i64) == (b as i64)) as u8
}

#[inline(always)]
pub fn lean_int64_dec_lt(a: u64, b: u64) -> u8 {
    ((a as i64) < (b as i64)) as u8
}

#[inline(always)]
pub fn lean_int64_dec_le(a: u64, b: u64) -> u8 {
    ((a as i64) <= (b as i64)) as u8
}

// --- Int64 conversions ---

#[inline(always)]
pub fn lean_int64_to_int8(a: u64) -> u8 {
    (a as i64 as i8) as u8
}

#[inline(always)]
pub fn lean_int64_to_int16(a: u64) -> u16 {
    (a as i64 as i16) as u16
}

#[inline(always)]
pub fn lean_int64_to_int32(a: u64) -> u32 {
    (a as i64 as i32) as u32
}

#[inline(always)]
pub fn lean_int64_to_isize(a: u64) -> usize {
    (a as i64 as isize) as usize
}

// --- Int64 <-> Int/Nat ---

#[inline(always)]
pub unsafe fn lean_int64_of_int(a: *mut LeanObject) -> u64 {
    if lean_is_scalar(a) {
        (lean_unbox(a) as u32 as i32 as i64) as u64
    } else {
        use num_traits::ToPrimitive;
        if (*a).tag == crate::object::LEAN_MPZ_TAG {
            let v = crate::bignat::lean_bignat_value(a);
            v.to_u64().unwrap_or(0)
        } else {
            let v = crate::int::lean_bigint_value(a);
            v.to_i64().unwrap_or(0) as u64
        }
    }
}

#[inline(always)]
pub unsafe fn lean_int64_of_nat(a: *mut LeanObject) -> u64 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u64
    } else {
        let v = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        v.to_u64().unwrap_or(0)
    }
}

/// Convert Int64 to a boxed Int (Lean integer object).
/// The value may not fit in scalar range, so we must handle the big case.
#[inline]
pub fn lean_int64_to_int(a: u64) -> *mut LeanObject {
    crate::int::lean_int64_to_int(a as i64)
}

// ============================================================================
// ISize (stored as usize)
// ============================================================================

#[inline(always)]
pub fn lean_isize_neg(a: usize) -> usize {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_isize_add(a: usize, b: usize) -> usize {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_isize_sub(a: usize, b: usize) -> usize {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_isize_mul(a: usize, b: usize) -> usize {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_isize_div(a: usize, b: usize) -> usize {
    let lhs = a as isize;
    let rhs = b as isize;
    if rhs == 0 {
        0
    } else {
        // Widen to i128 to avoid overflow trap on isize::MIN / -1
        ((lhs as i128) / (rhs as i128)) as usize
    }
}

#[inline(always)]
pub fn lean_isize_mod(a: usize, b: usize) -> usize {
    let lhs = a as isize;
    let rhs = b as isize;
    if rhs == 0 {
        a
    } else {
        ((lhs as i128) % (rhs as i128)) as usize
    }
}

#[inline(always)]
pub fn lean_isize_land(a: usize, b: usize) -> usize {
    a & b
}

#[inline(always)]
pub fn lean_isize_lor(a: usize, b: usize) -> usize {
    a | b
}

#[inline(always)]
pub fn lean_isize_xor(a: usize, b: usize) -> usize {
    a ^ b
}

#[inline(always)]
pub fn lean_isize_shift_right(a: usize, b: usize) -> usize {
    let lhs = a as isize;
    let bits = usize::BITS as isize;
    // smod (usize::BITS)
    let rhs = (((b as isize) % bits) + bits) % bits;
    (lhs >> rhs) as usize
}

#[inline(always)]
pub fn lean_isize_shift_left(a: usize, b: usize) -> usize {
    let bits = usize::BITS as isize;
    // smod (usize::BITS)
    let rhs = (((b as isize) % bits) + bits) % bits;
    a << (rhs as usize)
}

#[inline(always)]
pub fn lean_isize_complement(a: usize) -> usize {
    !(a as isize) as usize
}

#[inline(always)]
pub fn lean_isize_abs(a: usize) -> usize {
    if (a as isize) < 0 {
        a.wrapping_neg()
    } else {
        a
    }
}

#[inline(always)]
pub fn lean_isize_dec_eq(a: usize, b: usize) -> u8 {
    ((a as isize) == (b as isize)) as u8
}

#[inline(always)]
pub fn lean_isize_dec_lt(a: usize, b: usize) -> u8 {
    ((a as isize) < (b as isize)) as u8
}

#[inline(always)]
pub fn lean_isize_dec_le(a: usize, b: usize) -> u8 {
    ((a as isize) <= (b as isize)) as u8
}

// --- ISize conversions ---

#[inline(always)]
pub fn lean_isize_to_int8(a: usize) -> u8 {
    (a as isize as i8) as u8
}

#[inline(always)]
pub fn lean_isize_to_int16(a: usize) -> u16 {
    (a as isize as i16) as u16
}

#[inline(always)]
pub fn lean_isize_to_int32(a: usize) -> u32 {
    (a as isize as i32) as u32
}

#[inline(always)]
pub fn lean_isize_to_int64(a: usize) -> u64 {
    (a as isize as i64) as u64
}

// --- ISize <-> Int/Nat ---

#[inline(always)]
pub unsafe fn lean_isize_of_int(a: *mut LeanObject) -> usize {
    if lean_is_scalar(a) {
        (lean_unbox(a) as u32 as i32 as isize) as usize
    } else {
        use num_traits::ToPrimitive;
        if (*a).tag == crate::object::LEAN_MPZ_TAG {
            let v = crate::bignat::lean_bignat_value(a);
            v.to_usize().unwrap_or(0)
        } else {
            let v = crate::int::lean_bigint_value(a);
            v.to_isize().unwrap_or(0) as usize
        }
    }
}

#[inline(always)]
pub unsafe fn lean_isize_of_nat(a: *mut LeanObject) -> usize {
    if lean_is_scalar(a) {
        lean_unbox(a)
    } else {
        let v = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        v.to_usize().unwrap_or(0)
    }
}

/// Convert ISize to a boxed Int (Lean integer object).
/// The value may not fit in scalar range, so we must handle the big case.
#[inline]
pub fn lean_isize_to_int(a: usize) -> *mut LeanObject {
    crate::int::lean_int64_to_int(a as isize as i64)
}

// ============================================================================
// Bool-to-signed conversions
// ============================================================================

#[inline(always)]
pub fn lean_bool_to_int8(a: u8) -> u8 {
    a
}

#[inline(always)]
pub fn lean_bool_to_int16(a: u8) -> u16 {
    a as u16
}

#[inline(always)]
pub fn lean_bool_to_int32(a: u8) -> u32 {
    a as u32
}

#[inline(always)]
pub fn lean_bool_to_int64(a: u8) -> u64 {
    a as u64
}

#[inline(always)]
pub fn lean_bool_to_isize(a: u8) -> usize {
    a as usize
}

// Bool-to-unsigned conversions

#[inline(always)]
pub fn lean_bool_to_uint8(a: u8) -> u8 {
    a
}

#[inline(always)]
pub fn lean_bool_to_uint16(a: u8) -> u16 {
    a as u16
}

#[inline(always)]
pub fn lean_bool_to_uint32(a: u8) -> u32 {
    a as u32
}

#[inline(always)]
pub fn lean_bool_to_uint64(a: u8) -> u64 {
    a as u64
}

#[inline(always)]
pub fn lean_bool_to_usize(a: u8) -> usize {
    a as usize
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Int8 tests ---

    #[test]
    fn int8_neg() {
        // -1 as u8 = 0xFF
        assert_eq!(lean_int8_neg(1), 0xFF);
        // -(-128) wraps to -128 (0x80)
        assert_eq!(lean_int8_neg(0x80), 0x80);
        assert_eq!(lean_int8_neg(0), 0);
    }

    #[test]
    fn int8_add_wrapping() {
        // 127 + 1 wraps to -128 (0x80)
        assert_eq!(lean_int8_add(127, 1), 0x80);
        // -128 + (-1) wraps to 127
        assert_eq!(lean_int8_add(0x80, 0xFF), 0x7F);
    }

    #[test]
    fn int8_sub_wrapping() {
        // -128 - 1 wraps to 127
        assert_eq!(lean_int8_sub(0x80, 1), 0x7F);
    }

    #[test]
    fn int8_mul_wrapping() {
        // 64 * 2 = 128 wraps to -128 (0x80)
        assert_eq!(lean_int8_mul(64, 2), 0x80);
    }

    #[test]
    fn int8_div_basic() {
        // 7 / 2 = 3
        assert_eq!(lean_int8_div(7, 2), 3);
        // -7 / 2 = -3 (truncation toward zero)
        assert_eq!(lean_int8_div((-7i8) as u8, 2), (-3i8) as u8);
    }

    #[test]
    fn int8_div_by_zero() {
        assert_eq!(lean_int8_div(42, 0), 0);
    }

    #[test]
    fn int8_div_min_by_neg1() {
        // i8::MIN / -1 would overflow; widening avoids trap, result = 128 truncated to u8 = 0x80
        // Actually: -128 / -1 = 128, which as i16 is 128, cast to u8 = 0x80
        let result = lean_int8_div(0x80, (-1i8) as u8);
        // 128 as u8 = 0x80 = -128 (wraps)
        assert_eq!(result, 0x80);
    }

    #[test]
    fn int8_mod_basic() {
        // 7 % 2 = 1
        assert_eq!(lean_int8_mod(7, 2), 1);
        // -7 % 2 = -1
        assert_eq!(lean_int8_mod((-7i8) as u8, 2), (-1i8) as u8);
    }

    #[test]
    fn int8_mod_by_zero() {
        // mod by 0 returns a (the dividend)
        assert_eq!(lean_int8_mod(42, 0), 42);
    }

    #[test]
    fn int8_abs() {
        assert_eq!(lean_int8_abs(5), 5);
        assert_eq!(lean_int8_abs((-5i8) as u8), 5);
        assert_eq!(lean_int8_abs(0), 0);
        // abs(i8::MIN) wraps to i8::MIN (0x80)
        assert_eq!(lean_int8_abs(0x80), 0x80);
    }

    #[test]
    fn int8_shift_right() {
        // Arithmetic shift right: -4 >> 1 = -2
        assert_eq!(lean_int8_shift_right((-4i8) as u8, 1), (-2i8) as u8);
        // 64 >> 2 = 16
        assert_eq!(lean_int8_shift_right(64, 2), 16);
    }

    #[test]
    fn int8_shift_left() {
        // 1 << 3 = 8
        assert_eq!(lean_int8_shift_left(1, 3), 8);
        // 1 << 7 = 128 = 0x80
        assert_eq!(lean_int8_shift_left(1, 7), 0x80);
    }

    #[test]
    fn int8_complement() {
        assert_eq!(lean_int8_complement(0), 0xFF);
        assert_eq!(lean_int8_complement(0xFF), 0);
    }

    #[test]
    fn int8_comparisons() {
        // 5 == 5
        assert_eq!(lean_int8_dec_eq(5, 5), 1);
        assert_eq!(lean_int8_dec_eq(5, 6), 0);
        // -1 < 0
        assert_eq!(lean_int8_dec_lt((-1i8) as u8, 0), 1);
        assert_eq!(lean_int8_dec_lt(0, (-1i8) as u8), 0);
        // -1 <= -1
        assert_eq!(lean_int8_dec_le((-1i8) as u8, (-1i8) as u8), 1);
        assert_eq!(lean_int8_dec_le(0, (-1i8) as u8), 0);
    }

    #[test]
    fn int8_bitwise() {
        assert_eq!(lean_int8_land(0xF0, 0x0F), 0x00);
        assert_eq!(lean_int8_lor(0xF0, 0x0F), 0xFF);
        assert_eq!(lean_int8_xor(0xFF, 0x0F), 0xF0);
    }

    #[test]
    fn int8_conversions() {
        // -1 as i8 sign-extends to -1 in all wider types
        let neg1_u8 = (-1i8) as u8; // 0xFF
        assert_eq!(lean_int8_to_int16(neg1_u8), (-1i16) as u16);
        assert_eq!(lean_int8_to_int32(neg1_u8), (-1i32) as u32);
        assert_eq!(lean_int8_to_int64(neg1_u8), (-1i64) as u64);
        assert_eq!(lean_int8_to_isize(neg1_u8), (-1isize) as usize);
    }

    #[test]
    fn int8_to_int_roundtrip() {
        let boxed = lean_int8_to_int((-42i8) as u8);
        assert!(lean_is_scalar(boxed));
        // The boxed value encodes i8(-42) sign-extended to i64, then cast to u32
        let unboxed = lean_unbox(boxed) as u32 as i32;
        assert_eq!(unboxed, -42);
    }

    // --- Int16 tests ---

    #[test]
    fn int16_neg() {
        assert_eq!(lean_int16_neg(1), 0xFFFF);
        assert_eq!(lean_int16_neg(0x8000), 0x8000);
        assert_eq!(lean_int16_neg(0), 0);
    }

    #[test]
    fn int16_add_wrapping() {
        assert_eq!(lean_int16_add(0x7FFF, 1), 0x8000);
    }

    #[test]
    fn int16_div_basic() {
        assert_eq!(lean_int16_div(100, 7), 14);
        assert_eq!(lean_int16_div((-100i16) as u16, 7), (-14i16) as u16);
    }

    #[test]
    fn int16_div_by_zero() {
        assert_eq!(lean_int16_div(42, 0), 0);
    }

    #[test]
    fn int16_div_min_by_neg1() {
        let result = lean_int16_div(0x8000, (-1i16) as u16);
        assert_eq!(result, 0x8000);
    }

    #[test]
    fn int16_mod_basic() {
        assert_eq!(lean_int16_mod(100, 7), 2);
        assert_eq!(lean_int16_mod((-100i16) as u16, 7), (-2i16) as u16);
    }

    #[test]
    fn int16_mod_by_zero() {
        assert_eq!(lean_int16_mod(42, 0), 42);
    }

    #[test]
    fn int16_abs() {
        assert_eq!(lean_int16_abs(5), 5);
        assert_eq!(lean_int16_abs((-5i16) as u16), 5);
        assert_eq!(lean_int16_abs(0x8000), 0x8000);
    }

    #[test]
    fn int16_comparisons() {
        assert_eq!(lean_int16_dec_eq(100, 100), 1);
        assert_eq!(lean_int16_dec_lt((-1i16) as u16, 0), 1);
        assert_eq!(lean_int16_dec_le((-1i16) as u16, (-1i16) as u16), 1);
    }

    #[test]
    fn int16_shift() {
        assert_eq!(lean_int16_shift_left(1, 15), 0x8000);
        assert_eq!(lean_int16_shift_right((-4i16) as u16, 1), (-2i16) as u16);
    }

    #[test]
    fn int16_conversions() {
        let neg1_u16 = (-1i16) as u16;
        assert_eq!(lean_int16_to_int8(neg1_u16), (-1i8) as u8);
        assert_eq!(lean_int16_to_int32(neg1_u16), (-1i32) as u32);
        assert_eq!(lean_int16_to_int64(neg1_u16), (-1i64) as u64);
        assert_eq!(lean_int16_to_isize(neg1_u16), (-1isize) as usize);
    }

    // --- Int32 tests ---

    #[test]
    fn int32_neg() {
        assert_eq!(lean_int32_neg(1), 0xFFFF_FFFF);
        assert_eq!(lean_int32_neg(0x8000_0000), 0x8000_0000);
    }

    #[test]
    fn int32_add_wrapping() {
        assert_eq!(lean_int32_add(0x7FFF_FFFF, 1), 0x8000_0000);
    }

    #[test]
    fn int32_div_basic() {
        assert_eq!(lean_int32_div(100, 7), 14);
        assert_eq!(lean_int32_div((-100i32) as u32, 7), (-14i32) as u32);
    }

    #[test]
    fn int32_div_by_zero() {
        assert_eq!(lean_int32_div(42, 0), 0);
    }

    #[test]
    fn int32_div_min_by_neg1() {
        let result = lean_int32_div(0x8000_0000, (-1i32) as u32);
        assert_eq!(result, 0x8000_0000);
    }

    #[test]
    fn int32_mod_basic() {
        assert_eq!(lean_int32_mod(100, 7), 2);
        assert_eq!(lean_int32_mod((-100i32) as u32, 7), (-2i32) as u32);
    }

    #[test]
    fn int32_mod_by_zero() {
        assert_eq!(lean_int32_mod(42, 0), 42);
    }

    #[test]
    fn int32_abs() {
        assert_eq!(lean_int32_abs(42), 42);
        assert_eq!(lean_int32_abs((-42i32) as u32), 42);
        assert_eq!(lean_int32_abs(0x8000_0000), 0x8000_0000);
    }

    #[test]
    fn int32_comparisons() {
        assert_eq!(lean_int32_dec_eq(100, 100), 1);
        assert_eq!(lean_int32_dec_lt((-1i32) as u32, 0), 1);
        assert_eq!(lean_int32_dec_le((-1i32) as u32, (-1i32) as u32), 1);
    }

    #[test]
    fn int32_shift() {
        assert_eq!(lean_int32_shift_left(1, 31), 0x8000_0000);
        assert_eq!(lean_int32_shift_right((-4i32) as u32, 1), (-2i32) as u32);
    }

    #[test]
    fn int32_conversions() {
        let neg1_u32 = (-1i32) as u32;
        assert_eq!(lean_int32_to_int8(neg1_u32), (-1i8) as u8);
        assert_eq!(lean_int32_to_int16(neg1_u32), (-1i16) as u16);
        assert_eq!(lean_int32_to_int64(neg1_u32), (-1i64) as u64);
        assert_eq!(lean_int32_to_isize(neg1_u32), (-1isize) as usize);
    }

    // --- Int64 tests ---

    #[test]
    fn int64_neg() {
        assert_eq!(lean_int64_neg(1), 0xFFFF_FFFF_FFFF_FFFF);
        assert_eq!(lean_int64_neg(0x8000_0000_0000_0000), 0x8000_0000_0000_0000);
    }

    #[test]
    fn int64_add_wrapping() {
        assert_eq!(
            lean_int64_add(0x7FFF_FFFF_FFFF_FFFF, 1),
            0x8000_0000_0000_0000
        );
    }

    #[test]
    fn int64_div_basic() {
        assert_eq!(lean_int64_div(100, 7), 14);
        assert_eq!(lean_int64_div((-100i64) as u64, 7), (-14i64) as u64);
    }

    #[test]
    fn int64_div_by_zero() {
        assert_eq!(lean_int64_div(42, 0), 0);
    }

    #[test]
    fn int64_div_min_by_neg1() {
        let result = lean_int64_div(0x8000_0000_0000_0000, (-1i64) as u64);
        assert_eq!(result, 0x8000_0000_0000_0000);
    }

    #[test]
    fn int64_mod_basic() {
        assert_eq!(lean_int64_mod(100, 7), 2);
        assert_eq!(lean_int64_mod((-100i64) as u64, 7), (-2i64) as u64);
    }

    #[test]
    fn int64_mod_by_zero() {
        assert_eq!(lean_int64_mod(42, 0), 42);
    }

    #[test]
    fn int64_abs() {
        assert_eq!(lean_int64_abs(42), 42);
        assert_eq!(lean_int64_abs((-42i64) as u64), 42);
        assert_eq!(lean_int64_abs(0x8000_0000_0000_0000), 0x8000_0000_0000_0000);
    }

    #[test]
    fn int64_comparisons() {
        assert_eq!(lean_int64_dec_eq(100, 100), 1);
        assert_eq!(lean_int64_dec_eq(100, 200), 0);
        assert_eq!(lean_int64_dec_lt((-1i64) as u64, 0), 1);
        assert_eq!(lean_int64_dec_lt(0, (-1i64) as u64), 0);
        assert_eq!(lean_int64_dec_le((-1i64) as u64, (-1i64) as u64), 1);
    }

    #[test]
    fn int64_shift() {
        assert_eq!(lean_int64_shift_left(1, 63), 0x8000_0000_0000_0000);
        assert_eq!(lean_int64_shift_right((-4i64) as u64, 1), (-2i64) as u64);
    }

    #[test]
    fn int64_bitwise() {
        assert_eq!(lean_int64_land(0xF0F0, 0x0F0F), 0x0000);
        assert_eq!(lean_int64_lor(0xF0F0, 0x0F0F), 0xFFFF);
        assert_eq!(lean_int64_xor(0xFFFF, 0x0F0F), 0xF0F0);
        assert_eq!(lean_int64_complement(0), 0xFFFF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn int64_conversions() {
        let neg1_u64 = (-1i64) as u64;
        assert_eq!(lean_int64_to_int8(neg1_u64), (-1i8) as u8);
        assert_eq!(lean_int64_to_int16(neg1_u64), (-1i16) as u16);
        assert_eq!(lean_int64_to_int32(neg1_u64), (-1i32) as u32);
        assert_eq!(lean_int64_to_isize(neg1_u64), (-1isize) as usize);
    }

    #[test]
    fn int64_to_int_small() {
        let boxed = lean_int64_to_int(42);
        assert!(lean_is_scalar(boxed));
        let v = lean_unbox(boxed) as u32 as i32;
        assert_eq!(v, 42);

        let boxed_neg = lean_int64_to_int((-42i64) as u64);
        assert!(lean_is_scalar(boxed_neg));
        let v_neg = lean_unbox(boxed_neg) as u32 as i32;
        assert_eq!(v_neg, -42);
    }

    #[test]
    fn int64_to_int_big() {
        unsafe {
            // i64::MAX does not fit in i32 scalar range
            let boxed = lean_int64_to_int(i64::MAX as u64);
            assert!(!lean_is_scalar(boxed));
            // Clean up via standard dec path
            crate::lean_dec(boxed);
        }
    }

    // --- ISize tests ---

    #[test]
    fn isize_neg() {
        assert_eq!(lean_isize_neg(1), usize::MAX);
        assert_eq!(lean_isize_neg(0), 0);
    }

    #[test]
    fn isize_add_wrapping() {
        assert_eq!(lean_isize_add(isize::MAX as usize, 1), isize::MIN as usize);
    }

    #[test]
    fn isize_div_basic() {
        assert_eq!(lean_isize_div(100, 7), 14);
        assert_eq!(lean_isize_div((-100isize) as usize, 7), (-14isize) as usize);
    }

    #[test]
    fn isize_div_by_zero() {
        assert_eq!(lean_isize_div(42, 0), 0);
    }

    #[test]
    fn isize_div_min_by_neg1() {
        let result = lean_isize_div(isize::MIN as usize, (-1isize) as usize);
        assert_eq!(result, isize::MIN as usize);
    }

    #[test]
    fn isize_mod_basic() {
        assert_eq!(lean_isize_mod(100, 7), 2);
        assert_eq!(lean_isize_mod((-100isize) as usize, 7), (-2isize) as usize);
    }

    #[test]
    fn isize_mod_by_zero() {
        assert_eq!(lean_isize_mod(42, 0), 42);
    }

    #[test]
    fn isize_abs() {
        assert_eq!(lean_isize_abs(42), 42);
        assert_eq!(lean_isize_abs((-42isize) as usize), 42);
        assert_eq!(lean_isize_abs(isize::MIN as usize), isize::MIN as usize);
    }

    #[test]
    fn isize_comparisons() {
        assert_eq!(lean_isize_dec_eq(100, 100), 1);
        assert_eq!(lean_isize_dec_eq(100, 200), 0);
        assert_eq!(lean_isize_dec_lt((-1isize) as usize, 0), 1);
        assert_eq!(lean_isize_dec_lt(0, (-1isize) as usize), 0);
        assert_eq!(lean_isize_dec_le((-1isize) as usize, (-1isize) as usize), 1);
    }

    #[test]
    fn isize_shift() {
        let bits = usize::BITS;
        assert_eq!(
            lean_isize_shift_left(1, (bits - 1) as usize),
            1usize << (bits - 1)
        );
        assert_eq!(
            lean_isize_shift_right((-4isize) as usize, 1),
            (-2isize) as usize
        );
    }

    #[test]
    fn isize_bitwise() {
        assert_eq!(lean_isize_land(0xF0, 0x0F), 0x00);
        assert_eq!(lean_isize_lor(0xF0, 0x0F), 0xFF);
        assert_eq!(lean_isize_xor(0xFF, 0x0F), 0xF0);
        assert_eq!(lean_isize_complement(0), usize::MAX);
    }

    #[test]
    fn isize_conversions() {
        let neg1 = (-1isize) as usize;
        assert_eq!(lean_isize_to_int8(neg1), (-1i8) as u8);
        assert_eq!(lean_isize_to_int16(neg1), (-1i16) as u16);
        assert_eq!(lean_isize_to_int32(neg1), (-1i32) as u32);
        assert_eq!(lean_isize_to_int64(neg1), (-1i64) as u64);
    }

    // --- Bool conversion tests ---

    #[test]
    fn bool_to_signed() {
        assert_eq!(lean_bool_to_int8(0), 0u8);
        assert_eq!(lean_bool_to_int8(1), 1u8);
        assert_eq!(lean_bool_to_int16(1), 1u16);
        assert_eq!(lean_bool_to_int32(1), 1u32);
        assert_eq!(lean_bool_to_int64(1), 1u64);
        assert_eq!(lean_bool_to_isize(1), 1usize);
    }

    #[test]
    fn bool_to_unsigned() {
        assert_eq!(lean_bool_to_uint8(0), 0u8);
        assert_eq!(lean_bool_to_uint8(1), 1u8);
        assert_eq!(lean_bool_to_uint16(1), 1u16);
        assert_eq!(lean_bool_to_uint32(1), 1u32);
        assert_eq!(lean_bool_to_uint64(1), 1u64);
        assert_eq!(lean_bool_to_usize(1), 1usize);
    }

    // --- of_int/of_nat tests ---

    #[test]
    fn int8_of_nat_small() {
        unsafe {
            assert_eq!(lean_int8_of_nat(lean_box(42)), 42u8);
            assert_eq!(lean_int8_of_nat(lean_box(255)), 255u8);
        }
    }

    #[test]
    fn int16_of_nat_small() {
        unsafe {
            assert_eq!(lean_int16_of_nat(lean_box(1000)), 1000u16);
        }
    }

    #[test]
    fn int32_of_nat_small() {
        unsafe {
            assert_eq!(lean_int32_of_nat(lean_box(100000)), 100000u32);
        }
    }

    #[test]
    fn int64_of_nat_small() {
        unsafe {
            assert_eq!(lean_int64_of_nat(lean_box(123456)), 123456u64);
        }
    }

    #[test]
    fn isize_of_nat_small() {
        unsafe {
            assert_eq!(lean_isize_of_nat(lean_box(999)), 999usize);
        }
    }
}
