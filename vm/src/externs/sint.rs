//! Signed fixed-width integer operations (Int8, Int16, Int32, Int64, ISize)
//!
//! These are two's complement signed integers stored as boxed scalars.

use super::int::int_to_i64;
use super::{
    signed_binop, signed_bitop, signed_cmp, signed_complement, signed_div, signed_mod,
    signed_of_nat, signed_shift, signed_unary, ExternFn, Result,
};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Int8 operations
        ("lean_int8_of_nat", int8_of_nat as ExternFn),
        ("lean_int8_of_int", int8_of_int),
        ("lean_int8_neg", int8_neg),
        ("lean_int8_add", int8_add),
        ("lean_int8_sub", int8_sub),
        ("lean_int8_mul", int8_mul),
        ("lean_int8_div", int8_div),
        ("lean_int8_mod", int8_mod),
        ("lean_int8_abs", int8_abs),
        ("lean_int8_land", int8_land),
        ("lean_int8_lor", int8_lor),
        ("lean_int8_xor", int8_xor),
        ("lean_int8_shift_left", int8_shift_left),
        ("lean_int8_shift_right", int8_shift_right),
        ("lean_int8_complement", int8_complement),
        ("lean_int8_dec_eq", int8_dec_eq),
        ("lean_int8_dec_lt", int8_dec_lt),
        ("lean_int8_dec_le", int8_dec_le),
        // Int16 operations
        ("lean_int16_of_nat", int16_of_nat),
        ("lean_int16_of_int", int16_of_int),
        ("lean_int16_neg", int16_neg),
        ("lean_int16_add", int16_add),
        ("lean_int16_sub", int16_sub),
        ("lean_int16_mul", int16_mul),
        ("lean_int16_div", int16_div),
        ("lean_int16_mod", int16_mod),
        ("lean_int16_abs", int16_abs),
        ("lean_int16_land", int16_land),
        ("lean_int16_lor", int16_lor),
        ("lean_int16_xor", int16_xor),
        ("lean_int16_shift_left", int16_shift_left),
        ("lean_int16_shift_right", int16_shift_right),
        ("lean_int16_complement", int16_complement),
        ("lean_int16_dec_eq", int16_dec_eq),
        ("lean_int16_dec_lt", int16_dec_lt),
        ("lean_int16_dec_le", int16_dec_le),
        // Int32 operations
        ("lean_int32_of_nat", int32_of_nat),
        ("lean_int32_of_int", int32_of_int),
        ("lean_int32_neg", int32_neg),
        ("lean_int32_add", int32_add),
        ("lean_int32_sub", int32_sub),
        ("lean_int32_mul", int32_mul),
        ("lean_int32_div", int32_div),
        ("lean_int32_mod", int32_mod),
        ("lean_int32_abs", int32_abs),
        ("lean_int32_land", int32_land),
        ("lean_int32_lor", int32_lor),
        ("lean_int32_xor", int32_xor),
        ("lean_int32_shift_left", int32_shift_left),
        ("lean_int32_shift_right", int32_shift_right),
        ("lean_int32_complement", int32_complement),
        ("lean_int32_dec_eq", int32_dec_eq),
        ("lean_int32_dec_lt", int32_dec_lt),
        ("lean_int32_dec_le", int32_dec_le),
        // Int64 operations
        ("lean_int64_of_nat", int64_of_nat),
        ("lean_int64_of_int", int64_of_int),
        ("lean_int64_neg", int64_neg),
        ("lean_int64_add", int64_add),
        ("lean_int64_sub", int64_sub),
        ("lean_int64_mul", int64_mul),
        ("lean_int64_div", int64_div),
        ("lean_int64_mod", int64_mod),
        ("lean_int64_abs", int64_abs),
        ("lean_int64_land", int64_land),
        ("lean_int64_lor", int64_lor),
        ("lean_int64_xor", int64_xor),
        ("lean_int64_shift_left", int64_shift_left),
        ("lean_int64_shift_right", int64_shift_right),
        ("lean_int64_complement", int64_complement),
        ("lean_int64_dec_eq", int64_dec_eq),
        ("lean_int64_dec_lt", int64_dec_lt),
        ("lean_int64_dec_le", int64_dec_le),
        // ISize operations
        ("lean_isize_of_nat", isize_of_nat),
        ("lean_isize_of_int", isize_of_int),
        ("lean_isize_neg", isize_neg),
        ("lean_isize_add", isize_add),
        ("lean_isize_sub", isize_sub),
        ("lean_isize_mul", isize_mul),
        ("lean_isize_div", isize_div),
        ("lean_isize_mod", isize_mod),
        ("lean_isize_abs", isize_abs),
        ("lean_isize_land", isize_land),
        ("lean_isize_lor", isize_lor),
        ("lean_isize_xor", isize_xor),
        ("lean_isize_shift_left", isize_shift_left),
        ("lean_isize_shift_right", isize_shift_right),
        ("lean_isize_complement", isize_complement),
        ("lean_isize_dec_eq", isize_dec_eq),
        ("lean_isize_dec_lt", isize_dec_lt),
        ("lean_isize_dec_le", isize_dec_le),
    ]
}

