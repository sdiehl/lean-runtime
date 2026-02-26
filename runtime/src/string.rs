//! String support

use crate::object::*;
use std::ptr;
use std::slice;

#[repr(C)]
pub struct LeanString {
    pub header: LeanObject,
    pub byte_len: usize,
    pub utf8_len: usize,
    pub capacity: usize,
}

impl LeanString {
    #[inline(always)]
    pub unsafe fn data_ptr(&self) -> *mut u8 {
        (self as *const Self as *mut u8).add(std::mem::size_of::<Self>())
    }
}

#[inline(always)]
pub const fn lean_string_object_size(capacity: usize) -> usize {
    std::mem::size_of::<LeanString>() + capacity + 1
}

#[inline]
pub unsafe fn lean_mk_string_unchecked(
    s: *const u8,
    byte_len: usize,
    utf8_len: usize,
) -> *mut LeanObject {
    let size = lean_string_object_size(byte_len);
    let obj = lean_alloc_object(size);
    let str_obj = obj as *mut LeanString;
    (*str_obj).header.rc = 1;
    (*str_obj).header.tag = LEAN_STRING_TAG;
    (*str_obj).header.other = 0;
    (*str_obj).header.cs_sz = 0;
    (*str_obj).byte_len = byte_len;
    (*str_obj).utf8_len = utf8_len;
    (*str_obj).capacity = byte_len;
    let data = (*str_obj).data_ptr();
    ptr::copy_nonoverlapping(s, data, byte_len);
    *data.add(byte_len) = 0;
    obj
}

pub fn lean_mk_string(s: &str) -> *mut LeanObject {
    let byte_len = s.len();
    let utf8_len = s.chars().count();
    unsafe { lean_mk_string_unchecked(s.as_ptr(), byte_len, utf8_len) }
}

#[inline(always)]
pub unsafe fn lean_string_byte_len(s: *mut LeanObject) -> usize {
    (*(s as *mut LeanString)).byte_len
}

#[inline(always)]
pub unsafe fn lean_string_utf8_len(s: *mut LeanObject) -> usize {
    (*(s as *mut LeanString)).utf8_len
}

#[inline(always)]
pub unsafe fn lean_string_cstr(s: *mut LeanObject) -> *const u8 {
    (*(s as *mut LeanString)).data_ptr()
}

pub unsafe fn lean_string_to_str(s: *mut LeanObject) -> &'static str {
    let str_obj = s as *mut LeanString;
    let data = (*str_obj).data_ptr();
    let bytes = slice::from_raw_parts(data, (*str_obj).byte_len);
    std::str::from_utf8_unchecked(bytes)
}

/// Append two strings. s1 is owned (consumed), s2 is borrowed.
/// When s1 is exclusive and has enough capacity, appends in-place to avoid a copy.
pub unsafe fn lean_string_append(s1: *mut LeanObject, s2: *mut LeanObject) -> *mut LeanObject {
    let len1 = lean_string_byte_len(s1);
    let len2 = lean_string_byte_len(s2);
    let new_len = len1 + len2;

    // Fast path: reuse s1's buffer if exclusive and has capacity
    if crate::rc::lean_is_exclusive(s1) {
        let str1 = s1 as *mut LeanString;
        if (*str1).capacity >= new_len {
            let data = (*str1).data_ptr();
            ptr::copy_nonoverlapping(lean_string_cstr(s2), data.add(len1), len2);
            *data.add(new_len) = 0;
            (*str1).byte_len = new_len;
            (*str1).utf8_len += lean_string_utf8_len(s2);
            return s1;
        }
    }

    // Slow path: allocate new string with 2x growth for amortized O(1) appends
    let utf8_len = lean_string_utf8_len(s1) + lean_string_utf8_len(s2);
    let capacity = new_len.max(len1 * 2);
    let size = lean_string_object_size(capacity);
    let obj = lean_alloc_object(size);
    let str_obj = obj as *mut LeanString;
    (*str_obj).header.rc = 1;
    (*str_obj).header.tag = LEAN_STRING_TAG;
    (*str_obj).header.other = 0;
    (*str_obj).header.cs_sz = 0;
    (*str_obj).byte_len = new_len;
    (*str_obj).utf8_len = utf8_len;
    (*str_obj).capacity = capacity;
    let data = (*str_obj).data_ptr();
    ptr::copy_nonoverlapping(lean_string_cstr(s1), data, len1);
    ptr::copy_nonoverlapping(lean_string_cstr(s2), data.add(len1), len2);
    *data.add(new_len) = 0;
    crate::lean_dec(s1);
    obj
}

pub unsafe fn lean_string_eq(s1: *mut LeanObject, s2: *mut LeanObject) -> bool {
    if s1 == s2 {
        return true;
    }
    let len1 = lean_string_byte_len(s1);
    let len2 = lean_string_byte_len(s2);
    if len1 != len2 {
        return false;
    }
    libc::memcmp(lean_string_cstr(s1) as _, lean_string_cstr(s2) as _, len1) == 0
}

pub unsafe fn lean_string_dec_eq(s1: *mut LeanObject, s2: *mut LeanObject) -> u8 {
    lean_string_eq(s1, s2) as u8
}

pub unsafe fn lean_nat_to_string(n: *mut LeanObject) -> *mut LeanObject {
    if crate::lean_is_scalar(n) {
        lean_mk_string(&crate::lean_unbox(n).to_string())
    } else {
        let val = crate::bignat::lean_bignat_value(n);
        let s = val.to_string();
        let r = lean_mk_string(&s);
        crate::lean_dec(n);
        r
    }
}

