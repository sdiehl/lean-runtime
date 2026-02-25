//! Reference counting

use crate::object::*;
use crate::r#box::lean_is_scalar;

#[inline(always)]
pub unsafe fn lean_inc(o: *mut LeanObject) {
    if !lean_is_scalar(o) {
        lean_inc_ref(o);
    }
}

#[inline(always)]
pub unsafe fn lean_dec(o: *mut LeanObject) {
    if !lean_is_scalar(o) {
        lean_dec_ref(o);
    }
}

/// Increment reference count by n. Used for bulk inc operations.
#[inline(always)]
pub unsafe fn lean_inc_n(o: *mut LeanObject, n: u32) {
    if !lean_is_scalar(o) && (*o).rc > 0 {
        (*o).rc += n as i32;
    }
}

#[inline(always)]
pub unsafe fn lean_inc_ref(o: *mut LeanObject) {
    #[cfg(any(debug_assertions, feature = "runtime-debug"))]
    crate::debug::lean_debug_assert_valid(o, "lean_inc_ref");
    if (*o).rc > 0 {
        (*o).rc += 1;
    }
}

/// Bulk increment reference count (non-scalar only, no scalar check).
#[inline(always)]
pub unsafe fn lean_inc_ref_n(o: *mut LeanObject, n: u32) {
    #[cfg(any(debug_assertions, feature = "runtime-debug"))]
    crate::debug::lean_debug_assert_valid(o, "lean_inc_ref_n");
    if (*o).rc > 0 {
        (*o).rc += n as i32;
    }
}

#[inline(always)]
pub unsafe fn lean_dec_ref(o: *mut LeanObject) {
    #[cfg(any(debug_assertions, feature = "runtime-debug"))]
    crate::debug::lean_debug_assert_valid(o, "lean_dec_ref");
    if (*o).rc > 0 {
        (*o).rc -= 1;
        if (*o).rc == 0 {
            lean_free_object_full(o);
        }
    }
}

/// Free an object and decrement all its children.
/// Uses an explicit worklist to avoid stack overflow on deep object graphs.
pub unsafe fn lean_free_object_full(o: *mut LeanObject) {
    let mut worklist: Vec<*mut LeanObject> = Vec::new();
    worklist.push(o);

    while let Some(obj) = worklist.pop() {
        let tag = (*obj).tag;
        if tag <= LEAN_MAX_CTOR_TAG {
            let num_objs = (*obj).num_objs();
            let objs = (*obj).obj_fields_ptr();
            for i in 0..num_objs {
                let child = *objs.add(i as usize);
                if !lean_is_scalar(child) && (*child).rc > 0 {
                    (*child).rc -= 1;
                    if (*child).rc == 0 {
                        worklist.push(child);
                    }
                }
            }
            lean_free_object(obj, lean_ctor_object_size(num_objs, (*obj).cs_sz as u32));
        } else if tag == LEAN_CLOSURE_TAG {
            let closure = obj as *mut crate::closure::LeanClosure;
            let num_fixed = (*closure).num_fixed as u32;
            let args = (*closure).fixed_args_ptr();
            for i in 0..num_fixed {
                lean_dec(*args.add(i as usize));
            }
            lean_free_object(obj, crate::closure::lean_closure_object_size(num_fixed));
        } else if tag == LEAN_STRING_TAG {
            let str_obj = obj as *mut crate::string::LeanString;
            lean_free_object(
                obj,
                crate::string::lean_string_object_size((*str_obj).capacity),
            );
        } else if tag == LEAN_ARRAY_TAG {
            let arr = obj as *mut crate::array::LeanArray;
            let data = (*arr).data_ptr();
            for i in 0..(*arr).size {
                let child = *data.add(i);
                if !lean_is_scalar(child) && (*child).rc > 0 {
                    (*child).rc -= 1;
                    if (*child).rc == 0 {
                        worklist.push(child);
                    }
                }
            }
            lean_free_object(
                obj,
                std::mem::size_of::<crate::array::LeanArray>() + (*arr).capacity * 8,
            );
        } else if tag == LEAN_SCALAR_ARRAY_TAG {
            let sarr = obj as *mut crate::sarray::LeanSArray;
            lean_free_object(
                obj,
                crate::sarray::lean_sarray_object_size((*sarr).capacity, (*obj).other),
            );
        } else if tag == LEAN_MPZ_TAG {
            crate::bignat::lean_free_bignat(obj);
        } else if tag == crate::object::LEAN_BIGINT_TAG {
            crate::int::lean_free_bigint(obj);
        } else if tag == LEAN_THUNK_TAG {
            let thunk = obj as *mut crate::thunk::LeanThunk;
            let value = (*thunk).value;
            if !value.is_null() {
                lean_dec(value);
            }
            let closure = (*thunk).closure;
            if !closure.is_null() {
                lean_dec(closure);
            }
            lean_free_object(obj, std::mem::size_of::<crate::thunk::LeanThunk>());
        } else if tag == LEAN_REF_TAG {
            let r = obj as *mut crate::stref::LeanRef;
            let value = (*r).value;
            if !value.is_null() {
                lean_dec(value);
            }
            lean_free_object(obj, std::mem::size_of::<crate::stref::LeanRef>());
        } else if tag == LEAN_EXTERNAL_TAG {
            let ext = obj as *mut crate::external::LeanExternal;
            let cls = (*ext).class;
            if !cls.is_null() {
                if let Some(finalize) = (*cls).finalize {
                    finalize((*ext).data);
                }
            }
            lean_free_object(obj, std::mem::size_of::<crate::external::LeanExternal>());
        } else {
            lean_free_object(obj, LEAN_OBJECT_SIZE);
        }
    }
}

