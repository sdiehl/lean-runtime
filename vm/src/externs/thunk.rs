//! Thunk (lazy evaluation) operations

use super::{ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        ("lean_thunk_pure", thunk_pure as ExternFn),
        ("lean_thunk_get_own", thunk_get_own),
        ("lean_thunk_map", thunk_map),
        ("lean_thunk_bind", thunk_bind),
    ]
}

fn thunk_pure(args: &[LeanValue]) -> Result<LeanValue> {
    // Create a thunk that immediately returns the given value
    // We represent as a ctor with tag 1 (evaluated) holding the value
    unsafe {
        lean_inc(args[0].as_ptr());
        let thunk = lean_alloc_ctor(1, 1, 0); // tag 1 = evaluated
        lean_ctor_set(thunk, 0, args[0].as_ptr());
        Ok(LeanValue::from_raw(thunk))
    }
}

fn thunk_get_own(args: &[LeanValue]) -> Result<LeanValue> {
    // Get the value from a thunk (force evaluation)
    // For pure thunks, just return the value
    unsafe {
        let thunk = args[0].as_ptr();
        let tag = lean_obj_tag(thunk);
        if tag == 1 {
            // Already evaluated
            let val = lean_ctor_get(thunk, 0);
            lean_inc(val);
            Ok(LeanValue::from_raw(val))
        } else {
            // Unevaluated thunk - would need to run the computation
            // For now, return unit
            Ok(LeanValue::unit())
        }
    }
}

fn thunk_map(args: &[LeanValue]) -> Result<LeanValue> {
    // Map a function over a thunk (stub - returns unit thunk)
    let _ = args;
    thunk_pure(&[LeanValue::unit()])
}

fn thunk_bind(args: &[LeanValue]) -> Result<LeanValue> {
    // Bind operation on thunks (stub - returns unit thunk)
    let _ = args;
    thunk_pure(&[LeanValue::unit()])
}
