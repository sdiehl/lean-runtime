//! String operations

use super::{delegate, delegate_bool, ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        ("lean_string_append", string_append as ExternFn),
        ("lean_string_length", string_length),
        ("lean_string_mk", string_mk),
        ("lean_string_dec_eq", string_dec_eq),
        ("lean_string_hash", string_hash),
        ("lean_string_utf8_byte_size", string_utf8_byte_size),
        ("lean_string_utf8_next", string_utf8_next),
        ("lean_string_utf8_next_fast", string_utf8_next),
        ("lean_string_utf8_prev", string_utf8_prev),
        ("lean_string_utf8_get", string_utf8_get),
        ("lean_string_utf8_get_fast", string_utf8_get),
        ("lean_string_utf8_set", string_utf8_set),
        ("lean_string_utf8_extract", string_utf8_extract),
        ("lean_string_to_utf8", string_to_utf8),
        (
            "lean_string_from_utf8_unchecked",
            string_from_utf8_unchecked,
        ),
        ("lean_string_push", string_push),
        ("lean_string_of_usize", string_of_usize),
        ("lean_string_isempty", string_isempty),
        ("lean_string_foldl", string_foldl),
        ("lean_string_dec_lt", string_dec_lt),
        ("lean_substring_tostring", substring_tostring),
        ("lean_string_iterator_mk", string_iterator_mk),
        ("lean_string_iterator_curr", string_iterator_curr),
        ("lean_string_iterator_next", string_iterator_next),
        ("lean_string_iterator_has_next", string_iterator_has_next),
        ("lean_string_utf8_at_end", string_utf8_at_end),
        ("lean_string_memcmp", string_memcmp),
        ("lean_string_data", string_data),
        ("lean_string_isprefixof", string_isprefixof),
        ("lean_string_any", string_any),
    ]
}

// Simple delegations
delegate!(string_data, lean_string_data, 1);
delegate!(string_append, lean_string_append, 2);
delegate!(string_length, lean_string_length, 1);
delegate!(string_mk, lean_string_mk, 1);
delegate!(string_utf8_byte_size, lean_string_utf8_byte_size, 1);
delegate!(string_utf8_next, lean_string_utf8_next_fast, 2);
delegate!(string_utf8_prev, lean_string_utf8_prev, 2);
delegate!(string_utf8_extract, lean_string_utf8_extract, 3);
delegate!(string_to_utf8, lean_string_to_utf8, 1);
delegate!(
    string_from_utf8_unchecked,
    lean_string_from_utf8_unchecked,
    1
);

// Bool delegations
delegate_bool!(string_dec_eq, lean_string_dec_eq, 2);
delegate_bool!(string_utf8_at_end, lean_string_utf8_at_end, 2);
delegate_bool!(string_dec_lt, lean_string_dec_lt, 2);

// Custom implementations
fn string_memcmp(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_string_memcmp(
            args[0].as_ptr(),
            args[1].as_ptr(),
            args[2].as_ptr(),
            args[3].as_ptr(),
            args[4].as_ptr(),
        );
        Ok(LeanValue::from_bool(result != 0))
    }
}

fn string_hash(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_string_hash(args[0].as_ptr());
        Ok(LeanValue::from_raw(lean_box(result as usize)))
    }
}

fn string_push(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_string_push(args[0].as_ptr(), lean_unbox(args[1].as_ptr()) as u32);
        Ok(LeanValue::from_raw(result))
    }
}

fn string_of_usize(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let n = lean_usize_of_nat(args[0].as_ptr());
        Ok(LeanValue::from_string(&n.to_string()))
    }
}

fn string_isempty(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let len = lean_string_byte_len(args[0].as_ptr());
        Ok(LeanValue::from_bool(len == 0))
    }
}

fn string_foldl(args: &[LeanValue]) -> Result<LeanValue> {
    // string_foldl f init s - fold over characters
    // For now, just return the initial value (stub)
    Ok(args[1].clone())
}

fn substring_tostring(_args: &[LeanValue]) -> Result<LeanValue> {
    // Convert substring to string - for now return empty string
    Ok(LeanValue::from_string(""))
}

fn string_utf8_get(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_string_utf8_get_fast(args[0].as_ptr(), args[1].as_ptr());
        Ok(LeanValue::from_raw(lean_box(result as usize)))
    }
}

fn string_utf8_set(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let c = lean_unbox(args[2].as_ptr()) as u32;
        let result = lean_string_utf8_set(args[0].as_ptr(), args[1].as_ptr(), c);
        Ok(LeanValue::from_raw(result))
    }
}

fn string_isprefixof(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let prefix_str = lean_string_to_str(args[0].as_ptr());
        let s_str = lean_string_to_str(args[1].as_ptr());
        Ok(LeanValue::from_bool(s_str.starts_with(prefix_str)))
    }
}

fn string_any(_args: &[LeanValue]) -> Result<LeanValue> {
    // String.any : String → (Char → Bool) → Bool
    // TODO: implement properly - requires handling bytecode closures
    Ok(LeanValue::from_bool(false))
}

// String iterator: represented as (string, position)
fn string_iterator_mk(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        lean_inc(args[0].as_ptr());
        let iter = lean_alloc_ctor(0, 2, 0);
        lean_ctor_set(iter, 0, args[0].as_ptr());
        lean_ctor_set(iter, 1, lean_box(0));
        Ok(LeanValue::from_raw(iter))
    }
}

fn string_iterator_curr(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let iter = args[0].as_ptr();
        let s = lean_ctor_get(iter, 0);
        let pos = lean_ctor_get(iter, 1);
        let c = lean_string_utf8_get_fast(s, pos);
        Ok(LeanValue::from_raw(lean_box(c as usize)))
    }
}

fn string_iterator_next(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let iter = args[0].as_ptr();
        let s = lean_ctor_get(iter, 0);
        let pos = lean_ctor_get(iter, 1);
        let next_pos = lean_string_utf8_next_fast(s, pos);
        lean_inc(s);
        let new_iter = lean_alloc_ctor(0, 2, 0);
        lean_ctor_set(new_iter, 0, s);
        lean_ctor_set(new_iter, 1, next_pos);
        Ok(LeanValue::from_raw(new_iter))
    }
}

fn string_iterator_has_next(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let iter = args[0].as_ptr();
        let s = lean_ctor_get(iter, 0);
        let pos = lean_ctor_get(iter, 1);
        let pos_val = lean_unbox(pos);
        let size = lean_string_byte_len(s);
        Ok(LeanValue::from_bool(pos_val < size))
    }
}
