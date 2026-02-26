//! IO primitives

#[allow(unused_imports)]
use crate::object::*;
#[allow(unused_imports)]
use crate::r#box::*;
#[allow(unused_imports)]
use crate::string::*;

#[inline]
pub unsafe fn lean_io_result_mk_ok(value: *mut LeanObject) -> *mut LeanObject {
    let obj = crate::lean_alloc_ctor(0, 2, 0);
    crate::lean_ctor_set(obj, 0, value);
    crate::lean_ctor_set(obj, 1, lean_box(0));
    obj
}

#[inline]
pub unsafe fn lean_io_result_mk_error(error: *mut LeanObject) -> *mut LeanObject {
    let obj = crate::lean_alloc_ctor(1, 2, 0);
    crate::lean_ctor_set(obj, 0, error);
    crate::lean_ctor_set(obj, 1, lean_box(0));
    obj
}

#[inline]
pub unsafe fn lean_io_result_is_ok(r: *mut LeanObject) -> bool {
    (*r).tag == 0
}

#[inline]
pub unsafe fn lean_io_result_get_value(r: *mut LeanObject) -> *mut LeanObject {
    crate::lean_ctor_get(r, 0)
}

pub unsafe fn lean_io_prim_println(s: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    println!("{}", lean_string_to_str(s));
    crate::lean_dec(s);
    lean_io_result_mk_ok(lean_box(0))
}

pub unsafe fn lean_io_prim_print(s: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    print!("{}", lean_string_to_str(s));
    crate::lean_dec(s);
    lean_io_result_mk_ok(lean_box(0))
}

pub unsafe fn lean_io_prim_eprintln(s: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    eprintln!("{}", lean_string_to_str(s));
    crate::lean_dec(s);
    lean_io_result_mk_ok(lean_box(0))
}

pub unsafe fn lean_io_pure(value: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(value)
}

pub unsafe fn lean_io_bind(
    action: *mut LeanObject,
    cont: *mut LeanObject,
    rw: *mut LeanObject,
) -> *mut LeanObject {
    let result = crate::lean_apply_1(action, rw);
    if lean_io_result_is_ok(result) {
        let value = lean_io_result_get_value(result);
        crate::lean_inc(value);
        let new_rw = crate::lean_ctor_get(result, 1);
        crate::lean_inc(new_rw);
        crate::lean_dec(result);
        crate::lean_apply_2(cont, value, new_rw)
    } else {
        crate::lean_dec(cont);
        result
    }
}

pub unsafe fn lean_io_exit(code: *mut LeanObject, _rw: *mut LeanObject) -> ! {
    std::process::exit(lean_unbox(code) as i32)
}

pub unsafe fn lean_io_getenv(name: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    let name_str = lean_string_to_str(name);
    let result = match std::env::var(name_str) {
        Ok(val) => {
            let str_obj = lean_mk_string(&val);
            let opt = crate::lean_alloc_ctor(1, 1, 0);
            crate::lean_ctor_set(opt, 0, str_obj);
            opt
        }
        Err(_) => lean_box(0),
    };
    crate::lean_dec(name);
    lean_io_result_mk_ok(result)
}

unsafe fn stdout_write_impl(s: *mut LeanObject, _rw: *mut LeanObject) -> *mut LeanObject {
    print!("{}", lean_string_to_str(s));
    crate::lean_dec(s);
    lean_io_result_mk_ok(lean_box(0))
}

static mut STDOUT_WRITE_CLOSURE: *mut LeanObject = std::ptr::null_mut();
static mut STDOUT_HANDLE: *mut LeanObject = std::ptr::null_mut();

pub unsafe fn lean_get_stdout() -> *mut LeanObject {
    if STDOUT_HANDLE.is_null() {
        STDOUT_WRITE_CLOSURE = crate::lean_alloc_closure(stdout_write_impl as *const (), 2, 0);
        crate::lean_mark_persistent(STDOUT_WRITE_CLOSURE);

        STDOUT_HANDLE = crate::lean_alloc_ctor(0, 5, 0);
        crate::lean_mark_persistent(STDOUT_HANDLE);
        crate::lean_ctor_set(STDOUT_HANDLE, 0, lean_box(0));
        crate::lean_ctor_set(STDOUT_HANDLE, 1, lean_box(0));
        crate::lean_ctor_set(STDOUT_HANDLE, 2, lean_box(0));
        crate::lean_ctor_set(STDOUT_HANDLE, 3, lean_box(0));
        crate::lean_ctor_set(STDOUT_HANDLE, 4, STDOUT_WRITE_CLOSURE);
    }
    STDOUT_HANDLE
}

