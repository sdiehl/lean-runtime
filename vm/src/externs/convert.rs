//! Type conversion functions between numeric types

use super::int::i64_to_int;
use super::{
    bool_to_uint, sint_to_float32, sint_to_float64, sint_to_sint, uint_to_float32, uint_to_float64,
    uint_to_uint, ExternFn, Result,
};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Bool to integer conversions
        ("lean_bool_to_uint8", bool_to_uint8 as ExternFn),
        ("lean_bool_to_uint16", bool_to_uint16),
        ("lean_bool_to_uint32", bool_to_uint32),
        ("lean_bool_to_uint64", bool_to_uint64),
        ("lean_bool_to_usize", bool_to_usize),
        ("lean_bool_to_int8", bool_to_int8),
        ("lean_bool_to_int16", bool_to_int16),
        ("lean_bool_to_int32", bool_to_int32),
        ("lean_bool_to_int64", bool_to_int64),
        ("lean_bool_to_isize", bool_to_isize),
        // UInt8 conversions
        ("lean_uint8_to_uint16", uint8_to_uint16),
        ("lean_uint8_to_uint32", uint8_to_uint32),
        ("lean_uint8_to_uint64", uint8_to_uint64),
        ("lean_uint8_to_usize", uint8_to_usize),
        ("lean_uint8_to_float", uint8_to_float),
        ("lean_uint8_to_float32", uint8_to_float32),
        // UInt16 conversions
        ("lean_uint16_to_uint8", uint16_to_uint8),
        ("lean_uint16_to_uint32", uint16_to_uint32),
        ("lean_uint16_to_uint64", uint16_to_uint64),
        ("lean_uint16_to_usize", uint16_to_usize),
        ("lean_uint16_to_float", uint16_to_float),
        ("lean_uint16_to_float32", uint16_to_float32),
        // UInt32 conversions
        ("lean_uint32_to_uint8", uint32_to_uint8),
        ("lean_uint32_to_uint16", uint32_to_uint16),
        ("lean_uint32_to_uint64", uint32_to_uint64),
        ("lean_uint32_to_usize", uint32_to_usize),
        ("lean_uint32_to_float", uint32_to_float),
        ("lean_uint32_to_float32", uint32_to_float32),
        // UInt64 conversions
        ("lean_uint64_to_uint8", uint64_to_uint8),
        ("lean_uint64_to_uint16", uint64_to_uint16),
        ("lean_uint64_to_uint32", uint64_to_uint32),
        ("lean_uint64_to_float", uint64_to_float),
        ("lean_uint64_to_float32", uint64_to_float32),
        // USize conversions
        ("lean_usize_to_uint8", usize_to_uint8),
        ("lean_usize_to_uint16", usize_to_uint16),
        ("lean_usize_to_uint32", usize_to_uint32),
        ("lean_usize_to_uint64", usize_to_uint64),
        ("lean_usize_to_float", usize_to_float),
        ("lean_usize_to_float32", usize_to_float32),
        // Int8 conversions
        ("lean_int8_to_int", int8_to_int),
        ("lean_int8_to_int16", int8_to_int16),
        ("lean_int8_to_int32", int8_to_int32),
        ("lean_int8_to_int64", int8_to_int64),
        ("lean_int8_to_isize", int8_to_isize),
        ("lean_int8_to_float", int8_to_float),
        ("lean_int8_to_float32", int8_to_float32),
        // Int16 conversions
        ("lean_int16_to_int", int16_to_int),
        ("lean_int16_to_int8", int16_to_int8),
        ("lean_int16_to_int32", int16_to_int32),
        ("lean_int16_to_int64", int16_to_int64),
        ("lean_int16_to_isize", int16_to_isize),
        ("lean_int16_to_float", int16_to_float),
        ("lean_int16_to_float32", int16_to_float32),
        // Int32 conversions
        ("lean_int32_to_int", int32_to_int),
        ("lean_int32_to_int8", int32_to_int8),
        ("lean_int32_to_int16", int32_to_int16),
        ("lean_int32_to_int64", int32_to_int64),
        ("lean_int32_to_isize", int32_to_isize),
        ("lean_int32_to_float", int32_to_float),
        ("lean_int32_to_float32", int32_to_float32),
        // Int64 conversions
        ("lean_int64_to_int_sint", int64_to_int),
        ("lean_int64_to_int8", int64_to_int8),
        ("lean_int64_to_int16", int64_to_int16),
        ("lean_int64_to_int32", int64_to_int32),
        ("lean_int64_to_isize", int64_to_isize),
        ("lean_int64_to_float", int64_to_float),
        ("lean_int64_to_float32", int64_to_float32),
        // ISize conversions
        ("lean_isize_to_int", isize_to_int),
        ("lean_isize_to_int8", isize_to_int8),
        ("lean_isize_to_int16", isize_to_int16),
        ("lean_isize_to_int32", isize_to_int32),
        ("lean_isize_to_int64", isize_to_int64),
        ("lean_isize_to_float", isize_to_float),
        ("lean_isize_to_float32", isize_to_float32),
        // Float (f64) conversions
        ("lean_float_to_uint8", float_to_uint8),
        ("lean_float_to_uint16", float_to_uint16),
        ("lean_float_to_uint32", float_to_uint32),
        ("lean_float_to_usize", float_to_usize),
        ("lean_float_to_int8", float_to_int8),
        ("lean_float_to_int16", float_to_int16),
        ("lean_float_to_int32", float_to_int32),
        ("lean_float_to_int64", float_to_int64),
        ("lean_float_to_isize", float_to_isize),
        ("lean_float_to_float32", float_to_float32),
        ("lean_float_to_bits", float_to_bits),
        ("lean_float_of_bits", float_of_bits),
        // Float32 (f32) conversions
        ("lean_float32_to_uint8", float32_to_uint8),
        ("lean_float32_to_uint16", float32_to_uint16),
        ("lean_float32_to_uint32", float32_to_uint32),
        ("lean_float32_to_usize", float32_to_usize),
        ("lean_float32_to_int8", float32_to_int8),
        ("lean_float32_to_int16", float32_to_int16),
        ("lean_float32_to_int32", float32_to_int32),
        ("lean_float32_to_int64", float32_to_int64),
        ("lean_float32_to_isize", float32_to_isize),
        ("lean_float32_to_float", float32_to_float),
        ("lean_float32_to_bits", float32_to_bits),
        ("lean_float32_of_bits", float32_of_bits),
    ]
}