/// Parse a string into a Nat. Accepts `&str` from generated Rust code.
pub fn lean_cstr_to_nat(s: &str) -> *mut LeanObject {
    use num_bigint::BigUint;
    match BigUint::parse_bytes(s.as_bytes(), 10) {
        Some(val) => unsafe { crate::bignat::lean_bignat_to_nat(val) },
        None => crate::lean_box(0),
    }
}

/// Return the byte size of a string as a boxed Nat.
pub unsafe fn lean_string_utf8_byte_size(s: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(lean_string_byte_len(s))
}

/// Convert a String to a ByteArray.
pub unsafe fn lean_string_to_utf8(s: *mut LeanObject) -> *mut LeanObject {
    let byte_len = lean_string_byte_len(s);
    let data = lean_string_cstr(s);
    let arr = crate::sarray::lean_alloc_sarray(1, byte_len, byte_len);
    let arr_data = crate::sarray::lean_sarray_data(arr);
    ptr::copy_nonoverlapping(data, arr_data, byte_len);
    crate::lean_dec(s);
    arr
}

/// Convert a ByteArray to a String (unchecked — assumes valid UTF-8).
pub unsafe fn lean_string_from_utf8_unchecked(a: *mut LeanObject) -> *mut LeanObject {
    let sarr = a as *mut crate::sarray::LeanSArray;
    let size = (*sarr).size;
    let data = crate::sarray::lean_sarray_data(a);
    let bytes = std::slice::from_raw_parts(data, size);
    // Count UTF-8 characters
    let utf8_len = std::str::from_utf8_unchecked(bytes).chars().count();
    let r = lean_mk_string_unchecked(data, size, utf8_len);
    crate::lean_dec(a);
    r
}

pub unsafe fn lean_string_push(s: *mut LeanObject, c: u32) -> *mut LeanObject {
    let ch = char::from_u32(c).unwrap_or('\u{FFFD}');
    let mut buf = [0u8; 4];
    let char_str = ch.encode_utf8(&mut buf);
    let char_len = char_str.len();

    let old_byte_len = lean_string_byte_len(s);
    let old_utf8_len = lean_string_utf8_len(s);
    let new_byte_len = old_byte_len + char_len;
    let new_utf8_len = old_utf8_len + 1;

    let size = lean_string_object_size(new_byte_len);
    let obj = lean_alloc_object(size);
    let str_obj = obj as *mut LeanString;
    (*str_obj).header.rc = 1;
    (*str_obj).header.tag = LEAN_STRING_TAG;
    (*str_obj).header.other = 0;
    (*str_obj).header.cs_sz = 0;
    (*str_obj).byte_len = new_byte_len;
    (*str_obj).utf8_len = new_utf8_len;
    (*str_obj).capacity = new_byte_len;

    let data = (*str_obj).data_ptr();
    ptr::copy_nonoverlapping(lean_string_cstr(s), data, old_byte_len);
    ptr::copy_nonoverlapping(char_str.as_ptr(), data.add(old_byte_len), char_len);
    *data.add(new_byte_len) = 0;

    crate::lean_dec(s);
    obj
}

// ---------------------------------------------------------------------------
// UTF-8 helpers
// ---------------------------------------------------------------------------

/// Number of bytes in a UTF-8 character given its first byte.
#[inline(always)]
fn utf8_char_width(first_byte: u8) -> usize {
    if first_byte < 0x80 {
        1
    } else if first_byte < 0xE0 {
        2
    } else if first_byte < 0xF0 {
        3
    } else {
        4
    }
}

/// Decode a UTF-8 code point starting at `p` with `remaining` bytes available.
/// Returns (codepoint, byte_width). Returns (replacement_char, 1) on truncation.
#[inline]
unsafe fn utf8_decode(p: *const u8, remaining: usize) -> (u32, usize) {
    if remaining == 0 {
        return (0xFFFD, 1);
    }
    let b0 = *p;
    if b0 < 0x80 {
        (b0 as u32, 1)
    } else if b0 < 0xE0 {
        if remaining < 2 {
            return (0xFFFD, 1);
        }
        let c = ((b0 as u32 & 0x1F) << 6) | (*p.add(1) as u32 & 0x3F);
        (c, 2)
    } else if b0 < 0xF0 {
        if remaining < 3 {
            return (0xFFFD, 1);
        }
        let c = ((b0 as u32 & 0x0F) << 12)
            | ((*p.add(1) as u32 & 0x3F) << 6)
            | (*p.add(2) as u32 & 0x3F);
        (c, 3)
    } else {
        if remaining < 4 {
            return (0xFFFD, 1);
        }
        let c = ((b0 as u32 & 0x07) << 18)
            | ((*p.add(1) as u32 & 0x3F) << 12)
            | ((*p.add(2) as u32 & 0x3F) << 6)
            | (*p.add(3) as u32 & 0x3F);
        (c, 4)
    }
}

// ---------------------------------------------------------------------------
// New string functions
// ---------------------------------------------------------------------------

/// Decode the UTF-8 character at byte offset `i`, return as UInt32.
pub unsafe fn lean_string_utf8_get(s: *mut LeanObject, i: *mut LeanObject) -> u32 {
    let idx = crate::lean_unbox(i);
    let byte_len = lean_string_byte_len(s);
    if idx >= byte_len {
        0
    } else {
        let data = lean_string_cstr(s);
        let (cp, _) = utf8_decode(data.add(idx), byte_len - idx);
        cp
    }
}

