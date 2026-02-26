//! Built-in extern function implementations for the VM
//!
//! This module is organized by semantic grouping:
//! - string: String operations
//! - nat: Natural number operations
//! - uint: Unsigned integer operations (UInt8, UInt16, UInt32, UInt64, USize)
//! - int: Signed integer operations
//! - convert: Type conversions between numeric types
//! - array: Array operations
//! - list: List operations
//! - io: IO operations (file, stdin/stdout, environment)
//! - float: Floating point operations
//! - refs: IO.Ref and ST.Ref mutable references
//! - system: System operations (platform, exit, time, process)
//! - format: Format/pretty-printing operations
//! - misc: Miscellaneous (panic, sorry, name, char)
//! - thunk: Thunk/lazy evaluation operations
//! - task: Task/Thread concurrency operations

mod array;
mod convert;
mod float;
mod format;
pub mod int;
mod io;
mod list;
mod misc;
mod nat;
mod refs;
mod sint;
mod string;
mod system;
mod task;
mod thunk;
mod uint;

use crate::value::LeanValue;
use crate::VMError;

pub type Result<T> = std::result::Result<T, VMError>;
pub type ExternFn = fn(&[LeanValue]) -> Result<LeanValue>;

// ============================================================================
// Macros for generating extern wrappers
// ============================================================================

/// Generate a wrapper that delegates to a lean_runtime function returning *mut LeanObject.
/// Usage: delegate!(wrapper_name, lean_fn, arity)
macro_rules! delegate {
    ($wrapper:ident, $lean_fn:ident, 1) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result = lean_runtime::$lean_fn(args[0].as_ptr());
                Ok(LeanValue::from_raw(result))
            }
        }
    };
    ($wrapper:ident, $lean_fn:ident, 2) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result = lean_runtime::$lean_fn(args[0].as_ptr(), args[1].as_ptr());
                Ok(LeanValue::from_raw(result))
            }
        }
    };
    ($wrapper:ident, $lean_fn:ident, 3) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result =
                    lean_runtime::$lean_fn(args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr());
                Ok(LeanValue::from_raw(result))
            }
        }
    };
    ($wrapper:ident, $lean_fn:ident, 4) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result = lean_runtime::$lean_fn(
                    args[0].as_ptr(),
                    args[1].as_ptr(),
                    args[2].as_ptr(),
                    args[3].as_ptr(),
                );
                Ok(LeanValue::from_raw(result))
            }
        }
    };
    ($wrapper:ident, $lean_fn:ident, 5) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result = lean_runtime::$lean_fn(
                    args[0].as_ptr(),
                    args[1].as_ptr(),
                    args[2].as_ptr(),
                    args[3].as_ptr(),
                    args[4].as_ptr(),
                );
                Ok(LeanValue::from_raw(result))
            }
        }
    };
}

/// Generate a wrapper that delegates to a lean_runtime function returning u8 (bool).
/// Usage: delegate_bool!(wrapper_name, lean_fn, arity)
macro_rules! delegate_bool {
    ($wrapper:ident, $lean_fn:ident, 1) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result = lean_runtime::$lean_fn(args[0].as_ptr());
                Ok(LeanValue::from_bool(result != 0))
            }
        }
    };
    ($wrapper:ident, $lean_fn:ident, 2) => {
        fn $wrapper(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let result = lean_runtime::$lean_fn(args[0].as_ptr(), args[1].as_ptr());
                Ok(LeanValue::from_bool(result != 0))
            }
        }
    };
}

pub(crate) use delegate;
pub(crate) use delegate_bool;

// ============================================================================
// Macros for scalar uint operations (unbox -> compute -> rebox)
// ============================================================================

/// Generate scalar binary arithmetic operations (add, sub, mul with wrapping)
/// Usage: scalar_binop!(name, type, op, wrapping)
macro_rules! scalar_binop {
    ($name:ident, $ty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $ty;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    (a.$op(b)) as usize,
                )))
            }
        }
    };
}

/// Generate scalar binary bitwise operations (land, lor, xor)
macro_rules! scalar_bitop {
    ($name:ident, $ty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $ty;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box((a $op b) as usize))) }
        }
    };
}

