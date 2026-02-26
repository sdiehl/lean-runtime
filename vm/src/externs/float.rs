//! Floating point operations (Float64 and Float32)

use super::int::int_to_i64;
use super::{
    float32_binop, float32_cmp, float32_pred, float32_to_int, float32_unary, float64_binop,
    float64_binop_method, float64_cmp, float64_pred, float64_unary, float_to_int, int_to_float64,
    ExternFn, Result,
};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Float (f64) operations
        ("lean_float_add", float_add as ExternFn),
        ("lean_float_sub", float_sub),
        ("lean_float_mul", float_mul),
        ("lean_float_div", float_div),
        ("lean_float_neg", float_neg),
        ("lean_float_of_nat", float_of_nat),
        ("lean_float_of_int", float_of_int),
        ("lean_float_to_string", float_to_string),
        ("lean_float_of_scientific", float_of_scientific),
        ("lean_float_dec_lt", float_dec_lt),
        ("lean_float_dec_le", float_dec_le),
        ("lean_float_dec_eq", float_dec_eq),
        ("lean_float_decLt", float_dec_lt),
        ("lean_float_decLe", float_dec_le),
        ("lean_float_decEq", float_dec_eq),
        ("lean_float_sqrt", float_sqrt),
        ("lean_float_sin", float_sin),
        ("lean_float_cos", float_cos),
        ("lean_float_tan", float_tan),
        ("lean_float_exp", float_exp),
        ("lean_float_log", float_log),
        ("lean_float_pow", float_pow),
        ("lean_float_abs", float_abs),
        ("lean_float_floor", float_floor),
        ("lean_float_ceil", float_ceil),
        ("lean_float_round", float_round),
        ("lean_float_is_nan", float_is_nan),
        ("lean_float_is_inf", float_is_inf),
        ("lean_float_to_uint64", float_to_uint64),
        ("lean_float_to_uint8", float_to_uint8),
        ("lean_float_of_bits", float_of_bits),
        ("lean_float_to_bits", float_to_bits),
        ("lean_float_to_float32", float_to_float32),
        ("lean_float_frexp", float_frexp),
        ("lean_float_scaleb", float_scaleb),
        ("lean_float_is_finite", float_is_finite),
        ("lean_float_negate", float_neg),
        ("sqrt", float_sqrt),
        ("sin", float_sin),
        ("cos", float_cos),
        ("tan", float_tan),
        ("exp", float_exp),
        ("log", float_log),
        ("fabs", float_abs),
        ("floor", float_floor),
        ("ceil", float_ceil),
        ("round", float_round),
        ("pow", float_pow),
        ("fmod", float_fmod),
        ("asin", float_asin),
        ("acos", float_acos),
        ("atan", float_atan),
        ("atan2", float_atan2),
        ("sinh", float_sinh),
        ("cosh", float_cosh),
        ("tanh", float_tanh),
        ("asinh", float_asinh),
        ("acosh", float_acosh),
        ("atanh", float_atanh),
        ("exp2", float_exp2),
        ("log2", float_log2),
        ("log10", float_log10),
        ("cbrt", float_cbrt),
        // Integer to Float conversions
        ("lean_uint64_to_float", uint64_to_float),
        ("lean_uint32_to_float", uint32_to_float),
        ("lean_uint16_to_float", uint16_to_float),
        ("lean_uint8_to_float", uint8_to_float),
        ("lean_usize_to_float", usize_to_float),
        ("lean_int64_to_float", int64_to_float),
        ("lean_int32_to_float", int32_to_float),
        ("lean_int16_to_float", int16_to_float),
        ("lean_int8_to_float", int8_to_float),
        ("lean_isize_to_float", isize_to_float),
        // Float32 (f32) operations
        ("lean_float32_add", float32_add),
        ("lean_float32_sub", float32_sub),
        ("lean_float32_mul", float32_mul),
        ("lean_float32_div", float32_div),
        ("lean_float32_neg", float32_neg),
        ("lean_float32_of_bits", float32_of_bits),
        ("lean_float32_to_bits", float32_to_bits),
        ("lean_float32_to_float", float32_to_float),
        ("lean_float32_to_string", float32_to_string),
        ("lean_float32_dec_lt", float32_dec_lt),
        ("lean_float32_dec_le", float32_dec_le),
        ("lean_float32_dec_eq", float32_dec_eq),
        ("lean_float32_is_nan", float32_is_nan),
        ("lean_float32_is_inf", float32_is_inf),
        ("lean_float32_is_finite", float32_is_finite),
        ("lean_float32_frexp", float32_frexp),
        ("lean_float32_scaleb", float32_scaleb),
        // Float32 conversions
        ("lean_float32_to_uint8", float32_to_uint8),
        ("lean_float32_to_uint16", float32_to_uint16),
        ("lean_float32_to_uint32", float32_to_uint32),
        ("lean_float32_to_uint64", float32_to_uint64),
        ("lean_float32_to_usize", float32_to_usize),
        ("lean_float32_to_int8", float32_to_int8),
        ("lean_float32_to_int16", float32_to_int16),
        ("lean_float32_to_int32", float32_to_int32),
        ("lean_float32_to_int64", float32_to_int64),
        ("lean_float32_to_isize", float32_to_isize),
    ]
}

