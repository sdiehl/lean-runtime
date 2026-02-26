//! Unsigned integer operations (UInt8, UInt16, UInt32, UInt64, USize)

use super::{
    scalar_binop, scalar_bitop, scalar_cmp, scalar_complement, scalar_div, scalar_mod,
    scalar_of_nat, scalar_shift, scalar_to_nat, ExternFn, Result,
};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // UInt8
        ("lean_uint8_of_nat_mk", uint8_of_nat_mk as ExternFn),
        ("lean_uint8_to_nat", uint8_to_nat),
        ("lean_uint8_of_nat", uint8_of_nat),
        ("lean_uint8_dec_eq", uint8_dec_eq),
        ("lean_uint8_dec_lt", uint8_dec_lt),
        ("lean_uint8_dec_le", uint8_dec_le),
        ("lean_uint8_add", uint8_add),
        ("lean_uint8_sub", uint8_sub),
        ("lean_uint8_mul", uint8_mul),
        ("lean_uint8_div", uint8_div),
        ("lean_uint8_mod", uint8_mod),
        ("lean_uint8_land", uint8_land),
        ("lean_uint8_lor", uint8_lor),
        ("lean_uint8_xor", uint8_xor),
        ("lean_uint8_shift_left", uint8_shift_left),
        ("lean_uint8_shift_right", uint8_shift_right),
        ("lean_uint8_complement", uint8_complement),
        // UInt16
        ("lean_uint16_of_nat_mk", uint16_of_nat_mk),
        ("lean_uint16_to_nat", uint16_to_nat),
        ("lean_uint16_of_nat", uint16_of_nat),
        ("lean_uint16_dec_eq", uint16_dec_eq),
        ("lean_uint16_dec_lt", uint16_dec_lt),
        ("lean_uint16_dec_le", uint16_dec_le),
        ("lean_uint16_add", uint16_add),
        ("lean_uint16_sub", uint16_sub),
        ("lean_uint16_mul", uint16_mul),
        ("lean_uint16_div", uint16_div),
        ("lean_uint16_mod", uint16_mod),
        ("lean_uint16_land", uint16_land),
        ("lean_uint16_lor", uint16_lor),
        ("lean_uint16_xor", uint16_xor),
        ("lean_uint16_shift_left", uint16_shift_left),
        ("lean_uint16_shift_right", uint16_shift_right),
        ("lean_uint16_complement", uint16_complement),
        // UInt32
        ("lean_uint32_of_nat_mk", uint32_of_nat_mk),
        ("lean_uint32_to_nat", uint32_to_nat),
        ("lean_uint32_of_nat", uint32_of_nat),
        ("lean_uint32_dec_eq", uint32_dec_eq),
        ("lean_uint32_dec_lt", uint32_dec_lt),
        ("lean_uint32_dec_le", uint32_dec_le),
        ("lean_uint32_add", uint32_add),
        ("lean_uint32_sub", uint32_sub),
        ("lean_uint32_mul", uint32_mul),
        ("lean_uint32_div", uint32_div),
        ("lean_uint32_mod", uint32_mod),
        ("lean_uint32_land", uint32_land),
        ("lean_uint32_lor", uint32_lor),
        ("lean_uint32_xor", uint32_xor),
        ("lean_uint32_shift_left", uint32_shift_left),
        ("lean_uint32_shift_right", uint32_shift_right),
        ("lean_uint32_complement", uint32_complement),
        // UInt64
        ("lean_uint64_of_nat_mk", uint64_of_nat_mk),
        ("lean_uint64_to_nat", uint64_to_nat),
        ("lean_uint64_of_nat", uint64_of_nat),
        ("lean_uint64_dec_eq", uint64_dec_eq),
        ("lean_uint64_dec_lt", uint64_dec_lt),
        ("lean_uint64_dec_le", uint64_dec_le),
        ("lean_uint64_add", uint64_add),
        ("lean_uint64_sub", uint64_sub),
        ("lean_uint64_mul", uint64_mul),
        ("lean_uint64_div", uint64_div),
        ("lean_uint64_mod", uint64_mod),
        ("lean_uint64_land", uint64_land),
        ("lean_uint64_lor", uint64_lor),
        ("lean_uint64_xor", uint64_xor),
        ("lean_uint64_shift_left", uint64_shift_left),
        ("lean_uint64_shift_right", uint64_shift_right),
        ("lean_uint64_complement", uint64_complement),
        ("lean_uint64_to_usize", uint64_to_usize),
        ("lean_uint64_mix_hash", uint64_mix_hash),
        // USize
        ("lean_usize_of_nat_mk", usize_of_nat_mk),
        ("lean_usize_to_nat", usize_to_nat),
        ("lean_usize_of_nat", usize_of_nat),
        ("lean_usize_dec_eq", usize_dec_eq),
        ("lean_usize_dec_lt", usize_dec_lt),
        ("lean_usize_dec_le", usize_dec_le),
        ("lean_usize_add", usize_add),
        ("lean_usize_sub", usize_sub),
        ("lean_usize_mul", usize_mul),
        ("lean_usize_div", usize_div),
        ("lean_usize_mod", usize_mod),
        ("lean_usize_land", usize_land),
        ("lean_usize_lor", usize_lor),
        ("lean_usize_xor", usize_xor),
        ("lean_usize_shift_left", usize_shift_left),
        ("lean_usize_shift_right", usize_shift_right),
        ("lean_usize_complement", usize_complement),
    ]
}