// ============================================================================
// Bool conversions
// ============================================================================

bool_to_uint!(bool_to_uint8);
bool_to_uint!(bool_to_uint16);
bool_to_uint!(bool_to_uint32);
bool_to_uint!(bool_to_uint64);
bool_to_uint!(bool_to_usize);
bool_to_uint!(bool_to_int8);
bool_to_uint!(bool_to_int16);
bool_to_uint!(bool_to_int32);
bool_to_uint!(bool_to_int64);
bool_to_uint!(bool_to_isize);

// ============================================================================
// UInt conversions
// ============================================================================

uint_to_uint!(uint8_to_uint16, u8, u16);
uint_to_uint!(uint8_to_uint32, u8, u32);
uint_to_uint!(uint8_to_uint64, u8, u64);
uint_to_uint!(uint8_to_usize, u8, usize);
uint_to_float64!(uint8_to_float, u8);
uint_to_float32!(uint8_to_float32, u8);

uint_to_uint!(uint16_to_uint8, u16, u8);
uint_to_uint!(uint16_to_uint32, u16, u32);
uint_to_uint!(uint16_to_uint64, u16, u64);
uint_to_uint!(uint16_to_usize, u16, usize);
uint_to_float64!(uint16_to_float, u16);
uint_to_float32!(uint16_to_float32, u16);

uint_to_uint!(uint32_to_uint8, u32, u8);
uint_to_uint!(uint32_to_uint16, u32, u16);
uint_to_uint!(uint32_to_uint64, u32, u64);
uint_to_uint!(uint32_to_usize, u32, usize);
uint_to_float64!(uint32_to_float, u32);
uint_to_float32!(uint32_to_float32, u32);

