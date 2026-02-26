//! Array support

use crate::object::*;

#[repr(C)]
pub struct LeanArray {
    pub header: LeanObject,
    pub size: usize,
    pub capacity: usize,
}

impl LeanArray {
    #[inline(always)]
    pub unsafe fn data_ptr(&self) -> *mut *mut LeanObject {
        (self as *const Self as *mut u8).add(std::mem::size_of::<Self>()) as *mut *mut LeanObject
    }
}

#[inline]
pub unsafe fn lean_alloc_array(size: usize, capacity: usize) -> *mut LeanObject {
    let obj_size = std::mem::size_of::<LeanArray>() + capacity * 8;
    let obj = lean_alloc_object(obj_size);
    let arr = obj as *mut LeanArray;
    (*arr).header.rc = 1;
    (*arr).header.tag = LEAN_ARRAY_TAG;
    (*arr).header.other = 0;
    (*arr).header.cs_sz = 0;
    (*arr).size = size;
    (*arr).capacity = capacity;
    obj
}

#[inline]
pub unsafe fn lean_mk_empty_array() -> *mut LeanObject {
    lean_alloc_array(0, 0)
}

#[inline]
pub unsafe fn lean_mk_empty_array_with_capacity(cap: *mut LeanObject) -> *mut LeanObject {
    let c = crate::lean_unbox(cap);
    lean_alloc_array(0, c)
}

#[inline(always)]
pub unsafe fn lean_array_size(a: *mut LeanObject) -> usize {
    (*(a as *mut LeanArray)).size
}

#[inline(always)]
pub unsafe fn lean_array_get_size(a: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(lean_array_size(a))
}

#[inline(always)]
pub unsafe fn lean_array_data(a: *mut LeanObject) -> *mut *mut LeanObject {
    (*(a as *mut LeanArray)).data_ptr()
}

#[inline(always)]
pub unsafe fn lean_array_get_core(a: *mut LeanObject, i: usize) -> *mut LeanObject {
    *lean_array_data(a).add(i)
}

/// Borrowed variant of lean_array_get: returns element without incrementing RC.
/// The caller must ensure the array outlives the returned reference.
pub unsafe fn lean_array_get_borrowed(
    def: *mut LeanObject,
    a: *mut LeanObject,
    i: *mut LeanObject,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    if idx < lean_array_size(a) {
        crate::lean_dec(def);
        lean_array_get_core(a, idx)
    } else {
        def
    }
}

pub unsafe fn lean_array_get(
    def: *mut LeanObject,
    a: *mut LeanObject,
    i: *mut LeanObject,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    if idx < lean_array_size(a) {
        crate::lean_dec(def);
        let v = lean_array_get_core(a, idx);
        crate::lean_inc(v);
        v
    } else {
        def
    }
}

pub unsafe fn lean_array_push(a: *mut LeanObject, v: *mut LeanObject) -> *mut LeanObject {
    let arr = a as *mut LeanArray;
    let size = (*arr).size;
    let cap = (*arr).capacity;

    if crate::lean_is_exclusive(a) && size < cap {
        *lean_array_data(a).add(size) = v;
        (*arr).size = size + 1;
        a
    } else {
        let new_cap = if cap == 0 { 4 } else { cap * 2 };
        let new_arr = lean_alloc_array(size + 1, new_cap.max(size + 1));
        let new_data = lean_array_data(new_arr);
        let old_data = lean_array_data(a);
        for i in 0..size {
            let elem = *old_data.add(i);
            crate::lean_inc(elem);
            *new_data.add(i) = elem;
        }
        *new_data.add(size) = v;
        crate::lean_dec(a);
        new_arr
    }
}

/// Get array element at a (Fin-style boxed) index. Increments RC of the returned element.
pub unsafe fn lean_array_fget(a: *mut LeanObject, i: *mut LeanObject) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    let v = lean_array_get_core(a, idx);
    crate::lean_inc(v);
    v
}

/// Get array element at a (Fin-style boxed) index. Does NOT increment RC (borrowed).
pub unsafe fn lean_array_fget_borrowed(a: *mut LeanObject, i: *mut LeanObject) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    lean_array_get_core(a, idx)
}

/// Set array element at a (Fin-style boxed) index. `a` is owned, `v` is owned.
pub unsafe fn lean_array_fset(
    a: *mut LeanObject,
    i: *mut LeanObject,
    v: *mut LeanObject,
) -> *mut LeanObject {
    lean_array_set(a, i, v)
}