/// Generate scalar comparison operations (dec_eq, dec_lt, dec_le)
macro_rules! scalar_cmp {
    ($name:ident, $ty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $ty;
            Ok(LeanValue::from_bool(a $op b))
        }
    };
}

/// Generate scalar div operation (with div-by-zero check returning 0)
macro_rules! scalar_div {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $ty;
            let result = if b == 0 { 0 } else { a / b };
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box(result as usize))) }
        }
    };
}

/// Generate scalar mod operation (with mod-by-zero check returning a)
macro_rules! scalar_mod {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $ty;
            let result = if b == 0 { a } else { a % b };
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box(result as usize))) }
        }
    };
}

/// Generate scalar shift operations (with modulo to prevent overflow)
macro_rules! scalar_shift {
    ($name:ident, $ty:ty, $bits:expr, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $ty;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box((a $op (b % $bits)) as usize))) }
        }
    };
}

/// Generate scalar complement operation
macro_rules! scalar_complement {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box((!a) as usize))) }
        }
    };
}

/// Generate of_nat/of_nat_mk (truncate Nat to fixed-width)
macro_rules! scalar_of_nat {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let n = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
                Ok(LeanValue::from_raw(lean_runtime::lean_box(n as usize)))
            }
        }
    };
}

/// Generate to_nat (identity for scalars stored as Nat)
macro_rules! scalar_to_nat {
    ($name:ident) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            Ok(args[0].clone())
        }
    };
}

pub(crate) use scalar_binop;
pub(crate) use scalar_bitop;
pub(crate) use scalar_cmp;
pub(crate) use scalar_complement;
pub(crate) use scalar_div;
pub(crate) use scalar_mod;
pub(crate) use scalar_of_nat;
pub(crate) use scalar_shift;
pub(crate) use scalar_to_nat;

// ============================================================================
// Macros for signed scalar operations (iN types, need iN -> uN -> usize casting)
// ============================================================================

/// Generate signed scalar binary arithmetic operations (add, sub, mul with wrapping)
macro_rules! signed_binop {
    ($name:ident, $sty:ty, $uty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $sty;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    a.$op(b) as $uty as usize,
                )))
            }
        }
    };
}

/// Generate signed scalar unary operations (neg, abs)
macro_rules! signed_unary {
    ($name:ident, $sty:ty, $uty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    a.$op() as $uty as usize
                )))
            }
        }
    };
}

/// Generate signed scalar binary bitwise operations (land, lor, xor)
macro_rules! signed_bitop {
    ($name:ident, $sty:ty, $uty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $sty;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box((a $op b) as $uty as usize))) }
        }
    };
}

/// Generate signed scalar comparison operations
macro_rules! signed_cmp {
    ($name:ident, $sty:ty, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $sty;
            Ok(LeanValue::from_bool(a $op b))
        }
    };
}

/// Generate signed div operation (with div-by-zero check returning 0)
macro_rules! signed_div {
    ($name:ident, $sty:ty, $uty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $sty;
            let result = if b == 0 { 0 } else { a.wrapping_div(b) };
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    result as $uty as usize,
                )))
            }
        }
    };
}

/// Generate signed mod operation (with mod-by-zero check returning a)
macro_rules! signed_mod {
    ($name:ident, $sty:ty, $uty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $sty;
            let result = if b == 0 { a } else { a.wrapping_rem(b) };
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    result as $uty as usize,
                )))
            }
        }
    };
}

/// Generate signed scalar shift operations
macro_rules! signed_shift {
    ($name:ident, $sty:ty, $uty:ty, $bits:expr, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            let b = lean_runtime::lean_unbox(args[1].as_ptr()) as $uty;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box((a $op (b % $bits)) as $uty as usize))) }
        }
    };
}

/// Generate signed scalar complement operation
macro_rules! signed_complement {
    ($name:ident, $sty:ty, $uty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let a = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    (!a) as $uty as usize,
                )))
            }
        }
    };
}