// ============================================================================
// UInt8 Operations
// ============================================================================

scalar_of_nat!(uint8_of_nat_mk, u8);
scalar_to_nat!(uint8_to_nat);
scalar_of_nat!(uint8_of_nat, u8);
scalar_cmp!(uint8_dec_eq, u8, ==);
scalar_cmp!(uint8_dec_lt, u8, <);
scalar_cmp!(uint8_dec_le, u8, <=);
scalar_binop!(uint8_add, u8, wrapping_add);
scalar_binop!(uint8_sub, u8, wrapping_sub);
scalar_binop!(uint8_mul, u8, wrapping_mul);
scalar_div!(uint8_div, u8);
scalar_mod!(uint8_mod, u8);
scalar_bitop!(uint8_land, u8, &);
scalar_bitop!(uint8_lor, u8, |);
scalar_bitop!(uint8_xor, u8, ^);
scalar_shift!(uint8_shift_left, u8, 8, <<);
scalar_shift!(uint8_shift_right, u8, 8, >>);
scalar_complement!(uint8_complement, u8);

// ============================================================================
// UInt16 Operations
// ============================================================================

scalar_of_nat!(uint16_of_nat_mk, u16);
scalar_to_nat!(uint16_to_nat);
scalar_of_nat!(uint16_of_nat, u16);
scalar_cmp!(uint16_dec_eq, u16, ==);
scalar_cmp!(uint16_dec_lt, u16, <);
scalar_cmp!(uint16_dec_le, u16, <=);
scalar_binop!(uint16_add, u16, wrapping_add);
scalar_binop!(uint16_sub, u16, wrapping_sub);
scalar_binop!(uint16_mul, u16, wrapping_mul);
scalar_div!(uint16_div, u16);
scalar_mod!(uint16_mod, u16);
scalar_bitop!(uint16_land, u16, &);
scalar_bitop!(uint16_lor, u16, |);
scalar_bitop!(uint16_xor, u16, ^);
scalar_shift!(uint16_shift_left, u16, 16, <<);
scalar_shift!(uint16_shift_right, u16, 16, >>);
scalar_complement!(uint16_complement, u16);

// ============================================================================
// UInt32 Operations
// ============================================================================

scalar_of_nat!(uint32_of_nat_mk, u32);
scalar_to_nat!(uint32_to_nat);
scalar_of_nat!(uint32_of_nat, u32);
scalar_cmp!(uint32_dec_eq, u32, ==);
scalar_cmp!(uint32_dec_lt, u32, <);
scalar_cmp!(uint32_dec_le, u32, <=);
scalar_binop!(uint32_add, u32, wrapping_add);
scalar_binop!(uint32_sub, u32, wrapping_sub);
scalar_binop!(uint32_mul, u32, wrapping_mul);
scalar_div!(uint32_div, u32);
scalar_mod!(uint32_mod, u32);
scalar_bitop!(uint32_land, u32, &);
scalar_bitop!(uint32_lor, u32, |);
scalar_bitop!(uint32_xor, u32, ^);
scalar_shift!(uint32_shift_left, u32, 32, <<);
scalar_shift!(uint32_shift_right, u32, 32, >>);
scalar_complement!(uint32_complement, u32);

// ============================================================================
// UInt64 Operations (uses runtime functions for big nat handling)
// ============================================================================

#[inline]
fn nat_to_u64(ptr: *mut LeanObject) -> u64 {
    unsafe { lean_uint64_of_nat(ptr) }
}