// ---------------------------------------------------------------------------
// IO Error constructors
// ---------------------------------------------------------------------------
//
// IO.Error is an inductive type defined in Init/System/IOError.lean.
// Constructor tags (0-indexed, in declaration order):
//   0: alreadyExists     (filename : Option String) (osCode : UInt32) (details : String) -- 2 obj, 4 scalar
//   1: otherError         (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//   2: resourceBusy       (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//   3: resourceVanished   (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//   4: unsupportedOperation (osCode : UInt32) (details : String)                          -- 1 obj, 4 scalar
//   5: hardwareFault      (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//   6: unsatisfiedConstraints (osCode : UInt32) (details : String)                        -- 1 obj, 4 scalar
//   7: illegalOperation   (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//   8: protocolError      (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//   9: timeExpired        (osCode : UInt32) (details : String)                            -- 1 obj, 4 scalar
//  10: interrupted        (filename : String) (osCode : UInt32) (details : String)        -- 2 obj, 4 scalar
//  11: noFileOrDirectory  (filename : String) (osCode : UInt32) (details : String)        -- 2 obj, 4 scalar
//  12: invalidArgument    (filename : Option String) (osCode : UInt32) (details : String) -- 2 obj, 4 scalar
//  13: permissionDenied   (filename : Option String) (osCode : UInt32) (details : String) -- 2 obj, 4 scalar
//  14: resourceExhausted  (filename : Option String) (osCode : UInt32) (details : String) -- 2 obj, 4 scalar
//  15: inappropriateType  (filename : Option String) (osCode : UInt32) (details : String) -- 2 obj, 4 scalar
//  16: noSuchThing        (filename : Option String) (osCode : UInt32) (details : String) -- 2 obj, 4 scalar
//  17: unexpectedEof                                                                      -- 0 obj, 0 scalar
//  18: userError          (msg : String)                                                  -- 1 obj, 0 scalar

/// Helper: build a 2-obj-field + UInt32-scalar error (constructors with Option String filename or String filename).
#[inline]
unsafe fn mk_io_error_2obj(
    tag: u32,
    field0: *mut LeanObject,
    field1: *mut LeanObject,
    errno: u32,
) -> *mut LeanObject {
    let ptr_sz = std::mem::size_of::<*mut LeanObject>() as u32;
    let err = crate::lean_alloc_ctor(tag, 2, 4);
    crate::lean_ctor_set(err, 0, field0);
    crate::lean_ctor_set(err, 1, field1);
    crate::lean_ctor_set_uint32(err, 2 * ptr_sz, errno);
    err
}

/// Helper: build a 1-obj-field + UInt32-scalar error.
#[inline]
unsafe fn mk_io_error_1obj(tag: u32, details: *mut LeanObject, errno: u32) -> *mut LeanObject {
    let ptr_sz = std::mem::size_of::<*mut LeanObject>() as u32;
    let err = crate::lean_alloc_ctor(tag, 1, 4);
    crate::lean_ctor_set(err, 0, details);
    crate::lean_ctor_set_uint32(err, ptr_sz, errno);
    err
}

/// Create a user error from a string (tag 18).
pub unsafe fn lean_mk_io_user_error(s: *mut LeanObject) -> *mut LeanObject {
    let err = crate::lean_alloc_ctor(18, 1, 0);
    crate::lean_ctor_set(err, 0, s);
    err
}

// --- Constructors without file parameter ---
// These pass Option.none as the filename (where applicable) or have no filename field.

/// alreadyExists none errno details (tag 0)
pub unsafe fn lean_mk_io_error_already_exists(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(0, lean_box(0), details, errno)
}

/// alreadyExists (some fname) errno details (tag 0)
pub unsafe fn lean_mk_io_error_already_exists_file(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    let opt_some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(opt_some, 0, fname);
    mk_io_error_2obj(0, opt_some, details, errno)
}

/// otherError errno details (tag 1)
pub unsafe fn lean_mk_io_error_other_error(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(1, details, errno)
}

/// resourceBusy errno details (tag 2)
pub unsafe fn lean_mk_io_error_resource_busy(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(2, details, errno)
}

/// resourceVanished errno details (tag 3)
pub unsafe fn lean_mk_io_error_resource_vanished(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(3, details, errno)
}

/// unsupportedOperation errno details (tag 4)
pub unsafe fn lean_mk_io_error_unsupported_operation(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(4, details, errno)
}

/// hardwareFault errno details (tag 5)
pub unsafe fn lean_mk_io_error_hardware_fault(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(5, details, errno)
}

/// unsatisfiedConstraints errno details (tag 6)
pub unsafe fn lean_mk_io_error_unsatisfied_constraints(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(6, details, errno)
}

/// illegalOperation errno details (tag 7)
pub unsafe fn lean_mk_io_error_illegal_operation(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(7, details, errno)
}

