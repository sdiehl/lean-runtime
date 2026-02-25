//! Scalar array (ByteArray) support.
//!
//! A scalar array stores unboxed elements of uniform size.
//! Tag 248 (LEAN_SCALAR_ARRAY_TAG). The `other` field in the header stores `elem_size`.
//!
//! Layout: [LeanObject header: 8 bytes][size: usize][capacity: usize][data: elem_size * capacity bytes]

use crate::object::*;

#[repr(C)]
pub struct LeanSArray {
    pub header: LeanObject,
    pub size: usize,
    pub capacity: usize,
    // Followed by `capacity * elem_size` bytes of data
}

impl LeanSArray {
    #[inline(always)]
    pub unsafe fn data_ptr(&self) -> *mut u8 {
        (self as *const Self as *mut u8).add(std::mem::size_of::<Self>())
    }
}

/// Compute the total allocation size for a scalar array.
#[inline(always)]
pub fn lean_sarray_object_size(capacity: usize, elem_size: u8) -> usize {
    let data_size = capacity
        .checked_mul(elem_size as usize)
        .expect("lean_sarray_object_size: capacity overflow");
    std::mem::size_of::<LeanSArray>()
        .checked_add(data_size)
        .expect("lean_sarray_object_size: total size overflow")
}

/// Get a pointer to the raw data of a scalar array.
#[inline(always)]
pub unsafe fn lean_sarray_data(a: *mut LeanObject) -> *mut u8 {
    (*(a as *mut LeanSArray)).data_ptr()
}

/// Allocate a scalar array with given element size, initial size, and capacity.
pub unsafe fn lean_alloc_sarray(elem_size: u32, size: usize, capacity: usize) -> *mut LeanObject {
    let obj_size = lean_sarray_object_size(capacity, elem_size as u8);
    let obj = lean_alloc_object(obj_size);
    let sarr = obj as *mut LeanSArray;
    (*sarr).header.rc = 1;
    (*sarr).header.tag = LEAN_SCALAR_ARRAY_TAG;
    (*sarr).header.other = elem_size as u8;
    (*sarr).header.cs_sz = 0;
    (*sarr).size = size;
    (*sarr).capacity = capacity;
    obj
}

/// Create an empty ByteArray with given capacity.
pub unsafe fn lean_mk_empty_byte_array(cap: *mut LeanObject) -> *mut LeanObject {
    let c = crate::lean_unbox(cap);
    lean_alloc_sarray(1, 0, c)
}

/// Get the size of a ByteArray as a boxed Nat.
pub unsafe fn lean_byte_array_size(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    crate::lean_box((*sarr).size)
}

/// Push a byte onto a ByteArray. `a` is owned, `b` is an unboxed u8.
/// The Lean IR unboxes UInt8 before passing to this extern function.
pub unsafe fn lean_byte_array_push(a: *mut LeanObject, b: u8) -> *mut LeanObject {
    let byte_val = b;
    let sarr = a as *mut LeanSArray;
    let size = (*sarr).size;
    let cap = (*sarr).capacity;

    if crate::lean_is_exclusive(a) && size < cap {
        let data = lean_sarray_data(a);
        *data.add(size) = byte_val;
        (*sarr).size = size + 1;
        a
    } else {
        let new_cap = if cap == 0 { 4 } else { cap * 2 };
        let new_cap = new_cap.max(size + 1);
        let new_obj = lean_alloc_sarray(1, size + 1, new_cap);
        let new_data = lean_sarray_data(new_obj);
        let old_data = lean_sarray_data(a);
        std::ptr::copy_nonoverlapping(old_data, new_data, size);
        *new_data.add(size) = byte_val;
        crate::lean_dec(a);
        new_obj
    }
}

/// Get a byte from a ByteArray at a given (boxed Fin) index.
pub unsafe fn lean_byte_array_fget(a: *mut LeanObject, i: *mut LeanObject) -> u8 {
    let idx = crate::lean_unbox(i);
    let data = lean_sarray_data(a);
    *data.add(idx)
}