/// Convert a List to an Array.
pub unsafe fn lean_array_mk(list: *mut LeanObject) -> *mut LeanObject {
    // First pass: count elements
    let mut count = 0usize;
    let mut cur = list;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        count += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }

    let arr = lean_alloc_array(count, count);
    let data = lean_array_data(arr);

    // Second pass: fill array
    cur = list;
    let mut i = 0;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        let elem = crate::lean_ctor_get(cur, 0);
        crate::lean_inc(elem);
        *data.add(i) = elem;
        i += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }

    crate::lean_dec(list);
    arr
}

/// Convert an Array to a List.
pub unsafe fn lean_array_to_list(a: *mut LeanObject) -> *mut LeanObject {
    let size = lean_array_size(a);
    let data = lean_array_data(a);

    let mut list = crate::lean_box(0); // nil
    for i in (0..size).rev() {
        let elem = *data.add(i);
        crate::lean_inc(elem);
        let cons = crate::lean_alloc_ctor(1, 2, 0);
        crate::lean_ctor_set(cons, 0, elem);
        crate::lean_ctor_set(cons, 1, list);
        list = cons;
    }

    crate::lean_dec(a);
    list
}

pub unsafe fn lean_array_set(
    a: *mut LeanObject,
    i: *mut LeanObject,
    v: *mut LeanObject,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    let arr = a as *mut LeanArray;
    let size = (*arr).size;

    if idx >= size {
        crate::lean_dec(v);
        return a;
    }

    if crate::lean_is_exclusive(a) {
        let data = lean_array_data(a);
        crate::lean_dec(*data.add(idx));
        *data.add(idx) = v;
        a
    } else {
        let cap = (*arr).capacity;
        let new_arr = lean_alloc_array(size, cap);
        let new_data = lean_array_data(new_arr);
        let old_data = lean_array_data(a);
        for j in 0..size {
            if j == idx {
                *new_data.add(j) = v;
            } else {
                let elem = *old_data.add(j);
                crate::lean_inc(elem);
                *new_data.add(j) = elem;
            }
        }
        crate::lean_dec(a);
        new_arr
    }
}

/// Array.uget: get element by unboxed usize index.
pub unsafe fn lean_array_uget(a: *mut LeanObject, i: usize) -> *mut LeanObject {
    let data = lean_array_data(a);
    let v = *data.add(i);
    crate::lean_inc(v);
    v
}

/// Array.uset: set element by unboxed usize index (owned array).
pub unsafe fn lean_array_uset(a: *mut LeanObject, i: usize, v: *mut LeanObject) -> *mut LeanObject {
    if crate::lean_is_exclusive(a) {
        let data = lean_array_data(a);
        let old = *data.add(i);
        *data.add(i) = v;
        crate::lean_dec(old);
        a
    } else {
        let arr = a as *mut LeanArray;
        let sz = (*arr).size;
        let new_arr = lean_alloc_array(sz, sz);
        let new_data = lean_array_data(new_arr);
        let old_data = lean_array_data(a);
        for j in 0..sz {
            let elem = *old_data.add(j);
            crate::lean_inc(elem);
            *new_data.add(j) = elem;
        }
        // Set the new value
        let old = *new_data.add(i);
        *new_data.add(i) = v;
        crate::lean_dec(old);
        (*(new_arr as *mut LeanArray)).size = sz;
        crate::lean_dec(a);
        new_arr
    }
}

/// Array.pop: remove last element.
pub unsafe fn lean_array_pop(a: *mut LeanObject) -> *mut LeanObject {
    let arr = a as *mut LeanArray;
    let sz = (*arr).size;
    if sz == 0 {
        return a;
    }
    if crate::lean_is_exclusive(a) {
        let data = lean_array_data(a);
        let last = *data.add(sz - 1);
        crate::lean_dec(last);
        (*arr).size = sz - 1;
        a
    } else {
        let new_sz = sz - 1;
        let new_arr = lean_alloc_array(new_sz, new_sz);
        let new_data = lean_array_data(new_arr);
        let old_data = lean_array_data(a);
        for j in 0..new_sz {
            let elem = *old_data.add(j);
            crate::lean_inc(elem);
            *new_data.add(j) = elem;
        }
        (*(new_arr as *mut LeanArray)).size = new_sz;
        crate::lean_dec(a);
        new_arr
    }
}

