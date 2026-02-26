//! UInt8/16/32/64/USize operations for the Lean runtime.

use crate::object::LeanObject;
use crate::r#box::*;

// ============================================================================
// UInt8
// ============================================================================

#[inline(always)]
pub unsafe fn lean_uint8_of_nat(a: *mut LeanObject) -> u8 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u8
    } else {
        let val = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        val.to_u8().unwrap_or(0)
    }
}

/// Owned version: dec-refs `a`.
#[inline]
pub unsafe fn lean_uint8_of_nat_mk(a: *mut LeanObject) -> u8 {
    let r = lean_uint8_of_nat(a);
    crate::lean_dec(a);
    r
}

#[inline(always)]
pub fn lean_uint8_to_nat(a: u8) -> *mut LeanObject {
    lean_box(a as usize)
}

#[inline(always)]
pub fn lean_uint8_dec_eq(a: u8, b: u8) -> u8 {
    (a == b) as u8
}

#[inline(always)]
pub fn lean_uint8_dec_lt(a: u8, b: u8) -> u8 {
    (a < b) as u8
}

#[inline(always)]
pub fn lean_uint8_dec_le(a: u8, b: u8) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
pub fn lean_uint8_add(a: u8, b: u8) -> u8 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_uint8_sub(a: u8, b: u8) -> u8 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_uint8_mul(a: u8, b: u8) -> u8 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_uint8_div(a: u8, b: u8) -> u8 {
    if b == 0 {
        0
    } else {
        a / b
    }
}

#[inline(always)]
pub fn lean_uint8_mod(a: u8, b: u8) -> u8 {
    if b == 0 {
        0
    } else {
        a % b
    }
}

#[inline(always)]
pub fn lean_uint8_land(a: u8, b: u8) -> u8 {
    a & b
}

#[inline(always)]
pub fn lean_uint8_lor(a: u8, b: u8) -> u8 {
    a | b
}

#[inline(always)]
pub fn lean_uint8_lxor(a: u8, b: u8) -> u8 {
    a ^ b
}

#[inline(always)]
pub fn lean_uint8_shift_left(a: u8, b: u8) -> u8 {
    a.wrapping_shl(b as u32)
}

#[inline(always)]
pub fn lean_uint8_shift_right(a: u8, b: u8) -> u8 {
    a.wrapping_shr(b as u32)
}

#[inline(always)]
pub fn lean_uint8_complement(a: u8) -> u8 {
    !a
}

#[inline(always)]
pub fn lean_uint8_to_uint32(a: u8) -> u32 {
    a as u32
}

#[inline(always)]
pub fn lean_uint8_to_uint16(a: u8) -> u16 {
    a as u16
}

#[inline(always)]
pub fn lean_uint8_to_uint64(a: u8) -> u64 {
    a as u64
}

// ============================================================================
// UInt16
// ============================================================================

#[inline(always)]
pub unsafe fn lean_uint16_of_nat(a: *mut LeanObject) -> u16 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u16
    } else {
        let val = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        val.to_u16().unwrap_or(0)
    }
}

#[inline]
pub unsafe fn lean_uint16_of_nat_mk(a: *mut LeanObject) -> u16 {
    let r = lean_uint16_of_nat(a);
    crate::lean_dec(a);
    r
}

#[inline(always)]
pub fn lean_uint16_to_nat(a: u16) -> *mut LeanObject {
    lean_box(a as usize)
}

#[inline(always)]
pub fn lean_uint16_dec_eq(a: u16, b: u16) -> u8 {
    (a == b) as u8
}

#[inline(always)]
pub fn lean_uint16_dec_lt(a: u16, b: u16) -> u8 {
    (a < b) as u8
}

#[inline(always)]
pub fn lean_uint16_dec_le(a: u16, b: u16) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
pub fn lean_uint16_add(a: u16, b: u16) -> u16 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_uint16_sub(a: u16, b: u16) -> u16 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_uint16_mul(a: u16, b: u16) -> u16 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_uint16_div(a: u16, b: u16) -> u16 {
    if b == 0 {
        0
    } else {
        a / b
    }
}

#[inline(always)]
pub fn lean_uint16_mod(a: u16, b: u16) -> u16 {
    if b == 0 {
        0
    } else {
        a % b
    }
}