// ============================================================================
// Float64 Operations
// ============================================================================

// Arithmetic
float64_binop!(float_add, +);
float64_binop!(float_sub, -);
float64_binop!(float_mul, *);
float64_binop!(float_div, /);
float64_binop!(float_fmod, %);

// Unary math
float64_unary!(float_sqrt, sqrt);
float64_unary!(float_sin, sin);
float64_unary!(float_cos, cos);
float64_unary!(float_tan, tan);
float64_unary!(float_exp, exp);
float64_unary!(float_abs, abs);
float64_unary!(float_floor, floor);
float64_unary!(float_ceil, ceil);
float64_unary!(float_round, round);
float64_unary!(float_asin, asin);
float64_unary!(float_acos, acos);
float64_unary!(float_atan, atan);
float64_unary!(float_sinh, sinh);
float64_unary!(float_cosh, cosh);
float64_unary!(float_tanh, tanh);
float64_unary!(float_asinh, asinh);
float64_unary!(float_acosh, acosh);
float64_unary!(float_atanh, atanh);
float64_unary!(float_exp2, exp2);
float64_unary!(float_log2, log2);
float64_unary!(float_log10, log10);
float64_unary!(float_cbrt, cbrt);
float64_unary!(float_log, ln);

// Binary math
float64_binop_method!(float_pow, powf);
float64_binop_method!(float_atan2, atan2);

// Comparisons
float64_cmp!(float_dec_lt, <);
float64_cmp!(float_dec_le, <=);
float64_cmp!(float_dec_eq, ==);

// Predicates
float64_pred!(float_is_nan, is_nan);
float64_pred!(float_is_inf, is_infinite);
float64_pred!(float_is_finite, is_finite);

// Conversions
float_to_int!(float_to_uint64, u64);
float_to_int!(float_to_uint8, u8);

fn float_neg(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = lean_unbox_float(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box_float(-a)))
    }
}

fn float_of_nat(args: &[LeanValue]) -> Result<LeanValue> {
    let n = lean_unbox(args[0].as_ptr()) as f64;
    unsafe { Ok(LeanValue::from_raw(lean_box_float(n))) }
}

fn float_of_int(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr()) as f64;
        Ok(LeanValue::from_raw(lean_box_float(i)))
    }
}

fn float_to_string(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float(args[0].as_ptr());
        Ok(LeanValue::from_string(&f.to_string()))
    }
}

fn float_of_scientific(args: &[LeanValue]) -> Result<LeanValue> {
    let mantissa = lean_unbox(args[0].as_ptr()) as f64;
    let sign = lean_unbox(args[1].as_ptr()) != 0;
    let exponent = lean_unbox(args[2].as_ptr()) as i32;
    let exp = if sign { -exponent } else { exponent };
    let result = mantissa * 10f64.powi(exp);
    unsafe { Ok(LeanValue::from_raw(lean_box_float(result))) }
}

fn float_frexp(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float(args[0].as_ptr());
        if f == 0.0 || f.is_nan() || f.is_infinite() {
            let pair = lean_alloc_ctor(0, 2, 0);
            lean_ctor_set(pair, 0, lean_box_float(f));
            lean_ctor_set(pair, 1, lean_box(0));
            return Ok(LeanValue::from_raw(pair));
        }
        let bits = f.to_bits();
        let sign = bits & 0x8000000000000000;
        let exp = ((bits >> 52) & 0x7FF) as i32;
        let mantissa_bits = bits & 0x000FFFFFFFFFFFFF;
        let exponent = exp - 1022;
        let new_mantissa_bits = sign | (1022u64 << 52) | mantissa_bits;
        let mantissa = f64::from_bits(new_mantissa_bits);
        let pair = lean_alloc_ctor(0, 2, 0);
        lean_ctor_set(pair, 0, lean_box_float(mantissa));
        lean_ctor_set(pair, 1, lean_box(exponent as usize));
        Ok(LeanValue::from_raw(pair))
    }
}

fn float_scaleb(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float(args[0].as_ptr());
        let n = int_to_i64(args[1].as_ptr()) as i32;
        Ok(LeanValue::from_raw(lean_box_float(f * 2.0f64.powi(n))))
    }
}

fn float_of_bits(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let bits = lean_unbox(args[0].as_ptr()) as u64;
        Ok(LeanValue::from_raw(lean_box_float(f64::from_bits(bits))))
    }
}

