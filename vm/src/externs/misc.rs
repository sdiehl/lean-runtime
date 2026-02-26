//! Miscellaneous operations (panic, sorry, name, char)

use super::{ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Panic/sorry
        ("lean_sorry", sorry as ExternFn),
        ("lean_panic_fn", panic_fn),
        // Name operations
        ("lean_name_eq", name_eq),
        // Note: l_Char_ofNat comes from Init bytecode
    ]
}

fn sorry(_args: &[LeanValue]) -> Result<LeanValue> {
    eprintln!("sorry: unimplemented");
    Ok(LeanValue::unit())
}

fn panic_fn(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        if args.len() >= 2 && !lean_is_scalar(args[1].as_ptr()) {
            let cstr = lean_string_cstr(args[1].as_ptr());
            let len = lean_string_byte_len(args[1].as_ptr());
            let slice = std::slice::from_raw_parts(cstr, len);
            if let Ok(msg) = std::str::from_utf8(slice) {
                panic!("lean_panic: {}", msg);
            }
        }
    }
    panic!("lean_panic");
}

fn name_eq(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_name_eq(args[0].as_ptr(), args[1].as_ptr());
        Ok(LeanValue::from_bool(result != 0))
    }
}