uint_to_uint!(uint64_to_uint8, u64, u8);
uint_to_uint!(uint64_to_uint16, u64, u16);
uint_to_uint!(uint64_to_uint32, u64, u32);
uint_to_float64!(uint64_to_float, u64);
uint_to_float32!(uint64_to_float32, u64);

uint_to_uint!(usize_to_uint8, usize, u8);
uint_to_uint!(usize_to_uint16, usize, u16);
uint_to_uint!(usize_to_uint32, usize, u32);
uint_to_uint!(usize_to_uint64, usize, u64);
uint_to_float64!(usize_to_float, usize);
uint_to_float32!(usize_to_float32, usize);

// ============================================================================
// Signed int conversions
// ============================================================================

sint_to_sint!(int8_to_int16, i8, i16, u16);
sint_to_sint!(int8_to_int32, i8, i32, u32);
sint_to_sint!(int8_to_int64, i8, i64, u64);
sint_to_sint!(int8_to_isize, i8, isize, usize);
sint_to_float64!(int8_to_float, i8);
sint_to_float32!(int8_to_float32, i8);

sint_to_sint!(int16_to_int8, i16, i8, u8);
sint_to_sint!(int16_to_int32, i16, i32, u32);
sint_to_sint!(int16_to_int64, i16, i64, u64);
sint_to_sint!(int16_to_isize, i16, isize, usize);
sint_to_float64!(int16_to_float, i16);
sint_to_float32!(int16_to_float32, i16);

sint_to_sint!(int32_to_int8, i32, i8, u8);
sint_to_sint!(int32_to_int16, i32, i16, u16);
sint_to_sint!(int32_to_int64, i32, i64, u64);
sint_to_sint!(int32_to_isize, i32, isize, usize);
sint_to_float64!(int32_to_float, i32);
sint_to_float32!(int32_to_float32, i32);

sint_to_sint!(int64_to_int8, i64, i8, u8);
sint_to_sint!(int64_to_int16, i64, i16, u16);
sint_to_sint!(int64_to_int32, i64, i32, u32);
sint_to_sint!(int64_to_isize, i64, isize, usize);
sint_to_float64!(int64_to_float, i64);
sint_to_float32!(int64_to_float32, i64);

sint_to_sint!(isize_to_int8, isize, i8, u8);
sint_to_sint!(isize_to_int16, isize, i16, u16);
sint_to_sint!(isize_to_int32, isize, i32, u32);
sint_to_sint!(isize_to_int64, isize, i64, u64);
sint_to_float64!(isize_to_float, isize);
sint_to_float32!(isize_to_float32, isize);

// Signed int to Lean Int (uses i64_to_int helper)
fn int8_to_int(args: &[LeanValue]) -> Result<LeanValue> {
    let v = lean_unbox(args[0].as_ptr()) as i8;
    unsafe { Ok(LeanValue::from_raw(i64_to_int(v as i64))) }
}

fn int16_to_int(args: &[LeanValue]) -> Result<LeanValue> {
    let v = lean_unbox(args[0].as_ptr()) as i16;
    unsafe { Ok(LeanValue::from_raw(i64_to_int(v as i64))) }
}

fn int32_to_int(args: &[LeanValue]) -> Result<LeanValue> {
    let v = lean_unbox(args[0].as_ptr()) as i32;
    unsafe { Ok(LeanValue::from_raw(i64_to_int(v as i64))) }
}

fn int64_to_int(args: &[LeanValue]) -> Result<LeanValue> {
    let v = lean_unbox(args[0].as_ptr()) as i64;
    unsafe { Ok(LeanValue::from_raw(i64_to_int(v))) }
}

fn isize_to_int(args: &[LeanValue]) -> Result<LeanValue> {
    let v = lean_unbox(args[0].as_ptr()) as isize;
    unsafe { Ok(LeanValue::from_raw(i64_to_int(v as i64))) }
}

// ============================================================================
// Float64 conversions (with saturation)
// ============================================================================