/// Convert a List UInt8 to a ByteArray.
pub unsafe fn lean_byte_array_mk(list: *mut LeanObject) -> *mut LeanObject {
    // First pass: count elements
    let mut count = 0usize;
    let mut cur = list;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        count += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }

    let obj = lean_alloc_sarray(1, count, count);
    let data = lean_sarray_data(obj);

    // Second pass: fill data
    cur = list;
    let mut i = 0;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        let val = crate::lean_unbox(crate::lean_ctor_get(cur, 0)) as u8;
        *data.add(i) = val;
        i += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }

    crate::lean_dec(list);
    obj
}

/// Convert a ByteArray to a List UInt8.
pub unsafe fn lean_byte_array_data(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    let size = (*sarr).size;
    let data = lean_sarray_data(a);

    let mut list = crate::lean_box(0); // nil
    for i in (0..size).rev() {
        let cons = crate::lean_alloc_ctor(1, 2, 0);
        crate::lean_ctor_set(cons, 0, crate::lean_box(*data.add(i) as usize));
        crate::lean_ctor_set(cons, 1, list);
        list = cons;
    }

    crate::lean_dec(a);
    list
}

/// Get the raw byte data size of a ByteArray (not boxed).
#[inline(always)]
pub unsafe fn lean_sarray_size(a: *mut LeanObject) -> usize {
    (*(a as *mut LeanSArray)).size
}

/// Get byte at index (raw, not boxed).
#[inline(always)]
pub unsafe fn lean_sarray_get_byte(a: *mut LeanObject, i: usize) -> u8 {
    *lean_sarray_data(a).add(i)
}

/// ByteArray.get (bounds-checked, returns default on out-of-bounds).
pub unsafe fn lean_byte_array_get(a: *mut LeanObject, i: *mut LeanObject) -> u8 {
    if crate::lean_is_scalar(i) {
        let idx = crate::lean_unbox(i);
        let sarr = a as *mut LeanSArray;
        if idx < (*sarr).size {
            let data = lean_sarray_data(a);
            *data.add(idx)
        } else {
            0
        }
    } else {
        // Index is a big nat — must be out of bounds
        0
    }
}

/// ByteArray.set (bounds-checked, no-op on out-of-bounds).
pub unsafe fn lean_byte_array_set(
    a: *mut LeanObject,
    i: *mut LeanObject,
    v: u8,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    let sarr = a as *mut LeanSArray;
    if idx >= (*sarr).size {
        return a;
    }

    if crate::lean_is_exclusive(a) {
        let data = lean_sarray_data(a);
        *data.add(idx) = v;
        a
    } else {
        let sz = (*sarr).size;
        let new_obj = lean_alloc_sarray(1, sz, sz);
        let new_data = lean_sarray_data(new_obj);
        let old_data = lean_sarray_data(a);
        std::ptr::copy_nonoverlapping(old_data, new_data, sz);
        *new_data.add(idx) = v;
        crate::lean_dec(a);
        new_obj
    }
}

/// ByteArray.fset (no bounds check).
pub unsafe fn lean_byte_array_fset(
    a: *mut LeanObject,
    i: *mut LeanObject,
    v: u8,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    if crate::lean_is_exclusive(a) {
        let data = lean_sarray_data(a);
        *data.add(idx) = v;
        a
    } else {
        let sarr = a as *mut LeanSArray;
        let sz = (*sarr).size;
        let new_obj = lean_alloc_sarray(1, sz, sz);
        let new_data = lean_sarray_data(new_obj);
        let old_data = lean_sarray_data(a);
        std::ptr::copy_nonoverlapping(old_data, new_data, sz);
        *new_data.add(idx) = v;
        crate::lean_dec(a);
        new_obj
    }
}

