//! Float64 and Float32 operations for the Lean 4 runtime.

#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::object::{LeanObject, LEAN_OBJECT_SIZE};

// ============================================================================
// Internal helpers: frexp / ldexp
// ============================================================================

/// Decomposes `x` into a normalized fraction and exponent such that
/// `x = frac * 2^exp` with `0.5 <= |frac| < 1.0` (when `x` is normal).
fn frexp_impl_f64(x: f64) -> (f64, i32) {
    if x == 0.0 || x.is_nan() || x.is_infinite() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let sign = bits & 0x8000_0000_0000_0000;
    let exp = ((bits >> 52) & 0x7FF) as i32;
    let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;

    if exp == 0 {
        // Denormalized: multiply by 2^63 to normalize, then subtract 63 from exponent
        let (f, e) = frexp_impl_f64(x * (1u64 << 63) as f64);
        return (f, e - 63);
    }

    // Set exponent to -1 (biased: 1022) to get 0.5 <= |result| < 1.0
    let new_bits = sign | (0x3FE_u64 << 52) | mantissa;
    (f64::from_bits(new_bits), exp - 1022)
}

/// Computes `x * 2^exp`, handling overflow and underflow.
fn ldexp_impl_f64(x: f64, exp: i32) -> f64 {
    if x == 0.0 || x.is_nan() || x.is_infinite() || exp == 0 {
        return x;
    }
    let mut result = x;
    let mut e = exp;
    // 2^1023
    const TWO_1023: f64 = f64::from_bits(0x7FE0_0000_0000_0000_u64);
    // 2^-1022
    const TWO_NEG1022: f64 = f64::from_bits(0x0010_0000_0000_0000_u64);
    while e > 1023 {
        result *= TWO_1023;
        e -= 1023;
    }
    while e < -1022 {
        result *= TWO_NEG1022;
        e += 1022;
    }
    result * f64::from_bits(((e + 1023) as u64) << 52)
}

/// Decomposes `x` (f32) into a normalized fraction and exponent.
fn frexp_impl_f32(x: f32) -> (f32, i32) {
    if x == 0.0 || x.is_nan() || x.is_infinite() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let sign = bits & 0x8000_0000;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let mantissa = bits & 0x007F_FFFF;

    if exp == 0 {
        // Denormalized: multiply by 2^23 to normalize
        let (f, e) = frexp_impl_f32(x * (1u32 << 23) as f32);
        return (f, e - 23);
    }

    // Set exponent to -1 (biased: 126 = 0x7E) to get 0.5 <= |result| < 1.0
    let new_bits = sign | (0x7E_u32 << 23) | mantissa;
    (f32::from_bits(new_bits), exp - 126)
}

/// Computes `x * 2^exp` for f32.
fn ldexp_impl_f32(x: f32, exp: i32) -> f32 {
    if x == 0.0 || x.is_nan() || x.is_infinite() || exp == 0 {
        return x;
    }
    let mut result = x;
    let mut e = exp;
    // 2^127
    const TWO_127: f32 = f32::from_bits(0x7F00_0000_u32);
    // 2^-126
    const TWO_NEG126: f32 = f32::from_bits(0x0080_0000_u32);
    while e > 127 {
        result *= TWO_127;
        e -= 127;
    }
    while e < -126 {
        result *= TWO_NEG126;
        e += 126;
    }
    result * f32::from_bits(((e + 127) as u32) << 23)
}

/// Convert an i32 to a boxed Lean Int using the `crate::int` module.
unsafe fn lean_small_int_to_obj(v: i32) -> *mut LeanObject {
    crate::int::lean_int64_to_int(v as i64)
}

/// Extract an i32 from a boxed Lean Int using the `crate::int` module.
/// If the value is out of i32 range, it is clamped.
unsafe fn lean_obj_to_int32(o: *mut LeanObject) -> i32 {
    if crate::lean_is_scalar(o) {
        crate::int::lean_scalar_to_int64(o) as i32
    } else {
        use num_traits::ToPrimitive;
        let tag = (*o).tag;
        if tag == crate::object::LEAN_MPZ_TAG {
            // Large non-negative BigUint (tag 250)
            let biguint = crate::bignat::lean_bignat_value(o);
            biguint.to_i32().unwrap_or(i32::MAX)
        } else {
            // Large negative BigInt (tag 251)
            let big = crate::int::lean_bigint_value(o);
            big.to_i32().unwrap_or_else(|| {
                if big.sign() == num_bigint::Sign::Minus {
                    i32::MIN
                } else {
                    i32::MAX
                }
            })
        }
    }
}

// ============================================================================
// Float64 ctor get/set
// ============================================================================

/// Read an f64 from the scalar area of a ctor at the given byte offset.
#[inline(always)]
pub unsafe fn lean_ctor_get_float(o: *mut LeanObject, offset: u32) -> f64 {
    let num_objs = (*o).other as usize;
    let base = (o as *const u8).add(LEAN_OBJECT_SIZE).add(num_objs * 8);
    let ptr = base.add(offset as usize) as *const f64;
    std::ptr::read_unaligned(ptr)
}

/// Write an f64 to the scalar area of a ctor at the given byte offset.
#[inline(always)]
pub unsafe fn lean_ctor_set_float(o: *mut LeanObject, offset: u32, v: f64) {
    let num_objs = (*o).other as usize;
    let base = (o as *mut u8).add(LEAN_OBJECT_SIZE).add(num_objs * 8);
    let ptr = base.add(offset as usize) as *mut f64;
    std::ptr::write_unaligned(ptr, v);
}

// ============================================================================
// Float64 boxing/unboxing
// ============================================================================

/// Box an f64 as a Lean object: ctor with tag 0, 0 obj fields, 8 bytes scalar.
#[inline]
pub unsafe fn lean_box_float(v: f64) -> *mut LeanObject {
    let r = crate::lean_alloc_ctor(0, 0, std::mem::size_of::<f64>() as u32);
    lean_ctor_set_float(r, 0, v);
    r
}