// ============================================================================
// Int8 Operations
// ============================================================================

signed_of_nat!(int8_of_nat, i8, u8);
signed_unary!(int8_neg, i8, u8, wrapping_neg);
signed_binop!(int8_add, i8, u8, wrapping_add);
signed_binop!(int8_sub, i8, u8, wrapping_sub);
signed_binop!(int8_mul, i8, u8, wrapping_mul);
signed_div!(int8_div, i8, u8);
signed_mod!(int8_mod, i8, u8);
signed_unary!(int8_abs, i8, u8, wrapping_abs);
signed_bitop!(int8_land, i8, u8, &);
signed_bitop!(int8_lor, i8, u8, |);
signed_bitop!(int8_xor, i8, u8, ^);
signed_shift!(int8_shift_left, i8, u8, 8, <<);
signed_shift!(int8_shift_right, i8, u8, 8, >>);
signed_complement!(int8_complement, i8, u8);
signed_cmp!(int8_dec_eq, i8, ==);
signed_cmp!(int8_dec_lt, i8, <);
signed_cmp!(int8_dec_le, i8, <=);

fn int8_of_int(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr()) as i8;
        Ok(LeanValue::from_raw(lean_box(i as u8 as usize)))
    }
}

// ============================================================================
// Int16 Operations
// ============================================================================

signed_of_nat!(int16_of_nat, i16, u16);
signed_unary!(int16_neg, i16, u16, wrapping_neg);
signed_binop!(int16_add, i16, u16, wrapping_add);
signed_binop!(int16_sub, i16, u16, wrapping_sub);
signed_binop!(int16_mul, i16, u16, wrapping_mul);
signed_div!(int16_div, i16, u16);
signed_mod!(int16_mod, i16, u16);
signed_unary!(int16_abs, i16, u16, wrapping_abs);
signed_bitop!(int16_land, i16, u16, &);
signed_bitop!(int16_lor, i16, u16, |);
signed_bitop!(int16_xor, i16, u16, ^);
signed_shift!(int16_shift_left, i16, u16, 16, <<);
signed_shift!(int16_shift_right, i16, u16, 16, >>);
signed_complement!(int16_complement, i16, u16);
signed_cmp!(int16_dec_eq, i16, ==);
signed_cmp!(int16_dec_lt, i16, <);
signed_cmp!(int16_dec_le, i16, <=);

fn int16_of_int(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr()) as i16;
        Ok(LeanValue::from_raw(lean_box(i as u16 as usize)))
    }
}

// ============================================================================
// Int32 Operations
// ============================================================================

