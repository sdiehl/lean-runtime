//! Thunk primitives.

use crate::object::LeanObject;

const LEAN_THUNK_TAG: u8 = 247;

#[repr(C)]
pub struct LeanThunk {
    pub header: LeanObject,
    pub value: *mut LeanObject,
    pub closure: *mut LeanObject,
}

/// Create a thunk from a closure (Unit -> A).
pub unsafe fn lean_mk_thunk(c: *mut LeanObject) -> *mut LeanObject {
    let size = std::mem::size_of::<LeanThunk>();
    let obj = crate::object::lean_alloc_object(size);
    let t = obj as *mut LeanThunk;
    (*t).header.rc = 1;
    (*t).header.tag = LEAN_THUNK_TAG;
    (*t).header.other = 0;
    (*t).header.cs_sz = 0;
    (*t).value = std::ptr::null_mut();
    (*t).closure = c;
    obj
}

/// Thunk.pure: wrap a value as a thunk.
pub unsafe fn lean_thunk_pure(v: *mut LeanObject) -> *mut LeanObject {
    let size = std::mem::size_of::<LeanThunk>();
    let obj = crate::object::lean_alloc_object(size);
    let t = obj as *mut LeanThunk;
    (*t).header.rc = 1;
    (*t).header.tag = LEAN_THUNK_TAG;
    (*t).header.other = 0;
    (*t).header.cs_sz = 0;
    (*t).value = v;
    (*t).closure = std::ptr::null_mut();
    obj
}

/// Get the value of a thunk, forcing it if needed.
pub unsafe fn lean_thunk_get(t: *mut LeanObject) -> *mut LeanObject {
    let thunk = t as *mut LeanThunk;
    let v = (*thunk).value;
    if !v.is_null() {
        return v;
    }
    // Force the thunk
    lean_thunk_get_core(t)
}

/// Force a thunk by evaluating its closure.
pub unsafe fn lean_thunk_get_core(t: *mut LeanObject) -> *mut LeanObject {
    let thunk = t as *mut LeanThunk;
    let c = (*thunk).closure;
    if c.is_null() {
        return (*thunk).value; // Already forced or no closure
    }
    crate::lean_inc(c);
    let result = crate::lean_apply_1(c, crate::lean_box(0));
    // Store the result
    (*thunk).value = result;
    // Don't free the closure yet (might be shared)
    result
}

/// Thunk.get : Thunk A -> A (owned version)
pub unsafe fn lean_thunk_get_own(t: *mut LeanObject) -> *mut LeanObject {
    let r = lean_thunk_get(t);
    crate::lean_inc(r);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thunk_pure_returns_value() {
        unsafe {
            let v = crate::lean_box(42);
            let t = lean_thunk_pure(v);
            let result = lean_thunk_get(t);
            assert_eq!(crate::lean_unbox(result), 42);
            crate::lean_dec(t);
        }
    }

    #[test]
    fn thunk_get_own_increments_rc() {
        unsafe {
            let v = crate::lean_mk_string("hello");
            let t = lean_thunk_pure(v);
            let result = lean_thunk_get_own(t);
            // result should have been inc'd, so rc=2 (one from thunk, one from get_own)
            assert_eq!((*result).rc, 2);
            crate::lean_dec(result); // drop the extra ref
            crate::lean_dec(t); // frees thunk and its value
        }
    }

    #[test]
    fn thunk_from_closure_forces() {
        unsafe {
            // Create a closure that returns lean_box(99)
            unsafe fn return_99(_arg: *mut LeanObject) -> *mut LeanObject {
                crate::lean_box(99)
            }
            let c = crate::lean_alloc_closure(return_99 as *const (), 1, 0);
            let t = lean_mk_thunk(c);
            let result = lean_thunk_get(t);
            assert_eq!(crate::lean_unbox(result), 99);
            crate::lean_dec(t);
        }
    }
}
