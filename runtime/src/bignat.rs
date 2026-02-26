//! Big natural number support using num-bigint.
//!
//! A big nat is stored as a heap object with tag LEAN_MPZ_TAG (250).
//! Layout: [LeanObject header: 8 bytes][BigUint: 24 bytes] = 32 bytes total.

use crate::object::*;
use crate::r#box::*;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::mem;

/// Maximum value that fits in a scalar (tagged pointer).
/// Scalars encode as `(n << 1) | 1`, so the max value is `usize::MAX >> 1`.
pub const LEAN_MAX_SMALL_NAT: usize = usize::MAX >> 1;

/// Size of a big nat object in bytes.
const LEAN_BIGNAT_OBJECT_SIZE: usize = LEAN_OBJECT_SIZE + mem::size_of::<BigUint>();

/// Allocate a new big nat object from a `BigUint`.
/// The returned object has rc=1 and tag=LEAN_MPZ_TAG.
pub unsafe fn lean_alloc_bignat(val: BigUint) -> *mut LeanObject {
    let obj = lean_alloc_object(LEAN_BIGNAT_OBJECT_SIZE);
    (*obj).rc = 1;
    (*obj).tag = LEAN_MPZ_TAG;
    (*obj).other = 0;
    (*obj).cs_sz = 0;
    let data_ptr = (obj as *mut u8).add(LEAN_OBJECT_SIZE) as *mut BigUint;
    std::ptr::write(data_ptr, val);
    obj
}

/// Get a reference to the BigUint stored in a big nat object.
/// The object must have tag LEAN_MPZ_TAG.
#[inline]
pub unsafe fn lean_bignat_value(o: *mut LeanObject) -> &'static BigUint {
    debug_assert_eq!((*o).tag, LEAN_MPZ_TAG);
    let data_ptr = (o as *mut u8).add(LEAN_OBJECT_SIZE) as *const BigUint;
    &*data_ptr
}

/// Get a mutable reference to the BigUint stored in a big nat object.
#[inline]
pub unsafe fn lean_bignat_value_mut(o: *mut LeanObject) -> &'static mut BigUint {
    debug_assert_eq!((*o).tag, LEAN_MPZ_TAG);
    let data_ptr = (o as *mut u8).add(LEAN_OBJECT_SIZE) as *mut BigUint;
    &mut *data_ptr
}

/// Convert a BigUint to a Lean object: scalar if it fits, otherwise heap big nat.
pub unsafe fn lean_bignat_to_nat(val: BigUint) -> *mut LeanObject {
    match val.to_usize() {
        Some(n) if n <= LEAN_MAX_SMALL_NAT => lean_box(n),
        _ => lean_alloc_bignat(val),
    }
}

/// Extract a BigUint from any Lean nat (scalar or big nat).
pub unsafe fn lean_nat_to_biguint(o: *mut LeanObject) -> BigUint {
    if lean_is_scalar(o) {
        BigUint::from(lean_unbox(o))
    } else {
        lean_bignat_value(o).clone()
    }
}

/// Drop the BigUint inside a big nat object (must be called before freeing).
pub unsafe fn lean_bignat_drop(o: *mut LeanObject) {
    debug_assert_eq!((*o).tag, LEAN_MPZ_TAG);
    let data_ptr = (o as *mut u8).add(LEAN_OBJECT_SIZE) as *mut BigUint;
    std::ptr::drop_in_place(data_ptr);
}

/// Free a big nat object: drop the BigUint, then free the memory.
pub unsafe fn lean_free_bignat(o: *mut LeanObject) {
    lean_bignat_drop(o);
    lean_free_object(o, LEAN_BIGNAT_OBJECT_SIZE);
}

/// Size of a big nat object (constant).
pub const fn lean_bignat_object_size() -> usize {
    LEAN_BIGNAT_OBJECT_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_read() {
        unsafe {
            let val = BigUint::from(12345u64);
            let obj = lean_alloc_bignat(val.clone());
            assert_eq!(*lean_bignat_value(obj), val);
            assert_eq!((*obj).tag, LEAN_MPZ_TAG);
            assert_eq!((*obj).rc, 1);
            lean_free_bignat(obj);
        }
    }

    #[test]
    fn demote_small() {
        unsafe {
            let val = BigUint::from(42u64);
            let obj = lean_bignat_to_nat(val);
            assert!(lean_is_scalar(obj));
            assert_eq!(lean_unbox(obj), 42);
        }
    }

    #[test]
    fn keep_big() {
        unsafe {
            let val = BigUint::from(u128::MAX);
            let obj = lean_bignat_to_nat(val.clone());
            assert!(!lean_is_scalar(obj));
            assert_eq!(*lean_bignat_value(obj), val);
            lean_free_bignat(obj);
        }
    }

    #[test]
    fn nat_to_biguint_scalar() {
        unsafe {
            let obj = lean_box(99);
            let val = lean_nat_to_biguint(obj);
            assert_eq!(val, BigUint::from(99u64));
        }
    }

    #[test]
    fn nat_to_biguint_big() {
        unsafe {
            let big = BigUint::from(u128::MAX);
            let obj = lean_alloc_bignat(big.clone());
            let val = lean_nat_to_biguint(obj);
            assert_eq!(val, big);
            lean_free_bignat(obj);
        }
    }

    #[test]
    fn roundtrip_large() {
        unsafe {
            // 2^128 + 1
            let val: BigUint = BigUint::from(1u64) << 128 | BigUint::from(1u64);
            let obj = lean_bignat_to_nat(val.clone());
            assert!(!lean_is_scalar(obj));
            let back = lean_nat_to_biguint(obj);
            assert_eq!(back, val);
            lean_free_bignat(obj);
        }
    }
}