#[inline(always)]
pub fn lean_uint16_land(a: u16, b: u16) -> u16 {
    a & b
}

#[inline(always)]
pub fn lean_uint16_lor(a: u16, b: u16) -> u16 {
    a | b
}

#[inline(always)]
pub fn lean_uint16_lxor(a: u16, b: u16) -> u16 {
    a ^ b
}

#[inline(always)]
pub fn lean_uint16_shift_left(a: u16, b: u16) -> u16 {
    a.wrapping_shl(b as u32)
}

#[inline(always)]
pub fn lean_uint16_shift_right(a: u16, b: u16) -> u16 {
    a.wrapping_shr(b as u32)
}

#[inline(always)]
pub fn lean_uint16_complement(a: u16) -> u16 {
    !a
}

#[inline(always)]
pub fn lean_uint16_to_uint8(a: u16) -> u8 {
    a as u8
}

#[inline(always)]
pub fn lean_uint16_to_uint32(a: u16) -> u32 {
    a as u32
}

#[inline(always)]
pub fn lean_uint16_to_uint64(a: u16) -> u64 {
    a as u64
}

// ============================================================================
// UInt32
// ============================================================================

#[inline(always)]
pub unsafe fn lean_uint32_of_nat(a: *mut LeanObject) -> u32 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u32
    } else {
        let val = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        val.to_u32().unwrap_or(0)
    }
}

#[inline]
pub unsafe fn lean_uint32_of_nat_mk(a: *mut LeanObject) -> u32 {
    let r = lean_uint32_of_nat(a);
    crate::lean_dec(a);
    r
}

#[inline(always)]
pub fn lean_uint32_to_nat(a: u32) -> *mut LeanObject {
    lean_box(a as usize)
}

#[inline(always)]
pub fn lean_uint32_dec_le(a: u32, b: u32) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
pub fn lean_uint32_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_uint32_sub(a: u32, b: u32) -> u32 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_uint32_mul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_uint32_div(a: u32, b: u32) -> u32 {
    if b == 0 {
        0
    } else {
        a / b
    }
}

#[inline(always)]
pub fn lean_uint32_mod(a: u32, b: u32) -> u32 {
    if b == 0 {
        0
    } else {
        a % b
    }
}

#[inline(always)]
pub fn lean_uint32_land(a: u32, b: u32) -> u32 {
    a & b
}

#[inline(always)]
pub fn lean_uint32_lor(a: u32, b: u32) -> u32 {
    a | b
}

#[inline(always)]
pub fn lean_uint32_lxor(a: u32, b: u32) -> u32 {
    a ^ b
}

#[inline(always)]
pub fn lean_uint32_shift_left(a: u32, b: u32) -> u32 {
    a.wrapping_shl(b)
}

#[inline(always)]
pub fn lean_uint32_shift_right(a: u32, b: u32) -> u32 {
    a.wrapping_shr(b)
}

#[inline(always)]
pub fn lean_uint32_complement(a: u32) -> u32 {
    !a
}

#[inline(always)]
pub fn lean_uint32_to_uint8(a: u32) -> u8 {
    a as u8
}

#[inline(always)]
pub fn lean_uint32_to_uint16(a: u32) -> u16 {
    a as u16
}

#[inline(always)]
pub fn lean_uint32_to_uint64(a: u32) -> u64 {
    a as u64
}

// ============================================================================
// UInt64
// ============================================================================

#[inline(always)]
pub unsafe fn lean_uint64_of_nat(a: *mut LeanObject) -> u64 {
    if lean_is_scalar(a) {
        lean_unbox(a) as u64
    } else {
        let val = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        val.to_u64().unwrap_or(0)
    }
}

#[inline]
pub unsafe fn lean_uint64_of_nat_mk(a: *mut LeanObject) -> u64 {
    let r = lean_uint64_of_nat(a);
    crate::lean_dec(a);
    r
}

#[inline(always)]
pub fn lean_uint64_to_nat(a: u64) -> *mut LeanObject {
    crate::nat::lean_uint64_to_nat(a)
}

#[inline(always)]
pub fn lean_uint64_dec_eq(a: u64, b: u64) -> u8 {
    (a == b) as u8
}