/// Return the byte offset of the next UTF-8 character after byte offset `i`.
pub unsafe fn lean_string_utf8_next(s: *mut LeanObject, i: *mut LeanObject) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    let byte_len = lean_string_byte_len(s);
    if idx >= byte_len {
        crate::lean_box(idx + 1)
    } else {
        let data = lean_string_cstr(s);
        let w = utf8_char_width(*data.add(idx));
        crate::lean_box(idx + w)
    }
}

/// Return the byte offset of the previous UTF-8 character before byte offset `i`.
pub unsafe fn lean_string_utf8_prev(s: *mut LeanObject, i: *mut LeanObject) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    if idx == 0 {
        return crate::lean_box(0);
    }
    let data = lean_string_cstr(s);
    let byte_len = lean_string_byte_len(s);
    let mut pos = if idx > byte_len { byte_len } else { idx };
    pos -= 1;
    while pos > 0 && (*data.add(pos) & 0xC0) == 0x80 {
        pos -= 1;
    }
    crate::lean_box(pos)
}

/// Check if byte offset `i` is at or past the end of the string.
pub unsafe fn lean_string_utf8_at_end(s: *mut LeanObject, i: *mut LeanObject) -> u8 {
    let idx = crate::lean_unbox(i);
    (idx >= lean_string_byte_len(s)) as u8
}

/// Extract a substring from byte offset `b` to byte offset `e`.
pub unsafe fn lean_string_utf8_extract(
    s: *mut LeanObject,
    b: *mut LeanObject,
    e: *mut LeanObject,
) -> *mut LeanObject {
    let byte_len = lean_string_byte_len(s);
    let mut begin = crate::lean_unbox(b);
    let mut end = crate::lean_unbox(e);
    if begin > byte_len {
        begin = byte_len;
    }
    if end > byte_len {
        end = byte_len;
    }
    if begin >= end {
        return lean_mk_string("");
    }
    let data = lean_string_cstr(s);
    let sub_bytes = end - begin;
    let mut utf8_count = 0usize;
    let mut pos = begin;
    while pos < end {
        let w = utf8_char_width(*data.add(pos));
        utf8_count += 1;
        pos += w;
    }
    lean_mk_string_unchecked(data.add(begin), sub_bytes, utf8_count)
}

/// Replace the UTF-8 character at byte offset `i` with character `c`.
pub unsafe fn lean_string_utf8_set(
    s: *mut LeanObject,
    i: *mut LeanObject,
    c: u32,
) -> *mut LeanObject {
    let idx = crate::lean_unbox(i);
    let byte_len = lean_string_byte_len(s);
    if idx >= byte_len {
        return s;
    }
    let data = lean_string_cstr(s);
    let old_w = utf8_char_width(*data.add(idx));

    let ch = char::from_u32(c).unwrap_or('\u{FFFD}');
    let mut buf = [0u8; 4];
    let encoded = ch.encode_utf8(&mut buf);
    let new_w = encoded.len();

    let new_byte_len = byte_len - old_w + new_w;
    let old_utf8_len = lean_string_utf8_len(s);

    let size = lean_string_object_size(new_byte_len);
    let obj = lean_alloc_object(size);
    let str_obj = obj as *mut LeanString;
    (*str_obj).header.rc = 1;
    (*str_obj).header.tag = LEAN_STRING_TAG;
    (*str_obj).header.other = 0;
    (*str_obj).header.cs_sz = 0;
    (*str_obj).byte_len = new_byte_len;
    (*str_obj).utf8_len = old_utf8_len;
    (*str_obj).capacity = new_byte_len;

    let new_data = (*str_obj).data_ptr();
    ptr::copy_nonoverlapping(data, new_data, idx);
    ptr::copy_nonoverlapping(encoded.as_ptr(), new_data.add(idx), new_w);
    let suffix_len = byte_len - idx - old_w;
    ptr::copy_nonoverlapping(data.add(idx + old_w), new_data.add(idx + new_w), suffix_len);
    *new_data.add(new_byte_len) = 0;

    crate::lean_dec(s);
    obj
}

/// Return the UTF-8 character count as a boxed Nat.
pub unsafe fn lean_string_length(s: *mut LeanObject) -> *mut LeanObject {
    crate::lean_box(lean_string_utf8_len(s))
}

/// Convert a List Char to a String.
pub unsafe fn lean_string_mk(cs: *mut LeanObject) -> *mut LeanObject {
    let mut total_bytes = 0usize;
    let mut total_chars = 0usize;
    let mut cur = cs;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        let c_val = crate::lean_unbox(crate::lean_ctor_get(cur, 0));
        let ch = char::from_u32(c_val as u32).unwrap_or('\u{FFFD}');
        total_bytes += ch.len_utf8();
        total_chars += 1;
        cur = crate::lean_ctor_get(cur, 1);
    }

    let size = lean_string_object_size(total_bytes);
    let obj = lean_alloc_object(size);
    let str_obj = obj as *mut LeanString;
    (*str_obj).header.rc = 1;
    (*str_obj).header.tag = LEAN_STRING_TAG;
    (*str_obj).header.other = 0;
    (*str_obj).header.cs_sz = 0;
    (*str_obj).byte_len = total_bytes;
    (*str_obj).utf8_len = total_chars;
    (*str_obj).capacity = total_bytes;

    let data = (*str_obj).data_ptr();
    let mut pos = 0;
    cur = cs;
    while !crate::lean_is_scalar(cur) && crate::lean_obj_tag(cur) == 1 {
        let c_val = crate::lean_unbox(crate::lean_ctor_get(cur, 0));
        let ch = char::from_u32(c_val as u32).unwrap_or('\u{FFFD}');
        let mut buf = [0u8; 4];
        let encoded = ch.encode_utf8(&mut buf);
        ptr::copy_nonoverlapping(encoded.as_ptr(), data.add(pos), encoded.len());
        pos += encoded.len();
        cur = crate::lean_ctor_get(cur, 1);
    }
    *data.add(total_bytes) = 0;

    crate::lean_dec(cs);
    obj
}

