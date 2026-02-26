//! IO.Ref and ST.Ref mutable reference operations

use super::{io_result_ok_val, ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // IO References
        ("lean_io_ref_new", io_ref_new as ExternFn),
        ("lean_io_ref_get", io_ref_get),
        ("lean_io_ref_set", io_ref_set),
        ("lean_io_ref_swap", io_ref_swap),
        ("lean_io_ref_take", io_ref_take),
        // ST References (same implementation)
        ("lean_st_ref_new", st_ref_new),
        ("lean_st_ref_get", st_ref_get),
        ("lean_st_ref_set", st_ref_set),
        ("lean_st_ref_swap", st_ref_swap),
    ]
}

fn io_ref_new(args: &[LeanValue]) -> Result<LeanValue> {
    // Create a new mutable reference containing the given value
    // We represent a Ref as a ctor with one field holding the value
    unsafe {
        lean_inc(args[0].as_ptr());
        let ref_obj = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(ref_obj, 0, args[0].as_ptr());
        Ok(io_result_ok_val(LeanValue::from_raw(ref_obj)))
    }
}

fn io_ref_get(args: &[LeanValue]) -> Result<LeanValue> {
    // Get the value from a reference
    unsafe {
        let ref_obj = args[0].as_ptr();
        let val = lean_ctor_get(ref_obj, 0);
        lean_inc(val);
        Ok(io_result_ok_val(LeanValue::from_raw(val)))
    }
}

fn io_ref_set(args: &[LeanValue]) -> Result<LeanValue> {
    // Set the value in a reference
    unsafe {
        let ref_obj = args[0].as_ptr();
        let new_val = args[1].as_ptr();

        // Dec old value
        let old_val = lean_ctor_get(ref_obj, 0);
        lean_dec(old_val);

        // Inc and set new value
        lean_inc(new_val);
        lean_ctor_set(ref_obj, 0, new_val);

        Ok(io_result_ok_val(LeanValue::unit()))
    }
}

fn io_ref_swap(args: &[LeanValue]) -> Result<LeanValue> {
    // Swap the value in a reference, return old value
    unsafe {
        let ref_obj = args[0].as_ptr();
        let new_val = args[1].as_ptr();

        let old_val = lean_ctor_get(ref_obj, 0);
        // Don't dec old_val since we're returning it

        lean_inc(new_val);
        lean_ctor_set(ref_obj, 0, new_val);

        Ok(io_result_ok_val(LeanValue::from_raw(old_val)))
    }
}

fn io_ref_take(args: &[LeanValue]) -> Result<LeanValue> {
    // Take the value from a reference (replaces with unit)
    unsafe {
        let ref_obj = args[0].as_ptr();

        let val = lean_ctor_get(ref_obj, 0);
        // Don't dec val since we're returning it

        lean_ctor_set(ref_obj, 0, lean_box(0)); // set to unit

        Ok(io_result_ok_val(LeanValue::from_raw(val)))
    }
}

// ST References - same as IO.Ref but in ST monad
fn st_ref_new(args: &[LeanValue]) -> Result<LeanValue> {
    io_ref_new(args)
}

fn st_ref_get(args: &[LeanValue]) -> Result<LeanValue> {
    io_ref_get(args)
}

fn st_ref_set(args: &[LeanValue]) -> Result<LeanValue> {
    io_ref_set(args)
}

fn st_ref_swap(args: &[LeanValue]) -> Result<LeanValue> {
    io_ref_swap(args)
}
