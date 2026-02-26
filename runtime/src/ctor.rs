//! Constructor allocation and field access

use crate::object::*;

#[inline]
pub unsafe fn lean_alloc_ctor(tag: u32, num_objs: u32, scalar_sz: u32) -> *mut LeanObject {
    let size = lean_ctor_object_size(num_objs, scalar_sz);
    let obj = lean_alloc_object(size);
    (*obj).rc = 1;
    (*obj).tag = tag as u8;
    (*obj).other = num_objs as u8;
    (*obj).cs_sz = scalar_sz as u16;
    obj
}

#[inline(always)]
pub unsafe fn lean_ctor_get(o: *mut LeanObject, i: u32) -> *mut LeanObject {
    #[cfg(any(debug_assertions, feature = "runtime-debug"))]
    {
        crate::debug::lean_debug_assert_valid(o, "lean_ctor_get");
        crate::debug::lean_debug_check_ctor_bounds(o, i, "lean_ctor_get");
    }
    *(*o).obj_fields_ptr().add(i as usize)
}

#[inline(always)]
pub unsafe fn lean_ctor_set(o: *mut LeanObject, i: u32, v: *mut LeanObject) {
    *(*o).obj_fields_ptr().add(i as usize) = v;
}

#[inline(always)]
pub unsafe fn lean_ctor_get_uint8(o: *mut LeanObject, offset: u32) -> u8 {
    *(*o).scalar_data_ptr().add(offset as usize)
}

#[inline(always)]
pub unsafe fn lean_ctor_set_uint8(o: *mut LeanObject, offset: u32, v: u8) {
    *(*o).scalar_data_ptr().add(offset as usize) = v;
}

#[inline(always)]
pub unsafe fn lean_ctor_get_uint16(o: *mut LeanObject, offset: u32) -> u16 {
    std::ptr::read_unaligned((*o).scalar_data_ptr().add(offset as usize) as *const u16)
}

#[inline(always)]
pub unsafe fn lean_ctor_set_uint16(o: *mut LeanObject, offset: u32, v: u16) {
    std::ptr::write_unaligned((*o).scalar_data_ptr().add(offset as usize) as *mut u16, v);
}

#[inline(always)]
pub unsafe fn lean_ctor_get_uint32(o: *mut LeanObject, offset: u32) -> u32 {
    std::ptr::read_unaligned((*o).scalar_data_ptr().add(offset as usize) as *const u32)
}

#[inline(always)]
pub unsafe fn lean_ctor_set_uint32(o: *mut LeanObject, offset: u32, v: u32) {
    std::ptr::write_unaligned((*o).scalar_data_ptr().add(offset as usize) as *mut u32, v);
}

#[inline(always)]
pub unsafe fn lean_ctor_get_uint64(o: *mut LeanObject, offset: u32) -> u64 {
    std::ptr::read_unaligned((*o).scalar_data_ptr().add(offset as usize) as *const u64)
}

#[inline(always)]
pub unsafe fn lean_ctor_set_uint64(o: *mut LeanObject, offset: u32, v: u64) {
    std::ptr::write_unaligned((*o).scalar_data_ptr().add(offset as usize) as *mut u64, v);
}

#[inline(always)]
pub unsafe fn lean_ctor_get_usize(o: *mut LeanObject, i: u32) -> usize {
    std::ptr::read_unaligned((*o).scalar_data_ptr().add(i as usize * 8) as *const usize)
}

#[inline(always)]
pub unsafe fn lean_ctor_set_usize(o: *mut LeanObject, i: u32, v: usize) {
    std::ptr::write_unaligned((*o).scalar_data_ptr().add(i as usize * 8) as *mut usize, v);
}

/// Release the i-th object field: decrement its RC and set to lean_box(0).
/// Used by reset/reuse optimization.
#[inline(always)]
pub unsafe fn lean_ctor_release(o: *mut LeanObject, i: u32) {
    let objs = (*o).obj_fields_ptr();
    crate::lean_dec(*objs.add(i as usize));
    *objs.add(i as usize) = crate::lean_box(0);
}

/// Set the tag on a constructor object. Used by reset/reuse optimization.
#[inline(always)]
pub unsafe fn lean_ctor_set_tag(o: *mut LeanObject, tag: u8) {
    (*o).tag = tag;
}

