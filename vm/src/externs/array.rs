//! Array operations

use super::{ExternFn, Result};
use crate::value::LeanValue;
use crate::VMError;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        ("lean_array_mk", array_mk as ExternFn),
        ("lean_mk_array", mk_array),
        (
            "lean_mk_empty_array_with_capacity",
            array_empty_with_capacity,
        ),
        ("lean_array_get_size", array_get_size),
        ("lean_array_size", array_get_size),
        ("lean_array_push", array_push),
        ("lean_array_get", array_get),
        ("lean_array_get_borrowed", array_get),
        ("lean_array_fget", array_fget),
        ("lean_array_fget_borrowed", array_fget_borrowed),
        ("lean_array_to_list", array_to_list),
        ("lean_array_uget", array_uget),
        ("lean_array_uset", array_uset),
        ("lean_array_fset", array_fset),
        ("lean_array_set", array_set),
        ("lean_array_pop", array_pop),
        ("lean_array_uswap", array_uswap),
        ("lean_array_fswap", array_fswap),
        ("lean_array_swap", array_swap),
        ("lean_copy_array", copy_array),
        // Additional Array operations
        ("lean_array_data", array_data),
        ("lean_array_set_panic", array_set_panic),
        ("lean_array_get_panic", array_get_panic),
        // ByteArray operations
        ("lean_byte_array_mk", byte_array_mk),
        ("lean_byte_array_data", byte_array_data),
        ("lean_mk_empty_byte_array", byte_array_empty),
        ("lean_byte_array_push", byte_array_push),
        ("lean_byte_array_size", byte_array_size),
        ("lean_byte_array_uget", byte_array_uget),
        ("lean_byte_array_uset", byte_array_uset),
        ("lean_byte_array_copy", byte_array_copy),
        ("lean_byte_array_dec_eq", byte_array_dec_eq),
        // FloatArray operations (stubs)
        ("lean_float_array_mk", float_array_mk),
        ("lean_float_array_push", float_array_push),
        ("lean_float_array_size", float_array_size),
        ("lean_float_array_uget", float_array_uget),
        ("lean_float_array_uset", float_array_uset),
    ]
}

