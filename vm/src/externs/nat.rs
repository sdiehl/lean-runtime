//! Natural number operations

use super::{delegate, delegate_bool, ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

// Delegating wrappers (2-arg -> *mut LeanObject)
delegate!(lean_nat_add, lean_nat_add, 2);
delegate!(lean_nat_sub, lean_nat_sub, 2);
delegate!(lean_nat_mul, lean_nat_mul, 2);
delegate!(lean_nat_div, lean_nat_div, 2);
delegate!(lean_nat_mod, lean_nat_mod, 2);
delegate!(lean_nat_pow, lean_nat_pow, 2);
delegate!(lean_nat_shiftr, lean_nat_shiftr, 2);
delegate!(lean_nat_shiftl, lean_nat_shiftl, 2);
delegate!(lean_nat_land, lean_nat_land, 2);
delegate!(lean_nat_lor, lean_nat_lor, 2);
delegate!(lean_nat_lxor, lean_nat_lxor, 2);
delegate!(lean_nat_log2, lean_nat_log2, 1);

// Delegating wrappers (2-arg -> bool)
delegate_bool!(lean_nat_dec_lt, lean_nat_dec_lt, 2);
delegate_bool!(lean_nat_dec_le, lean_nat_dec_le, 2);
delegate_bool!(lean_nat_dec_eq, lean_nat_dec_eq, 2);

// Custom wrappers
fn nat_pred(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_nat_pred(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn nat_succ(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let n = args[0].as_ptr();
        let one = lean_box(1);
        let result = lean_runtime::lean_nat_add(n, one);
        Ok(LeanValue::from_raw(result))
    }
}

fn nat_abs(args: &[LeanValue]) -> Result<LeanValue> {
    // For Nat, abs is identity
    unsafe {
        lean_inc(args[0].as_ptr());
        Ok(LeanValue::from_raw(args[0].as_ptr()))
    }
}

// Boolean aliases (same as dec_* but explicitly named)
fn nat_ble(args: &[LeanValue]) -> Result<LeanValue> {
    lean_nat_dec_le(args)
}

fn nat_blt(args: &[LeanValue]) -> Result<LeanValue> {
    lean_nat_dec_lt(args)
}

fn nat_beq(args: &[LeanValue]) -> Result<LeanValue> {
    lean_nat_dec_eq(args)
}

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Core runtime primitives
        ("lean_nat_add", lean_nat_add as ExternFn),
        ("lean_nat_sub", lean_nat_sub),
        ("lean_nat_mul", lean_nat_mul),
        ("lean_nat_div", lean_nat_div),
        ("lean_nat_mod", lean_nat_mod),
        ("lean_nat_pow", lean_nat_pow),
        ("lean_nat_pred", nat_pred),
        ("lean_nat_succ", nat_succ),
        ("lean_nat_shiftr", lean_nat_shiftr),
        ("lean_nat_shiftl", lean_nat_shiftl),
        ("lean_nat_land", lean_nat_land),
        ("lean_nat_lor", lean_nat_lor),
        ("lean_nat_lxor", lean_nat_lxor),
        ("lean_nat_dec_lt", lean_nat_dec_lt),
        ("lean_nat_dec_le", lean_nat_dec_le),
        ("lean_nat_dec_eq", lean_nat_dec_eq),
        ("lean_nat_ble", nat_ble),
        ("lean_nat_blt", nat_blt),
        ("lean_nat_beq", nat_beq),
        ("lean_nat_log2", lean_nat_log2),
        ("lean_nat_abs", nat_abs),
    ]
}