/// Array.fswap: swap two elements at Fin indices.
pub unsafe fn lean_array_fswap(
    a: *mut LeanObject,
    i: *mut LeanObject,
    j: *mut LeanObject,
) -> *mut LeanObject {
    let idx_i = crate::lean_unbox(i);
    let idx_j = crate::lean_unbox(j);
    if idx_i == idx_j {
        return a;
    }

    if crate::lean_is_exclusive(a) {
        let data = lean_array_data(a);
        let tmp = *data.add(idx_i);
        *data.add(idx_i) = *data.add(idx_j);
        *data.add(idx_j) = tmp;
        a
    } else {
        // Copy and swap
        let arr = a as *mut LeanArray;
        let sz = (*arr).size;
        let new_arr = lean_alloc_array(sz, sz);
        let new_data = lean_array_data(new_arr);
        let old_data = lean_array_data(a);
        for k in 0..sz {
            let elem = *old_data.add(k);
            crate::lean_inc(elem);
            *new_data.add(k) = elem;
        }
        let tmp = *new_data.add(idx_i);
        *new_data.add(idx_i) = *new_data.add(idx_j);
        *new_data.add(idx_j) = tmp;
        (*(new_arr as *mut LeanArray)).size = sz;
        crate::lean_dec(a);
        new_arr
    }
}

/// Array.swap: swap two elements at Nat indices with bounds checking.
pub unsafe fn lean_array_swap(
    a: *mut LeanObject,
    i: *mut LeanObject,
    j: *mut LeanObject,
) -> *mut LeanObject {
    let arr = a as *mut LeanArray;
    let sz = (*arr).size;
    let idx_i = crate::lean_unbox(i);
    let idx_j = crate::lean_unbox(j);
    if idx_i >= sz || idx_j >= sz || idx_i == idx_j {
        return a;
    }
    lean_array_fswap(a, i, j)
}

/// mkArray: create an array of n copies of a value.
pub unsafe fn lean_mk_array(n: *mut LeanObject, v: *mut LeanObject) -> *mut LeanObject {
    let sz = crate::lean_unbox(n);
    let arr = lean_alloc_array(sz, sz);
    let data = lean_array_data(arr);
    for k in 0..sz {
        crate::lean_inc(v);
        *data.add(k) = v;
    }
    (*(arr as *mut LeanArray)).size = sz;
    crate::lean_dec(v);
    arr
}