/// protocolError errno details (tag 8)
pub unsafe fn lean_mk_io_error_protocol_error(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(8, details, errno)
}

/// timeExpired errno details (tag 9)
pub unsafe fn lean_mk_io_error_time_expired(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_1obj(9, details, errno)
}

/// interrupted fname errno details (tag 10)
pub unsafe fn lean_mk_io_error_interrupted(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(10, fname, details, errno)
}

/// noFileOrDirectory fname errno details (tag 11)
pub unsafe fn lean_mk_io_error_no_file_or_directory(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(11, fname, details, errno)
}

/// invalidArgument none errno details (tag 12)
pub unsafe fn lean_mk_io_error_invalid_argument(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(12, lean_box(0), details, errno)
}

/// invalidArgument (some fname) errno details (tag 12)
pub unsafe fn lean_mk_io_error_invalid_argument_file(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    let opt_some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(opt_some, 0, fname);
    mk_io_error_2obj(12, opt_some, details, errno)
}

/// permissionDenied none errno details (tag 13)
pub unsafe fn lean_mk_io_error_permission_denied(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(13, lean_box(0), details, errno)
}

/// permissionDenied (some fname) errno details (tag 13)
pub unsafe fn lean_mk_io_error_permission_denied_file(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    let opt_some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(opt_some, 0, fname);
    mk_io_error_2obj(13, opt_some, details, errno)
}

/// resourceExhausted none errno details (tag 14)
pub unsafe fn lean_mk_io_error_resource_exhausted(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(14, lean_box(0), details, errno)
}

/// resourceExhausted (some fname) errno details (tag 14)
pub unsafe fn lean_mk_io_error_resource_exhausted_file(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    let opt_some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(opt_some, 0, fname);
    mk_io_error_2obj(14, opt_some, details, errno)
}

/// inappropriateType none errno details (tag 15)
pub unsafe fn lean_mk_io_error_inappropriate_type(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(15, lean_box(0), details, errno)
}

/// inappropriateType (some fname) errno details (tag 15)
pub unsafe fn lean_mk_io_error_inappropriate_type_file(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    let opt_some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(opt_some, 0, fname);
    mk_io_error_2obj(15, opt_some, details, errno)
}

/// noSuchThing none errno details (tag 16)
pub unsafe fn lean_mk_io_error_no_such_thing(
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    mk_io_error_2obj(16, lean_box(0), details, errno)
}

/// noSuchThing (some fname) errno details (tag 16)
pub unsafe fn lean_mk_io_error_no_such_thing_file(
    fname: *mut LeanObject,
    errno: u32,
    details: *mut LeanObject,
) -> *mut LeanObject {
    let opt_some = crate::lean_alloc_ctor(1, 1, 0);
    crate::lean_ctor_set(opt_some, 0, fname);
    mk_io_error_2obj(16, opt_some, details, errno)
}

/// unexpectedEof (tag 17) -- nullary constructor, encoded as scalar
pub unsafe fn lean_mk_io_error_eof(_unit: *mut LeanObject) -> *mut LeanObject {
    // unexpectedEof is a nullary constructor (tag 17, 0 fields).
    // As an optimization, Lean encodes nullary constructors as scalars: lean_box(tag).
    lean_box(17)
}

// ---------------------------------------------------------------------------
// decode_io_error: convert errno to IO.Error
// ---------------------------------------------------------------------------

