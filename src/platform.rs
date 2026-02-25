//! Platform, version, and miscellaneous helper functions.

use crate::object::LeanObject;

// Version info (hardcoded for the Rust backend)
pub unsafe fn lean_version_get_major(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(4)
}
pub unsafe fn lean_version_get_minor(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(0)
}
pub unsafe fn lean_version_get_patch(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(0)
}
pub unsafe fn lean_version_get_is_release(_unit: *mut LeanObject) -> u8 {
    0
}
pub unsafe fn lean_version_get_special_desc(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::string::lean_mk_string("rust-backend")
}

// Platform
pub unsafe fn lean_system_platform_target(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::string::lean_mk_string(std::env::consts::ARCH)
}

pub fn lean_internal_is_stage0(_unit: *mut LeanObject) -> u8 {
    0
}
pub fn lean_internal_has_llvm_backend(_unit: *mut LeanObject) -> u8 {
    0
}

// Ctor limits
pub fn lean_get_max_ctor_fields(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(256)
}
pub fn lean_get_max_ctor_scalars_size(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(1024)
}
pub fn lean_get_usize_size(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(std::mem::size_of::<usize>())
}
pub fn lean_get_max_ctor_tag(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(243)
}

// Strict logical operators (both args already evaluated)
pub fn lean_strict_or(a: u8, b: u8) -> u8 {
    (a != 0 || b != 0) as u8
}
pub fn lean_strict_and(a: u8, b: u8) -> u8 {
    (a != 0 && b != 0) as u8
}

// Pointer address
pub fn lean_ptr_addr(a: *mut LeanObject) -> usize {
    a as usize
}

// Void
pub unsafe fn lean_void_mk(a: *mut LeanObject) -> *mut LeanObject {
    crate::lean_dec(a);
    crate::lean_box(0)
}

// HashMap/HashSet index helpers
pub fn lean_hashmap_mk_idx(sz: *mut LeanObject, hash: u64) -> usize {
    let s = crate::lean_unbox(sz);
    (hash as usize) & (s - 1)
}

pub fn lean_hashset_mk_idx(sz: *mut LeanObject, hash: u64) -> usize {
    let s = crate::lean_unbox(sz);
    (hash as usize) & (s - 1)
}

// Runtime initialization stubs
pub unsafe fn lean_initialize_runtime_module() -> *mut LeanObject {
    crate::io::lean_io_result_mk_ok(crate::lean_box(0))
}

pub fn lean_io_mark_end_initialization() {}
pub fn lean_set_panic_messages(_val: u8) {}
pub fn lean_init_task_manager() {}
pub fn lean_finalize_task_manager() {}

// IO result error display
pub unsafe fn lean_io_result_show_error(r: *mut LeanObject) {
    if crate::io::lean_io_result_is_ok(r) {
        return;
    }
    let err = crate::lean_ctor_get(r, 0);
    // Try to extract a string from the error
    eprintln!("[lean-runtime] IO error");
    let _ = err;
}

// IO initializing flag
pub unsafe fn lean_io_initializing(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::io::lean_io_result_mk_ok(crate::lean_box(0))
}

// Debug sleep
pub unsafe fn lean_dbg_sleep(ms: u32, thunk: *mut LeanObject) -> *mut LeanObject {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    crate::lean_apply_1(thunk, crate::lean_box(0))
}

// lean_is_exclusive_obj - boxed version
pub unsafe fn lean_is_exclusive_obj(a: *mut LeanObject) -> *mut LeanObject {
    let r = if crate::lean_is_scalar(a) {
        0
    } else {
        crate::lean_is_exclusive(a) as usize
    };
    crate::lean_box(r)
}

// Name hash
pub unsafe fn lean_name_hash(n: *mut LeanObject) -> u64 {
    if crate::lean_is_scalar(n) {
        1723
    } else {
        crate::lean_ctor_get_uint64(n, (std::mem::size_of::<*mut LeanObject>() * 2) as u32)
    }
}

// Expr data
pub unsafe fn lean_expr_data(expr: *mut LeanObject) -> u64 {
    let num_objs = (*expr).other as u32;
    let offset = num_objs as usize * std::mem::size_of::<*mut LeanObject>();
    crate::lean_ctor_get_uint64(expr, offset as u32)
}

