//! FloatArray support — scalar array of f64 values.

use crate::object::*;
use crate::sarray::*;

/// Create an empty FloatArray with given capacity.
pub unsafe fn lean_mk_empty_float_array(cap: *mut LeanObject) -> *mut LeanObject {
    let c = crate::lean_unbox(cap);
    lean_alloc_sarray(std::mem::size_of::<f64>() as u32, 0, c)
}

/// Get FloatArray size.
pub unsafe fn lean_float_array_size(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    crate::lean_box((*sarr).size)
}

/// Push a f64 onto a FloatArray.
pub unsafe fn lean_float_array_push(a: *mut LeanObject, v: f64) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    let size = (*sarr).size;
    let cap = (*sarr).capacity;

    if crate::lean_is_exclusive(a) && size < cap {
        let data = lean_sarray_data(a) as *mut f64;
        *data.add(size) = v;
        (*sarr).size = size + 1;
        a
    } else {
        let new_cap = if cap == 0 { 4 } else { cap * 2 };
        let new_cap = new_cap.max(size + 1);
        let elem_size = std::mem::size_of::<f64>();
        let new_obj = lean_alloc_sarray(elem_size as u32, size + 1, new_cap);
        let new_data = lean_sarray_data(new_obj) as *mut f64;
        let old_data = lean_sarray_data(a) as *const f64;
        std::ptr::copy_nonoverlapping(old_data, new_data, size);
        *new_data.add(size) = v;
        crate::lean_dec(a);
        new_obj
    }
}

/// Get element at unboxed index.
pub unsafe fn lean_float_array_uget(a: *mut LeanObject, i: usize) -> f64 {
    let data = lean_sarray_data(a) as *const f64;
    *data.add(i)
}

/// Get element at boxed Fin index.
pub unsafe fn lean_float_array_fget(a: *mut LeanObject, i: *mut LeanObject) -> f64 {
    lean_float_array_uget(a, crate::lean_unbox(i))
}

/// Get element with bounds check and default.
pub unsafe fn lean_float_array_get(a: *mut LeanObject, dflt: f64, i: *mut LeanObject) -> f64 {
    let idx = crate::lean_unbox(i);
    let sarr = a as *mut LeanSArray;
    if idx >= (*sarr).size {
        dflt
    } else {
        lean_float_array_uget(a, idx)
    }
}

/// Set element at unboxed index.
pub unsafe fn lean_float_array_uset(a: *mut LeanObject, i: usize, v: f64) -> *mut LeanObject {
    if crate::lean_is_exclusive(a) {
        let data = lean_sarray_data(a) as *mut f64;
        *data.add(i) = v;
        a
    } else {
        let sarr = a as *mut LeanSArray;
        let sz = (*sarr).size;
        let elem_size = std::mem::size_of::<f64>();
        let new_obj = lean_alloc_sarray(elem_size as u32, sz, sz);
        let new_data = lean_sarray_data(new_obj) as *mut f64;
        let old_data = lean_sarray_data(a) as *const f64;
        std::ptr::copy_nonoverlapping(old_data, new_data, sz);
        *new_data.add(i) = v;
        crate::lean_dec(a);
        new_obj
    }
}

/// Set element at boxed Fin index.
pub unsafe fn lean_float_array_fset(
    a: *mut LeanObject,
    i: *mut LeanObject,
    v: f64,
) -> *mut LeanObject {
    lean_float_array_uset(a, crate::lean_unbox(i), v)
}

/// Set element with bounds check.
pub unsafe fn lean_float_array_set(
    a: *mut LeanObject,
    i: *mut LeanObject,
    v: f64,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    let sarr = a as *mut LeanSArray;
    if idx >= (*sarr).size {
        return a;
    }
    lean_float_array_uset(a, idx, v)
}

/// FloatArray.mk: List Float -> FloatArray
pub unsafe fn lean_float_array_mk(list: *mut LeanObject) -> *mut LeanObject {
    // Count elements
    let mut count = 0usize;
    let mut cur = list;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        count += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }
    let elem_size = std::mem::size_of::<f64>();
    let obj = lean_alloc_sarray(elem_size as u32, count, count);
    let data = lean_sarray_data(obj) as *mut f64;
    cur = list;
    let mut i = 0;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        let val_obj = crate::lean_ctor_get(cur, 0);
        *data.add(i) = crate::float::lean_unbox_float(val_obj);
        i += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }
    crate::lean_dec(list);
    obj
}

/// FloatArray.data: FloatArray -> List Float
pub unsafe fn lean_float_array_data(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    let size = (*sarr).size;
    let data = lean_sarray_data(a) as *const f64;
    let mut list = crate::lean_box(0); // nil
    for i in (0..size).rev() {
        let cons = crate::lean_alloc_ctor(1, 2, 0);
        crate::lean_ctor_set(cons, 0, crate::float::lean_box_float(*data.add(i)));
        crate::lean_ctor_set(cons, 1, list);
        list = cons;
    }
    crate::lean_dec(a);
    list
}

/// Copy a float array.
pub unsafe fn lean_copy_float_array(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut LeanSArray;
    let sz = (*sarr).size;
    let elem_size = std::mem::size_of::<f64>();
    let new_obj = lean_alloc_sarray(elem_size as u32, sz, sz);
    let new_data = lean_sarray_data(new_obj);
    let old_data = lean_sarray_data(a);
    std::ptr::copy_nonoverlapping(old_data, new_data, sz * elem_size);
    new_obj
}