/// Map a system errno to the corresponding IO.Error constructor.
/// `fname` may be null (represented as lean_box(0)) when no filename context is available.
pub unsafe fn lean_decode_io_error(errno: i32, fname: *mut LeanObject) -> *mut LeanObject {
    let details = lean_mk_string("system error");
    let has_fname = !lean_is_scalar(fname);

    match errno {
        // EINTR
        4 => {
            if has_fname {
                crate::lean_inc(fname);
            }
            let f = if has_fname { fname } else { lean_mk_string("") };
            lean_mk_io_error_interrupted(f, errno as u32, details)
        }
        // EBADF, EINVAL, EDOM, ENOEXEC, EILSEQ
        9 | 22 | 33 | 8 | 84 => {
            if has_fname {
                crate::lean_inc(fname);
                lean_mk_io_error_invalid_argument_file(fname, errno as u32, details)
            } else {
                lean_mk_io_error_invalid_argument(errno as u32, details)
            }
        }
        // ENOENT
        2 => {
            if has_fname {
                crate::lean_inc(fname);
            }
            let f = if has_fname { fname } else { lean_mk_string("") };
            lean_mk_io_error_no_file_or_directory(f, errno as u32, details)
        }
        // EACCES, EPERM, EROFS
        13 | 1 | 30 => {
            if has_fname {
                crate::lean_inc(fname);
                lean_mk_io_error_permission_denied_file(fname, errno as u32, details)
            } else {
                lean_mk_io_error_permission_denied(errno as u32, details)
            }
        }
        // ENOMEM, ENOSPC, EMFILE, ENFILE
        12 | 28 | 24 | 23 => {
            if has_fname {
                crate::lean_inc(fname);
                lean_mk_io_error_resource_exhausted_file(fname, errno as u32, details)
            } else {
                lean_mk_io_error_resource_exhausted(errno as u32, details)
            }
        }
        // EISDIR, ENOTDIR
        21 | 20 => {
            if has_fname {
                crate::lean_inc(fname);
                lean_mk_io_error_inappropriate_type_file(fname, errno as u32, details)
            } else {
                lean_mk_io_error_inappropriate_type(errno as u32, details)
            }
        }
        // ESRCH, ECHILD, ENXIO, ECONNREFUSED
        3 | 10 | 6 | 61 => {
            if has_fname {
                crate::lean_inc(fname);
                lean_mk_io_error_no_such_thing_file(fname, errno as u32, details)
            } else {
                lean_mk_io_error_no_such_thing(errno as u32, details)
            }
        }
        // EEXIST
        17 => {
            if has_fname {
                crate::lean_inc(fname);
                lean_mk_io_error_already_exists_file(fname, errno as u32, details)
            } else {
                lean_mk_io_error_already_exists(errno as u32, details)
            }
        }
        // EIO
        5 => lean_mk_io_error_hardware_fault(errno as u32, details),
        // ENOTEMPTY
        66 => lean_mk_io_error_unsatisfied_constraints(errno as u32, details),
        // ENOTTY
        25 => lean_mk_io_error_illegal_operation(errno as u32, details),
        // ECONNRESET, EPIPE
        54 | 32 => lean_mk_io_error_resource_vanished(errno as u32, details),
        // EPROTO, EPROTOTYPE
        100 | 41 => lean_mk_io_error_protocol_error(errno as u32, details),
        // ETIMEDOUT
        60 => lean_mk_io_error_time_expired(errno as u32, details),
        // EBUSY, EADDRINUSE
        16 | 48 => lean_mk_io_error_resource_busy(errno as u32, details),
        // ENOSYS, EOPNOTSUPP, ERANGE
        78 | 45 | 34 => lean_mk_io_error_unsupported_operation(errno as u32, details),
        // Default
        _ => lean_mk_io_error_other_error(errno as u32, details),
    }
}

// ---------------------------------------------------------------------------
// stdin / stderr handles
// ---------------------------------------------------------------------------

/// Get the stdin handle (boxed scalar 0).
pub unsafe fn lean_get_stdin(_unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Get the stderr handle (boxed scalar 2).
pub unsafe fn lean_get_stderr(_unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(2))
}

/// Swap stdin handle (returns old handle).
pub unsafe fn lean_get_set_stdin(h: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(h)
}

/// Swap stdout handle (returns old handle).
pub unsafe fn lean_get_set_stdout(h: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(h)
}

/// Swap stderr handle (returns old handle).
pub unsafe fn lean_get_set_stderr(h: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(h)
}

// ---------------------------------------------------------------------------
// Time functions
// ---------------------------------------------------------------------------

/// Return monotonic time in milliseconds as a boxed Nat.
pub unsafe fn lean_io_mono_ms_now(_unit: *mut LeanObject) -> *mut LeanObject {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as usize)
        .unwrap_or(0);
    lean_io_result_mk_ok(lean_box(ms))
}

/// Return monotonic time in nanoseconds as a boxed Nat.
pub unsafe fn lean_io_mono_nanos_now(_unit: *mut LeanObject) -> *mut LeanObject {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    lean_io_result_mk_ok(lean_box(ns as usize))
}

// ---------------------------------------------------------------------------
// Random bytes
// ---------------------------------------------------------------------------

/// Get `n` pseudo-random bytes as a ByteArray.
pub unsafe fn lean_io_get_random_bytes(
    n: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let count = crate::lean_unbox(n);
    let arr = crate::sarray::lean_alloc_sarray(1, count, count);
    let data = crate::sarray::lean_sarray_data(arr);
    for i in 0..count {
        *data.add(i) = (i.wrapping_mul(1103515245).wrapping_add(12345) >> 16) as u8;
    }
    lean_io_result_mk_ok(arr)
}

// ---------------------------------------------------------------------------
// Timeit / allocprof stubs
// ---------------------------------------------------------------------------

/// Time a thunk and print the elapsed time. In this runtime, just run the thunk.
pub unsafe fn lean_io_timeit(
    msg: *mut LeanObject,
    thunk: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    crate::lean_dec(msg);
    crate::lean_apply_1(thunk, lean_box(0))
}

