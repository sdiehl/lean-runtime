//! Tagged pointer boxing

use crate::object::LeanObject;

#[inline(always)]
pub fn lean_box(n: usize) -> *mut LeanObject {
    ((n << 1) | 1) as *mut LeanObject
}

#[inline(always)]
pub fn lean_unbox(o: *mut LeanObject) -> usize {
    (o as usize) >> 1
}

#[inline(always)]
pub fn lean_is_scalar(o: *mut LeanObject) -> bool {
    (o as usize & 1) == 1
}

#[inline(always)]
pub fn lean_box_uint32(n: u32) -> *mut LeanObject {
    lean_box(n as usize)
}

#[inline(always)]
pub fn lean_unbox_uint32(o: *mut LeanObject) -> u32 {
    lean_unbox(o) as u32
}

#[inline(always)]
pub fn lean_box_uint64(n: u64) -> *mut LeanObject {
    lean_box(n as usize)
}

#[inline(always)]
pub fn lean_unbox_uint64(o: *mut LeanObject) -> u64 {
    lean_unbox(o) as u64
}

#[inline(always)]
pub fn lean_box_usize(n: usize) -> *mut LeanObject {
    lean_box(n)
}

#[inline(always)]
pub fn lean_unbox_usize(o: *mut LeanObject) -> usize {
    lean_unbox(o)
}

#[inline(always)]
pub fn lean_uint32_dec_eq(a: u32, b: u32) -> u8 {
    (a == b) as u8
}

#[inline(always)]
pub fn lean_uint32_dec_lt(a: u32, b: u32) -> u8 {
    (a < b) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_unbox_roundtrip() {
        let v = 42usize;
        let boxed = lean_box(v);
        assert_eq!(lean_unbox(boxed), v);
    }

    #[test]
    fn is_scalar() {
        assert!(lean_is_scalar(lean_box(0)));
        assert!(lean_is_scalar(lean_box(100)));
        // A heap pointer (even address) is not scalar
        let ptr = 0x1000 as *mut LeanObject;
        assert!(!lean_is_scalar(ptr));
    }

    #[test]
    fn uint32_roundtrip() {
        let v: u32 = 12345;
        let boxed = lean_box_uint32(v);
        assert_eq!(lean_unbox_uint32(boxed), v);
    }

    #[test]
    fn uint64_roundtrip() {
        let v: u64 = 99999;
        let boxed = lean_box_uint64(v);
        assert_eq!(lean_unbox_uint64(boxed), v);
    }

    #[test]
    fn usize_roundtrip() {
        let v: usize = 777;
        let boxed = lean_box_usize(v);
        assert_eq!(lean_unbox_usize(boxed), v);
    }

    #[test]
    fn box_zero() {
        let boxed = lean_box(0);
        assert_eq!(lean_unbox(boxed), 0);
        assert!(lean_is_scalar(boxed));
        // box(0) should be 0x1 (low bit set, value 0)
        assert_eq!(boxed as usize, 1);
    }

    #[test]
    fn box_large_value() {
        // Max safe value: (usize::MAX >> 1), since we shift left by 1
        let v = usize::MAX >> 1;
        let boxed = lean_box(v);
        assert_eq!(lean_unbox(boxed), v);
        assert!(lean_is_scalar(boxed));
    }

    #[test]
    fn box_all_small_values() {
        for i in 0..256 {
            let boxed = lean_box(i);
            assert!(lean_is_scalar(boxed));
            assert_eq!(lean_unbox(boxed), i);
        }
    }

    #[test]
    fn heap_pointer_not_scalar() {
        // Any even address looks like a heap pointer
        for addr in [0x8, 0x10, 0x1000, 0xFFFF_FFF0usize] {
            let p = addr as *mut LeanObject;
            assert!(!lean_is_scalar(p));
        }
    }

    #[test]
    fn uint32_max() {
        let v: u32 = u32::MAX;
        let boxed = lean_box_uint32(v);
        assert_eq!(lean_unbox_uint32(boxed), v);
    }
}