#[inline(always)]
pub unsafe fn lean_obj_tag(o: *mut LeanObject) -> u8 {
    if crate::lean_is_scalar(o) {
        crate::lean_unbox(o) as u8
    } else {
        (*o).tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_get_set_roundtrip() {
        unsafe {
            let obj = lean_alloc_ctor(5, 2, 0);
            let v0 = crate::lean_box(10);
            let v1 = crate::lean_box(20);
            lean_ctor_set(obj, 0, v0);
            lean_ctor_set(obj, 1, v1);
            assert_eq!(crate::lean_unbox(lean_ctor_get(obj, 0)), 10);
            assert_eq!(crate::lean_unbox(lean_ctor_get(obj, 1)), 20);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn scalar_uint8() {
        unsafe {
            let obj = lean_alloc_ctor(0, 0, 2);
            lean_ctor_set_uint8(obj, 0, 42);
            lean_ctor_set_uint8(obj, 1, 255);
            assert_eq!(lean_ctor_get_uint8(obj, 0), 42);
            assert_eq!(lean_ctor_get_uint8(obj, 1), 255);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn scalar_uint32() {
        unsafe {
            let obj = lean_alloc_ctor(0, 0, 4);
            lean_ctor_set_uint32(obj, 0, 0xDEAD_BEEF);
            assert_eq!(lean_ctor_get_uint32(obj, 0), 0xDEAD_BEEF);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn scalar_uint64() {
        unsafe {
            let obj = lean_alloc_ctor(0, 0, 8);
            lean_ctor_set_uint64(obj, 0, 0x0123_4567_89AB_CDEF);
            assert_eq!(lean_ctor_get_uint64(obj, 0), 0x0123_4567_89AB_CDEF);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn obj_tag() {
        unsafe {
            let scalar = crate::lean_box(3);
            assert_eq!(lean_obj_tag(scalar), 3);
            let scalar0 = crate::lean_box(0);
            assert_eq!(lean_obj_tag(scalar0), 0);
            let obj = lean_alloc_ctor(7, 0, 0);
            assert_eq!(lean_obj_tag(obj), 7);
            crate::lean_dec(obj);
        }
    }

    /// Mixed object fields and scalar fields in one ctor.
    #[test]
    fn mixed_obj_and_scalar_fields() {
        unsafe {
            // 2 obj fields + 12 bytes scalar (u32 + u64)
            let obj = lean_alloc_ctor(0, 2, 12);
            lean_ctor_set(obj, 0, crate::lean_box(42));
            lean_ctor_set(obj, 1, crate::lean_box(99));
            lean_ctor_set_uint32(obj, 0, 0xCAFE);
            lean_ctor_set_uint64(obj, 4, 0xDEAD_BEEF_1234_5678);
            assert_eq!(crate::lean_unbox(lean_ctor_get(obj, 0)), 42);
            assert_eq!(crate::lean_unbox(lean_ctor_get(obj, 1)), 99);
            assert_eq!(lean_ctor_get_uint32(obj, 0), 0xCAFE);
            assert_eq!(lean_ctor_get_uint64(obj, 4), 0xDEAD_BEEF_1234_5678);
            crate::lean_dec(obj);
        }
    }

    /// usize scalar fields.
    #[test]
    fn usize_fields() {
        unsafe {
            let obj = lean_alloc_ctor(0, 0, 16);
            lean_ctor_set_usize(obj, 0, 0x1234_5678_9ABC_DEF0);
            lean_ctor_set_usize(obj, 1, usize::MAX);
            assert_eq!(lean_ctor_get_usize(obj, 0), 0x1234_5678_9ABC_DEF0);
            assert_eq!(lean_ctor_get_usize(obj, 1), usize::MAX);
            crate::lean_dec(obj);
        }
    }

    /// Multiple uint8 fields.
    #[test]
    fn multiple_uint8_fields() {
        unsafe {
            let obj = lean_alloc_ctor(0, 0, 4);
            lean_ctor_set_uint8(obj, 0, 0x00);
            lean_ctor_set_uint8(obj, 1, 0x7F);
            lean_ctor_set_uint8(obj, 2, 0x80);
            lean_ctor_set_uint8(obj, 3, 0xFF);
            assert_eq!(lean_ctor_get_uint8(obj, 0), 0x00);
            assert_eq!(lean_ctor_get_uint8(obj, 1), 0x7F);
            assert_eq!(lean_ctor_get_uint8(obj, 2), 0x80);
            assert_eq!(lean_ctor_get_uint8(obj, 3), 0xFF);
            crate::lean_dec(obj);
        }
    }

    /// Ctor holding heap children (strings).
    #[test]
    fn ctor_with_heap_children() {
        unsafe {
            let obj = lean_alloc_ctor(1, 2, 0);
            let s1 = crate::lean_mk_string("hello");
            let s2 = crate::lean_mk_string("world");
            lean_ctor_set(obj, 0, s1);
            lean_ctor_set(obj, 1, s2);
            let r1 = lean_ctor_get(obj, 0);
            let r2 = lean_ctor_get(obj, 1);
            assert_eq!(crate::lean_string_to_str(r1), "hello");
            assert_eq!(crate::lean_string_to_str(r2), "world");
            // lean_dec should recursively free children
            crate::lean_dec(obj);
        }
    }

    /// Nested ctors (tree structure).
    #[test]
    fn nested_ctors() {
        unsafe {
            // Build: Pair(Pair(1, 2), Pair(3, 4))
            let inner1 = lean_alloc_ctor(0, 2, 0);
            lean_ctor_set(inner1, 0, crate::lean_box(1));
            lean_ctor_set(inner1, 1, crate::lean_box(2));
            let inner2 = lean_alloc_ctor(0, 2, 0);
            lean_ctor_set(inner2, 0, crate::lean_box(3));
            lean_ctor_set(inner2, 1, crate::lean_box(4));
            let outer = lean_alloc_ctor(0, 2, 0);
            lean_ctor_set(outer, 0, inner1);
            lean_ctor_set(outer, 1, inner2);
            // Navigate: outer.0.1 = 2
            let left = lean_ctor_get(outer, 0);
            assert_eq!(crate::lean_unbox(lean_ctor_get(left, 1)), 2);
            // outer.1.0 = 3
            let right = lean_ctor_get(outer, 1);
            assert_eq!(crate::lean_unbox(lean_ctor_get(right, 0)), 3);
            crate::lean_dec(outer);
        }
    }

    /// All 245 valid ctor tags.
    #[test]
    fn all_valid_tags() {
        unsafe {
            for tag in 0..=244u32 {
                let obj = lean_alloc_ctor(tag, 0, 0);
                assert_eq!(lean_obj_tag(obj), tag as u8);
                crate::lean_dec(obj);
            }
        }
    }
}