#[inline(always)]
pub fn lean_uint64_dec_lt(a: u64, b: u64) -> u8 {
    (a < b) as u8
}

#[inline(always)]
pub fn lean_uint64_dec_le(a: u64, b: u64) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
pub fn lean_uint64_add(a: u64, b: u64) -> u64 {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_uint64_sub(a: u64, b: u64) -> u64 {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_uint64_mul(a: u64, b: u64) -> u64 {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_uint64_div(a: u64, b: u64) -> u64 {
    if b == 0 {
        0
    } else {
        a / b
    }
}

#[inline(always)]
pub fn lean_uint64_mod(a: u64, b: u64) -> u64 {
    if b == 0 {
        0
    } else {
        a % b
    }
}

#[inline(always)]
pub fn lean_uint64_land(a: u64, b: u64) -> u64 {
    a & b
}

#[inline(always)]
pub fn lean_uint64_lor(a: u64, b: u64) -> u64 {
    a | b
}

#[inline(always)]
pub fn lean_uint64_lxor(a: u64, b: u64) -> u64 {
    a ^ b
}

#[inline(always)]
pub fn lean_uint64_shift_left(a: u64, b: u64) -> u64 {
    a.wrapping_shl(b as u32)
}

#[inline(always)]
pub fn lean_uint64_shift_right(a: u64, b: u64) -> u64 {
    a.wrapping_shr(b as u32)
}

#[inline(always)]
pub fn lean_uint64_complement(a: u64) -> u64 {
    !a
}

#[inline(always)]
pub fn lean_uint64_to_uint8(a: u64) -> u8 {
    a as u8
}

#[inline(always)]
pub fn lean_uint64_to_uint16(a: u64) -> u16 {
    a as u16
}

#[inline(always)]
pub fn lean_uint64_to_uint32(a: u64) -> u32 {
    a as u32
}

/// Hash mixing for UInt64 (used by Hashable instances).
#[inline(always)]
pub fn lean_uint64_mix_hash(a: u64, b: u64) -> u64 {
    // Same mix function as the C runtime
    let mut h = a;
    h ^= b
        .wrapping_add(0x9e3779b9)
        .wrapping_add(h << 6)
        .wrapping_add(h >> 2);
    h
}

// ============================================================================
// USize
// ============================================================================

#[inline(always)]
pub unsafe fn lean_usize_of_nat(a: *mut LeanObject) -> usize {
    if lean_is_scalar(a) {
        lean_unbox(a)
    } else {
        let val = crate::bignat::lean_bignat_value(a);
        use num_traits::ToPrimitive;
        val.to_usize().unwrap_or(0)
    }
}

#[inline]
pub unsafe fn lean_usize_of_nat_mk(a: *mut LeanObject) -> usize {
    let r = lean_usize_of_nat(a);
    crate::lean_dec(a);
    r
}

#[inline(always)]
pub fn lean_usize_to_nat(a: usize) -> *mut LeanObject {
    crate::nat::lean_usize_to_nat(a)
}

#[inline(always)]
pub fn lean_usize_dec_eq(a: usize, b: usize) -> u8 {
    (a == b) as u8
}

#[inline(always)]
pub fn lean_usize_dec_lt(a: usize, b: usize) -> u8 {
    (a < b) as u8
}

#[inline(always)]
pub fn lean_usize_dec_le(a: usize, b: usize) -> u8 {
    (a <= b) as u8
}

#[inline(always)]
pub fn lean_usize_add(a: usize, b: usize) -> usize {
    a.wrapping_add(b)
}

#[inline(always)]
pub fn lean_usize_sub(a: usize, b: usize) -> usize {
    a.wrapping_sub(b)
}

#[inline(always)]
pub fn lean_usize_mul(a: usize, b: usize) -> usize {
    a.wrapping_mul(b)
}

#[inline(always)]
pub fn lean_usize_div(a: usize, b: usize) -> usize {
    if b == 0 {
        0
    } else {
        a / b
    }
}

#[inline(always)]
pub fn lean_usize_mod(a: usize, b: usize) -> usize {
    if b == 0 {
        0
    } else {
        a % b
    }
}

#[inline(always)]
pub fn lean_usize_land(a: usize, b: usize) -> usize {
    a & b
}

#[inline(always)]
pub fn lean_usize_lor(a: usize, b: usize) -> usize {
    a | b
}

#[inline(always)]
pub fn lean_usize_lxor(a: usize, b: usize) -> usize {
    a ^ b
}

#[inline(always)]
pub fn lean_usize_shift_left(a: usize, b: usize) -> usize {
    a.wrapping_shl(b as u32)
}

#[inline(always)]
pub fn lean_usize_shift_right(a: usize, b: usize) -> usize {
    a.wrapping_shr(b as u32)
}

#[inline(always)]
pub fn lean_usize_complement(a: usize) -> usize {
    !a
}

#[inline(always)]
pub fn lean_usize_mix_hash(a: usize, b: usize) -> usize {
    lean_uint64_mix_hash(a as u64, b as u64) as usize
}

// ============================================================================
// Negation (wrapping)
// ============================================================================

#[inline(always)]
pub fn lean_uint8_neg(a: u8) -> u8 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_uint16_neg(a: u16) -> u16 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_uint32_neg(a: u32) -> u32 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_uint64_neg(a: u64) -> u64 {
    a.wrapping_neg()
}

#[inline(always)]
pub fn lean_usize_neg(a: usize) -> usize {
    a.wrapping_neg()
}

// ============================================================================
// Log2
// ============================================================================

#[inline(always)]
pub fn lean_uint8_log2(a: u8) -> u8 {
    let mut v = a;
    let mut r: u8 = 0;
    while v >= 2 {
        r += 1;
        v /= 2;
    }
    r
}

#[inline(always)]
pub fn lean_uint16_log2(a: u16) -> u16 {
    let mut v = a;
    let mut r: u16 = 0;
    while v >= 2 {
        r += 1;
        v /= 2;
    }
    r
}

#[inline(always)]
pub fn lean_uint32_log2(a: u32) -> u32 {
    let mut v = a;
    let mut r: u32 = 0;
    while v >= 2 {
        r += 1;
        v /= 2;
    }
    r
}

#[inline(always)]
pub fn lean_uint64_log2(a: u64) -> u64 {
    let mut v = a;
    let mut r: u64 = 0;
    while v >= 2 {
        r += 1;
        v /= 2;
    }
    r
}

#[inline(always)]
pub fn lean_usize_log2(a: usize) -> usize {
    let mut v = a;
    let mut r: usize = 0;
    while v >= 2 {
        r += 1;
        v /= 2;
    }
    r
}

// ============================================================================
// To/from usize conversions
// ============================================================================

#[inline(always)]
pub fn lean_uint8_to_usize(a: u8) -> usize {
    a as usize
}

#[inline(always)]
pub fn lean_uint16_to_usize(a: u16) -> usize {
    a as usize
}

#[inline(always)]
pub fn lean_uint32_to_usize(a: u32) -> usize {
    a as usize
}

#[inline(always)]
pub fn lean_uint64_to_usize(a: u64) -> usize {
    a as usize
}

#[inline(always)]
pub fn lean_usize_to_uint8(a: usize) -> u8 {
    a as u8
}

#[inline(always)]
pub fn lean_usize_to_uint16(a: usize) -> u16 {
    a as u16
}

#[inline(always)]
pub fn lean_usize_to_uint32(a: usize) -> u32 {
    a as u32
}

#[inline(always)]
pub fn lean_usize_to_uint64(a: usize) -> u64 {
    a as u64
}

// ============================================================================
// XOR aliases (lean_uint*_xor -> lean_uint*_lxor)
// ============================================================================

#[inline(always)]
pub fn lean_uint8_xor(a: u8, b: u8) -> u8 {
    lean_uint8_lxor(a, b)
}

#[inline(always)]
pub fn lean_uint16_xor(a: u16, b: u16) -> u16 {
    lean_uint16_lxor(a, b)
}

#[inline(always)]
pub fn lean_uint32_xor(a: u32, b: u32) -> u32 {
    lean_uint32_lxor(a, b)
}

#[inline(always)]
pub fn lean_uint64_xor(a: u64, b: u64) -> u64 {
    lean_uint64_lxor(a, b)
}

#[inline(always)]
pub fn lean_usize_xor(a: usize, b: usize) -> usize {
    lean_usize_lxor(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uint8_of_nat_small() {
        unsafe {
            assert_eq!(lean_uint8_of_nat(lean_box(42)), 42u8);
            assert_eq!(lean_uint8_of_nat(lean_box(255)), 255u8);
            // Truncation
            assert_eq!(lean_uint8_of_nat(lean_box(256)), 0u8);
        }
    }

    #[test]
    fn uint32_of_nat_small() {
        unsafe {
            assert_eq!(lean_uint32_of_nat(lean_box(1000)), 1000u32);
        }
    }

    #[test]
    fn uint64_of_nat_small() {
        unsafe {
            assert_eq!(lean_uint64_of_nat(lean_box(123456)), 123456u64);
        }
    }

    #[test]
    fn uint8_to_nat_roundtrip() {
        let n = lean_uint8_to_nat(200);
        assert_eq!(lean_unbox(n), 200);
    }

    #[test]
    fn uint32_to_nat_roundtrip() {
        let n = lean_uint32_to_nat(0xCAFE);
        assert_eq!(lean_unbox(n), 0xCAFE);
    }

    #[test]
    fn uint64_mix_hash_deterministic() {
        let h1 = lean_uint64_mix_hash(1, 2);
        let h2 = lean_uint64_mix_hash(1, 2);
        assert_eq!(h1, h2);
        let h3 = lean_uint64_mix_hash(2, 1);
        assert_ne!(h1, h3);
    }

    #[test]
    fn uint8_arithmetic() {
        assert_eq!(lean_uint8_add(200, 100), 44); // wrapping
        assert_eq!(lean_uint8_sub(10, 20), 246); // wrapping
        assert_eq!(lean_uint8_mul(16, 16), 0); // wrapping
        assert_eq!(lean_uint8_div(100, 7), 14);
        assert_eq!(lean_uint8_mod(100, 7), 2);
        assert_eq!(lean_uint8_div(10, 0), 0);
    }

    #[test]
    fn uint8_bitwise() {
        assert_eq!(lean_uint8_land(0xF0, 0x0F), 0x00);
        assert_eq!(lean_uint8_lor(0xF0, 0x0F), 0xFF);
        assert_eq!(lean_uint8_lxor(0xFF, 0x0F), 0xF0);
        assert_eq!(lean_uint8_shift_left(1, 7), 128);
        assert_eq!(lean_uint8_shift_right(128, 7), 1);
        assert_eq!(lean_uint8_complement(0), 0xFF);
    }

    #[test]
    fn uint32_arithmetic() {
        assert_eq!(lean_uint32_add(u32::MAX, 1), 0);
        assert_eq!(lean_uint32_sub(0, 1), u32::MAX);
        assert_eq!(lean_uint32_div(100, 0), 0);
    }

    #[test]
    fn uint64_arithmetic() {
        assert_eq!(lean_uint64_add(u64::MAX, 1), 0);
        assert_eq!(lean_uint64_div(100, 0), 0);
    }

    #[test]
    fn usize_dec_comparisons() {
        assert_eq!(lean_usize_dec_eq(5, 5), 1);
        assert_eq!(lean_usize_dec_eq(5, 6), 0);
        assert_eq!(lean_usize_dec_lt(3, 5), 1);
        assert_eq!(lean_usize_dec_lt(5, 3), 0);
        assert_eq!(lean_usize_dec_le(5, 5), 1);
    }

    #[test]
    fn uint8_conversions() {
        assert_eq!(lean_uint8_to_uint32(42), 42u32);
        assert_eq!(lean_uint8_to_uint16(42), 42u16);
        assert_eq!(lean_uint8_to_uint64(42), 42u64);
    }

    #[test]
    fn uint32_conversions() {
        assert_eq!(lean_uint32_to_uint8(256), 0u8);
        assert_eq!(lean_uint32_to_uint16(0x1FFFF), 0xFFFFu16);
        assert_eq!(lean_uint32_to_uint64(42), 42u64);
    }

    #[test]
    fn uint64_conversions() {
        assert_eq!(lean_uint64_to_uint8(256), 0u8);
        assert_eq!(lean_uint64_to_uint32(u64::MAX), u32::MAX);
    }
}