/// Unbox an f64 from a Lean object.
#[inline]
pub unsafe fn lean_unbox_float(o: *mut LeanObject) -> f64 {
    lean_ctor_get_float(o, 0)
}

// ============================================================================
// Float64 arithmetic
// ============================================================================

#[inline(always)]
pub fn lean_float_add(a: f64, b: f64) -> f64 {
    a + b
}

#[inline(always)]
pub fn lean_float_sub(a: f64, b: f64) -> f64 {
    a - b
}

#[inline(always)]
pub fn lean_float_mul(a: f64, b: f64) -> f64 {
    a * b
}

#[inline(always)]
pub fn lean_float_div(a: f64, b: f64) -> f64 {
    a / b
}

#[inline(always)]
pub fn lean_float_negate(a: f64) -> f64 {
    -a
}

// ============================================================================
// Float64 comparison (return u8 boolean)
// ============================================================================

#[inline(always)]
pub fn lean_float_beq(a: f64, b: f64) -> u8 {
    (a == b) as u8
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lean_float_decLe(a: f64, b: f64) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lean_float_decLt(a: f64, b: f64) -> u8 {
    (a < b) as u8
}

// ============================================================================
// Float64 classification
// ============================================================================

#[inline(always)]
pub fn lean_float_isnan(a: f64) -> u8 {
    a.is_nan() as u8
}

#[inline(always)]
pub fn lean_float_isfinite(a: f64) -> u8 {
    a.is_finite() as u8
}

#[inline(always)]
pub fn lean_float_isinf(a: f64) -> u8 {
    a.is_infinite() as u8
}

// ============================================================================
// Float64 bit conversion
// ============================================================================

#[inline(always)]
pub fn lean_float_of_bits(u: u64) -> f64 {
    f64::from_bits(u)
}

#[inline(always)]
pub fn lean_float_to_bits(d: f64) -> u64 {
    d.to_bits()
}

// ============================================================================
// Float64 string conversion
// ============================================================================

/// Convert an f64 to a Lean string object.
/// Matches the C runtime formatting: 6 decimal places, special cases for
/// NaN, inf, -inf, and -0.
pub unsafe fn lean_float_to_string(a: f64) -> *mut LeanObject {
    let s = if a.is_nan() {
        "NaN".to_string()
    } else if a == f64::INFINITY {
        "inf".to_string()
    } else if a == f64::NEG_INFINITY {
        "-inf".to_string()
    } else if a == 0.0 && a.is_sign_negative() {
        "-0.000000".to_string()
    } else {
        format!("{:.6}", a)
    };
    crate::string::lean_mk_string(&s)
}

// ============================================================================
// Float64 frexp / scaleb
// ============================================================================

/// `frexp(a)` returns `(frac, exp)` as a Lean pair `(Float, Int)`.
/// Result is a ctor with tag 0, 2 obj fields: field 0 = boxed Float,
/// field 1 = boxed Int.
pub unsafe fn lean_float_frexp(a: f64) -> *mut LeanObject {
    let (frac, exp) = frexp_impl_f64(a);
    let pair = crate::lean_alloc_ctor(0, 2, 0);
    crate::lean_ctor_set(pair, 0, lean_box_float(frac));
    crate::lean_ctor_set(pair, 1, lean_small_int_to_obj(exp));
    pair
}

/// `scaleb(a, b)` computes `a * 2^b` where `b` is a boxed Lean Int.
pub unsafe fn lean_float_scaleb(a: f64, b: *mut LeanObject) -> f64 {
    let exp = lean_obj_to_int32(b);
    ldexp_impl_f64(a, exp)
}

// ============================================================================
// Float64 to unsigned integer conversions (with clamping)
// ============================================================================

#[inline(always)]
pub fn lean_float_to_uint8(a: f64) -> u8 {
    if !(a >= 0.0) {
        0
    } else if a < 256.0 {
        a as u8
    } else {
        u8::MAX
    }
}

#[inline(always)]
pub fn lean_float_to_uint16(a: f64) -> u16 {
    if !(a >= 0.0) {
        0
    } else if a < 65536.0 {
        a as u16
    } else {
        u16::MAX
    }
}

#[inline(always)]
pub fn lean_float_to_uint32(a: f64) -> u32 {
    if !(a >= 0.0) {
        0
    } else if a < 4_294_967_296.0 {
        a as u32
    } else {
        u32::MAX
    }
}

#[inline(always)]
pub fn lean_float_to_uint64(a: f64) -> u64 {
    if !(a >= 0.0) {
        0
    } else if a < 18_446_744_073_709_551_616.0 {
        a as u64
    } else {
        u64::MAX
    }
}

#[inline(always)]
pub fn lean_float_to_usize(a: f64) -> usize {
    lean_float_to_uint64(a) as usize
}

// ============================================================================
// Float64 to signed integer conversions (with clamping, NaN -> 0)
// ============================================================================

#[inline(always)]
pub fn lean_float_to_int8(a: f64) -> u8 {
    if a.is_nan() {
        0u8
    } else {
        let r: i8 = if a < -128.0 {
            i8::MIN
        } else if a >= 128.0 {
            i8::MAX
        } else {
            a as i8
        };
        r as u8
    }
}

#[inline(always)]
pub fn lean_float_to_int16(a: f64) -> u16 {
    if a.is_nan() {
        0u16
    } else {
        let r: i16 = if a < -32768.0 {
            i16::MIN
        } else if a >= 32768.0 {
            i16::MAX
        } else {
            a as i16
        };
        r as u16
    }
}

#[inline(always)]
pub fn lean_float_to_int32(a: f64) -> u32 {
    if a.is_nan() {
        0u32
    } else {
        let r: i32 = if a < -2_147_483_648.0 {
            i32::MIN
        } else if a >= 2_147_483_648.0 {
            i32::MAX
        } else {
            a as i32
        };
        r as u32
    }
}

#[inline(always)]
pub fn lean_float_to_int64(a: f64) -> u64 {
    if a.is_nan() {
        0u64
    } else {
        let r: i64 = if a < -9_223_372_036_854_775_808.0 {
            i64::MIN
        } else if a >= 9_223_372_036_854_775_808.0 {
            i64::MAX
        } else {
            a as i64
        };
        r as u64
    }
}

#[inline(always)]
pub fn lean_float_to_isize(a: f64) -> usize {
    lean_float_to_int64(a) as usize
}

// ============================================================================
// Unsigned int to Float64
// ============================================================================

#[inline(always)]
pub fn lean_uint8_to_float(a: u8) -> f64 {
    a as f64
}

#[inline(always)]
pub fn lean_uint16_to_float(a: u16) -> f64 {
    a as f64
}

#[inline(always)]
pub fn lean_uint32_to_float(a: u32) -> f64 {
    a as f64
}

#[inline(always)]
pub fn lean_uint64_to_float(a: u64) -> f64 {
    a as f64
}

#[inline(always)]
pub fn lean_usize_to_float(a: usize) -> f64 {
    a as f64
}

// ============================================================================
// Signed int to Float64 (input is unsigned representation)
// ============================================================================

#[inline(always)]
pub fn lean_int8_to_float(a: u8) -> f64 {
    (a as i8) as f64
}

#[inline(always)]
pub fn lean_int16_to_float(a: u16) -> f64 {
    (a as i16) as f64
}

#[inline(always)]
pub fn lean_int32_to_float(a: u32) -> f64 {
    (a as i32) as f64
}

#[inline(always)]
pub fn lean_int64_to_float(a: u64) -> f64 {
    (a as i64) as f64
}

#[inline(always)]
pub fn lean_isize_to_float(a: usize) -> f64 {
    (a as isize) as f64
}

// ============================================================================
// Float32 ctor get/set
// ============================================================================

/// Read an f32 from the scalar area of a ctor at the given byte offset.
#[inline(always)]
pub unsafe fn lean_ctor_get_float32(o: *mut LeanObject, offset: u32) -> f32 {
    let num_objs = (*o).other as usize;
    let base = (o as *const u8).add(LEAN_OBJECT_SIZE).add(num_objs * 8);
    let ptr = base.add(offset as usize) as *const f32;
    std::ptr::read_unaligned(ptr)
}

/// Write an f32 to the scalar area of a ctor at the given byte offset.
#[inline(always)]
pub unsafe fn lean_ctor_set_float32(o: *mut LeanObject, offset: u32, v: f32) {
    let num_objs = (*o).other as usize;
    let base = (o as *mut u8).add(LEAN_OBJECT_SIZE).add(num_objs * 8);
    let ptr = base.add(offset as usize) as *mut f32;
    std::ptr::write_unaligned(ptr, v);
}

// ============================================================================
// Float32 boxing/unboxing
// ============================================================================

/// Box an f32 as a Lean object: ctor with tag 0, 0 obj fields, 4 bytes scalar.
#[inline]
pub unsafe fn lean_box_float32(v: f32) -> *mut LeanObject {
    let r = crate::lean_alloc_ctor(0, 0, std::mem::size_of::<f32>() as u32);
    lean_ctor_set_float32(r, 0, v);
    r
}

/// Unbox an f32 from a Lean object.
#[inline]
pub unsafe fn lean_unbox_float32(o: *mut LeanObject) -> f32 {
    lean_ctor_get_float32(o, 0)
}

// ============================================================================
// Float32 arithmetic
// ============================================================================

#[inline(always)]
pub fn lean_float32_add(a: f32, b: f32) -> f32 {
    a + b
}

#[inline(always)]
pub fn lean_float32_sub(a: f32, b: f32) -> f32 {
    a - b
}

#[inline(always)]
pub fn lean_float32_mul(a: f32, b: f32) -> f32 {
    a * b
}

#[inline(always)]
pub fn lean_float32_div(a: f32, b: f32) -> f32 {
    a / b
}

#[inline(always)]
pub fn lean_float32_negate(a: f32) -> f32 {
    -a
}

// ============================================================================
// Float32 comparison (return u8 boolean)
// ============================================================================

#[inline(always)]
pub fn lean_float32_beq(a: f32, b: f32) -> u8 {
    (a == b) as u8
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lean_float32_decLe(a: f32, b: f32) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lean_float32_decLt(a: f32, b: f32) -> u8 {
    (a < b) as u8
}

// ============================================================================
// Float32 classification
// ============================================================================

#[inline(always)]
pub fn lean_float32_isnan(a: f32) -> u8 {
    a.is_nan() as u8
}

#[inline(always)]
pub fn lean_float32_isfinite(a: f32) -> u8 {
    a.is_finite() as u8
}

#[inline(always)]
pub fn lean_float32_isinf(a: f32) -> u8 {
    a.is_infinite() as u8
}

// ============================================================================
// Float32 bit conversion
// ============================================================================

#[inline(always)]
pub fn lean_float32_of_bits(u: u32) -> f32 {
    f32::from_bits(u)
}

#[inline(always)]
pub fn lean_float32_to_bits(d: f32) -> u32 {
    d.to_bits()
}

// ============================================================================
// Float32 string conversion
// ============================================================================

/// Convert an f32 to a Lean string object.
pub unsafe fn lean_float32_to_string(a: f32) -> *mut LeanObject {
    let s = if a.is_nan() {
        "NaN".to_string()
    } else if a == f32::INFINITY {
        "inf".to_string()
    } else if a == f32::NEG_INFINITY {
        "-inf".to_string()
    } else if a == 0.0 && a.is_sign_negative() {
        "-0.000000".to_string()
    } else {
        format!("{:.6}", a)
    };
    crate::string::lean_mk_string(&s)
}

// ============================================================================
// Float32 frexp / scaleb
// ============================================================================

/// `frexp(a)` for f32. Returns `(frac, exp)` as a Lean pair `(Float32, Int)`.
pub unsafe fn lean_float32_frexp(a: f32) -> *mut LeanObject {
    let (frac, exp) = frexp_impl_f32(a);
    let pair = crate::lean_alloc_ctor(0, 2, 0);
    crate::lean_ctor_set(pair, 0, lean_box_float32(frac));
    crate::lean_ctor_set(pair, 1, lean_small_int_to_obj(exp));
    pair
}

/// `scaleb(a, b)` for f32. Computes `a * 2^b` where `b` is a boxed Lean Int.
pub unsafe fn lean_float32_scaleb(a: f32, b: *mut LeanObject) -> f32 {
    let exp = lean_obj_to_int32(b);
    ldexp_impl_f32(a, exp)
}

// ============================================================================
// Float32 to unsigned integer conversions (with clamping)
// ============================================================================

#[inline(always)]
pub fn lean_float32_to_uint8(a: f32) -> u8 {
    if !(a >= 0.0) {
        0
    } else if a < 256.0 {
        a as u8
    } else {
        u8::MAX
    }
}

#[inline(always)]
pub fn lean_float32_to_uint16(a: f32) -> u16 {
    if !(a >= 0.0) {
        0
    } else if a < 65536.0 {
        a as u16
    } else {
        u16::MAX
    }
}

#[inline(always)]
pub fn lean_float32_to_uint32(a: f32) -> u32 {
    if !(a >= 0.0) {
        0
    } else if a < 4_294_967_296.0 {
        a as u32
    } else {
        u32::MAX
    }
}

#[inline(always)]
pub fn lean_float32_to_uint64(a: f32) -> u64 {
    if !(a >= 0.0) {
        0
    } else if a < 18_446_744_073_709_551_616.0 {
        a as u64
    } else {
        u64::MAX
    }
}

#[inline(always)]
pub fn lean_float32_to_usize(a: f32) -> usize {
    lean_float32_to_uint64(a) as usize
}

// ============================================================================
// Float32 to signed integer conversions (with clamping, NaN -> 0)
// ============================================================================

#[inline(always)]
pub fn lean_float32_to_int8(a: f32) -> u8 {
    if a.is_nan() {
        0u8
    } else {
        let r: i8 = if a < -128.0 {
            i8::MIN
        } else if a >= 128.0 {
            i8::MAX
        } else {
            a as i8
        };
        r as u8
    }
}

#[inline(always)]
pub fn lean_float32_to_int16(a: f32) -> u16 {
    if a.is_nan() {
        0u16
    } else {
        let r: i16 = if a < -32768.0 {
            i16::MIN
        } else if a >= 32768.0 {
            i16::MAX
        } else {
            a as i16
        };
        r as u16
    }
}

#[inline(always)]
pub fn lean_float32_to_int32(a: f32) -> u32 {
    if a.is_nan() {
        0u32
    } else {
        let r: i32 = if a < -2_147_483_648.0 {
            i32::MIN
        } else if a >= 2_147_483_648.0 {
            i32::MAX
        } else {
            a as i32
        };
        r as u32
    }
}

#[inline(always)]
pub fn lean_float32_to_int64(a: f32) -> u64 {
    if a.is_nan() {
        0u64
    } else {
        let r: i64 = if a < -9_223_372_036_854_775_808.0 {
            i64::MIN
        } else if a >= 9_223_372_036_854_775_808.0 {
            i64::MAX
        } else {
            a as i64
        };
        r as u64
    }
}

#[inline(always)]
pub fn lean_float32_to_isize(a: f32) -> usize {
    lean_float32_to_int64(a) as usize
}

// ============================================================================
// Unsigned int to Float32
// ============================================================================

#[inline(always)]
pub fn lean_uint8_to_float32(a: u8) -> f32 {
    a as f32
}

#[inline(always)]
pub fn lean_uint16_to_float32(a: u16) -> f32 {
    a as f32
}

#[inline(always)]
pub fn lean_uint32_to_float32(a: u32) -> f32 {
    a as f32
}

#[inline(always)]
pub fn lean_uint64_to_float32(a: u64) -> f32 {
    a as f32
}

#[inline(always)]
pub fn lean_usize_to_float32(a: usize) -> f32 {
    a as f32
}

// ============================================================================
// Signed int to Float32 (input is unsigned representation)
// ============================================================================

#[inline(always)]
pub fn lean_int8_to_float32(a: u8) -> f32 {
    (a as i8) as f32
}

#[inline(always)]
pub fn lean_int16_to_float32(a: u16) -> f32 {
    (a as i16) as f32
}

#[inline(always)]
pub fn lean_int32_to_float32(a: u32) -> f32 {
    (a as i32) as f32
}

#[inline(always)]
pub fn lean_int64_to_float32(a: u64) -> f32 {
    (a as i64) as f32
}

#[inline(always)]
pub fn lean_isize_to_float32(a: usize) -> f32 {
    (a as isize) as f32
}

// ============================================================================
// Cross-conversion between Float64 and Float32
// ============================================================================

#[inline(always)]
pub fn lean_float_to_float32(a: f64) -> f32 {
    a as f32
}

#[inline(always)]
pub fn lean_float32_to_float(a: f32) -> f64 {
    a as f64
}

// ============================================================================
// C math library function stubs (f64)
//
// These are referenced by name in generated Rust code from the Lean emitter.
// The Init.Data.Float module declares them via @[extern "sin"], etc.
// ============================================================================

#[inline(always)]
pub fn sin(a: f64) -> f64 {
    a.sin()
}

#[inline(always)]
pub fn cos(a: f64) -> f64 {
    a.cos()
}

#[inline(always)]
pub fn tan(a: f64) -> f64 {
    a.tan()
}

#[inline(always)]
pub fn asin(a: f64) -> f64 {
    a.asin()
}

#[inline(always)]
pub fn acos(a: f64) -> f64 {
    a.acos()
}

#[inline(always)]
pub fn atan(a: f64) -> f64 {
    a.atan()
}

#[inline(always)]
pub fn atan2(a: f64, b: f64) -> f64 {
    a.atan2(b)
}

#[inline(always)]
pub fn sinh(a: f64) -> f64 {
    a.sinh()
}

#[inline(always)]
pub fn cosh(a: f64) -> f64 {
    a.cosh()
}

#[inline(always)]
pub fn tanh(a: f64) -> f64 {
    a.tanh()
}

#[inline(always)]
pub fn asinh(a: f64) -> f64 {
    a.asinh()
}

#[inline(always)]
pub fn acosh(a: f64) -> f64 {
    a.acosh()
}

#[inline(always)]
pub fn atanh(a: f64) -> f64 {
    a.atanh()
}

#[inline(always)]
pub fn exp(a: f64) -> f64 {
    a.exp()
}

#[inline(always)]
pub fn exp2(a: f64) -> f64 {
    a.exp2()
}

#[inline(always)]
pub fn log(a: f64) -> f64 {
    a.ln()
}

#[inline(always)]
pub fn log2(a: f64) -> f64 {
    a.log2()
}

#[inline(always)]
pub fn log10(a: f64) -> f64 {
    a.log10()
}

#[inline(always)]
pub fn pow(a: f64, b: f64) -> f64 {
    a.powf(b)
}

#[inline(always)]
pub fn sqrt(a: f64) -> f64 {
    a.sqrt()
}

#[inline(always)]
pub fn cbrt(a: f64) -> f64 {
    a.cbrt()
}

#[inline(always)]
pub fn ceil(a: f64) -> f64 {
    a.ceil()
}

#[inline(always)]
pub fn floor(a: f64) -> f64 {
    a.floor()
}

#[inline(always)]
pub fn round(a: f64) -> f64 {
    a.round()
}

#[inline(always)]
pub fn fabs(a: f64) -> f64 {
    a.abs()
}

#[inline(always)]
pub fn fmod(a: f64, b: f64) -> f64 {
    a % b
}

// ============================================================================
// C math library function stubs (f32)
// ============================================================================

#[inline(always)]
pub fn sinf(a: f32) -> f32 {
    a.sin()
}

#[inline(always)]
pub fn cosf(a: f32) -> f32 {
    a.cos()
}

#[inline(always)]
pub fn tanf(a: f32) -> f32 {
    a.tan()
}

#[inline(always)]
pub fn asinf(a: f32) -> f32 {
    a.asin()
}

#[inline(always)]
pub fn acosf(a: f32) -> f32 {
    a.acos()
}

#[inline(always)]
pub fn atanf(a: f32) -> f32 {
    a.atan()
}

#[inline(always)]
pub fn atan2f(a: f32, b: f32) -> f32 {
    a.atan2(b)
}

#[inline(always)]
pub fn sinhf(a: f32) -> f32 {
    a.sinh()
}

#[inline(always)]
pub fn coshf(a: f32) -> f32 {
    a.cosh()
}

#[inline(always)]
pub fn tanhf(a: f32) -> f32 {
    a.tanh()
}

#[inline(always)]
pub fn asinhf(a: f32) -> f32 {
    a.asinh()
}

#[inline(always)]
pub fn acoshf(a: f32) -> f32 {
    a.acosh()
}

#[inline(always)]
pub fn atanhf(a: f32) -> f32 {
    a.atanh()
}

#[inline(always)]
pub fn expf(a: f32) -> f32 {
    a.exp()
}

#[inline(always)]
pub fn exp2f(a: f32) -> f32 {
    a.exp2()
}

#[inline(always)]
pub fn logf(a: f32) -> f32 {
    a.ln()
}

#[inline(always)]
pub fn log2f(a: f32) -> f32 {
    a.log2()
}

#[inline(always)]
pub fn log10f(a: f32) -> f32 {
    a.log10()
}

#[inline(always)]
pub fn powf(a: f32, b: f32) -> f32 {
    a.powf(b)
}

#[inline(always)]
pub fn sqrtf(a: f32) -> f32 {
    a.sqrt()
}

#[inline(always)]
pub fn cbrtf(a: f32) -> f32 {
    a.cbrt()
}

#[inline(always)]
pub fn ceilf(a: f32) -> f32 {
    a.ceil()
}

#[inline(always)]
pub fn floorf(a: f32) -> f32 {
    a.floor()
}

#[inline(always)]
pub fn roundf(a: f32) -> f32 {
    a.round()
}

#[inline(always)]
pub fn fabsf(a: f32) -> f32 {
    a.abs()
}

#[inline(always)]
pub fn fmodf(a: f32, b: f32) -> f32 {
    a % b
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Float64 boxing/unboxing --

    #[test]
    fn box_unbox_float64_roundtrip() {
        unsafe {
            let v = 3.14159265358979;
            let boxed = lean_box_float(v);
            let result = lean_unbox_float(boxed);
            assert_eq!(result, v);
            crate::lean_dec(boxed);
        }
    }

    #[test]
    fn box_unbox_float64_special_values() {
        unsafe {
            for &v in &[0.0, -0.0, f64::INFINITY, f64::NEG_INFINITY] {
                let boxed = lean_box_float(v);
                let result = lean_unbox_float(boxed);
                assert_eq!(result.to_bits(), v.to_bits());
                crate::lean_dec(boxed);
            }
            // NaN
            let boxed = lean_box_float(f64::NAN);
            assert!(lean_unbox_float(boxed).is_nan());
            crate::lean_dec(boxed);
        }
    }

    // -- Float64 arithmetic --

    #[test]
    fn float64_arithmetic() {
        assert_eq!(lean_float_add(1.5, 2.5), 4.0);
        assert_eq!(lean_float_sub(5.0, 3.0), 2.0);
        assert_eq!(lean_float_mul(2.0, 3.0), 6.0);
        assert_eq!(lean_float_div(6.0, 2.0), 3.0);
        assert_eq!(lean_float_negate(1.0), -1.0);
    }

    #[test]
    fn float64_div_by_zero() {
        assert!(lean_float_div(1.0, 0.0).is_infinite());
        assert!(lean_float_div(0.0, 0.0).is_nan());
    }

    // -- Float64 comparison --

    #[test]
    fn float64_comparison() {
        assert_eq!(lean_float_beq(1.0, 1.0), 1);
        assert_eq!(lean_float_beq(1.0, 2.0), 0);
        assert_eq!(lean_float_beq(f64::NAN, f64::NAN), 0);
        assert_eq!(lean_float_decLe(1.0, 2.0), 1);
        assert_eq!(lean_float_decLe(2.0, 2.0), 1);
        assert_eq!(lean_float_decLe(3.0, 2.0), 0);
        assert_eq!(lean_float_decLt(1.0, 2.0), 1);
        assert_eq!(lean_float_decLt(2.0, 2.0), 0);
    }

    // -- Float64 classification --

    #[test]
    fn float64_classification() {
        assert_eq!(lean_float_isnan(f64::NAN), 1);
        assert_eq!(lean_float_isnan(1.0), 0);
        assert_eq!(lean_float_isfinite(1.0), 1);
        assert_eq!(lean_float_isfinite(f64::INFINITY), 0);
        assert_eq!(lean_float_isinf(f64::INFINITY), 1);
        assert_eq!(lean_float_isinf(1.0), 0);
    }

    // -- Float64 bit conversion --

    #[test]
    fn float64_bit_roundtrip() {
        let v = 3.14;
        assert_eq!(lean_float_of_bits(lean_float_to_bits(v)), v);
    }

    // -- Float64 string conversion --

    #[test]
    fn float64_to_string() {
        unsafe {
            let check = |v: f64, expected: &str| {
                let obj = lean_float_to_string(v);
                let s = crate::string::lean_string_to_str(obj);
                assert_eq!(
                    s, expected,
                    "lean_float_to_string({}) = {:?}, expected {:?}",
                    v, s, expected
                );
                crate::lean_dec(obj);
            };
            check(3.14, "3.140000");
            check(0.0, "0.000000");
            check(-0.0, "-0.000000");
            check(f64::NAN, "NaN");
            check(f64::INFINITY, "inf");
            check(f64::NEG_INFINITY, "-inf");
            check(1.0, "1.000000");
            check(-42.5, "-42.500000");
        }
    }

    // -- Float64 to unsigned int conversions --

    #[test]
    fn float64_to_uint8_clamping() {
        assert_eq!(lean_float_to_uint8(0.0), 0);
        assert_eq!(lean_float_to_uint8(42.9), 42);
        assert_eq!(lean_float_to_uint8(255.0), 255);
        assert_eq!(lean_float_to_uint8(256.0), 255);
        assert_eq!(lean_float_to_uint8(-1.0), 0);
        assert_eq!(lean_float_to_uint8(f64::NAN), 0);
        assert_eq!(lean_float_to_uint8(f64::INFINITY), 255);
    }

    #[test]
    fn float64_to_uint16_clamping() {
        assert_eq!(lean_float_to_uint16(1000.0), 1000);
        assert_eq!(lean_float_to_uint16(70000.0), u16::MAX);
        assert_eq!(lean_float_to_uint16(-1.0), 0);
    }

    #[test]
    fn float64_to_uint32_clamping() {
        assert_eq!(lean_float_to_uint32(1000.0), 1000);
        assert_eq!(lean_float_to_uint32(5_000_000_000.0), u32::MAX);
        assert_eq!(lean_float_to_uint32(-1.0), 0);
    }

    #[test]
    fn float64_to_uint64_clamping() {
        assert_eq!(lean_float_to_uint64(1000.0), 1000);
        assert_eq!(lean_float_to_uint64(-1.0), 0);
        assert_eq!(lean_float_to_uint64(f64::NAN), 0);
    }

    // -- Float64 to signed int conversions --

    #[test]
    fn float64_to_int8_clamping() {
        assert_eq!(lean_float_to_int8(42.0), 42u8);
        assert_eq!(lean_float_to_int8(-1.0), 255u8); // -1 as u8
        assert_eq!(lean_float_to_int8(-128.0), 128u8); // i8::MIN as u8
        assert_eq!(lean_float_to_int8(-200.0), 128u8); // clamped to i8::MIN
        assert_eq!(lean_float_to_int8(200.0), 127u8); // clamped to i8::MAX
        assert_eq!(lean_float_to_int8(f64::NAN), 0u8);
    }

    #[test]
    fn float64_to_int32_clamping() {
        assert_eq!(lean_float_to_int32(42.0), 42u32);
        assert_eq!(lean_float_to_int32(-1.0), (-1i32) as u32);
        assert_eq!(lean_float_to_int32(3_000_000_000.0), i32::MAX as u32);
        assert_eq!(lean_float_to_int32(f64::NAN), 0u32);
    }

    // -- Unsigned int to Float64 --

    #[test]
    fn uint_to_float64() {
        assert_eq!(lean_uint8_to_float(42), 42.0);
        assert_eq!(lean_uint16_to_float(1000), 1000.0);
        assert_eq!(lean_uint32_to_float(100000), 100000.0);
        assert_eq!(lean_uint64_to_float(123456), 123456.0);
        assert_eq!(lean_usize_to_float(999), 999.0);
    }

    // -- Signed int to Float64 --

    #[test]
    fn int_to_float64() {
        assert_eq!(lean_int8_to_float(255), -1.0); // 255u8 as i8 = -1
        assert_eq!(lean_int16_to_float(65535), -1.0); // 65535u16 as i16 = -1
        assert_eq!(lean_int32_to_float((-1i32) as u32), -1.0);
        assert_eq!(lean_int64_to_float((-42i64) as u64), -42.0);
    }

    // -- Float32 boxing/unboxing --

    #[test]
    fn box_unbox_float32_roundtrip() {
        unsafe {
            let v: f32 = 2.71828;
            let boxed = lean_box_float32(v);
            let result = lean_unbox_float32(boxed);
            assert_eq!(result, v);
            crate::lean_dec(boxed);
        }
    }

    #[test]
    fn box_unbox_float32_special_values() {
        unsafe {
            for &v in &[0.0f32, -0.0, f32::INFINITY, f32::NEG_INFINITY] {
                let boxed = lean_box_float32(v);
                let result = lean_unbox_float32(boxed);
                assert_eq!(result.to_bits(), v.to_bits());
                crate::lean_dec(boxed);
            }
            let boxed = lean_box_float32(f32::NAN);
            assert!(lean_unbox_float32(boxed).is_nan());
            crate::lean_dec(boxed);
        }
    }

    // -- Float32 arithmetic --

    #[test]
    fn float32_arithmetic() {
        assert_eq!(lean_float32_add(1.5, 2.5), 4.0);
        assert_eq!(lean_float32_sub(5.0, 3.0), 2.0);
        assert_eq!(lean_float32_mul(2.0, 3.0), 6.0);
        assert_eq!(lean_float32_div(6.0, 2.0), 3.0);
        assert_eq!(lean_float32_negate(1.0), -1.0);
    }

    // -- Float32 comparison --

    #[test]
    fn float32_comparison() {
        assert_eq!(lean_float32_beq(1.0, 1.0), 1);
        assert_eq!(lean_float32_beq(1.0, 2.0), 0);
        assert_eq!(lean_float32_beq(f32::NAN, f32::NAN), 0);
        assert_eq!(lean_float32_decLe(1.0, 2.0), 1);
        assert_eq!(lean_float32_decLt(1.0, 2.0), 1);
        assert_eq!(lean_float32_decLt(2.0, 2.0), 0);
    }

    // -- Float32 classification --

    #[test]
    fn float32_classification() {
        assert_eq!(lean_float32_isnan(f32::NAN), 1);
        assert_eq!(lean_float32_isnan(1.0), 0);
        assert_eq!(lean_float32_isfinite(1.0), 1);
        assert_eq!(lean_float32_isfinite(f32::INFINITY), 0);
        assert_eq!(lean_float32_isinf(f32::INFINITY), 1);
        assert_eq!(lean_float32_isinf(1.0), 0);
    }

    // -- Float32 bit conversion --

    #[test]
    fn float32_bit_roundtrip() {
        let v: f32 = 3.14;
        assert_eq!(lean_float32_of_bits(lean_float32_to_bits(v)), v);
    }

    // -- Float32 string conversion --

    #[test]
    fn float32_to_string() {
        unsafe {
            let check = |v: f32, expected: &str| {
                let obj = lean_float32_to_string(v);
                let s = crate::string::lean_string_to_str(obj);
                assert_eq!(
                    s, expected,
                    "lean_float32_to_string({}) = {:?}, expected {:?}",
                    v, s, expected
                );
                crate::lean_dec(obj);
            };
            check(f32::NAN, "NaN");
            check(f32::INFINITY, "inf");
            check(f32::NEG_INFINITY, "-inf");
            check(-0.0, "-0.000000");
            check(1.0, "1.000000");
        }
    }

    // -- Float32 to unsigned int conversions --

    #[test]
    fn float32_to_uint8_clamping() {
        assert_eq!(lean_float32_to_uint8(0.0), 0);
        assert_eq!(lean_float32_to_uint8(42.9), 42);
        assert_eq!(lean_float32_to_uint8(256.0), 255);
        assert_eq!(lean_float32_to_uint8(-1.0), 0);
        assert_eq!(lean_float32_to_uint8(f32::NAN), 0);
    }

    #[test]
    fn float32_to_uint32_clamping() {
        assert_eq!(lean_float32_to_uint32(1000.0), 1000);
        assert_eq!(lean_float32_to_uint32(-1.0), 0);
    }

    // -- Float32 to signed int conversions --

    #[test]
    fn float32_to_int8_clamping() {
        assert_eq!(lean_float32_to_int8(42.0), 42u8);
        assert_eq!(lean_float32_to_int8(-1.0), 255u8);
        assert_eq!(lean_float32_to_int8(-200.0), 128u8);
        assert_eq!(lean_float32_to_int8(200.0), 127u8);
        assert_eq!(lean_float32_to_int8(f32::NAN), 0u8);
    }

    // -- Int to Float32 --

    #[test]
    fn uint_to_float32() {
        assert_eq!(lean_uint8_to_float32(42), 42.0);
        assert_eq!(lean_uint16_to_float32(1000), 1000.0);
        assert_eq!(lean_uint32_to_float32(100000), 100000.0);
    }

    #[test]
    fn int_to_float32() {
        assert_eq!(lean_int8_to_float32(255), -1.0);
        assert_eq!(lean_int16_to_float32(65535), -1.0);
    }

    // -- Cross conversion --

    #[test]
    fn cross_conversion() {
        let v: f64 = 3.14;
        let f32_val = lean_float_to_float32(v);
        let back = lean_float32_to_float(f32_val);
        // f64 -> f32 -> f64 loses precision
        assert!((back - v).abs() < 1e-6);
    }

    // -- frexp / ldexp (f64) --

    #[test]
    fn frexp_impl_f64_basic() {
        let (frac, exp) = frexp_impl_f64(8.0);
        assert_eq!(frac, 0.5);
        assert_eq!(exp, 4);
        // 0.5 * 2^4 = 8.0
        assert_eq!(ldexp_impl_f64(frac, exp), 8.0);
    }

    #[test]
    fn frexp_impl_f64_one() {
        let (frac, exp) = frexp_impl_f64(1.0);
        assert_eq!(frac, 0.5);
        assert_eq!(exp, 1);
    }

    #[test]
    fn frexp_impl_f64_negative() {
        let (frac, exp) = frexp_impl_f64(-4.0);
        assert_eq!(frac, -0.5);
        assert_eq!(exp, 3);
        assert_eq!(ldexp_impl_f64(frac, exp), -4.0);
    }

    #[test]
    fn frexp_impl_f64_zero() {
        let (frac, exp) = frexp_impl_f64(0.0);
        assert_eq!(frac, 0.0);
        assert_eq!(exp, 0);
    }

    #[test]
    fn frexp_impl_f64_nan() {
        let (frac, exp) = frexp_impl_f64(f64::NAN);
        assert!(frac.is_nan());
        assert_eq!(exp, 0);
    }

    #[test]
    fn frexp_impl_f64_inf() {
        let (frac, exp) = frexp_impl_f64(f64::INFINITY);
        assert!(frac.is_infinite());
        assert_eq!(exp, 0);
    }

    #[test]
    fn frexp_ldexp_f64_roundtrip() {
        for &v in &[0.1, 0.5, 1.0, 2.0, 100.0, -42.5, 1e-10, 1e100] {
            let (frac, exp) = frexp_impl_f64(v);
            assert!(
                frac.abs() >= 0.5 && frac.abs() < 1.0,
                "frexp({}) = ({}, {}): frac out of range",
                v,
                frac,
                exp
            );
            let reconstructed = ldexp_impl_f64(frac, exp);
            assert_eq!(reconstructed, v, "ldexp(frexp({})) != {}", v, v);
        }
    }

    // -- frexp / ldexp (f32) --

    #[test]
    fn frexp_impl_f32_basic() {
        let (frac, exp) = frexp_impl_f32(8.0f32);
        assert_eq!(frac, 0.5);
        assert_eq!(exp, 4);
        assert_eq!(ldexp_impl_f32(frac, exp), 8.0);
    }

    #[test]
    fn frexp_ldexp_f32_roundtrip() {
        for &v in &[0.1f32, 0.5, 1.0, 2.0, 100.0, -42.5] {
            let (frac, exp) = frexp_impl_f32(v);
            assert!(
                frac.abs() >= 0.5 && frac.abs() < 1.0,
                "frexp({}) = ({}, {}): frac out of range",
                v,
                frac,
                exp
            );
            let reconstructed = ldexp_impl_f32(frac, exp);
            assert_eq!(reconstructed, v, "ldexp(frexp({})) != {}", v, v);
        }
    }

    // -- lean_float_frexp / lean_float_scaleb --

    #[test]
    fn float_frexp_pair() {
        unsafe {
            let pair = lean_float_frexp(8.0);
            // pair is (Float, Int) -- ctor tag 0, 2 fields
            assert_eq!((*pair).tag, 0);
            let float_obj = crate::lean_ctor_get(pair, 0);
            let frac = lean_unbox_float(float_obj);
            assert_eq!(frac, 0.5);
            // Clean up
            crate::lean_dec(pair);
        }
    }

    #[test]
    fn float32_frexp_pair() {
        unsafe {
            let pair = lean_float32_frexp(8.0f32);
            assert_eq!((*pair).tag, 0);
            let float_obj = crate::lean_ctor_get(pair, 0);
            let frac = lean_unbox_float32(float_obj);
            assert_eq!(frac, 0.5);
            crate::lean_dec(pair);
        }
    }

    // -- ldexp via scaleb with boxed int --

    #[test]
    fn float_scaleb_basic() {
        unsafe {
            // scaleb(0.5, 4) = 0.5 * 2^4 = 8.0
            // Use lean_int64_to_int to create the boxed Int (small values become scalars)
            let int_obj = crate::int::lean_int64_to_int(4);
            let result = lean_float_scaleb(0.5, int_obj);
            assert_eq!(result, 8.0);
            crate::lean_dec(int_obj);
        }
    }

    #[test]
    fn float_scaleb_scalar_int() {
        unsafe {
            // If the Int is passed as a scalar (small non-negative), extract directly
            let int_obj = crate::lean_box(3);
            let result = lean_float_scaleb(1.0, int_obj);
            assert_eq!(result, 8.0); // 1.0 * 2^3 = 8.0
        }
    }

    // -- C math stubs --

    #[test]
    fn math_stubs_f64() {
        let pi = std::f64::consts::PI;
        assert!((sin(pi / 2.0) - 1.0).abs() < 1e-10);
        assert!((cos(0.0) - 1.0).abs() < 1e-10);
        assert!((tan(0.0)).abs() < 1e-10);
        assert!((exp(0.0) - 1.0).abs() < 1e-10);
        assert!((log(1.0)).abs() < 1e-10);
        assert_eq!(sqrt(4.0), 2.0);
        assert_eq!(ceil(1.5), 2.0);
        assert_eq!(floor(1.5), 1.0);
        assert_eq!(round(1.5), 2.0);
        assert_eq!(fabs(-3.0), 3.0);
        assert_eq!(pow(2.0, 10.0), 1024.0);
        assert_eq!(fmod(5.0, 3.0), 2.0);
    }

    #[test]
    fn math_stubs_f32() {
        let pi = std::f32::consts::PI;
        assert!((sinf(pi / 2.0) - 1.0).abs() < 1e-5);
        assert!((cosf(0.0) - 1.0).abs() < 1e-5);
        assert_eq!(sqrtf(4.0), 2.0);
        assert_eq!(ceilf(1.5), 2.0);
        assert_eq!(floorf(1.5), 1.0);
        assert_eq!(roundf(1.5), 2.0);
        assert_eq!(fabsf(-3.0), 3.0);
        assert_eq!(fmodf(5.0, 3.0), 2.0);
    }

    // -- ctor get/set float in mixed objects --

    #[test]
    fn ctor_float64_in_mixed_object() {
        unsafe {
            // Ctor with 1 obj field + 8 bytes scalar (f64)
            let obj = crate::lean_alloc_ctor(0, 1, 8);
            crate::lean_ctor_set(obj, 0, crate::lean_box(42));
            lean_ctor_set_float(obj, 0, 3.14);
            assert_eq!(crate::lean_unbox(crate::lean_ctor_get(obj, 0)), 42);
            assert_eq!(lean_ctor_get_float(obj, 0), 3.14);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn ctor_float32_in_mixed_object() {
        unsafe {
            // Ctor with 1 obj field + 4 bytes scalar (f32)
            let obj = crate::lean_alloc_ctor(0, 1, 4);
            crate::lean_ctor_set(obj, 0, crate::lean_box(99));
            lean_ctor_set_float32(obj, 0, 2.5);
            assert_eq!(crate::lean_unbox(crate::lean_ctor_get(obj, 0)), 99);
            assert_eq!(lean_ctor_get_float32(obj, 0), 2.5);
            crate::lean_dec(obj);
        }
    }
}