signed_of_nat!(int32_of_nat, i32, u32);
signed_unary!(int32_neg, i32, u32, wrapping_neg);
signed_binop!(int32_add, i32, u32, wrapping_add);
signed_binop!(int32_sub, i32, u32, wrapping_sub);
signed_binop!(int32_mul, i32, u32, wrapping_mul);
signed_div!(int32_div, i32, u32);
signed_mod!(int32_mod, i32, u32);
signed_unary!(int32_abs, i32, u32, wrapping_abs);
signed_bitop!(int32_land, i32, u32, &);
signed_bitop!(int32_lor, i32, u32, |);
signed_bitop!(int32_xor, i32, u32, ^);
signed_shift!(int32_shift_left, i32, u32, 32, <<);
signed_shift!(int32_shift_right, i32, u32, 32, >>);
signed_complement!(int32_complement, i32, u32);
signed_cmp!(int32_dec_eq, i32, ==);
signed_cmp!(int32_dec_lt, i32, <);
signed_cmp!(int32_dec_le, i32, <=);

fn int32_of_int(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr()) as i32;
        Ok(LeanValue::from_raw(lean_box(i as u32 as usize)))
    }
}

// ============================================================================
// Int64 Operations
// ============================================================================

signed_of_nat!(int64_of_nat, i64, u64);
signed_unary!(int64_neg, i64, u64, wrapping_neg);
signed_binop!(int64_add, i64, u64, wrapping_add);
signed_binop!(int64_sub, i64, u64, wrapping_sub);
signed_binop!(int64_mul, i64, u64, wrapping_mul);
signed_div!(int64_div, i64, u64);
signed_mod!(int64_mod, i64, u64);
signed_unary!(int64_abs, i64, u64, wrapping_abs);
signed_bitop!(int64_land, i64, u64, &);
signed_bitop!(int64_lor, i64, u64, |);
signed_bitop!(int64_xor, i64, u64, ^);
signed_shift!(int64_shift_left, i64, u64, 64, <<);
signed_shift!(int64_shift_right, i64, u64, 64, >>);
signed_complement!(int64_complement, i64, u64);
signed_cmp!(int64_dec_eq, i64, ==);
signed_cmp!(int64_dec_lt, i64, <);
signed_cmp!(int64_dec_le, i64, <=);

fn int64_of_int(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box(i as u64 as usize)))
    }
}

// ============================================================================
// ISize Operations
// ============================================================================

signed_of_nat!(isize_of_nat, isize, usize);
signed_unary!(isize_neg, isize, usize, wrapping_neg);
signed_binop!(isize_add, isize, usize, wrapping_add);
signed_binop!(isize_sub, isize, usize, wrapping_sub);
signed_binop!(isize_mul, isize, usize, wrapping_mul);
signed_div!(isize_div, isize, usize);
signed_mod!(isize_mod, isize, usize);
signed_unary!(isize_abs, isize, usize, wrapping_abs);
signed_bitop!(isize_land, isize, usize, &);
signed_bitop!(isize_lor, isize, usize, |);
signed_bitop!(isize_xor, isize, usize, ^);
signed_complement!(isize_complement, isize, usize);
signed_cmp!(isize_dec_eq, isize, ==);
signed_cmp!(isize_dec_lt, isize, <);
signed_cmp!(isize_dec_le, isize, <=);

fn isize_of_int(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr()) as isize;
        Ok(LeanValue::from_raw(lean_box(i as usize)))
    }
}

// ISize shift operations need dynamic bit width
fn isize_shift_left(args: &[LeanValue]) -> Result<LeanValue> {
    let a = lean_unbox(args[0].as_ptr()) as isize;
    let b = lean_unbox(args[1].as_ptr());
    let bits = std::mem::size_of::<isize>() * 8;
    unsafe { Ok(LeanValue::from_raw(lean_box((a << (b % bits)) as usize))) }
}

fn isize_shift_right(args: &[LeanValue]) -> Result<LeanValue> {
    let a = lean_unbox(args[0].as_ptr()) as isize;
    let b = lean_unbox(args[1].as_ptr());
    let bits = std::mem::size_of::<isize>() * 8;
    unsafe { Ok(LeanValue::from_raw(lean_box((a >> (b % bits)) as usize))) }
}