/// Free an object WITHOUT decrementing children (used by IR del instruction)
/// The caller is responsible for handling children's reference counts
pub unsafe fn lean_free_object_only(o: *mut LeanObject) {
    let tag = (*o).tag;
    if tag <= LEAN_MAX_CTOR_TAG {
        lean_free_object(o, lean_ctor_object_size((*o).num_objs(), (*o).cs_sz as u32));
    } else if tag == LEAN_CLOSURE_TAG {
        let closure = o as *mut crate::closure::LeanClosure;
        let num_fixed = (*closure).num_fixed as u32;
        lean_free_object(o, crate::closure::lean_closure_object_size(num_fixed));
    } else if tag == LEAN_STRING_TAG {
        let str_obj = o as *mut crate::string::LeanString;
        lean_free_object(
            o,
            crate::string::lean_string_object_size((*str_obj).capacity),
        );
    } else if tag == LEAN_ARRAY_TAG {
        let arr = o as *mut crate::array::LeanArray;
        lean_free_object(
            o,
            std::mem::size_of::<crate::array::LeanArray>() + (*arr).capacity * 8,
        );
    } else if tag == LEAN_SCALAR_ARRAY_TAG {
        let sarr = o as *mut crate::sarray::LeanSArray;
        lean_free_object(
            o,
            crate::sarray::lean_sarray_object_size((*sarr).capacity, (*o).other),
        );
    } else if tag == LEAN_MPZ_TAG {
        crate::bignat::lean_free_bignat(o);
    } else if tag == crate::object::LEAN_BIGINT_TAG {
        crate::int::lean_free_bigint(o);
    } else {
        lean_free_object(o, LEAN_OBJECT_SIZE);
    }
}

#[inline(always)]
pub unsafe fn lean_is_exclusive(o: *mut LeanObject) -> bool {
    (*o).rc == 1
}

