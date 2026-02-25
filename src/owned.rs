//! Safe, owned wrapper around Lean runtime objects.
//!
//! Provides RAII-based reference counting for `*mut LeanObject`.
//! Not used by generated code (which uses explicit lean_inc/lean_dec),
//! but useful for hand-written Rust that calls the runtime.

use crate::object::LeanObject;

/// An owned reference to a Lean object with automatic reference counting.
///
/// - `Clone` increments the reference count.
/// - `Drop` decrements the reference count (and frees if zero).
/// - Scalars (tagged pointers) are no-ops for inc/dec.
pub struct LeanOwnedValue {
    ptr: *mut LeanObject,
}

impl LeanOwnedValue {
    /// Wrap a raw pointer. Takes ownership (does NOT increment rc).
    pub unsafe fn from_raw(ptr: *mut LeanObject) -> Self {
        Self { ptr }
    }

    /// Get the raw pointer without releasing ownership.
    pub fn as_ptr(&self) -> *mut LeanObject {
        self.ptr
    }

    /// Release ownership, returning the raw pointer without decrementing rc.
    pub fn into_raw(self) -> *mut LeanObject {
        let ptr = self.ptr;
        std::mem::forget(self);
        ptr
    }

    /// Check if this is a scalar (unboxed) value.
    pub fn is_scalar(&self) -> bool {
        crate::lean_is_scalar(self.ptr)
    }

    /// Get the tag of the underlying object.
    pub fn tag(&self) -> u8 {
        unsafe { crate::lean_obj_tag(self.ptr) }
    }
}

impl Clone for LeanOwnedValue {
    fn clone(&self) -> Self {
        unsafe {
            crate::lean_inc(self.ptr);
        }
        Self { ptr: self.ptr }
    }
}

impl Drop for LeanOwnedValue {
    fn drop(&mut self) {
        unsafe {
            crate::lean_dec(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_roundtrip() {
        unsafe {
            let v = LeanOwnedValue::from_raw(crate::lean_box(42));
            assert!(v.is_scalar());
            assert_eq!(v.tag(), 42);
            assert_eq!(crate::lean_unbox(v.as_ptr()), 42);
        }
    }

    #[test]
    fn clone_increments_rc() {
        unsafe {
            let s = crate::lean_mk_string("hello");
            let v = LeanOwnedValue::from_raw(s);
            assert_eq!((*v.as_ptr()).rc, 1);
            let v2 = v.clone();
            assert_eq!((*v.as_ptr()).rc, 2);
            drop(v2);
            assert_eq!((*v.as_ptr()).rc, 1);
        }
    }

    #[test]
    fn into_raw_prevents_drop() {
        unsafe {
            let s = crate::lean_mk_string("keep");
            let v = LeanOwnedValue::from_raw(s);
            let ptr = v.into_raw();
            // rc should still be 1 (no dec happened)
            assert_eq!((*ptr).rc, 1);
            crate::lean_dec(ptr);
        }
    }

    #[test]
    fn drop_frees_heap_object() {
        unsafe {
            let s = crate::lean_mk_string("bye");
            let v = LeanOwnedValue::from_raw(s);
            assert_eq!((*v.as_ptr()).rc, 1);
            drop(v);
            // If we get here without a crash, the object was freed correctly.
        }
    }

    #[test]
    fn scalar_clone_drop_noop() {
        unsafe {
            let v = LeanOwnedValue::from_raw(crate::lean_box(7));
            let v2 = v.clone();
            drop(v);
            drop(v2);
            // Scalars should not crash on clone/drop.
        }
    }
}