#[inline]
fn u64_to_nat(n: u64) -> *mut LeanObject {
    lean_uint64_to_nat(n)
}

fn uint64_of_nat_mk(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        Ok(LeanValue::from_raw(u64_to_nat(nat_to_u64(
            args[0].as_ptr(),
        ))))
    }
}

fn uint64_to_nat(args: &[LeanValue]) -> Result<LeanValue> {
    Ok(args[0].clone())
}

fn uint64_of_nat(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        Ok(LeanValue::from_raw(u64_to_nat(nat_to_u64(
            args[0].as_ptr(),
        ))))
    }
}

fn uint64_dec_eq(args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(
        nat_to_u64(args[0].as_ptr()) == nat_to_u64(args[1].as_ptr()),
    ))
}

fn uint64_dec_lt(args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(
        nat_to_u64(args[0].as_ptr()) < nat_to_u64(args[1].as_ptr()),
    ))
}

fn uint64_dec_le(args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(
        nat_to_u64(args[0].as_ptr()) <= nat_to_u64(args[1].as_ptr()),
    ))
}

fn uint64_add(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a.wrapping_add(b)))) }
}

fn uint64_sub(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a.wrapping_sub(b)))) }
}

fn uint64_mul(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a.wrapping_mul(b)))) }
}

fn uint64_div(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    let result = if b == 0 { 0 } else { a / b };
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(result))) }
}

fn uint64_mod(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    let result = if b == 0 { a } else { a % b };
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(result))) }
}

fn uint64_land(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a & b))) }
}

fn uint64_lor(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a | b))) }
}

fn uint64_xor(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a ^ b))) }
}

fn uint64_shift_left(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a << ((b % 64) as u32)))) }
}

fn uint64_shift_right(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(a >> ((b % 64) as u32)))) }
}

fn uint64_complement(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        Ok(LeanValue::from_raw(u64_to_nat(!nat_to_u64(
            args[0].as_ptr(),
        ))))
    }
}

fn uint64_to_usize(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        Ok(LeanValue::from_raw(lean_box(
            nat_to_u64(args[0].as_ptr()) as usize
        )))
    }
}

fn uint64_mix_hash(args: &[LeanValue]) -> Result<LeanValue> {
    let a = nat_to_u64(args[0].as_ptr());
    let b = nat_to_u64(args[1].as_ptr());
    unsafe { Ok(LeanValue::from_raw(u64_to_nat(lean_uint64_mix_hash(a, b)))) }
}

// ============================================================================
// USize Operations
// ============================================================================

#[inline]
fn nat_to_usize(ptr: *mut LeanObject) -> usize {
    unsafe { lean_usize_of_nat(ptr) }
}

fn usize_of_nat_mk(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        Ok(LeanValue::from_raw(lean_usize_to_nat(nat_to_usize(
            args[0].as_ptr(),
        ))))
    }
}

fn usize_to_nat(args: &[LeanValue]) -> Result<LeanValue> {
    Ok(args[0].clone())
}

fn usize_of_nat(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        Ok(LeanValue::from_raw(lean_usize_to_nat(nat_to_usize(
            args[0].as_ptr(),
        ))))
    }
}

scalar_cmp!(usize_dec_eq, usize, ==);
scalar_cmp!(usize_dec_lt, usize, <);
scalar_cmp!(usize_dec_le, usize, <=);
scalar_binop!(usize_add, usize, wrapping_add);
scalar_binop!(usize_sub, usize, wrapping_sub);
scalar_binop!(usize_mul, usize, wrapping_mul);
scalar_div!(usize_div, usize);
scalar_mod!(usize_mod, usize);
scalar_bitop!(usize_land, usize, &);
scalar_bitop!(usize_lor, usize, |);
scalar_bitop!(usize_xor, usize, ^);
scalar_complement!(usize_complement, usize);

fn usize_shift_left(args: &[LeanValue]) -> Result<LeanValue> {
    let a = lean_unbox(args[0].as_ptr());
    let b = lean_unbox(args[1].as_ptr());
    let shift = (b % (std::mem::size_of::<usize>() * 8)) as u32;
    unsafe { Ok(LeanValue::from_raw(lean_box(a << shift))) }
}

fn usize_shift_right(args: &[LeanValue]) -> Result<LeanValue> {
    let a = lean_unbox(args[0].as_ptr());
    let b = lean_unbox(args[1].as_ptr());
    let shift = (b % (std::mem::size_of::<usize>() * 8)) as u32;
    unsafe { Ok(LeanValue::from_raw(lean_box(a >> shift))) }
}