/// ByteArray.uget (unboxed index).
pub unsafe fn lean_byte_array_uget(a: *mut LeanObject, i: usize) -> u8 {
    let data = lean_sarray_data(a);
    *data.add(i)
}

/// ByteArray.uset (unboxed index).
pub unsafe fn lean_byte_array_uset(a: *mut LeanObject, i: usize, v: u8) -> *mut LeanObject {
    if crate::lean_is_exclusive(a) {
        let data = lean_sarray_data(a);
        *data.add(i) = v;
        a
    } else {
        let sarr = a as *mut LeanSArray;
        let sz = (*sarr).size;
        let new_obj = lean_alloc_sarray(1, sz, sz);
        let new_data = lean_sarray_data(new_obj);
        let old_data = lean_sarray_data(a);
        std::ptr::copy_nonoverlapping(old_data, new_data, sz);
        *new_data.add(i) = v;
        crate::lean_dec(a);
        new_obj
    }
}

/// Validate that a byte array contains valid UTF-8.
pub unsafe fn lean_string_validate_utf8(a: *mut LeanObject) -> u8 {
    let sarr = a as *mut LeanSArray;
    let sz = (*sarr).size;
    let data = lean_sarray_data(a);
    let bytes = std::slice::from_raw_parts(data, sz);
    std::str::from_utf8(bytes).is_ok() as u8
}

/// Copy a byte array.
pub unsafe fn lean_copy_byte_array(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    let sz = (*sarr).size;
    let new_obj = lean_alloc_sarray(1, sz, sz);
    let new_data = lean_sarray_data(new_obj);
    let old_data = lean_sarray_data(a);
    std::ptr::copy_nonoverlapping(old_data, new_data, sz);
    new_obj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_push() {
        unsafe {
            let arr = lean_mk_empty_byte_array(crate::lean_box(4));
            let arr = lean_byte_array_push(arr, 10u8);
            let arr = lean_byte_array_push(arr, 20u8);
            let arr = lean_byte_array_push(arr, 255u8);
            assert_eq!(crate::lean_unbox(lean_byte_array_size(arr)), 3);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(0)), 10);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(1)), 20);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(2)), 255);
            crate::lean_dec(arr);
        }
    }

    #[test]
    fn push_beyond_capacity() {
        unsafe {
            let arr = lean_mk_empty_byte_array(crate::lean_box(2));
            let arr = lean_byte_array_push(arr, 1u8);
            let arr = lean_byte_array_push(arr, 2u8);
            let arr = lean_byte_array_push(arr, 3u8);
            assert_eq!(crate::lean_unbox(lean_byte_array_size(arr)), 3);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(2)), 3);
            crate::lean_dec(arr);
        }
    }

    #[test]
    fn mk_and_data_roundtrip() {
        unsafe {
            // Build list [10, 20, 30]
            let nil = crate::lean_box(0);
            let c3 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c3, 0, crate::lean_box(30));
            crate::lean_ctor_set(c3, 1, nil);
            let c2 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c2, 0, crate::lean_box(20));
            crate::lean_ctor_set(c2, 1, c3);
            let c1 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c1, 0, crate::lean_box(10));
            crate::lean_ctor_set(c1, 1, c2);

            let arr = lean_byte_array_mk(c1);
            assert_eq!(crate::lean_unbox(lean_byte_array_size(arr)), 3);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(0)), 10);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(1)), 20);
            assert_eq!(lean_byte_array_fget(arr, crate::lean_box(2)), 30);

            // Convert back to list
            crate::lean_inc(arr);
            let list = lean_byte_array_data(arr);
            let h0 = crate::lean_ctor_get(list, 0);
            assert_eq!(crate::lean_unbox(h0), 10);
            let t0 = crate::lean_ctor_get(list, 1);
            let h1 = crate::lean_ctor_get(t0, 0);
            assert_eq!(crate::lean_unbox(h1), 20);

            crate::lean_dec(list);
            crate::lean_dec(arr);
        }
    }
}