/// Profile allocations in a thunk. In this runtime, just run the thunk.
pub unsafe fn lean_io_allocprof(
    msg: *mut LeanObject,
    thunk: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    crate::lean_dec(msg);
    crate::lean_apply_1(thunk, lean_box(0))
}

// ---------------------------------------------------------------------------
// Heartbeat stubs
// ---------------------------------------------------------------------------

/// Get the number of heartbeats consumed so far.
pub unsafe fn lean_io_get_num_heartbeats(_unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Set the heartbeat counter.
pub unsafe fn lean_io_set_heartbeats(
    _n: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

// ---------------------------------------------------------------------------
// Thread ID
// ---------------------------------------------------------------------------

/// Get the current thread ID (stub: always 0).
pub unsafe fn lean_io_get_tid(_unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

// ---------------------------------------------------------------------------
// File handle operations
// ---------------------------------------------------------------------------

/// Open a file handle.
/// mode: 0=read, 1=write, 2=readWrite, 3=append
pub unsafe fn lean_io_prim_handle_mk(
    fname: *mut LeanObject,
    mode: u8,
    _bin: u8,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let name = lean_string_to_str(fname).to_string();
    crate::lean_dec(fname);
    use std::fs::{File, OpenOptions};
    let result = match mode {
        0 => File::open(&name),
        1 => File::create(&name),
        2 => OpenOptions::new().read(true).write(true).open(&name),
        3 => OpenOptions::new().append(true).create(true).open(&name),
        _ => File::open(&name),
    };
    match result {
        Ok(file) => {
            let handle = Box::into_raw(Box::new(file)) as usize;
            lean_io_result_mk_ok(lean_box(handle))
        }
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Flush a file handle. For scalar handles (stdin=0, stdout=1, stderr=2), flush stdio.
pub unsafe fn lean_io_prim_handle_flush(
    h: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    if lean_is_scalar(h) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
    }
    lean_io_result_mk_ok(lean_box(0))
}

/// Read up to `nbytes` bytes from a handle into a ByteArray.
pub unsafe fn lean_io_prim_handle_read(
    h: *mut LeanObject,
    nbytes: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let n = crate::lean_unbox(nbytes);
    let arr = crate::sarray::lean_alloc_sarray(1, 0, n);
    let data = crate::sarray::lean_sarray_data(arr);

    use std::io::Read;
    let bytes_read = if lean_is_scalar(h) {
        let handle_id = lean_unbox(h);
        if handle_id == 0 {
            std::io::stdin()
                .read(std::slice::from_raw_parts_mut(data, n))
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        let file = &mut *(lean_unbox(h) as *mut std::fs::File);
        file.read(std::slice::from_raw_parts_mut(data, n))
            .unwrap_or(0)
    };

    (*(arr as *mut crate::sarray::LeanSArray)).size = bytes_read;
    lean_io_result_mk_ok(arr)
}

/// Write a ByteArray to a handle.
pub unsafe fn lean_io_prim_handle_write(
    h: *mut LeanObject,
    buf: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let sarr = buf as *mut crate::sarray::LeanSArray;
    let size = (*sarr).size;
    let data = crate::sarray::lean_sarray_data(buf);
    let bytes = std::slice::from_raw_parts(data, size);

    use std::io::Write;
    if lean_is_scalar(h) {
        let handle_id = lean_unbox(h);
        match handle_id {
            1 => {
                let _ = std::io::stdout().write_all(bytes);
            }
            2 => {
                let _ = std::io::stderr().write_all(bytes);
            }
            _ => {}
        }
    } else {
        let file = &mut *(lean_unbox(h) as *mut std::fs::File);
        let _ = file.write_all(bytes);
    }
    crate::lean_dec(buf);
    lean_io_result_mk_ok(lean_box(0))
}

/// Read a line from a handle (returns a Lean String).
pub unsafe fn lean_io_prim_handle_get_line(
    h: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    use std::io::BufRead;
    let mut line = String::new();
    if lean_is_scalar(h) {
        let handle_id = lean_unbox(h);
        if handle_id == 0 {
            let _ = std::io::stdin().lock().read_line(&mut line);
        }
    } else {
        let file = &mut *(lean_unbox(h) as *mut std::fs::File);
        let mut buf_reader = std::io::BufReader::new(file);
        let _ = buf_reader.read_line(&mut line);
    }
    lean_io_result_mk_ok(lean_mk_string(&line))
}

/// Write a string to a handle.
pub unsafe fn lean_io_prim_handle_put_str(
    h: *mut LeanObject,
    s: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let str_val = lean_string_to_str(s);
    use std::io::Write;
    if lean_is_scalar(h) {
        let handle_id = lean_unbox(h);
        match handle_id {
            0 | 1 => {
                let _ = std::io::stdout().write_all(str_val.as_bytes());
            }
            2 => {
                let _ = std::io::stderr().write_all(str_val.as_bytes());
            }
            _ => {}
        }
    } else {
        let file = &mut *(lean_unbox(h) as *mut std::fs::File);
        let _ = file.write_all(str_val.as_bytes());
    }
    crate::lean_dec(s);
    lean_io_result_mk_ok(lean_box(0))
}

/// Check if a handle is a TTY (stub: always false).
pub unsafe fn lean_io_prim_handle_is_tty(
    _h: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0)) // false
}

/// Rewind a handle to the beginning (stub).
pub unsafe fn lean_io_prim_handle_rewind(
    _h: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Truncate a handle (stub).
pub unsafe fn lean_io_prim_handle_truncate(
    _h: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Lock a file handle (stub: always succeeds).
pub unsafe fn lean_io_prim_handle_lock(
    _h: *mut LeanObject,
    _exclusive: u8,
    _blocking: u8,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Try to lock a file handle (stub: always succeeds).
pub unsafe fn lean_io_prim_handle_try_lock(
    _h: *mut LeanObject,
    _exclusive: u8,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(1)) // true = success
}

/// Unlock a file handle (stub).
pub unsafe fn lean_io_prim_handle_unlock(
    _h: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

// ---------------------------------------------------------------------------
// Filesystem operations
// ---------------------------------------------------------------------------

/// Resolve a path to its canonical absolute form.
pub unsafe fn lean_io_realpath(fname: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    let name = lean_string_to_str(fname).to_string();
    crate::lean_dec(fname);
    match std::fs::canonicalize(&name) {
        Ok(path) => lean_io_result_mk_ok(lean_mk_string(&path.to_string_lossy())),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Remove a file.
pub unsafe fn lean_io_remove_file(
    fname: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let name = lean_string_to_str(fname).to_string();
    crate::lean_dec(fname);
    match std::fs::remove_file(&name) {
        Ok(()) => lean_io_result_mk_ok(lean_box(0)),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Remove a directory (optionally recursive).
pub unsafe fn lean_io_remove_dir(
    fname: *mut LeanObject,
    recursive: u8,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let name = lean_string_to_str(fname).to_string();
    crate::lean_dec(fname);
    let result = if recursive != 0 {
        std::fs::remove_dir_all(&name)
    } else {
        std::fs::remove_dir(&name)
    };
    match result {
        Ok(()) => lean_io_result_mk_ok(lean_box(0)),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Create a directory.
pub unsafe fn lean_io_create_dir(
    fname: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let name = lean_string_to_str(fname).to_string();
    crate::lean_dec(fname);
    match std::fs::create_dir(&name) {
        Ok(()) => lean_io_result_mk_ok(lean_box(0)),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Rename a file or directory.
pub unsafe fn lean_io_rename(
    old: *mut LeanObject,
    new_name: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let old_s = lean_string_to_str(old).to_string();
    let new_s = lean_string_to_str(new_name).to_string();
    crate::lean_dec(old);
    crate::lean_dec(new_name);
    match std::fs::rename(&old_s, &new_s) {
        Ok(()) => lean_io_result_mk_ok(lean_box(0)),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Create a hard link.
pub unsafe fn lean_io_hard_link(
    target: *mut LeanObject,
    link: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let t = lean_string_to_str(target).to_string();
    let l = lean_string_to_str(link).to_string();
    crate::lean_dec(target);
    crate::lean_dec(link);
    match std::fs::hard_link(&t, &l) {
        Ok(()) => lean_io_result_mk_ok(lean_box(0)),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Create a temporary file. Returns (path, handle).
pub unsafe fn lean_io_create_tempfile(_unit: *mut LeanObject) -> *mut LeanObject {
    let path = std::env::temp_dir().join(format!("lean_tmp_{}", std::process::id()));
    match std::fs::File::create(&path) {
        Ok(file) => {
            let handle = Box::into_raw(Box::new(file)) as usize;
            let pair = crate::lean_alloc_ctor(0, 2, 0);
            crate::lean_ctor_set(pair, 0, lean_mk_string(&path.to_string_lossy()));
            crate::lean_ctor_set(pair, 1, lean_box(handle));
            lean_io_result_mk_ok(pair)
        }
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Create a temporary directory. Returns path.
pub unsafe fn lean_io_create_tempdir(_unit: *mut LeanObject) -> *mut LeanObject {
    let path = std::env::temp_dir().join(format!("lean_tmpdir_{}", std::process::id()));
    match std::fs::create_dir_all(&path) {
        Ok(()) => lean_io_result_mk_ok(lean_mk_string(&path.to_string_lossy())),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Get the current working directory.
pub unsafe fn lean_io_current_dir(_unit: *mut LeanObject) -> *mut LeanObject {
    match std::env::current_dir() {
        Ok(p) => lean_io_result_mk_ok(lean_mk_string(&p.to_string_lossy())),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Get the path of the current executable.
pub unsafe fn lean_io_app_path(_unit: *mut LeanObject) -> *mut LeanObject {
    match std::env::current_exe() {
        Ok(p) => lean_io_result_mk_ok(lean_mk_string(&p.to_string_lossy())),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Set file permissions (stub: no-op on this platform).
pub unsafe fn lean_chmod(
    path: *mut LeanObject,
    _mode: u32,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    crate::lean_dec(path);
    lean_io_result_mk_ok(lean_box(0))
}

// ---------------------------------------------------------------------------
// Task stubs (single-threaded implementation)
// ---------------------------------------------------------------------------

/// Spawn a task. In single-threaded mode, evaluate the closure immediately.
pub unsafe fn lean_task_spawn(c: *mut LeanObject, _prio: *mut LeanObject) -> *mut LeanObject {
    crate::lean_apply_1(c, lean_box(0))
}

/// Wrap a pure value as a task result.
pub unsafe fn lean_task_pure(a: *mut LeanObject) -> *mut LeanObject {
    a
}

/// Bind a task to a continuation.
pub unsafe fn lean_task_bind(
    x: *mut LeanObject,
    f: *mut LeanObject,
    _prio: *mut LeanObject,
    _sync: u8,
) -> *mut LeanObject {
    crate::lean_apply_1(f, x)
}

/// Map a function over a task.
pub unsafe fn lean_task_map(
    f: *mut LeanObject,
    t: *mut LeanObject,
    _prio: *mut LeanObject,
    _sync: u8,
) -> *mut LeanObject {
    crate::lean_apply_1(f, t)
}

/// Get the value of a completed task (borrowed reference).
pub unsafe fn lean_task_get(t: *mut LeanObject) -> *mut LeanObject {
    crate::lean_inc(t);
    t
}

/// Get the value of a completed task (owned reference).
pub unsafe fn lean_task_get_own(t: *mut LeanObject) -> *mut LeanObject {
    t
}

/// Check if the current task has been canceled (stub: always false).
pub unsafe fn lean_io_check_canceled_core() -> u8 {
    0
}

/// Cancel a task (stub: no-op).
pub unsafe fn lean_io_cancel_core(_t: *mut LeanObject) {}

/// Get the state of a task (stub: always 2 = finished).
pub unsafe fn lean_io_get_task_state_core(_t: *mut LeanObject) -> u8 {
    2
}

/// Wait for any of a list of tasks to complete. Returns the first task.
pub unsafe fn lean_io_wait_any_core(task_list: *mut LeanObject) -> *mut LeanObject {
    if !lean_is_scalar(task_list) && crate::lean_obj_tag(task_list) == 1 {
        let head = crate::lean_ctor_get(task_list, 0);
        crate::lean_inc(head);
        head
    } else {
        lean_box(0)
    }
}

/// Run an IO action as a task.
pub unsafe fn lean_io_as_task(
    action: *mut LeanObject,
    _prio: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let result = crate::lean_apply_1(action, lean_box(0));
    if lean_io_result_is_ok(result) {
        let val = crate::lean_ctor_get(result, 0);
        crate::lean_inc(val);
        crate::lean_dec(result);
        lean_io_result_mk_ok(val)
    } else {
        result
    }
}

/// Map a function over a task result in IO.
pub unsafe fn lean_io_map_task(
    f: *mut LeanObject,
    t: *mut LeanObject,
    _prio: *mut LeanObject,
    _sync: u8,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let result = crate::lean_apply_1(f, t);
    lean_io_result_mk_ok(result)
}

/// Bind a task result to an IO continuation.
pub unsafe fn lean_io_bind_task(
    t: *mut LeanObject,
    f: *mut LeanObject,
    _prio: *mut LeanObject,
    _sync: u8,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let result = crate::lean_apply_1(f, t);
    lean_io_result_mk_ok(result)
}

// ---------------------------------------------------------------------------
// Promise stubs
// ---------------------------------------------------------------------------

/// Create a new promise.
pub unsafe fn lean_io_promise_new(_unit: *mut LeanObject) -> *mut LeanObject {
    let promise = crate::lean_alloc_ctor(0, 1, 0);
    crate::lean_ctor_set(promise, 0, lean_box(0));
    lean_io_result_mk_ok(promise)
}

/// Resolve a promise with a value.
pub unsafe fn lean_io_promise_resolve(
    val: *mut LeanObject,
    promise: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    if !lean_is_scalar(promise) {
        crate::lean_ctor_set(promise, 0, val);
    }
    lean_io_result_mk_ok(lean_box(0))
}

/// Get the result of a promise if available (returns Option).
pub unsafe fn lean_io_promise_result_opt(promise: *mut LeanObject) -> *mut LeanObject {
    if !lean_is_scalar(promise) {
        let v = crate::lean_ctor_get(promise, 0);
        if !lean_is_scalar(v) || lean_unbox(v) != 0 {
            crate::lean_inc(v);
            let some = crate::lean_alloc_ctor(1, 1, 0);
            crate::lean_ctor_set(some, 0, v);
            return some;
        }
    }
    lean_box(0) // none
}

/// Get the value from an Option, blocking if none (stub: returns value or default).
pub unsafe fn lean_option_get_or_block(promise: *mut LeanObject) -> *mut LeanObject {
    if !lean_is_scalar(promise) {
        let v = crate::lean_ctor_get(promise, 0);
        crate::lean_inc(v);
        return v;
    }
    lean_box(0)
}

// ---------------------------------------------------------------------------
// Process stubs
// ---------------------------------------------------------------------------

/// Spawn a child process (stub: returns error).
pub unsafe fn lean_io_process_spawn(
    cfg: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    crate::lean_dec(cfg);
    let msg = lean_mk_string("process spawn not fully implemented");
    lean_io_result_mk_error(lean_mk_io_user_error(msg))
}

/// Wait for a child process to exit.
pub unsafe fn lean_io_process_child_wait(
    _child: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Try to wait for a child process (non-blocking).
pub unsafe fn lean_io_process_child_try_wait(
    _child: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Kill a child process.
pub unsafe fn lean_io_process_child_kill(
    _child: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Take the stdin handle from a child process.
pub unsafe fn lean_io_process_child_take_stdin(
    _child: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Get the PID of a child process.
pub unsafe fn lean_io_process_child_pid(
    _child: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(0))
}

/// Get the current working directory (process variant).
pub unsafe fn lean_io_process_get_current_dir(_unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_current_dir(_unit)
}

/// Set the current working directory.
pub unsafe fn lean_io_process_set_current_dir(
    dir: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    let d = lean_string_to_str(dir).to_string();
    crate::lean_dec(dir);
    match std::env::set_current_dir(&d) {
        Ok(()) => lean_io_result_mk_ok(lean_box(0)),
        Err(e) => {
            let msg = lean_mk_string(&e.to_string());
            lean_io_result_mk_error(lean_mk_io_user_error(msg))
        }
    }
}

/// Get the current process ID.
pub unsafe fn lean_io_process_get_pid(_unit: *mut LeanObject) -> *mut LeanObject {
    lean_io_result_mk_ok(lean_box(std::process::id() as usize))
}

// ---------------------------------------------------------------------------
// Directory and metadata stubs
// ---------------------------------------------------------------------------

/// Read directory entries (stub: returns empty list).
pub unsafe fn lean_io_read_dir(path: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_dec(path);
    lean_io_result_mk_ok(lean_box(0)) // empty list (nil)
}

/// Get file metadata (stub: returns error).
pub unsafe fn lean_io_metadata(path: *mut LeanObject, _unit: *mut LeanObject) -> *mut LeanObject {
    crate::lean_dec(path);
    let msg = lean_mk_string("metadata not implemented");
    lean_io_result_mk_error(lean_mk_io_user_error(msg))
}

/// Get symlink metadata (stub: returns error).
pub unsafe fn lean_io_symlink_metadata(
    path: *mut LeanObject,
    _unit: *mut LeanObject,
) -> *mut LeanObject {
    lean_io_metadata(path, _unit)
}

// ---------------------------------------------------------------------------
// ShareCommon stubs
// ---------------------------------------------------------------------------

/// Check if two objects are pointer-equal.
pub unsafe fn lean_sharecommon_eq(a: *mut LeanObject, b: *mut LeanObject) -> u8 {
    (a == b) as u8
}

/// Hash an object by its pointer address.
pub unsafe fn lean_sharecommon_hash(a: *mut LeanObject) -> u64 {
    a as u64
}

/// Share-common with state: return (a, st) pair.
pub unsafe fn lean_state_sharecommon(a: *mut LeanObject, st: *mut LeanObject) -> *mut LeanObject {
    crate::lean_inc(a);
    let pair = crate::lean_alloc_ctor(0, 2, 0);
    crate::lean_ctor_set(pair, 0, a);
    crate::lean_ctor_set(pair, 1, st);
    pair
}

/// Quick share-common (no-op: return the object).
pub unsafe fn lean_sharecommon_quick(a: *mut LeanObject) -> *mut LeanObject {
    crate::lean_inc(a);
    a
}
