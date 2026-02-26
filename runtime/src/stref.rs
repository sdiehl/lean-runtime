//! ST/IO Ref primitives.

use crate::object::LeanObject;

const LEAN_REF_TAG: u8 = 253;

#[repr(C)]
pub struct LeanRef {
    pub header: LeanObject,
    pub value: *mut LeanObject,
}

/// ST.mkRef: create a new mutable reference.
pub unsafe fn lean_st_mk_ref(a: *mut LeanObject) -> *mut LeanObject {
    let size = std::mem::size_of::<LeanRef>();
    let obj = crate::object::lean_alloc_object(size);
    let r = obj as *mut LeanRef;
    (*r).header.rc = 1;
    (*r).header.tag = LEAN_REF_TAG;
    (*r).header.other = 0;
    (*r).header.cs_sz = 0;
    (*r).value = a;
    crate::io::lean_io_result_mk_ok(obj)
}

/// ST.Ref.get: read the reference value (borrowed ref arg).
pub unsafe fn lean_st_ref_get(ref_obj: *mut LeanObject) -> *mut LeanObject {
    let r = ref_obj as *mut LeanRef;
    let v = (*r).value;
    crate::lean_inc(v);
    crate::io::lean_io_result_mk_ok(v)
}

/// ST.Ref.set: write a new value (borrowed ref arg, owned new value).
pub unsafe fn lean_st_ref_set(
    ref_obj: *mut LeanObject,
    new_val: *mut LeanObject,
) -> *mut LeanObject {
    let r = ref_obj as *mut LeanRef;
    let old = (*r).value;
    (*r).value = new_val;
    crate::lean_dec(old);
    crate::io::lean_io_result_mk_ok(crate::lean_box(0))
}

/// ST.Ref.reset: reset to unit (lean_box(0)).
pub unsafe fn lean_st_ref_reset(ref_obj: *mut LeanObject) -> *mut LeanObject {
    lean_st_ref_set(ref_obj, crate::lean_box(0))
}

/// ST.Ref.swap: swap the value, returning the old one.
pub unsafe fn lean_st_ref_swap(
    ref_obj: *mut LeanObject,
    new_val: *mut LeanObject,
) -> *mut LeanObject {
    let r = ref_obj as *mut LeanRef;
    let old = (*r).value;
    (*r).value = new_val;
    crate::io::lean_io_result_mk_ok(old)
}

/// ST.Ref.take: take the value out, replacing with unit.
pub unsafe fn lean_st_ref_take(ref_obj: *mut LeanObject) -> *mut LeanObject {
    let r = ref_obj as *mut LeanRef;
    let old = (*r).value;
    (*r).value = crate::lean_box(0);
    crate::io::lean_io_result_mk_ok(old)
}

/// ST.Ref.ptrEq: check if two refs are the same pointer.
pub unsafe fn lean_st_ref_ptr_eq(a: *mut LeanObject, b: *mut LeanObject) -> *mut LeanObject {
    let eq = (a == b) as u8;
    crate::io::lean_io_result_mk_ok(crate::lean_box(eq as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mk_ref_and_get() {
        unsafe {
            let v = crate::lean_box(42);
            let io_res = lean_st_mk_ref(v);
            assert!(crate::io::lean_io_result_is_ok(io_res));
            let ref_obj = crate::io::lean_io_result_get_value(io_res);

            let get_res = lean_st_ref_get(ref_obj);
            assert!(crate::io::lean_io_result_is_ok(get_res));
            let val = crate::io::lean_io_result_get_value(get_res);
            assert_eq!(crate::lean_unbox(val), 42);

            // Clean up
            crate::lean_dec(get_res);
            crate::lean_dec(io_res);
        }
    }

    #[test]
    fn ref_set() {
        unsafe {
            let v = crate::lean_box(1);
            let io_res = lean_st_mk_ref(v);
            let ref_obj = crate::io::lean_io_result_get_value(io_res);

            let set_res = lean_st_ref_set(ref_obj, crate::lean_box(2));
            assert!(crate::io::lean_io_result_is_ok(set_res));
            crate::lean_dec(set_res);

            let get_res = lean_st_ref_get(ref_obj);
            let val = crate::io::lean_io_result_get_value(get_res);
            assert_eq!(crate::lean_unbox(val), 2);

            crate::lean_dec(get_res);
            crate::lean_dec(io_res);
        }
    }

    #[test]
    fn ref_swap() {
        unsafe {
            let v = crate::lean_box(10);
            let io_res = lean_st_mk_ref(v);
            let ref_obj = crate::io::lean_io_result_get_value(io_res);

            let swap_res = lean_st_ref_swap(ref_obj, crate::lean_box(20));
            assert!(crate::io::lean_io_result_is_ok(swap_res));
            let old = crate::io::lean_io_result_get_value(swap_res);
            assert_eq!(crate::lean_unbox(old), 10);
            crate::lean_dec(swap_res);

            let get_res = lean_st_ref_get(ref_obj);
            let val = crate::io::lean_io_result_get_value(get_res);
            assert_eq!(crate::lean_unbox(val), 20);

            crate::lean_dec(get_res);
            crate::lean_dec(io_res);
        }
    }

    #[test]
    fn ref_take() {
        unsafe {
            let v = crate::lean_box(55);
            let io_res = lean_st_mk_ref(v);
            let ref_obj = crate::io::lean_io_result_get_value(io_res);

            let take_res = lean_st_ref_take(ref_obj);
            assert!(crate::io::lean_io_result_is_ok(take_res));
            let taken = crate::io::lean_io_result_get_value(take_res);
            assert_eq!(crate::lean_unbox(taken), 55);
            crate::lean_dec(take_res);

            // After take, ref holds unit (lean_box(0))
            let get_res = lean_st_ref_get(ref_obj);
            let val = crate::io::lean_io_result_get_value(get_res);
            assert_eq!(crate::lean_unbox(val), 0);

            crate::lean_dec(get_res);
            crate::lean_dec(io_res);
        }
    }

    #[test]
    fn ref_ptr_eq() {
        unsafe {
            let v = crate::lean_box(1);
            let io_res = lean_st_mk_ref(v);
            let ref_obj = crate::io::lean_io_result_get_value(io_res);

            let eq_res = lean_st_ref_ptr_eq(ref_obj, ref_obj);
            assert!(crate::io::lean_io_result_is_ok(eq_res));
            let eq_val = crate::io::lean_io_result_get_value(eq_res);
            assert_eq!(crate::lean_unbox(eq_val), 1);
            crate::lean_dec(eq_res);

            // Different refs should not be equal
            let v2 = crate::lean_box(2);
            let io_res2 = lean_st_mk_ref(v2);
            let ref_obj2 = crate::io::lean_io_result_get_value(io_res2);

            let neq_res = lean_st_ref_ptr_eq(ref_obj, ref_obj2);
            let neq_val = crate::io::lean_io_result_get_value(neq_res);
            assert_eq!(crate::lean_unbox(neq_val), 0);
            crate::lean_dec(neq_res);

            crate::lean_dec(io_res2);
            crate::lean_dec(io_res);
        }
    }
}