/// Generate signed of_nat (truncate Nat to signed fixed-width)
macro_rules! signed_of_nat {
    ($name:ident, $sty:ty, $uty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let n = lean_runtime::lean_unbox(args[0].as_ptr()) as $sty;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    n as $uty as usize,
                )))
            }
        }
    };
}

pub(crate) use signed_binop;
pub(crate) use signed_bitop;
pub(crate) use signed_cmp;
pub(crate) use signed_complement;
pub(crate) use signed_div;
pub(crate) use signed_mod;
pub(crate) use signed_of_nat;
pub(crate) use signed_shift;
pub(crate) use signed_unary;

// ============================================================================
// Macros for float operations
// ============================================================================

/// Generate float64 binary operations
macro_rules! float64_binop {
    ($name:ident, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float(args[0].as_ptr());
                let b = lean_runtime::lean_unbox_float(args[1].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float(a $op b)))
            }
        }
    };
}

/// Generate float64 binary method operations (powf, atan2)
macro_rules! float64_binop_method {
    ($name:ident, $method:ident) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float(args[0].as_ptr());
                let b = lean_runtime::lean_unbox_float(args[1].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float(
                    a.$method(b),
                )))
            }
        }
    };
}

/// Generate float64 unary operations
macro_rules! float64_unary {
    ($name:ident, $method:ident) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float(args[0].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float(
                    a.$method(),
                )))
            }
        }
    };
}

/// Generate float64 comparison operations
macro_rules! float64_cmp {
    ($name:ident, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float(args[0].as_ptr());
                let b = lean_runtime::lean_unbox_float(args[1].as_ptr());
                Ok(LeanValue::from_bool(a $op b))
            }
        }
    };
}

/// Generate float64 predicate operations (is_nan, is_infinite, etc.)
macro_rules! float64_pred {
    ($name:ident, $method:ident) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float(args[0].as_ptr());
                Ok(LeanValue::from_bool(a.$method()))
            }
        }
    };
}

/// Generate float32 binary operations
macro_rules! float32_binop {
    ($name:ident, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float32(args[0].as_ptr());
                let b = lean_runtime::lean_unbox_float32(args[1].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float32(a $op b)))
            }
        }
    };
}

/// Generate float32 unary operations
macro_rules! float32_unary {
    ($name:ident, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float32(args[0].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float32($op a)))
            }
        }
    };
}

/// Generate float32 comparison operations
macro_rules! float32_cmp {
    ($name:ident, $op:tt) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float32(args[0].as_ptr());
                let b = lean_runtime::lean_unbox_float32(args[1].as_ptr());
                Ok(LeanValue::from_bool(a $op b))
            }
        }
    };
}

/// Generate float32 predicate operations
macro_rules! float32_pred {
    ($name:ident, $method:ident) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let a = lean_runtime::lean_unbox_float32(args[0].as_ptr());
                Ok(LeanValue::from_bool(a.$method()))
            }
        }
    };
}

/// Generate float to int conversion
macro_rules! float_to_int {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let f = lean_runtime::lean_unbox_float(args[0].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    f as $ty as usize,
                )))
            }
        }
    };
}

/// Generate int to float64 conversion
macro_rules! int_to_float64 {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let n = lean_runtime::lean_unbox(args[0].as_ptr()) as $ty;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box_float(n as f64))) }
        }
    };
}

/// Generate float32 to int conversion
macro_rules! float32_to_int {
    ($name:ident, $ty:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            unsafe {
                let f = lean_runtime::lean_unbox_float32(args[0].as_ptr());
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    f as $ty as usize,
                )))
            }
        }
    };
}

pub(crate) use float32_binop;
pub(crate) use float32_cmp;
pub(crate) use float32_pred;
pub(crate) use float32_to_int;
pub(crate) use float32_unary;
pub(crate) use float64_binop;
pub(crate) use float64_binop_method;
pub(crate) use float64_cmp;
pub(crate) use float64_pred;
pub(crate) use float64_unary;
pub(crate) use float_to_int;
pub(crate) use int_to_float64;

// ============================================================================
// Macros for type conversions
// ============================================================================