/// Copy an array.
pub unsafe fn lean_copy_array(a: *mut LeanObject) -> *mut LeanObject {
    let arr = a as *mut LeanArray;
    let sz = (*arr).size;
    let cap = (*arr).capacity;
    let new_arr = lean_alloc_array(sz, cap);
    let new_data = lean_array_data(new_arr);
    let old_data = lean_array_data(a);
    for k in 0..sz {
        let elem = *old_data.add(k);
        crate::lean_inc(elem);
        *new_data.add(k) = elem;
    }
    (*(new_arr as *mut LeanArray)).size = sz;
    new_arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_push_get() {
        unsafe {
            let arr = lean_mk_empty_array_with_capacity(crate::lean_box(4));
            let arr = lean_array_push(arr, crate::lean_box(10));
            let arr = lean_array_push(arr, crate::lean_box(20));
            let arr = lean_array_push(arr, crate::lean_box(30));
            assert_eq!(lean_array_size(arr), 3);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 0)), 10);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 1)), 20);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 2)), 30);
            crate::lean_dec(arr);
        }
    }

    #[test]
    fn get_with_default() {
        unsafe {
            let arr = lean_mk_empty_array_with_capacity(crate::lean_box(2));
            let arr = lean_array_push(arr, crate::lean_box(42));
            let v = lean_array_get(crate::lean_box(999), arr, crate::lean_box(0));
            assert_eq!(crate::lean_unbox(v), 42);
            let v2 = lean_array_get(crate::lean_box(999), arr, crate::lean_box(5));
            assert_eq!(crate::lean_unbox(v2), 999);
            crate::lean_dec(arr);
        }
    }

    #[test]
    fn set() {
        unsafe {
            let arr = lean_mk_empty_array_with_capacity(crate::lean_box(2));
            let arr = lean_array_push(arr, crate::lean_box(1));
            let arr = lean_array_push(arr, crate::lean_box(2));
            let arr = lean_array_set(arr, crate::lean_box(1), crate::lean_box(99));
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 0)), 1);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 1)), 99);
            crate::lean_dec(arr);
        }
    }

    #[test]
    fn size() {
        unsafe {
            let arr = lean_mk_empty_array();
            let sz = lean_array_get_size(arr);
            assert_eq!(crate::lean_unbox(sz), 0);
            let arr = lean_array_push(arr, crate::lean_box(1));
            let sz = lean_array_get_size(arr);
            assert_eq!(crate::lean_unbox(sz), 1);
            crate::lean_dec(arr);
        }
    }

    /// Push beyond initial capacity triggers reallocation.
    #[test]
    fn push_beyond_capacity() {
        unsafe {
            let arr = lean_mk_empty_array_with_capacity(crate::lean_box(2));
            let arr = lean_array_push(arr, crate::lean_box(1));
            let arr = lean_array_push(arr, crate::lean_box(2));
            // Third push exceeds capacity=2, triggers realloc
            let arr = lean_array_push(arr, crate::lean_box(3));
            assert_eq!(lean_array_size(arr), 3);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 0)), 1);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 1)), 2);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 2)), 3);
            crate::lean_dec(arr);
        }
    }

    /// Set out of bounds is a no-op.
    #[test]
    fn set_out_of_bounds() {
        unsafe {
            let arr = lean_mk_empty_array_with_capacity(crate::lean_box(2));
            let arr = lean_array_push(arr, crate::lean_box(10));
            // Set at index 5 (out of bounds) should not modify anything
            let arr = lean_array_set(arr, crate::lean_box(5), crate::lean_box(99));
            assert_eq!(lean_array_size(arr), 1);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 0)), 10);
            crate::lean_dec(arr);
        }
    }

    /// Copy-on-write: set on shared array produces a new copy.
    #[test]
    fn copy_on_write() {
        unsafe {
            let arr = lean_mk_empty_array_with_capacity(crate::lean_box(4));
            let arr = lean_array_push(arr, crate::lean_box(10));
            let arr = lean_array_push(arr, crate::lean_box(20));
            // Share the array (bump rc)
            crate::lean_inc(arr);
            // Set should produce a copy since arr is not exclusive
            let arr2 = lean_array_set(arr, crate::lean_box(0), crate::lean_box(99));
            // arr2 is a new object
            assert_ne!(arr, arr2);
            // Original is unchanged
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr, 0)), 10);
            // New copy has the update
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr2, 0)), 99);
            assert_eq!(crate::lean_unbox(lean_array_get_core(arr2, 1)), 20);
            crate::lean_dec(arr);
            crate::lean_dec(arr2);
        }
    }

    /// Push many elements, verifying correct growth.
    #[test]
    fn push_100_elements() {
        unsafe {
            let mut arr = lean_mk_empty_array_with_capacity(crate::lean_box(1));
            for i in 0..100 {
                arr = lean_array_push(arr, crate::lean_box(i));
            }
            assert_eq!(lean_array_size(arr), 100);
            for i in 0..100 {
                assert_eq!(crate::lean_unbox(lean_array_get_core(arr, i)), i);
            }
            crate::lean_dec(arr);
        }
    }

    /// Array holding heap objects: push strings and verify they survive.
    #[test]
    fn array_of_heap_objects() {
        unsafe {
            let mut arr = lean_mk_empty_array_with_capacity(crate::lean_box(4));
            let s1 = crate::lean_mk_string("hello");
            let s2 = crate::lean_mk_string("world");
            arr = lean_array_push(arr, s1);
            arr = lean_array_push(arr, s2);
            let r1 = lean_array_get_core(arr, 0);
            assert_eq!(crate::lean_string_to_str(r1), "hello");
            let r2 = lean_array_get_core(arr, 1);
            assert_eq!(crate::lean_string_to_str(r2), "world");
            crate::lean_dec(arr);
        }
    }

    /// lean_array_get increments refcount of returned element.
    #[test]
    fn get_increments_rc() {
        unsafe {
            let s = crate::lean_mk_string("test");
            let rc_before = (*(s as *const LeanObject)).rc;
            let mut arr = lean_mk_empty_array_with_capacity(crate::lean_box(2));
            arr = lean_array_push(arr, s);
            let elem = lean_array_get(crate::lean_box(0), arr, crate::lean_box(0));
            // lean_array_get should have incremented rc
            let rc_after = (*(elem as *const LeanObject)).rc;
            assert_eq!(rc_after, rc_before + 1);
            crate::lean_dec(elem);
            crate::lean_dec(arr);
        }
    }
}
