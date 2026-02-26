//! Signed integer operations

use super::{ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        ("lean_nat_to_int", nat_to_int as ExternFn),
        ("lean_int_neg", int_neg),
        ("lean_int_add", int_add),
        ("lean_int_sub", int_sub),
        ("lean_int_mul", int_mul),
        ("lean_int_div", int_div),
        ("lean_int_mod", int_mod),
        ("lean_int_ediv", int_ediv),
        ("lean_int_emod", int_emod),
        ("lean_int_div_exact", int_div_exact),
        ("lean_int_dec_lt", int_dec_lt),
        ("lean_int_dec_le", int_dec_le),
        ("lean_int_dec_eq", int_dec_eq),
        ("lean_int_dec_nonneg", int_dec_nonneg),
        ("lean_nat_abs", nat_abs),
        ("lean_int_neg_succ_of_nat", int_neg_succ_of_nat),
        ("l_instNatCastInt___lam__0", nat_cast_int_lam),
    ]
}

/// Extract i64 value from Lean Int (ofNat n | negSucc n representation)
///
/// # Safety
/// The caller must ensure `obj` is a valid pointer to a Lean Int object
/// (constructor with tag 0 for ofNat or tag 1 for negSucc).
pub unsafe fn int_to_i64(obj: *mut LeanObject) -> i64 {
    let tag = lean_obj_tag(obj);
    if tag == 0 {
        // ofNat n - non-negative
        let nat = lean_ctor_get(obj, 0);
        lean_unbox(nat) as i64
    } else {
        // negSucc n - represents -(n+1)
        let nat = lean_ctor_get(obj, 0);
        let n = lean_unbox(nat) as i64;
        -(n + 1)
    }
}

/// Create Lean Int from i64 value
///
/// # Safety
/// This function allocates memory and returns a raw pointer. The caller
/// is responsible for proper memory management of the returned object.
pub unsafe fn i64_to_int(val: i64) -> *mut LeanObject {
    if val >= 0 {
        // ofNat n (tag 0)
        let int_obj = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(int_obj, 0, lean_box(val as usize));
        int_obj
    } else {
        // negSucc n (tag 1) - represents -(n+1), so n = -val - 1
        let n = (-val - 1) as usize;
        let int_obj = lean_alloc_ctor(1, 1, 0);
        lean_ctor_set(int_obj, 0, lean_box(n));
        int_obj
    }
}

fn nat_to_int(args: &[LeanValue]) -> Result<LeanValue> {
    // Int is: ofNat n (tag 0) | negSucc n (tag 1)
    // For nat_to_int, we create ofNat
    unsafe {
        lean_inc(args[0].as_ptr());
        let int_obj = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(int_obj, 0, args[0].as_ptr());
        Ok(LeanValue::from_raw(int_obj))
    }
}

fn int_neg(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let val = int_to_i64(args[0].as_ptr());
        let result = i64_to_int(-val);
        Ok(LeanValue::from_raw(result))
    }
}

fn int_add(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        let result = i64_to_int(a + b);
        Ok(LeanValue::from_raw(result))
    }
}

fn int_sub(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        let result = i64_to_int(a - b);
        Ok(LeanValue::from_raw(result))
    }
}

fn int_mul(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        let result = i64_to_int(a * b);
        Ok(LeanValue::from_raw(result))
    }
}

fn int_div(args: &[LeanValue]) -> Result<LeanValue> {
    // Truncating division (toward zero)
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        let result = if b == 0 { 0 } else { a / b };
        Ok(LeanValue::from_raw(i64_to_int(result)))
    }
}

fn int_mod(args: &[LeanValue]) -> Result<LeanValue> {
    // Truncating modulo (sign follows dividend)
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        let result = if b == 0 { a } else { a % b };
        Ok(LeanValue::from_raw(i64_to_int(result)))
    }
}

fn int_ediv(args: &[LeanValue]) -> Result<LeanValue> {
    // Euclidean division (Daan Leijen's algorithm)
    // The remainder is always non-negative (0 <= r < |b|)
    unsafe {
        let n = int_to_i64(args[0].as_ptr());
        let d = int_to_i64(args[1].as_ptr());
        let result = if d == 0 {
            0
        } else {
            let q = n / d; // Rust truncates toward zero
            let r = n % d; // Rust remainder, sign follows dividend
                           // Only adjust when remainder is negative
            if r < 0 {
                if d > 0 {
                    q - 1
                } else {
                    q + 1
                }
            } else {
                q
            }
        };
        Ok(LeanValue::from_raw(i64_to_int(result)))
    }
}

fn int_emod(args: &[LeanValue]) -> Result<LeanValue> {
    // Euclidean modulo: result is always >= 0
    // emod(n, d): if d == 0, return n; else r = n % d; if r < 0: r = (d > 0) ? r + d : r - d
    unsafe {
        let n = int_to_i64(args[0].as_ptr());
        let d = int_to_i64(args[1].as_ptr());
        let result = if d == 0 {
            n
        } else {
            let r = n % d;
            if r < 0 {
                if d > 0 {
                    r + d
                } else {
                    r - d
                }
            } else {
                r
            }
        };
        Ok(LeanValue::from_raw(i64_to_int(result)))
    }
}

fn int_div_exact(args: &[LeanValue]) -> Result<LeanValue> {
    // Exact division (assumes b divides a evenly)
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        let result = if b == 0 { 0 } else { a / b };
        Ok(LeanValue::from_raw(i64_to_int(result)))
    }
}

fn int_dec_lt(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        Ok(LeanValue::from_bool(a < b))
    }
}

fn int_dec_le(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        Ok(LeanValue::from_bool(a <= b))
    }
}

fn int_dec_eq(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        let b = int_to_i64(args[1].as_ptr());
        Ok(LeanValue::from_bool(a == b))
    }
}

fn int_dec_nonneg(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let a = int_to_i64(args[0].as_ptr());
        Ok(LeanValue::from_bool(a >= 0))
    }
}

fn nat_abs(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let val = int_to_i64(args[0].as_ptr());
        Ok(LeanValue::from_nat(val.unsigned_abs()))
    }
}

fn int_neg_succ_of_nat(args: &[LeanValue]) -> Result<LeanValue> {
    // Create Int.negSucc n, representing -(n+1)
    unsafe {
        lean_inc(args[0].as_ptr());
        let int_obj = lean_alloc_ctor(1, 1, 0);
        lean_ctor_set(int_obj, 0, args[0].as_ptr());
        Ok(LeanValue::from_raw(int_obj))
    }
}

fn nat_cast_int_lam(args: &[LeanValue]) -> Result<LeanValue> {
    // Nat to Int cast
    unsafe {
        lean_inc(args[0].as_ptr());
        let int_obj = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(int_obj, 0, args[0].as_ptr());
        Ok(LeanValue::from_raw(int_obj))
    }
}