/// Convert a String to a List Char.
pub unsafe fn lean_string_data(s: *mut LeanObject) -> *mut LeanObject {
    let data = lean_string_cstr(s);
    let byte_len = lean_string_byte_len(s);

    let mut chars = Vec::new();
    let mut pos = 0;
    while pos < byte_len {
        let (cp, w) = utf8_decode(data.add(pos), byte_len - pos);
        chars.push(cp);
        pos += w;
    }

    let mut list = crate::lean_box(0); // nil
    for &cp in chars.iter().rev() {
        let cons = crate::lean_alloc_ctor(1, 2, 0);
        crate::lean_ctor_set(cons, 0, crate::lean_box(cp as usize));
        crate::lean_ctor_set(cons, 1, list);
        list = cons;
    }

    crate::lean_dec(s);
    list
}

/// FNV-1a hash of string bytes.
pub unsafe fn lean_string_hash(s: *mut LeanObject) -> u64 {
    let data = lean_string_cstr(s);
    let byte_len = lean_string_byte_len(s);
    let mut hash: u64 = 0xcbf29ce484222325;
    for i in 0..byte_len {
        hash ^= *data.add(i) as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Lexicographic less-than comparison.
pub unsafe fn lean_string_lt(s1: *mut LeanObject, s2: *mut LeanObject) -> bool {
    let len1 = lean_string_byte_len(s1);
    let len2 = lean_string_byte_len(s2);
    let min_len = len1.min(len2);
    let cmp = libc::memcmp(
        lean_string_cstr(s1) as _,
        lean_string_cstr(s2) as _,
        min_len,
    );
    if cmp < 0 {
        true
    } else if cmp > 0 {
        false
    } else {
        len1 < len2
    }
}

/// Less-than as u8.
pub unsafe fn lean_string_dec_lt(s1: *mut LeanObject, s2: *mut LeanObject) -> u8 {
    lean_string_lt(s1, s2) as u8
}

/// String.utf8GetFast -- same as lean_string_utf8_get but without the fallback closure param.
pub unsafe fn lean_string_utf8_get_fast(s: *mut LeanObject, i: *mut LeanObject) -> u32 {
    lean_string_utf8_get(s, i)
}

/// Cold path for utf8_get_fast (fallback for out-of-range).
pub unsafe fn lean_string_utf8_get_fast_cold(s: *mut LeanObject, i: *mut LeanObject) -> u32 {
    lean_string_utf8_get(s, i)
}

/// String.utf8NextFast -- same as lean_string_utf8_next.
pub unsafe fn lean_string_utf8_next_fast(
    s: *mut LeanObject,
    i: *mut LeanObject,
) -> *mut LeanObject {
    lean_string_utf8_next(s, i)
}

pub unsafe fn lean_string_utf8_next_fast_cold(
    s: *mut LeanObject,
    i: *mut LeanObject,
) -> *mut LeanObject {
    lean_string_utf8_next(s, i)
}

/// String.getByteAt -- get a single byte at position (not UTF-8 aware).
pub unsafe fn lean_string_get_byte_fast(s: *mut LeanObject, i: *mut LeanObject) -> u8 {
    let pos = crate::lean_unbox(i);
    let data = lean_string_cstr(s);
    *data.add(pos)
}

/// String.utf8GetOpt -- get char at position, return Option Char.
pub unsafe fn lean_string_utf8_get_opt(s: *mut LeanObject, i: *mut LeanObject) -> *mut LeanObject {
    let pos = crate::lean_unbox(i);
    let byte_len = lean_string_byte_len(s);
    if pos >= byte_len {
        return crate::lean_box(0); // Option.none
    }
    let ch = lean_string_utf8_get(s, i);
    let some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(some, 0, crate::lean_box(ch as usize));
    some
}

/// String.utf8Get! -- get char at position, panic on out of bounds.
pub unsafe fn lean_string_utf8_get_bang(s: *mut LeanObject, i: *mut LeanObject) -> u32 {
    lean_string_utf8_get(s, i)
}

/// String.isValidPos -- check if position is a valid UTF-8 boundary.
pub unsafe fn lean_string_is_valid_pos(s: *mut LeanObject, i: *mut LeanObject) -> u8 {
    let pos = crate::lean_unbox(i);
    let byte_len = lean_string_byte_len(s);
    if pos > byte_len {
        return 0;
    }
    if pos == 0 || pos == byte_len {
        return 1;
    }
    let data = lean_string_cstr(s);
    let byte = *data.add(pos);
    // A valid UTF-8 boundary is not a continuation byte (10xxxxxx)
    ((byte & 0xC0) != 0x80) as u8
}

/// String.ofUSize -- create a single-character string from a USize (Unicode code point).
pub unsafe fn lean_string_of_usize(c: usize) -> *mut LeanObject {
    if let Some(ch) = char::from_u32(c as u32) {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        lean_mk_string(s)
    } else {
        lean_mk_string("")
    }
}

/// String.memcmp -- compare substrings.
pub unsafe fn lean_string_memcmp(
    s1: *mut LeanObject,
    s2: *mut LeanObject,
    lstart: *mut LeanObject,
    rstart: *mut LeanObject,
    len: *mut LeanObject,
) -> u8 {
    let l = crate::lean_unbox(lstart);
    let r = crate::lean_unbox(rstart);
    let n = crate::lean_unbox(len);
    let d1 = lean_string_cstr(s1);
    let d2 = lean_string_cstr(s2);
    let bl1 = lean_string_byte_len(s1);
    let bl2 = lean_string_byte_len(s2);
    if l + n > bl1 || r + n > bl2 {
        return 0;
    }
    let slice1 = std::slice::from_raw_parts(d1.add(l), n);
    let slice2 = std::slice::from_raw_parts(d2.add(r), n);
    (slice1 == slice2) as u8
}

/// Slice hash
pub unsafe fn lean_slice_hash(s: *mut LeanObject) -> u64 {
    lean_string_hash(s)
}

/// Slice comparison
pub unsafe fn lean_slice_dec_lt(s1: *mut LeanObject, s2: *mut LeanObject) -> u8 {
    lean_string_lt(s1, s2) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mk_and_to_str() {
        let s = lean_mk_string("hello");
        unsafe {
            assert_eq!(lean_string_to_str(s), "hello");
            assert_eq!(lean_string_byte_len(s), 5);
            assert_eq!(lean_string_utf8_len(s), 5);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn multibyte_lengths() {
        let s = lean_mk_string("café");
        unsafe {
            assert_eq!(lean_string_byte_len(s), 5);
            assert_eq!(lean_string_utf8_len(s), 4);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn append() {
        unsafe {
            let s1 = lean_mk_string("hello ");
            let s2 = lean_mk_string("world");
            let s3 = lean_string_append(s1, s2);
            assert_eq!(lean_string_to_str(s3), "hello world");
            crate::lean_dec(s3);
        }
    }

    #[test]
    fn eq() {
        unsafe {
            let s1 = lean_mk_string("abc");
            let s2 = lean_mk_string("abc");
            let s3 = lean_mk_string("xyz");
            assert!(lean_string_eq(s1, s2));
            assert!(!lean_string_eq(s1, s3));
            crate::lean_dec(s1);
            crate::lean_dec(s2);
            crate::lean_dec(s3);
        }
    }

    #[test]
    fn push() {
        unsafe {
            let s = lean_mk_string("ab");
            let s2 = lean_string_push(s, 'c' as u32);
            assert_eq!(lean_string_to_str(s2), "abc");
            crate::lean_dec(s2);
        }
    }

    #[test]
    fn utf8_get_ascii() {
        unsafe {
            let s = lean_mk_string("abc");
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(0)), 'a' as u32);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(1)), 'b' as u32);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(2)), 'c' as u32);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(3)), 0);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_get_multibyte() {
        unsafe {
            let s = lean_mk_string("aé");
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(0)), 'a' as u32);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(1)), 'é' as u32);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_next() {
        unsafe {
            let s = lean_mk_string("aé");
            let n1 = lean_string_utf8_next(s, crate::lean_box(0));
            assert_eq!(crate::lean_unbox(n1), 1);
            let n2 = lean_string_utf8_next(s, crate::lean_box(1));
            assert_eq!(crate::lean_unbox(n2), 3);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_prev() {
        unsafe {
            let s = lean_mk_string("aé");
            let p = lean_string_utf8_prev(s, crate::lean_box(3));
            assert_eq!(crate::lean_unbox(p), 1);
            let p2 = lean_string_utf8_prev(s, crate::lean_box(1));
            assert_eq!(crate::lean_unbox(p2), 0);
            let p3 = lean_string_utf8_prev(s, crate::lean_box(0));
            assert_eq!(crate::lean_unbox(p3), 0);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_at_end() {
        unsafe {
            let s = lean_mk_string("ab");
            assert_eq!(lean_string_utf8_at_end(s, crate::lean_box(0)), 0);
            assert_eq!(lean_string_utf8_at_end(s, crate::lean_box(1)), 0);
            assert_eq!(lean_string_utf8_at_end(s, crate::lean_box(2)), 1);
            assert_eq!(lean_string_utf8_at_end(s, crate::lean_box(3)), 1);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_extract() {
        unsafe {
            let s = lean_mk_string("hello world");
            let sub = lean_string_utf8_extract(s, crate::lean_box(6), crate::lean_box(11));
            assert_eq!(lean_string_to_str(sub), "world");
            crate::lean_dec(sub);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_extract_empty() {
        unsafe {
            let s = lean_mk_string("abc");
            let sub = lean_string_utf8_extract(s, crate::lean_box(2), crate::lean_box(2));
            assert_eq!(lean_string_to_str(sub), "");
            crate::lean_dec(sub);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn utf8_set() {
        unsafe {
            let s = lean_mk_string("abc");
            let s2 = lean_string_utf8_set(s, crate::lean_box(1), 'X' as u32);
            assert_eq!(lean_string_to_str(s2), "aXc");
            crate::lean_dec(s2);
        }
    }

    #[test]
    fn utf8_set_multibyte() {
        unsafe {
            let s = lean_mk_string("abc");
            let s2 = lean_string_utf8_set(s, crate::lean_box(0), 'é' as u32);
            assert_eq!(lean_string_to_str(s2), "ébc");
            assert_eq!(lean_string_byte_len(s2), 4);
            crate::lean_dec(s2);
        }
    }

    #[test]
    fn string_length() {
        unsafe {
            let s = lean_mk_string("café");
            let len = lean_string_length(s);
            assert_eq!(crate::lean_unbox(len), 4);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn string_mk_data_roundtrip() {
        unsafe {
            let nil = crate::lean_box(0);
            let c3 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c3, 0, crate::lean_box('c' as usize));
            crate::lean_ctor_set(c3, 1, nil);
            let c2 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c2, 0, crate::lean_box('b' as usize));
            crate::lean_ctor_set(c2, 1, c3);
            let c1 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c1, 0, crate::lean_box('a' as usize));
            crate::lean_ctor_set(c1, 1, c2);

            let s = lean_string_mk(c1);
            assert_eq!(lean_string_to_str(s), "abc");

            crate::lean_inc(s);
            let list = lean_string_data(s);

            let h0 = crate::lean_ctor_get(list, 0);
            assert_eq!(crate::lean_unbox(h0), 'a' as usize);
            let t0 = crate::lean_ctor_get(list, 1);
            let h1 = crate::lean_ctor_get(t0, 0);
            assert_eq!(crate::lean_unbox(h1), 'b' as usize);
            let t1 = crate::lean_ctor_get(t0, 1);
            let h2 = crate::lean_ctor_get(t1, 0);
            assert_eq!(crate::lean_unbox(h2), 'c' as usize);
            let t2 = crate::lean_ctor_get(t1, 1);
            assert!(crate::lean_is_scalar(t2));

            crate::lean_dec(list);
            crate::lean_dec(s);
        }
    }

    #[test]
    fn string_hash_deterministic() {
        unsafe {
            let s1 = lean_mk_string("test");
            let s2 = lean_mk_string("test");
            assert_eq!(lean_string_hash(s1), lean_string_hash(s2));
            let s3 = lean_mk_string("other");
            assert_ne!(lean_string_hash(s1), lean_string_hash(s3));
            crate::lean_dec(s1);
            crate::lean_dec(s2);
            crate::lean_dec(s3);
        }
    }

    #[test]
    fn string_lt() {
        unsafe {
            let a = lean_mk_string("abc");
            let b = lean_mk_string("abd");
            let c = lean_mk_string("abc");
            let d = lean_mk_string("abcd");
            assert!(lean_string_lt(a, b));
            assert!(!lean_string_lt(b, a));
            assert!(!lean_string_lt(a, c));
            assert!(lean_string_lt(a, d));
            assert_eq!(lean_string_dec_lt(a, b), 1);
            assert_eq!(lean_string_dec_lt(b, a), 0);
            crate::lean_dec(a);
            crate::lean_dec(b);
            crate::lean_dec(c);
            crate::lean_dec(d);
        }
    }

    // -- non-trivial stress tests --

    /// 4-byte UTF-8 characters (emoji).
    #[test]
    fn four_byte_utf8() {
        // U+1F600 = 😀, 4 bytes in UTF-8
        let s = lean_mk_string("A😀B");
        unsafe {
            assert_eq!(lean_string_byte_len(s), 6); // 1 + 4 + 1
            assert_eq!(lean_string_utf8_len(s), 3);
            // get 'A' at byte 0
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(0)), 'A' as u32);
            // get 😀 at byte 1
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(1)), 0x1F600);
            // get 'B' at byte 5
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(5)), 'B' as u32);
            // next from 0 -> 1 (skip A)
            assert_eq!(
                crate::lean_unbox(lean_string_utf8_next(s, crate::lean_box(0))),
                1
            );
            // next from 1 -> 5 (skip 😀 = 4 bytes)
            assert_eq!(
                crate::lean_unbox(lean_string_utf8_next(s, crate::lean_box(1))),
                5
            );
            // next from 5 -> 6 (skip B)
            assert_eq!(
                crate::lean_unbox(lean_string_utf8_next(s, crate::lean_box(5))),
                6
            );
            crate::lean_dec(s);
        }
    }

    /// 3-byte CJK characters.
    #[test]
    fn three_byte_cjk() {
        // '漢' = U+6F22, 3 bytes; '字' = U+5B57, 3 bytes
        let s = lean_mk_string("漢字");
        unsafe {
            assert_eq!(lean_string_byte_len(s), 6);
            assert_eq!(lean_string_utf8_len(s), 2);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(0)), 0x6F22);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(3)), 0x5B57);
            crate::lean_dec(s);
        }
    }

    /// Empty string edge cases.
    #[test]
    fn empty_string_ops() {
        let s = lean_mk_string("");
        unsafe {
            assert_eq!(lean_string_byte_len(s), 0);
            assert_eq!(lean_string_utf8_len(s), 0);
            assert_eq!(lean_string_utf8_at_end(s, crate::lean_box(0)), 1);
            assert_eq!(lean_string_utf8_get(s, crate::lean_box(0)), 0);
            assert_eq!(
                crate::lean_unbox(lean_string_utf8_prev(s, crate::lean_box(0))),
                0
            );
            assert_eq!(crate::lean_unbox(lean_string_length(s)), 0);
            assert_eq!(lean_string_hash(s), lean_string_hash(lean_mk_string("")));
            crate::lean_dec(s);
        }
    }

    /// Full forward iteration with next + at_end.
    #[test]
    fn iterate_forward() {
        let s = lean_mk_string("aéb😀c");
        // chars: a(1) é(2) b(1) 😀(4) c(1) = 9 bytes, 5 chars
        unsafe {
            assert_eq!(lean_string_byte_len(s), 9);
            assert_eq!(lean_string_utf8_len(s), 5);
            let mut collected = Vec::new();
            let mut i = crate::lean_box(0);
            while lean_string_utf8_at_end(s, i) == 0 {
                let cp = lean_string_utf8_get(s, i);
                collected.push(char::from_u32(cp).unwrap());
                i = lean_string_utf8_next(s, i);
            }
            assert_eq!(collected, vec!['a', 'é', 'b', '😀', 'c']);
            crate::lean_dec(s);
        }
    }

    /// Full backward iteration with prev.
    #[test]
    fn iterate_backward() {
        let s = lean_mk_string("aéb");
        // a(1) é(2) b(1) = 4 bytes
        unsafe {
            let mut offsets = Vec::new();
            let mut i = crate::lean_box(lean_string_byte_len(s));
            loop {
                i = lean_string_utf8_prev(s, i);
                let off = crate::lean_unbox(i);
                offsets.push(off);
                if off == 0 {
                    break;
                }
            }
            // Should visit byte offsets: 3 (b at 3), 1 (é at 1), 0 (a at 0)
            assert_eq!(offsets, vec![3, 1, 0]);
            crate::lean_dec(s);
        }
    }

    /// Extract across multibyte boundaries.
    #[test]
    fn extract_multibyte_boundary() {
        // "aéb😀c"
        let s = lean_mk_string("aéb😀c");
        unsafe {
            // extract "éb" = bytes [1..4)
            let sub = lean_string_utf8_extract(s, crate::lean_box(1), crate::lean_box(4));
            assert_eq!(lean_string_to_str(sub), "éb");
            assert_eq!(lean_string_utf8_len(sub), 2);
            assert_eq!(lean_string_byte_len(sub), 3);
            crate::lean_dec(sub);
            // extract "b😀" = bytes [3..8)  (b=1byte, 😀=4bytes)
            let sub2 = lean_string_utf8_extract(s, crate::lean_box(3), crate::lean_box(8));
            assert_eq!(lean_string_to_str(sub2), "b😀");
            crate::lean_dec(sub2);
            crate::lean_dec(s);
        }
    }

    /// Extract with clamped out-of-bounds end.
    #[test]
    fn extract_beyond_end() {
        let s = lean_mk_string("abc");
        unsafe {
            let sub = lean_string_utf8_extract(s, crate::lean_box(1), crate::lean_box(100));
            assert_eq!(lean_string_to_str(sub), "bc");
            crate::lean_dec(sub);
            crate::lean_dec(s);
        }
    }

    /// Extract with begin > end returns empty.
    #[test]
    fn extract_inverted_range() {
        let s = lean_mk_string("abc");
        unsafe {
            let sub = lean_string_utf8_extract(s, crate::lean_box(2), crate::lean_box(1));
            assert_eq!(lean_string_to_str(sub), "");
            crate::lean_dec(sub);
            crate::lean_dec(s);
        }
    }

    /// Set: replace multibyte with multibyte.
    #[test]
    fn utf8_set_multibyte_to_multibyte() {
        // Replace é (2 bytes at offset 1) with 😀 (4 bytes)
        let s = lean_mk_string("aéc");
        unsafe {
            let s2 = lean_string_utf8_set(s, crate::lean_box(1), 0x1F600);
            assert_eq!(lean_string_to_str(s2), "a😀c");
            assert_eq!(lean_string_byte_len(s2), 6); // 1 + 4 + 1
            assert_eq!(lean_string_utf8_len(s2), 3); // char count unchanged
            crate::lean_dec(s2);
        }
    }

    /// Set: replace 4-byte with 1-byte (shrink).
    #[test]
    fn utf8_set_shrink() {
        let s = lean_mk_string("a😀c");
        unsafe {
            let s2 = lean_string_utf8_set(s, crate::lean_box(1), 'X' as u32);
            assert_eq!(lean_string_to_str(s2), "aXc");
            assert_eq!(lean_string_byte_len(s2), 3);
            crate::lean_dec(s2);
        }
    }

    /// Set out of bounds is a no-op.
    #[test]
    fn utf8_set_out_of_bounds() {
        let s = lean_mk_string("abc");
        unsafe {
            let s2 = lean_string_utf8_set(s, crate::lean_box(10), 'X' as u32);
            // Should return the same object since idx >= byte_len
            assert_eq!(lean_string_to_str(s2), "abc");
            crate::lean_dec(s2);
        }
    }

    /// Push multibyte characters.
    #[test]
    fn push_multibyte() {
        unsafe {
            let s = lean_mk_string("hi");
            let s = lean_string_push(s, 0x1F600); // 😀
            let s = lean_string_push(s, 0x6F22); // 漢
            assert_eq!(lean_string_to_str(s), "hi😀漢");
            assert_eq!(lean_string_byte_len(s), 9); // 2 + 4 + 3
            assert_eq!(lean_string_utf8_len(s), 4);
            crate::lean_dec(s);
        }
    }

    /// Append two multibyte strings.
    #[test]
    fn append_multibyte() {
        unsafe {
            let s1 = lean_mk_string("café");
            let s2 = lean_mk_string("😀漢");
            let s3 = lean_string_append(s1, s2);
            assert_eq!(lean_string_to_str(s3), "café😀漢");
            assert_eq!(lean_string_byte_len(s3), 12); // 5 + 4 + 3
            assert_eq!(lean_string_utf8_len(s3), 6);
            crate::lean_dec(s3);
        }
    }

    /// Build a string with many appends (stress allocation).
    #[test]
    fn many_appends() {
        unsafe {
            let mut s = lean_mk_string("");
            for i in 0..100 {
                let chunk = lean_mk_string(&i.to_string());
                s = lean_string_append(s, chunk);
            }
            let result = lean_string_to_str(s);
            // Verify first few and last few digits
            assert!(result.starts_with("012345"));
            assert!(result.ends_with("9899"));
            assert_eq!(lean_string_utf8_len(s), result.chars().count());
            crate::lean_dec(s);
        }
    }

    /// mk/data roundtrip with multibyte chars.
    #[test]
    fn mk_data_roundtrip_multibyte() {
        unsafe {
            // Build list: ['é', '😀']
            let nil = crate::lean_box(0);
            let c2 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c2, 0, crate::lean_box(0x1F600));
            crate::lean_ctor_set(c2, 1, nil);
            let c1 = crate::lean_alloc_ctor(1, 2, 0);
            crate::lean_ctor_set(c1, 0, crate::lean_box(0xE9)); // é
            crate::lean_ctor_set(c1, 1, c2);

            let s = lean_string_mk(c1);
            assert_eq!(lean_string_to_str(s), "é😀");
            assert_eq!(lean_string_byte_len(s), 6);
            assert_eq!(lean_string_utf8_len(s), 2);

            crate::lean_inc(s);
            let list = lean_string_data(s);

            let h0 = crate::lean_ctor_get(list, 0);
            assert_eq!(crate::lean_unbox(h0) as u32, 0xE9);
            let t0 = crate::lean_ctor_get(list, 1);
            let h1 = crate::lean_ctor_get(t0, 0);
            assert_eq!(crate::lean_unbox(h1) as u32, 0x1F600);
            let t1 = crate::lean_ctor_get(t0, 1);
            assert!(crate::lean_is_scalar(t1));

            crate::lean_dec(list);
            crate::lean_dec(s);
        }
    }

    /// mk from empty list produces empty string.
    #[test]
    fn mk_empty_list() {
        unsafe {
            let nil = crate::lean_box(0);
            let s = lean_string_mk(nil);
            assert_eq!(lean_string_to_str(s), "");
            assert_eq!(lean_string_byte_len(s), 0);
            crate::lean_dec(s);
        }
    }

    /// data of empty string produces nil list.
    #[test]
    fn data_empty_string() {
        unsafe {
            let s = lean_mk_string("");
            let list = lean_string_data(s);
            assert!(crate::lean_is_scalar(list)); // nil = box(0)
            assert_eq!(crate::lean_unbox(list), 0);
        }
    }

    /// Hash: different strings with same length have different hashes (probabilistic).
    #[test]
    fn hash_collision_resistance() {
        unsafe {
            let hashes: Vec<u64> = (0..50)
                .map(|i| {
                    let s = lean_mk_string(&format!("str_{i:04}"));
                    let h = lean_string_hash(s);
                    crate::lean_dec(s);
                    h
                })
                .collect();
            // All 50 should be distinct
            let mut sorted = hashes.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), 50);
        }
    }

    /// lt with multibyte strings.
    #[test]
    fn lt_multibyte() {
        unsafe {
            let a = lean_mk_string("café");
            let b = lean_mk_string("cafés");
            assert!(lean_string_lt(a, b)); // prefix is less
            let c = lean_mk_string("cafè"); // è > é byte-wise? No, é=C3A9, è=C3A8, so è < é
            assert!(lean_string_lt(c, a));
            crate::lean_dec(a);
            crate::lean_dec(b);
            crate::lean_dec(c);
        }
    }

    /// prev from beyond-end position clamps.
    #[test]
    fn prev_beyond_end() {
        let s = lean_mk_string("ab");
        unsafe {
            let p = lean_string_utf8_prev(s, crate::lean_box(100));
            assert_eq!(crate::lean_unbox(p), 1);
            crate::lean_dec(s);
        }
    }

    /// next from at-end position returns end+1.
    #[test]
    fn next_at_end() {
        let s = lean_mk_string("ab");
        unsafe {
            // byte_len = 2, next(2) should return 3
            let n = lean_string_utf8_next(s, crate::lean_box(2));
            assert_eq!(crate::lean_unbox(n), 3);
            crate::lean_dec(s);
        }
    }

    /// dec_eq with heap strings.
    #[test]
    fn dec_eq_strings() {
        unsafe {
            let a = lean_mk_string("hello");
            let b = lean_mk_string("hello");
            let c = lean_mk_string("world");
            assert_eq!(lean_string_dec_eq(a, b), 1);
            assert_eq!(lean_string_dec_eq(a, c), 0);
            crate::lean_dec(a);
            crate::lean_dec(b);
            crate::lean_dec(c);
        }
    }
}
