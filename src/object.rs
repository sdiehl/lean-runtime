//! Core Lean object representation

use std::alloc::{alloc, dealloc, Layout};

pub const LEAN_CLOSURE_TAG: u8 = 245;
pub const LEAN_ARRAY_TAG: u8 = 246;
pub const LEAN_SCALAR_ARRAY_TAG: u8 = 248;
pub const LEAN_STRING_TAG: u8 = 249;
pub const LEAN_MPZ_TAG: u8 = 250;
pub const LEAN_THUNK_TAG: u8 = 247;
pub const LEAN_BIGINT_TAG: u8 = 251;
pub const LEAN_REF_TAG: u8 = 253;
pub const LEAN_EXTERNAL_TAG: u8 = 254;
pub const LEAN_MAX_CTOR_TAG: u8 = 244;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LeanObject {
    pub rc: i32,
    pub cs_sz: u16,
    pub other: u8,
    pub tag: u8,
}

pub const LEAN_OBJECT_SIZE: usize = std::mem::size_of::<LeanObject>();

impl LeanObject {
    /// Pointer to the start of object (pointer) fields, immediately after the header.
    #[inline(always)]
    pub unsafe fn obj_fields_ptr(&self) -> *mut *mut LeanObject {
        (self as *const Self as *mut u8).add(LEAN_OBJECT_SIZE) as *mut *mut LeanObject
    }

    /// Pointer to scalar data region (after the object fields).
    #[inline(always)]
    pub unsafe fn scalar_data_ptr(&self) -> *mut u8 {
        (self as *const Self as *mut u8)
            .add(LEAN_OBJECT_SIZE)
            .add(self.num_objs() as usize * 8)
    }

    /// Number of object (pointer) fields. Stored in the `other` header byte.
    #[inline(always)]
    pub fn num_objs(&self) -> u32 {
        self.other as u32
    }
}

#[inline(always)]
pub const fn lean_ctor_object_size(num_objs: u32, scalar_sz: u32) -> usize {
    LEAN_OBJECT_SIZE + (num_objs as usize) * 8 + scalar_sz as usize
}

#[inline]
pub unsafe fn lean_alloc_object(size: usize) -> *mut LeanObject {
    let layout = Layout::from_size_align_unchecked(size, 8);
    let ptr = alloc(layout) as *mut LeanObject;
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    ptr
}

#[inline]
pub unsafe fn lean_free_object(o: *mut LeanObject, size: usize) {
    #[cfg(any(debug_assertions, feature = "runtime-debug"))]
    crate::debug::lean_debug_poison(o);
    let layout = Layout::from_size_align_unchecked(size, 8);
    dealloc(o as *mut u8, layout);
}