fn array_mk(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_array_mk(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn mk_array(args: &[LeanValue]) -> Result<LeanValue> {
    // lean_mk_array(n, v) creates array of size n filled with value v
    unsafe {
        let result = lean_mk_array(args[0].as_ptr(), args[1].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn array_empty_with_capacity(args: &[LeanValue]) -> Result<LeanValue> {
    // Bytecode may pass (default, capacity) or just (capacity)
    unsafe {
        let cap = if args.len() == 2 {
            args[1].as_ptr()
        } else {
            args[0].as_ptr()
        };
        let result = lean_mk_empty_array_with_capacity(cap);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_get_size(args: &[LeanValue]) -> Result<LeanValue> {
    // Bytecode may pass (default, array) or just (array)
    unsafe {
        let arr = if args.len() == 2 {
            args[1].as_ptr()
        } else {
            args[0].as_ptr()
        };
        let result = lean_array_get_size(arr);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_push(args: &[LeanValue]) -> Result<LeanValue> {
    // Bytecode passes (default, array, value) but runtime takes (array, value)
    unsafe {
        let (arr, val) = if args.len() == 3 {
            (args[1].as_ptr(), args[2].as_ptr())
        } else {
            (args[0].as_ptr(), args[1].as_ptr())
        };

        let result = lean_array_push(arr, val);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_get(args: &[LeanValue]) -> Result<LeanValue> {
    // Runtime: lean_array_get(def, array, index)
    // Bytecode may pass an extra leading default
    unsafe {
        let (def, arr, idx) = if args.len() == 4 {
            (args[1].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
        } else {
            (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr())
        };
        let result = lean_array_get(def, arr, idx);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_fget(args: &[LeanValue]) -> Result<LeanValue> {
    // Array.get! has extra params for type and proofs
    // Runtime just needs (array, index)
    unsafe {
        // Find the array by tag
        let mut arr_idx = None;
        for (i, arg) in args.iter().enumerate() {
            let ptr = arg.as_ptr();
            if !lean_is_scalar(ptr) && lean_obj_tag(ptr) == 246 {
                arr_idx = Some(i);
                break;
            }
        }
        let (arr, idx) = if let Some(ai) = arr_idx {
            // Index is the next argument after the array
            let ii = ai + 1;
            if ii < args.len() {
                (args[ai].as_ptr(), args[ii].as_ptr())
            } else {
                (args[ai].as_ptr(), args[args.len() - 1].as_ptr())
            }
        } else {
            // Fallback to last two args
            (args[args.len() - 2].as_ptr(), args[args.len() - 1].as_ptr())
        };
        let result = lean_array_fget(arr, idx);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_fget_borrowed(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        // Find the array - it's the non-scalar argument that has array tag (246)
        let mut arr_idx = None;
        let mut idx_idx = None;
        for (i, arg) in args.iter().enumerate() {
            let ptr = arg.as_ptr();
            if !lean_is_scalar(ptr) {
                let tag = lean_obj_tag(ptr);
                if tag == 246 {
                    // Array tag
                    arr_idx = Some(i);
                }
            }
        }

        // The index is the next argument after the array
        if let Some(ai) = arr_idx {
            // The first argument after the array is the index
            if ai + 1 < args.len() {
                idx_idx = Some(ai + 1);
            }
        }

        let (arr, idx) = if let (Some(ai), Some(ii)) = (arr_idx, idx_idx) {
            (args[ai].as_ptr(), args[ii].as_ptr())
        } else {
            // Fallback: last two args
            (args[args.len() - 2].as_ptr(), args[args.len() - 1].as_ptr())
        };

        // Use borrowed variant - does NOT increment RC
        let result = lean_array_fget_borrowed(arr, idx);
        // Need to inc because we're returning a new LeanValue that will be dropped
        lean_inc(result);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_to_list(args: &[LeanValue]) -> Result<LeanValue> {
    // Bytecode may pass (default, array) or just (array)
    unsafe {
        let arr = if args.len() == 2 {
            args[1].as_ptr()
        } else {
            args[0].as_ptr()
        };
        let result = lean_array_to_list(arr);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_uget(args: &[LeanValue]) -> Result<LeanValue> {
    // lean_array_uget(array, usize_index)
    // May receive extra type/universe params that should be ignored
    unsafe {
        if args.len() < 2 {
            return Err(VMError::TypeMismatch("array_uget: not enough arguments"));
        }
        // Find the array argument (first non-scalar with tag 246)
        let mut arr_idx = 0;
        for (i, arg) in args.iter().enumerate() {
            let ptr = arg.as_ptr();
            if !lean_is_scalar(ptr) && lean_obj_tag(ptr) == 246 {
                arr_idx = i;
                break;
            }
        }
        let arr = args[arr_idx].as_ptr();
        let idx = lean_unbox(args[arr_idx + 1].as_ptr());

        let result = lean_array_uget(arr, idx);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_uset(args: &[LeanValue]) -> Result<LeanValue> {
    // lean_array_uset(array, usize_index, value)
    unsafe {
        let arr = args[0].as_ptr();
        let idx = lean_unbox(args[1].as_ptr());
        let val = args[2].as_ptr();
        let result = lean_array_uset(arr, idx, val);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_fset(args: &[LeanValue]) -> Result<LeanValue> {
    // lean_array_fset(array, nat_index, value)
    unsafe {
        let arr = args[0].as_ptr();
        let idx = args[1].as_ptr();
        let val = args[2].as_ptr();
        let result = lean_array_fset(arr, idx, val);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_set(args: &[LeanValue]) -> Result<LeanValue> {
    // lean_array_set(array, nat_index, value)
    unsafe {
        let (arr, idx, val) = if args.len() == 4 {
            (args[1].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
        } else {
            (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr())
        };
        let result = lean_array_set(arr, idx, val);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_pop(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = if args.len() == 2 {
            args[1].as_ptr()
        } else {
            args[0].as_ptr()
        };
        let result = lean_array_pop(arr);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_uswap(args: &[LeanValue]) -> Result<LeanValue> {
    // array_uswap(a, i, j) - swap elements at indices i and j
    unsafe {
        let arr = args[0].as_ptr();
        let i = lean_unbox(args[1].as_ptr());
        let j = lean_unbox(args[2].as_ptr());
        // Get both elements
        let vi = lean_array_uget(arr, i);
        let vj = lean_array_uget(arr, j);
        // Set them at swapped positions
        let r = lean_array_uset(arr, i, vj);
        let r2 = lean_array_uset(r, j, vi);
        Ok(LeanValue::from_raw(r2))
    }
}

fn array_fswap(args: &[LeanValue]) -> Result<LeanValue> {
    // Array.fswap has signature: (a : Array α) (i j : Nat) (hi : i < a.size) (hj : j < a.size)
    // Bytecode may pass extra args for type params and proofs
    unsafe {
        let (arr, i, j) = match args.len() {
            3 => (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr()),
            4 => (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr()),
            5 => (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr()),
            6 => {
                // Check which arg is the array by looking for tag=246
                if !lean_is_scalar(args[0].as_ptr()) && lean_obj_tag(args[0].as_ptr()) == 246 {
                    (args[0].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
                } else if !lean_is_scalar(args[1].as_ptr()) && lean_obj_tag(args[1].as_ptr()) == 246
                {
                    (args[1].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
                } else {
                    // Fallback
                    (args[0].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
                }
            }
            _ => (
                args[0].as_ptr(),
                args[args.len() - 4].as_ptr(),
                args[args.len() - 3].as_ptr(),
            ),
        };
        let result = lean_array_fswap(arr, i, j);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_swap(args: &[LeanValue]) -> Result<LeanValue> {
    // Similar to fswap, may have extra type/proof params
    unsafe {
        let (arr, i, j) = match args.len() {
            3 => (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr()),
            4 => (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr()),
            5 => (args[0].as_ptr(), args[1].as_ptr(), args[2].as_ptr()),
            _ => {
                // Find array by tag
                if !lean_is_scalar(args[0].as_ptr()) && lean_obj_tag(args[0].as_ptr()) == 246 {
                    (args[0].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
                } else if args.len() > 1
                    && !lean_is_scalar(args[1].as_ptr())
                    && lean_obj_tag(args[1].as_ptr()) == 246
                {
                    (args[1].as_ptr(), args[2].as_ptr(), args[3].as_ptr())
                } else {
                    (
                        args[0].as_ptr(),
                        args[args.len() - 4].as_ptr(),
                        args[args.len() - 3].as_ptr(),
                    )
                }
            }
        };
        let result = lean_array_swap(arr, i, j);
        Ok(LeanValue::from_raw(result))
    }
}

fn copy_array(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = args[0].as_ptr();
        let result = lean_copy_array(arr);
        Ok(LeanValue::from_raw(result))
    }
}

// ByteArray operations
fn byte_array_mk(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_byte_array_mk(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn byte_array_data(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_byte_array_data(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn byte_array_empty(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_mk_empty_byte_array(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn byte_array_push(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let byte = lean_unbox(args[1].as_ptr()) as u8;
        let result = lean_byte_array_push(args[0].as_ptr(), byte);
        Ok(LeanValue::from_raw(result))
    }
}

fn byte_array_size(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_byte_array_size(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

// Additional Array operations
fn array_data(args: &[LeanValue]) -> Result<LeanValue> {
    // Return array data pointer as opaque value (for FFI)
    unsafe {
        let arr = args[0].as_ptr();
        let data = lean_array_data(arr);
        Ok(LeanValue::from_raw(lean_box(data as usize)))
    }
}

fn array_set_panic(args: &[LeanValue]) -> Result<LeanValue> {
    // Array.set! with panic on out-of-bounds
    unsafe {
        let arr = args[0].as_ptr();
        let idx = args[1].as_ptr();
        let val = args[2].as_ptr();
        let size = lean_unbox(lean_array_get_size(arr));
        let i = lean_unbox(idx);
        if i >= size {
            panic!("Array.set!: index {} out of bounds (size {})", i, size);
        }
        let result = lean_array_fset(arr, idx, val);
        Ok(LeanValue::from_raw(result))
    }
}

fn array_get_panic(args: &[LeanValue]) -> Result<LeanValue> {
    // Array.get! with panic on out-of-bounds
    unsafe {
        let arr = args[0].as_ptr();
        let idx = args[1].as_ptr();
        let size = lean_unbox(lean_array_get_size(arr));
        let i = lean_unbox(idx);
        if i >= size {
            panic!("Array.get!: index {} out of bounds (size {})", i, size);
        }
        let result = lean_array_fget(arr, idx);
        Ok(LeanValue::from_raw(result))
    }
}

// Additional ByteArray operations
fn byte_array_uget(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = args[0].as_ptr();
        let idx = lean_unbox(args[1].as_ptr());
        let byte = lean_byte_array_uget(arr, idx);
        Ok(LeanValue::from_raw(lean_box(byte as usize)))
    }
}

fn byte_array_uset(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = args[0].as_ptr();
        let idx = lean_unbox(args[1].as_ptr());
        let byte = lean_unbox(args[2].as_ptr()) as u8;
        let result = lean_byte_array_uset(arr, idx, byte);
        Ok(LeanValue::from_raw(result))
    }
}

fn byte_array_copy(args: &[LeanValue]) -> Result<LeanValue> {
    // Copy a range of bytes from one array to another
    unsafe {
        let src = args[0].as_ptr();
        let dest = args[1].as_ptr();
        let dest_off = lean_unbox(args[2].as_ptr());
        let src_off = lean_unbox(args[3].as_ptr());
        let count = lean_unbox(args[4].as_ptr());

        // For simplicity, do element-by-element copy
        let mut result = dest;
        for i in 0..count {
            let byte = lean_byte_array_uget(src, src_off + i);
            result = lean_byte_array_uset(result, dest_off + i, byte);
        }
        Ok(LeanValue::from_raw(result))
    }
}

fn byte_array_dec_eq(args: &[LeanValue]) -> Result<LeanValue> {
    // Compare two byte arrays for equality
    unsafe {
        let a = args[0].as_ptr();
        let b = args[1].as_ptr();
        let size_a = lean_unbox(lean_byte_array_size(a));
        let size_b = lean_unbox(lean_byte_array_size(b));
        if size_a != size_b {
            return Ok(LeanValue::from_bool(false));
        }
        for i in 0..size_a {
            if lean_byte_array_uget(a, i) != lean_byte_array_uget(b, i) {
                return Ok(LeanValue::from_bool(false));
            }
        }
        Ok(LeanValue::from_bool(true))
    }
}

// FloatArray operations (using regular arrays with boxed floats)
fn float_array_mk(_args: &[LeanValue]) -> Result<LeanValue> {
    // Create float array from list (stub - create empty)
    unsafe {
        let arr = lean_mk_empty_array_with_capacity(lean_box(0));
        Ok(LeanValue::from_raw(arr))
    }
}

fn float_array_push(args: &[LeanValue]) -> Result<LeanValue> {
    // Push float onto array
    unsafe {
        let arr = args[0].as_ptr();
        let f = args[1].as_ptr();
        let result = lean_array_push(arr, f);
        Ok(LeanValue::from_raw(result))
    }
}

fn float_array_size(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_array_get_size(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn float_array_uget(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = args[0].as_ptr();
        let idx = lean_unbox(args[1].as_ptr());
        let result = lean_array_uget(arr, idx);
        Ok(LeanValue::from_raw(result))
    }
}

fn float_array_uset(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = args[0].as_ptr();
        let idx = lean_unbox(args[1].as_ptr());
        let f = args[2].as_ptr();
        let result = lean_array_uset(arr, idx, f);
        Ok(LeanValue::from_raw(result))
    }
}