macro_rules! float64_to_uint_saturating {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = unsafe { lean_unbox_float(args[0].as_ptr()) };
            let result = if v.is_nan() || v < 0.0 {
                0 as $ty
            } else if v > <$ty>::MAX as f64 {
                <$ty>::MAX
            } else {
                v as $ty
            };
            unsafe { Ok(LeanValue::from_raw(lean_box(result as usize))) }
        }
    };
}

macro_rules! float64_to_sint_saturating {
    ($name:ident, $sty:ty, $uty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = unsafe { lean_unbox_float(args[0].as_ptr()) };
            let result = if v.is_nan() {
                0 as $sty
            } else if v < <$sty>::MIN as f64 {
                <$sty>::MIN
            } else if v > <$sty>::MAX as f64 {
                <$sty>::MAX
            } else {
                v as $sty
            };
            unsafe { Ok(LeanValue::from_raw(lean_box((result as $uty) as usize))) }
        }
    };
}

float64_to_uint_saturating!(float_to_uint8, u8);
float64_to_uint_saturating!(float_to_uint16, u16);
float64_to_uint_saturating!(float_to_uint32, u32);
float64_to_uint_saturating!(float_to_usize, usize);
float64_to_sint_saturating!(float_to_int8, i8, u8);
float64_to_sint_saturating!(float_to_int16, i16, u16);
float64_to_sint_saturating!(float_to_int32, i32, u32);
float64_to_sint_saturating!(float_to_int64, i64, u64);
float64_to_sint_saturating!(float_to_isize, isize, usize);

fn float_to_float32(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let v = lean_unbox_float(args[0].as_ptr()) as f32;
        Ok(LeanValue::from_raw(lean_box_float32(v)))
    }
}

fn float_to_bits(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let v = lean_unbox_float(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box(v.to_bits() as usize)))
    }
}

fn float_of_bits(args: &[LeanValue]) -> Result<LeanValue> {
    let bits = lean_unbox(args[0].as_ptr()) as u64;
    unsafe { Ok(LeanValue::from_raw(lean_box_float(f64::from_bits(bits)))) }
}

// ============================================================================
// Float32 conversions (with saturation)
// ============================================================================

macro_rules! float32_to_uint_saturating {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = unsafe { lean_unbox_float32(args[0].as_ptr()) };
            let result = if v.is_nan() || v < 0.0 {
                0 as $ty
            } else if v > <$ty>::MAX as f32 {
                <$ty>::MAX
            } else {
                v as $ty
            };
            unsafe { Ok(LeanValue::from_raw(lean_box(result as usize))) }
        }
    };
}

macro_rules! float32_to_sint_saturating {
    ($name:ident, $sty:ty, $uty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = unsafe { lean_unbox_float32(args[0].as_ptr()) };
            let result = if v.is_nan() {
                0 as $sty
            } else if v < <$sty>::MIN as f32 {
                <$sty>::MIN
            } else if v > <$sty>::MAX as f32 {
                <$sty>::MAX
            } else {
                v as $sty
            };
            unsafe { Ok(LeanValue::from_raw(lean_box((result as $uty) as usize))) }
        }
    };
}

float32_to_uint_saturating!(float32_to_uint8, u8);
float32_to_uint_saturating!(float32_to_uint16, u16);
float32_to_uint_saturating!(float32_to_uint32, u32);
float32_to_uint_saturating!(float32_to_usize, usize);
float32_to_sint_saturating!(float32_to_int8, i8, u8);
float32_to_sint_saturating!(float32_to_int16, i16, u16);
float32_to_sint_saturating!(float32_to_int32, i32, u32);
float32_to_sint_saturating!(float32_to_int64, i64, u64);
float32_to_sint_saturating!(float32_to_isize, isize, usize);

fn float32_to_float(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let v = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box_float(v as f64)))
    }
}

fn float32_to_bits(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let v = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box(v.to_bits() as usize)))
    }
}

fn float32_of_bits(args: &[LeanValue]) -> Result<LeanValue> {
    let bits = lean_unbox(args[0].as_ptr()) as u32;
    unsafe { Ok(LeanValue::from_raw(lean_box_float32(f32::from_bits(bits)))) }
}
