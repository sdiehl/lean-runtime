//! Safe wrapper around Lean objects with RAII reference counting

use lean_runtime::*;

/// Safe wrapper around LeanObject pointer.
/// RAII handles reference counting automatically.
#[repr(transparent)]
pub struct LeanValue(pub(crate) *mut LeanObject);

impl LeanValue {
    /// Create from raw pointer (takes ownership, does NOT increment refcount)
    ///
    /// # Safety
    /// The pointer must be a valid `LeanObject` pointer or a scalar (tagged pointer).
    /// Caller transfers ownership of the reference count to this `LeanValue`.
    #[inline]
    pub unsafe fn from_raw(ptr: *mut LeanObject) -> Self {
        LeanValue(ptr)
    }

    /// Create from raw pointer and increment refcount
    ///
    /// # Safety
    /// The pointer must be a valid `LeanObject` pointer or a scalar (tagged pointer).
    #[inline]
    pub unsafe fn from_raw_inc(ptr: *mut LeanObject) -> Self {
        if !lean_is_scalar(ptr) {
            lean_inc_ref(ptr);
        }
        LeanValue(ptr)
    }

    /// Get raw pointer without affecting refcount
    #[inline]
    pub fn as_ptr(&self) -> *mut LeanObject {
        self.0
    }

    /// Consume self and return raw pointer (does NOT decrement refcount)
    #[inline]
    pub fn into_raw(self) -> *mut LeanObject {
        let ptr = self.0;
        std::mem::forget(self);
        ptr
    }

    /// Check if this is a scalar (tagged pointer, no heap allocation)
    #[inline]
    pub fn is_scalar(&self) -> bool {
        lean_is_scalar(self.0)
    }

    /// Get constructor tag (or unboxed value for scalars)
    #[inline]
    pub fn tag(&self) -> usize {
        unsafe { lean_obj_tag(self.0) as usize }
    }

    /// Check if we have exclusive access (rc=1, can mutate in place)
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        if self.is_scalar() {
            false
        } else {
            unsafe { (*self.0).rc == 1 }
        }
    }

    /// Check if object is shared (rc > 1)
    #[inline]
    pub fn is_shared(&self) -> bool {
        if self.is_scalar() {
            false
        } else {
            unsafe { (*self.0).rc > 1 }
        }
    }

    // === Constructors for common values ===

    /// Create unit value (scalar 0)
    #[inline]
    pub fn unit() -> Self {
        LeanValue(lean_box(0))
    }

    /// Create boolean value
    #[inline]
    pub fn from_bool(b: bool) -> Self {
        LeanValue(lean_box(if b { 1 } else { 0 }))
    }

    /// Create from small nat (fits in tagged pointer)
    #[inline]
    pub fn from_small_nat(n: usize) -> Self {
        LeanValue(lean_box(n))
    }

    /// Create from nat (may allocate)
    #[inline]
    pub fn from_nat(n: u64) -> Self {
        if n <= LEAN_MAX_SMALL_NAT as u64 {
            LeanValue(lean_box(n as usize))
        } else {
            LeanValue(lean_uint64_to_nat(n))
        }
    }

    /// Create from string
    pub fn from_string(s: &str) -> Self {
        unsafe {
            let ptr = lean_mk_string_unchecked(s.as_ptr(), s.len(), s.chars().count());
            LeanValue(ptr)
        }
    }

    // === Accessors ===

    /// Unbox to usize (only valid for scalars)
    #[inline]
    pub fn unbox(&self) -> usize {
        debug_assert!(self.is_scalar());
        lean_unbox(self.0)
    }

    /// Get as u64 nat (handles both small and big nats)
    pub fn to_nat_u64(&self) -> Option<u64> {
        if self.is_scalar() {
            Some(self.unbox() as u64)
        } else if self.tag() == 0 {
            // Big nat - would need mpz conversion
            // For now, only support small nats
            None
        } else {
            None
        }
    }

    // === Constructor operations ===

    /// Get field from constructor (increments refcount of field)
    #[inline]
    pub fn ctor_get(&self, idx: u32) -> LeanValue {
        debug_assert!(!self.is_scalar());
        unsafe {
            let field = lean_ctor_get(self.0, idx);
            if !lean_is_scalar(field) {
                lean_inc_ref(field);
            }
            LeanValue(field)
        }
    }

    /// Set field in constructor.
    /// Takes ownership of val.
    /// Note: Lean's IR borrow analysis ensures this is safe even when rc > 1.
    /// The C runtime doesn't check rc here - it trusts the compiler's analysis.
    #[inline]
    pub fn ctor_set(&mut self, idx: u32, val: LeanValue) {
        unsafe {
            // Dec old field
            let old = lean_ctor_get(self.0, idx);
            if !lean_is_scalar(old) {
                lean_dec_ref(old);
            }
            // Set new field (val ownership transferred)
            lean_ctor_set(self.0, idx, val.into_raw());
        }
    }

    // === Closure operations ===

    /// Get captured variable from closure
    #[inline]
    pub fn closure_get(&self, idx: u32) -> LeanValue {
        debug_assert!(!self.is_scalar());
        unsafe {
            let field = lean_closure_get(self.0, idx);
            if !lean_is_scalar(field) {
                lean_inc_ref(field);
            }
            LeanValue(field)
        }
    }
}

impl Clone for LeanValue {
    #[inline]
    fn clone(&self) -> Self {
        if !self.is_scalar() {
            unsafe {
                lean_inc_ref(self.0);
            }
        }
        LeanValue(self.0)
    }
}

impl LeanValue {
    /// Copy the raw pointer WITHOUT incrementing ref count.
    /// This implements "borrow" semantics matching the IR's explicit ref counting model.
    /// Use this for LoadLocal - the IR has explicit Inc/Dec instructions for ref counting.
    #[inline]
    pub fn copy_ptr(&self) -> Self {
        LeanValue(self.0)
    }
}

impl Drop for LeanValue {
    #[inline]
    fn drop(&mut self) {
        if !self.is_scalar() {
            unsafe {
                lean_dec_ref(self.0);
            }
        }
    }
}

impl std::fmt::Debug for LeanValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_scalar() {
            write!(f, "Scalar({})", self.unbox())
        } else {
            write!(f, "Object(tag={}, rc={})", self.tag(), unsafe {
                (*self.0).rc
            })
        }
    }
}

// === Allocation functions ===

/// Allocate a new constructor
pub fn alloc_ctor(tag: u8, num_objs: u32, scalar_size: usize) -> LeanValue {
    unsafe {
        let obj = lean_alloc_ctor(tag as u32, num_objs, scalar_size as u32);
        LeanValue(obj)
    }
}

/// Allocate a new closure
///
/// # Safety
/// `func_ptr` must be a valid function pointer compatible with the Lean closure calling convention.
pub unsafe fn alloc_closure(func_ptr: *const (), arity: u8, num_fixed: u32) -> LeanValue {
    let obj = lean_alloc_closure(func_ptr, arity as u32, num_fixed);
    LeanValue(obj)
}