#[inline(always)]
pub unsafe fn lean_mark_persistent(o: *mut LeanObject) {
    (*o).rc = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inc_dec_scalar_noop() {
        unsafe {
            // Scalars should not crash on inc/dec
            let s = crate::lean_box(42);
            lean_inc(s);
            lean_dec(s);
        }
    }

    #[test]
    fn inc_ref_dec_ref() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            assert_eq!((*obj).rc, 1);
            lean_inc_ref(obj);
            assert_eq!((*obj).rc, 2);
            lean_dec_ref(obj);
            assert_eq!((*obj).rc, 1);
            lean_dec_ref(obj); // frees
        }
    }

    #[test]
    fn is_exclusive() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            assert!(lean_is_exclusive(obj));
            lean_inc_ref(obj);
            assert!(!lean_is_exclusive(obj));
            lean_dec_ref(obj);
            assert!(lean_is_exclusive(obj));
            lean_dec_ref(obj); // frees
        }
    }

    #[test]
    fn mark_persistent() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            lean_mark_persistent(obj);
            assert_eq!((*obj).rc, 0);
            // inc/dec should be no-ops on persistent objects
            lean_inc_ref(obj);
            assert_eq!((*obj).rc, 0);
            lean_dec_ref(obj);
            assert_eq!((*obj).rc, 0);
            // Clean up directly since lean_dec won't free persistent objects
            crate::lean_free_object(obj, crate::lean_ctor_object_size(0, 0));
        }
    }

    /// Recursive free: ctor with heap children.
    /// lean_dec on the parent should free children transitively.
    #[test]
    fn recursive_free_ctor_with_children() {
        unsafe {
            let child1 = crate::lean_mk_string("hello");
            let child2 = crate::lean_mk_string("world");
            let parent = crate::lean_alloc_ctor(0, 2, 0);
            crate::lean_ctor_set(parent, 0, child1);
            crate::lean_ctor_set(parent, 1, child2);
            // All rc=1. Dropping parent should cascade.
            lean_dec(parent);
            // If this doesn't crash/leak, the recursive free works.
        }
    }

    /// Deeply nested ctor tree: free should not stack overflow for moderate depth.
    #[test]
    fn recursive_free_nested() {
        unsafe {
            // Build a linked list of 100 cons cells: cons(i, cons(i-1, ... nil))
            let mut list = crate::lean_box(0); // nil
            for i in 0..100 {
                let cell = crate::lean_alloc_ctor(1, 2, 0);
                crate::lean_ctor_set(cell, 0, crate::lean_box(i));
                crate::lean_ctor_set(cell, 1, list);
                list = cell;
            }
            // Single lean_dec should free the entire chain
            lean_dec(list);
        }
    }

    /// Deep linked list: 200_000 nodes must not stack overflow.
    #[test]
    fn deep_free_no_stack_overflow() {
        unsafe {
            let mut list = crate::lean_box(0); // nil
            for i in 0..200_000 {
                let cell = crate::lean_alloc_ctor(1, 2, 0);
                crate::lean_ctor_set(cell, 0, crate::lean_box(i));
                crate::lean_ctor_set(cell, 1, list);
                list = cell;
            }
            lean_dec(list);
        }
    }

    /// lean_free_object_only frees the shell but does not touch children.
    #[test]
    fn free_object_only_preserves_children() {
        unsafe {
            let child = crate::lean_mk_string("survivor");
            // Bump rc so child survives parent free
            lean_inc(child);
            let parent = crate::lean_alloc_ctor(0, 1, 0);
            crate::lean_ctor_set(parent, 0, child);
            // free_object_only frees parent but not child
            lean_free_object_only(parent);
            // child should still be valid (rc was 2, now still 2 since we didn't dec)
            assert_eq!((*child).rc, 2);
            assert_eq!(crate::lean_string_to_str(child), "survivor");
            // Clean up child manually
            lean_dec(child); // rc 2->1
            lean_dec(child); // rc 1->0, freed
        }
    }

    /// Shared object: inc/dec many times, final dec frees.
    #[test]
    fn shared_object_many_refs() {
        unsafe {
            let obj = crate::lean_alloc_ctor(0, 0, 0);
            for _ in 0..99 {
                lean_inc_ref(obj);
            }
            assert_eq!((*obj).rc, 100);
            for _ in 0..99 {
                lean_dec_ref(obj);
            }
            assert_eq!((*obj).rc, 1);
            assert!(lean_is_exclusive(obj));
            lean_dec_ref(obj); // frees
        }
    }

    /// Closure objects are freed correctly.
    #[test]
    fn free_closure_with_fixed_args() {
        unsafe {
            unsafe fn dummy(_: *mut LeanObject, _: *mut LeanObject) -> *mut LeanObject {
                crate::lean_box(0)
            }
            let s = crate::lean_mk_string("captured");
            let c = crate::lean_alloc_closure(dummy as *const (), 2, 1);
            crate::lean_closure_set(c, 0, s);
            // s is owned by closure. lean_dec(c) should free s.
            lean_dec(c);
        }
    }

    /// String objects are freed correctly.
    #[test]
    fn free_string() {
        unsafe {
            let s = crate::lean_mk_string("this string will be freed");
            assert_eq!((*s).rc, 1);
            lean_dec(s);
        }
    }

    /// Array with heap elements freed correctly.
    #[test]
    fn free_array_with_heap_elements() {
        unsafe {
            let mut arr = crate::lean_mk_empty_array_with_capacity(crate::lean_box(4));
            arr = crate::lean_array_push(arr, crate::lean_mk_string("a"));
            arr = crate::lean_array_push(arr, crate::lean_mk_string("b"));
            arr = crate::lean_array_push(arr, crate::lean_mk_string("c"));
            lean_dec(arr);
        }
    }
}