fn float_to_bits(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box(f.to_bits() as usize)))
    }
}

fn float_to_float32(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float(args[0].as_ptr()) as f32;
        Ok(LeanValue::from_raw(lean_box_float32(f)))
    }
}

// ============================================================================
// Integer to Float64 conversions
// ============================================================================

fn uint64_to_float(args: &[LeanValue]) -> Result<LeanValue> {
    use num_traits::ToPrimitive;
    let ptr = args[0].as_ptr();
    let f: f64 = if lean_is_scalar(ptr) {
        lean_unbox(ptr) as f64
    } else {
        unsafe { lean_nat_to_biguint(ptr).to_f64().unwrap_or(f64::INFINITY) }
    };
    unsafe { Ok(LeanValue::from_raw(lean_box_float(f))) }
}

int_to_float64!(uint32_to_float, u32);
int_to_float64!(uint16_to_float, u16);
int_to_float64!(uint8_to_float, u8);
int_to_float64!(usize_to_float, usize);
int_to_float64!(int32_to_float, i32);
int_to_float64!(int16_to_float, i16);
int_to_float64!(int8_to_float, i8);
int_to_float64!(isize_to_float, isize);

fn int64_to_float(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let i = int_to_i64(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box_float(i as f64)))
    }
}

// ============================================================================
// Float32 Operations
// ============================================================================

float32_binop!(float32_add, +);
float32_binop!(float32_sub, -);
float32_binop!(float32_mul, *);
float32_binop!(float32_div, /);
float32_unary!(float32_neg, -);

float32_cmp!(float32_dec_lt, <);
float32_cmp!(float32_dec_le, <=);
float32_cmp!(float32_dec_eq, ==);

float32_pred!(float32_is_nan, is_nan);
float32_pred!(float32_is_inf, is_infinite);
float32_pred!(float32_is_finite, is_finite);

float32_to_int!(float32_to_uint8, u8);
float32_to_int!(float32_to_uint16, u16);
float32_to_int!(float32_to_uint32, u32);
float32_to_int!(float32_to_uint64, u64);
float32_to_int!(float32_to_usize, usize);

fn float32_of_bits(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let bits = lean_unbox(args[0].as_ptr()) as u32;
        Ok(LeanValue::from_raw(lean_box_float32(f32::from_bits(bits))))
    }
}

fn float32_to_bits(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box(f.to_bits() as usize)))
    }
}

fn float32_to_float(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box_float(f as f64)))
    }
}

fn float32_to_string(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_string(&f.to_string()))
    }
}

fn float32_frexp(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        if f == 0.0 || f.is_nan() || f.is_infinite() {
            let pair = lean_alloc_ctor(0, 2, 0);
            lean_ctor_set(pair, 0, lean_box_float32(f));
            lean_ctor_set(pair, 1, lean_box(0));
            return Ok(LeanValue::from_raw(pair));
        }
        let bits = f.to_bits();
        let sign = bits & 0x80000000;
        let exp = ((bits >> 23) & 0xFF) as i32;
        let mantissa_bits = bits & 0x007FFFFF;
        let exponent = exp - 126;
        let new_mantissa_bits = sign | (126 << 23) | mantissa_bits;
        let mantissa = f32::from_bits(new_mantissa_bits);
        let pair = lean_alloc_ctor(0, 2, 0);
        lean_ctor_set(pair, 0, lean_box_float32(mantissa));
        lean_ctor_set(pair, 1, lean_box(exponent as usize));
        Ok(LeanValue::from_raw(pair))
    }
}

fn float32_scaleb(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        let n = lean_unbox(args[1].as_ptr()) as i32;
        Ok(LeanValue::from_raw(lean_box_float32(f * 2.0f32.powi(n))))
    }
}

fn float32_to_int8(args: &[LeanValue]) -> Result<LeanValue> {
    use super::int::i64_to_int;
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(i64_to_int(f as i8 as i64)))
    }
}

fn float32_to_int16(args: &[LeanValue]) -> Result<LeanValue> {
    use super::int::i64_to_int;
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(i64_to_int(f as i16 as i64)))
    }
}

fn float32_to_int32(args: &[LeanValue]) -> Result<LeanValue> {
    use super::int::i64_to_int;
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(i64_to_int(f as i32 as i64)))
    }
}

fn float32_to_int64(args: &[LeanValue]) -> Result<LeanValue> {
    use super::int::i64_to_int;
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(i64_to_int(f as i64)))
    }
}

fn float32_to_isize(args: &[LeanValue]) -> Result<LeanValue> {
    use super::int::i64_to_int;
    unsafe {
        let f = lean_unbox_float32(args[0].as_ptr());
        Ok(LeanValue::from_raw(i64_to_int(f as isize as i64)))
    }
}