/// Bool to any unsigned int (get tag, box it)
macro_rules! bool_to_uint {
    ($name:ident) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let b = args[0].tag();
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box(b))) }
        }
    };
}

/// Unsigned int to unsigned int conversion
macro_rules! uint_to_uint {
    ($name:ident, $from:ty, $to:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = lean_runtime::lean_unbox(args[0].as_ptr()) as $from;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    (v as $to) as usize,
                )))
            }
        }
    };
}

/// Unsigned int to float64 conversion
macro_rules! uint_to_float64 {
    ($name:ident, $from:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = lean_runtime::lean_unbox(args[0].as_ptr()) as $from;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box_float(v as f64))) }
        }
    };
}

/// Unsigned int to float32 conversion
macro_rules! uint_to_float32 {
    ($name:ident, $from:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = lean_runtime::lean_unbox(args[0].as_ptr()) as $from;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float32(
                    v as f32,
                )))
            }
        }
    };
}

/// Signed int to signed int conversion (two's complement)
macro_rules! sint_to_sint {
    ($name:ident, $from:ty, $to:ty, $unsigned:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = lean_runtime::lean_unbox(args[0].as_ptr()) as $from;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box(
                    (v as $to as $unsigned) as usize,
                )))
            }
        }
    };
}

/// Signed int to float64 conversion
macro_rules! sint_to_float64 {
    ($name:ident, $from:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = lean_runtime::lean_unbox(args[0].as_ptr()) as $from;
            unsafe { Ok(LeanValue::from_raw(lean_runtime::lean_box_float(v as f64))) }
        }
    };
}

/// Signed int to float32 conversion
macro_rules! sint_to_float32 {
    ($name:ident, $from:ty) => {
        fn $name(args: &[LeanValue]) -> Result<LeanValue> {
            let v = lean_runtime::lean_unbox(args[0].as_ptr()) as $from;
            unsafe {
                Ok(LeanValue::from_raw(lean_runtime::lean_box_float32(
                    v as f32,
                )))
            }
        }
    };
}

pub(crate) use bool_to_uint;
pub(crate) use sint_to_float32;
pub(crate) use sint_to_float64;
pub(crate) use sint_to_sint;
pub(crate) use uint_to_float32;
pub(crate) use uint_to_float64;
pub(crate) use uint_to_uint;

/// Register all built-in extern functions
pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    let mut builtins = Vec::new();

    builtins.extend(string::get_builtins());
    builtins.extend(nat::get_builtins());
    builtins.extend(uint::get_builtins());
    builtins.extend(int::get_builtins());
    builtins.extend(sint::get_builtins());
    builtins.extend(convert::get_builtins());
    builtins.extend(array::get_builtins());
    builtins.extend(list::get_builtins());
    builtins.extend(io::get_builtins());
    builtins.extend(float::get_builtins());
    builtins.extend(refs::get_builtins());
    builtins.extend(system::get_builtins());
    builtins.extend(format::get_builtins());
    builtins.extend(misc::get_builtins());
    builtins.extend(thunk::get_builtins());
    builtins.extend(task::get_builtins());

    builtins
}

/// Create an IO result (Ok constructor with value + world token)
pub fn io_result_ok_val(val: LeanValue) -> LeanValue {
    unsafe {
        use lean_runtime::lean_io_result_mk_ok;
        let result = lean_io_result_mk_ok(val.into_raw());
        LeanValue::from_raw(result)
    }
}

/// Create an IO error result
pub fn io_result_error(msg: &str) -> LeanValue {
    unsafe {
        use lean_runtime::{lean_alloc_ctor, lean_ctor_set};
        // IO.Error.userError (msg : String) : IO.Error
        let msg_obj = LeanValue::from_string(msg).into_raw();
        let err = lean_alloc_ctor(6, 1, 0); // userError is tag 6
        lean_ctor_set(err, 0, msg_obj);

        // EStateM.Result.error (error : ε) : Result ε σ α
        let result = lean_alloc_ctor(1, 1, 0); // error is tag 1
        lean_ctor_set(result, 0, err);
        LeanValue::from_raw(result)
    }
}