// Nat.gcd and Nat.divExact stubs (used by Int)
pub unsafe fn lean_nat_gcd(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    // Euclidean GCD algorithm
    let mut x = a;
    let mut y = b;
    crate::lean_inc(x);
    crate::lean_inc(y);
    while !crate::lean_is_scalar(y) || crate::lean_unbox(y) != 0 {
        let temp = crate::nat::lean_nat_mod(x, y);
        crate::lean_inc(y); // y is borrowed by mod
        x = y;
        y = temp;
    }
    // Don't dec the final y (it's lean_box(0), a scalar)
    x
}

pub unsafe fn lean_nat_div_exact(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    crate::nat::lean_nat_div(a, b)
}

// lean_get_githash stub
pub unsafe fn lean_get_githash(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::string::lean_mk_string("unknown")
}

// lean_manual_get_root stub
pub unsafe fn lean_manual_get_root(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::string::lean_mk_string("")
}

// Runtime mark functions (IO-wrapped stubs)
pub unsafe fn lean_runtime_mark_multi_threaded(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::io::lean_io_result_mk_ok(crate::lean_box(0))
}

pub unsafe fn lean_runtime_mark_persistent(
    a: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    if !crate::lean_is_scalar(a) {
        crate::lean_mark_persistent(a);
    }
    crate::io::lean_io_result_mk_ok(crate::lean_box(0))
}

pub unsafe fn lean_runtime_forget(a: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    // "Forget" the object - don't dec it
    let _ = a;
    crate::io::lean_io_result_mk_ok(crate::lean_box(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_info() {
        unsafe {
            let unit = crate::lean_box(0);
            assert_eq!(crate::lean_unbox(lean_version_get_major(unit)), 4);
            assert_eq!(crate::lean_unbox(lean_version_get_minor(unit)), 0);
            assert_eq!(crate::lean_unbox(lean_version_get_patch(unit)), 0);
            assert_eq!(lean_version_get_is_release(unit), 0);
            let desc = lean_version_get_special_desc(unit);
            assert_eq!(crate::lean_string_to_str(desc), "rust-backend");
            crate::lean_dec(desc);
        }
    }

    #[test]
    fn strict_logic() {
        assert_eq!(lean_strict_or(0, 0), 0);
        assert_eq!(lean_strict_or(1, 0), 1);
        assert_eq!(lean_strict_or(0, 1), 1);
        assert_eq!(lean_strict_or(1, 1), 1);
        assert_eq!(lean_strict_and(0, 0), 0);
        assert_eq!(lean_strict_and(1, 0), 0);
        assert_eq!(lean_strict_and(0, 1), 0);
        assert_eq!(lean_strict_and(1, 1), 1);
    }

    #[test]
    fn ctor_limits() {
        let unit = crate::lean_box(0);
        assert_eq!(crate::lean_unbox(lean_get_max_ctor_fields(unit)), 256);
        assert_eq!(
            crate::lean_unbox(lean_get_max_ctor_scalars_size(unit)),
            1024
        );
        assert_eq!(
            crate::lean_unbox(lean_get_usize_size(unit)),
            std::mem::size_of::<usize>()
        );
        assert_eq!(crate::lean_unbox(lean_get_max_ctor_tag(unit)), 243);
    }

    #[test]
    fn hashmap_idx() {
        // size=8 (power of 2), hash=13 => 13 & 7 = 5
        let sz = crate::lean_box(8);
        assert_eq!(lean_hashmap_mk_idx(sz, 13), 5);
        assert_eq!(lean_hashset_mk_idx(sz, 13), 5);
    }

    #[test]
    fn void_mk_decs_arg() {
        unsafe {
            let s = crate::lean_mk_string("discard me");
            let result = lean_void_mk(s);
            // result should be lean_box(0) = unit
            assert!(crate::lean_is_scalar(result));
            assert_eq!(crate::lean_unbox(result), 0);
        }
    }

    #[test]
    fn initialize_runtime_module() {
        unsafe {
            let r = lean_initialize_runtime_module();
            assert!(crate::io::lean_io_result_is_ok(r));
            crate::lean_dec(r);
        }
    }

    #[test]
    fn platform_target_non_empty() {
        unsafe {
            let s = lean_system_platform_target(crate::lean_box(0));
            let str_val = crate::lean_string_to_str(s);
            assert!(!str_val.is_empty());
            crate::lean_dec(s);
        }
    }
}
