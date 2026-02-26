//! Miscellaneous runtime functions.

use crate::object::LeanObject;

/// Structural equality on Name constructors.
/// Name is: `.anonymous` (tag 0, 0 fields) | `.str parent string` (tag 1, 2 fields) | `.num parent nat` (tag 2, 2 fields).
pub unsafe fn lean_name_eq(n1: *mut LeanObject, n2: *mut LeanObject) -> u8 {
    lean_name_eq_inner(n1, n2) as u8
}

unsafe fn lean_name_eq_inner(n1: *mut LeanObject, n2: *mut LeanObject) -> bool {
    if n1 == n2 {
        return true;
    }
    if crate::lean_is_scalar(n1) || crate::lean_is_scalar(n2) {
        return n1 == n2;
    }
    let tag1 = crate::lean_obj_tag(n1);
    let tag2 = crate::lean_obj_tag(n2);
    if tag1 != tag2 {
        return false;
    }
    match tag1 {
        0 => true, // both anonymous
        1 => {
            // .str parent string
            let s1 = crate::lean_ctor_get(n1, 1);
            let s2 = crate::lean_ctor_get(n2, 1);
            if !crate::lean_string_eq(s1, s2) {
                return false;
            }
            let p1 = crate::lean_ctor_get(n1, 0);
            let p2 = crate::lean_ctor_get(n2, 0);
            lean_name_eq_inner(p1, p2)
        }
        2 => {
            // .num parent nat
            let v1 = crate::lean_ctor_get(n1, 1);
            let v2 = crate::lean_ctor_get(n2, 1);
            if !crate::nat::lean_nat_eq(v1, v2) {
                return false;
            }
            let p1 = crate::lean_ctor_get(n1, 0);
            let p2 = crate::lean_ctor_get(n2, 0);
            lean_name_eq_inner(p1, p2)
        }
        _ => false,
    }
}

/// `sorry` axiom stub — returns `lean_box(0)` (unit/default value).
/// Signature matches `sorryAx (α : Sort u) (synthetic : Bool) : α`.
/// The type argument is erased; the IR unboxes the Bool to u8.
pub unsafe fn lean_sorry(_synthetic: u8) -> *mut LeanObject {
    crate::lean_box(0)
}

/// Return the platform's pointer size in bits (always 64 on 64-bit systems).
pub unsafe fn lean_system_platform_nbits(_unit: *mut LeanObject) -> *mut LeanObject {
    crate::io::lean_io_result_mk_ok(crate::lean_box(std::mem::size_of::<usize>() * 8))
}

/// dbg_trace: print the string, then evaluate the thunk.
pub unsafe fn lean_dbg_trace(s: *mut LeanObject, thunk: *mut LeanObject) -> *mut LeanObject {
    eprintln!("{}", crate::lean_string_to_str(s));
    crate::lean_dec(s);
    crate::lean_apply_1(thunk, crate::lean_box(0))
}

/// dbg_trace_if_shared: if the object is shared (rc > 1), print trace.
pub unsafe fn lean_dbg_trace_if_shared(s: *mut LeanObject, a: *mut LeanObject) -> *mut LeanObject {
    if !crate::lean_is_scalar(a) && (*a).rc > 1 {
        eprintln!("[lean-debug] shared: {}", crate::lean_string_to_str(s));
    }
    crate::lean_dec(s);
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorry_returns_unit() {
        unsafe {
            let r = lean_sorry(0u8);
            assert!(crate::lean_is_scalar(r));
            assert_eq!(crate::lean_unbox(r), 0);
        }
    }

    #[test]
    fn platform_nbits() {
        unsafe {
            let r = lean_system_platform_nbits(crate::lean_box(0));
            assert!(crate::io::lean_io_result_is_ok(r));
            let val = crate::io::lean_io_result_get_value(r);
            assert_eq!(crate::lean_unbox(val), 64);
            crate::lean_dec(r);
        }
    }

    #[test]
    fn name_eq_anonymous() {
        unsafe {
            let n1 = crate::lean_alloc_ctor(0, 0, 0);
            let n2 = crate::lean_alloc_ctor(0, 0, 0);
            assert_eq!(lean_name_eq(n1, n2), 1);
            crate::lean_dec(n1);
            crate::lean_dec(n2);
        }
    }

    #[test]
    fn name_eq_str() {
        unsafe {
            // .str .anonymous "foo"
            let anon1 = crate::lean_alloc_ctor(0, 0, 0);
            let s1 = crate::lean_mk_string("foo");
            let n1 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(n1, 0, anon1);
            crate::lean_ctor_set(n1, 1, s1);

            let anon2 = crate::lean_alloc_ctor(0, 0, 0);
            let s2 = crate::lean_mk_string("foo");
            let n2 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(n2, 0, anon2);
            crate::lean_ctor_set(n2, 1, s2);

            assert_eq!(lean_name_eq(n1, n2), 1);

            // Different name
            let anon3 = crate::lean_alloc_ctor(0, 0, 0);
            let s3 = crate::lean_mk_string("bar");
            let n3 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(n3, 0, anon3);
            crate::lean_ctor_set(n3, 1, s3);

            assert_eq!(lean_name_eq(n1, n3), 0);

            crate::lean_dec(n1);
            crate::lean_dec(n2);
            crate::lean_dec(n3);
        }
    }
}
