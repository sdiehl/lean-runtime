//! Lean 4 Rust Runtime
//!
//! This is a low-level runtime library for compiled Lean programs.

pub mod array;
pub mod bignat;
pub mod r#box;
pub mod closure;
pub mod ctor;
pub mod debug;
pub mod external;
pub mod float;
pub mod floatarray;
pub mod int;
pub mod io;
pub mod misc;
pub mod nat;
pub mod object;
pub mod owned;
pub mod panic;
pub mod platform;
pub mod rc;
pub mod sarray;
pub mod sint;
pub mod stref;
pub mod string;
pub mod thunk;
pub mod uint;

// Re-export commonly used items
pub use array::{
    lean_alloc_array, lean_array_data, lean_array_fget, lean_array_fget_borrowed, lean_array_fset,
    lean_array_fswap, lean_array_get, lean_array_get_borrowed, lean_array_get_core,
    lean_array_get_size, lean_array_mk, lean_array_pop, lean_array_push, lean_array_set,
    lean_array_size, lean_array_swap, lean_array_to_list, lean_array_uget, lean_array_uset,
    lean_copy_array, lean_mk_array, lean_mk_empty_array, lean_mk_empty_array_with_capacity,
    LeanArray,
};

pub use bignat::{
    lean_alloc_bignat, lean_bignat_to_nat, lean_bignat_value, lean_free_bignat,
    lean_nat_to_biguint, LEAN_MAX_SMALL_NAT,
};

pub use int::{
    lean_alloc_bigint, lean_bigint_value, lean_cstr_to_int, lean_free_bigint, lean_int_add,
    lean_int_dec_eq, lean_int_dec_le, lean_int_dec_lt, lean_int_dec_nonneg, lean_int_div,
    lean_int_div_exact, lean_int_ediv, lean_int_emod, lean_int_eq, lean_int_le, lean_int_lt,
    lean_int_mod, lean_int_mul, lean_int_ne, lean_int_neg, lean_int_neg_succ_of_nat, lean_int_sub,
    lean_int_to_int, lean_int_to_nat, lean_nat_abs, lean_nat_succ, lean_nat_to_int,
    lean_scalar_to_int64, LEAN_MAX_SMALL_INT, LEAN_MIN_SMALL_INT,
};

pub use r#box::{
    lean_box, lean_box_uint32, lean_box_uint64, lean_box_usize, lean_is_scalar, lean_uint32_dec_eq,
    lean_uint32_dec_lt, lean_unbox, lean_unbox_uint32, lean_unbox_uint64, lean_unbox_usize,
};

pub use closure::{
    lean_alloc_closure, lean_apply_1, lean_apply_2, lean_apply_3, lean_apply_4, lean_apply_5,
    lean_apply_6, lean_apply_7, lean_apply_8, lean_apply_m, lean_closure_get,
    lean_closure_object_size, lean_closure_set, LeanClosure, LeanFn1, LeanFn2, LeanFn3, LeanFn4,
    LeanFn5, LeanFn6, LeanFn7, LeanFn8,
};

pub use ctor::{
    lean_alloc_ctor, lean_ctor_get, lean_ctor_get_uint16, lean_ctor_get_uint32,
    lean_ctor_get_uint64, lean_ctor_get_uint8, lean_ctor_get_usize, lean_ctor_release,
    lean_ctor_set, lean_ctor_set_tag, lean_ctor_set_uint16, lean_ctor_set_uint32,
    lean_ctor_set_uint64, lean_ctor_set_uint8, lean_ctor_set_usize, lean_obj_tag,
};

pub use float::{
    acos,
    acosf,
    acosh,
    acoshf,
    asin,
    asinf,
    asinh,
    asinhf,
    atan,
    atan2,
    atan2f,
    atanf,
    atanh,
    atanhf,
    cbrt,
    cbrtf,
    ceil,
    ceilf,
    cos,
    cosf,
    cosh,
    coshf,
    exp,
    exp2,
    exp2f,
    expf,
    fabs,
    fabsf,
    floor,
    floorf,
    fmod,
    fmodf,
    // Float64 boxing/unboxing and ctor access
    lean_box_float,
    // Float32 boxing/unboxing and ctor access
    lean_box_float32,
    lean_ctor_get_float,
    lean_ctor_get_float32,
    lean_ctor_set_float,
    lean_ctor_set_float32,
    // Float32 arithmetic
    lean_float32_add,
    // Float32 comparison
    lean_float32_beq,
    lean_float32_decLe,
    lean_float32_decLt,
    lean_float32_div,
    // Float32 frexp/scaleb
    lean_float32_frexp,
    lean_float32_isfinite,
    lean_float32_isinf,
    // Float32 classification
    lean_float32_isnan,
    lean_float32_mul,
    lean_float32_negate,
    // Float32 bit conversion
    lean_float32_of_bits,
    lean_float32_scaleb,
    lean_float32_sub,
    lean_float32_to_bits,
    lean_float32_to_float,
    lean_float32_to_int16,
    lean_float32_to_int32,
    lean_float32_to_int64,
    // Float32 to signed int
    lean_float32_to_int8,
    lean_float32_to_isize,
    // Float32 string conversion
    lean_float32_to_string,
    lean_float32_to_uint16,
    lean_float32_to_uint32,
    lean_float32_to_uint64,
    // Float32 to unsigned int
    lean_float32_to_uint8,
    lean_float32_to_usize,
    // Float64 arithmetic
    lean_float_add,
    // Float64 comparison
    lean_float_beq,
    lean_float_decLe,
    lean_float_decLt,
    lean_float_div,
    // Float64 frexp/scaleb
    lean_float_frexp,
    lean_float_isfinite,
    lean_float_isinf,
    // Float64 classification
    lean_float_isnan,
    lean_float_mul,
    lean_float_negate,
    // Float64 bit conversion
    lean_float_of_bits,
    lean_float_scaleb,
    lean_float_sub,
    lean_float_to_bits,
    // Cross-conversion
    lean_float_to_float32,
    lean_float_to_int16,
    lean_float_to_int32,
    lean_float_to_int64,
    // Float64 to signed int
    lean_float_to_int8,
    lean_float_to_isize,
    // Float64 string conversion
    lean_float_to_string,
    lean_float_to_uint16,
    lean_float_to_uint32,
    lean_float_to_uint64,
    // Float64 to unsigned int
    lean_float_to_uint8,
    lean_float_to_usize,
    lean_int16_to_float,
    lean_int16_to_float32,
    lean_int32_to_float,
    lean_int32_to_float32,
    lean_int64_to_float,
    lean_int64_to_float32,
    // Signed int to Float64
    lean_int8_to_float,
    // Signed int to Float32
    lean_int8_to_float32,
    lean_isize_to_float,
    lean_isize_to_float32,
    lean_uint16_to_float,
    lean_uint16_to_float32,
    lean_uint32_to_float,
    lean_uint32_to_float32,
    lean_uint64_to_float,
    lean_uint64_to_float32,
    // Unsigned int to Float64
    lean_uint8_to_float,
    // Unsigned int to Float32
    lean_uint8_to_float32,
    lean_unbox_float,
    lean_unbox_float32,
    lean_usize_to_float,
    lean_usize_to_float32,
    log,
    log10,
    log10f,
    log2,
    log2f,
    logf,
    pow,
    powf,
    round,
    roundf,
    // C math stubs (f64)
    sin,
    // C math stubs (f32)
    sinf,
    sinh,
    sinhf,
    sqrt,
    sqrtf,
    tan,
    tanf,
    tanh,
    tanhf,
};

pub use floatarray::{
    lean_copy_float_array, lean_float_array_data, lean_float_array_fget, lean_float_array_fset,
    lean_float_array_get, lean_float_array_mk, lean_float_array_push, lean_float_array_set,
    lean_float_array_size, lean_float_array_uget, lean_float_array_uset, lean_mk_empty_float_array,
};

pub use io::{
    lean_chmod,
    lean_decode_io_error,
    lean_get_set_stderr,
    lean_get_set_stdin,
    lean_get_set_stdout,
    lean_get_stderr,
    // Handles
    lean_get_stdin,
    lean_get_stdout,
    lean_io_allocprof,
    lean_io_app_path,
    lean_io_as_task,
    lean_io_bind,
    lean_io_bind_task,
    lean_io_cancel_core,
    lean_io_check_canceled_core,
    lean_io_create_dir,
    lean_io_create_tempdir,
    lean_io_create_tempfile,
    lean_io_current_dir,
    lean_io_exit,
    // Heartbeats
    lean_io_get_num_heartbeats,
    // Random
    lean_io_get_random_bytes,
    lean_io_get_task_state_core,
    // Thread ID
    lean_io_get_tid,
    lean_io_getenv,
    lean_io_hard_link,
    lean_io_map_task,
    lean_io_metadata,
    // Time
    lean_io_mono_ms_now,
    lean_io_mono_nanos_now,
    lean_io_prim_eprintln,
    lean_io_prim_handle_flush,
    lean_io_prim_handle_get_line,
    lean_io_prim_handle_is_tty,
    lean_io_prim_handle_lock,
    // File handle operations
    lean_io_prim_handle_mk,
    lean_io_prim_handle_put_str,
    lean_io_prim_handle_read,
    lean_io_prim_handle_rewind,
    lean_io_prim_handle_truncate,
    lean_io_prim_handle_try_lock,
    lean_io_prim_handle_unlock,
    lean_io_prim_handle_write,
    lean_io_prim_print,
    lean_io_prim_println,
    lean_io_process_child_kill,
    lean_io_process_child_pid,
    lean_io_process_child_take_stdin,
    lean_io_process_child_try_wait,
    lean_io_process_child_wait,
    lean_io_process_get_current_dir,
    lean_io_process_get_pid,
    lean_io_process_set_current_dir,
    // Process stubs
    lean_io_process_spawn,
    // Promise stubs
    lean_io_promise_new,
    lean_io_promise_resolve,
    lean_io_promise_result_opt,
    lean_io_pure,
    // Directory and metadata
    lean_io_read_dir,
    // Filesystem operations
    lean_io_realpath,
    lean_io_remove_dir,
    lean_io_remove_file,
    lean_io_rename,
    lean_io_result_get_value,
    lean_io_result_is_ok,
    lean_io_result_mk_error,
    lean_io_result_mk_ok,
    lean_io_set_heartbeats,
    lean_io_symlink_metadata,
    // Timeit / allocprof
    lean_io_timeit,
    lean_io_wait_any_core,
    lean_mk_io_error_already_exists,
    lean_mk_io_error_already_exists_file,
    lean_mk_io_error_eof,
    lean_mk_io_error_hardware_fault,
    lean_mk_io_error_illegal_operation,
    lean_mk_io_error_inappropriate_type,
    lean_mk_io_error_inappropriate_type_file,
    lean_mk_io_error_interrupted,
    lean_mk_io_error_invalid_argument,
    lean_mk_io_error_invalid_argument_file,
    lean_mk_io_error_no_file_or_directory,
    lean_mk_io_error_no_such_thing,
    lean_mk_io_error_no_such_thing_file,
    lean_mk_io_error_other_error,
    lean_mk_io_error_permission_denied,
    lean_mk_io_error_permission_denied_file,
    lean_mk_io_error_protocol_error,
    lean_mk_io_error_resource_busy,
    lean_mk_io_error_resource_exhausted,
    lean_mk_io_error_resource_exhausted_file,
    lean_mk_io_error_resource_vanished,
    lean_mk_io_error_time_expired,
    lean_mk_io_error_unsatisfied_constraints,
    lean_mk_io_error_unsupported_operation,
    // IO error constructors
    lean_mk_io_user_error,
    lean_option_get_or_block,
    // ShareCommon
    lean_sharecommon_eq,
    lean_sharecommon_hash,
    lean_sharecommon_quick,
    lean_state_sharecommon,
    lean_task_bind,
    lean_task_get,
    lean_task_get_own,
    lean_task_map,
    lean_task_pure,
    // Task stubs
    lean_task_spawn,
};

pub use misc::{
    lean_dbg_trace, lean_dbg_trace_if_shared, lean_name_eq, lean_sorry, lean_system_platform_nbits,
};

pub use nat::{
    l_Nat_decEq___boxed, lean_big_uint64_to_nat, lean_big_usize_to_nat, lean_nat_add,
    lean_nat_dec_eq, lean_nat_dec_le, lean_nat_dec_lt, lean_nat_div, lean_nat_eq, lean_nat_land,
    lean_nat_log2, lean_nat_lor, lean_nat_lxor, lean_nat_mod, lean_nat_mul, lean_nat_pow,
    lean_nat_pred, lean_nat_shiftl, lean_nat_shiftr, lean_nat_sub, lean_uint64_to_nat,
    lean_unsigned_to_nat, lean_usize_to_nat,
};

pub use owned::LeanOwnedValue;

pub use object::{
    lean_alloc_object, lean_ctor_object_size, lean_free_object, LeanObject, LEAN_ARRAY_TAG,
    LEAN_BIGINT_TAG, LEAN_CLOSURE_TAG, LEAN_EXTERNAL_TAG, LEAN_MAX_CTOR_TAG, LEAN_MPZ_TAG,
    LEAN_OBJECT_SIZE, LEAN_REF_TAG, LEAN_SCALAR_ARRAY_TAG, LEAN_STRING_TAG, LEAN_THUNK_TAG,
};

pub use panic::{
    lean_internal_panic, lean_internal_panic_out_of_memory, lean_internal_panic_unreachable,
    lean_panic_fn,
};

pub use rc::{
    lean_dec, lean_dec_ref, lean_free_object_full, lean_free_object_only, lean_inc, lean_inc_n,
    lean_inc_ref, lean_inc_ref_n, lean_is_exclusive, lean_mark_persistent,
};

pub use sarray::{
    lean_alloc_sarray, lean_byte_array_data, lean_byte_array_fget, lean_byte_array_fset,
    lean_byte_array_get, lean_byte_array_mk, lean_byte_array_push, lean_byte_array_set,
    lean_byte_array_size, lean_byte_array_uget, lean_byte_array_uset, lean_copy_byte_array,
    lean_mk_empty_byte_array, lean_sarray_data, lean_string_validate_utf8, LeanSArray,
};

pub use string::{
    lean_cstr_to_nat, lean_mk_string, lean_mk_string_unchecked, lean_nat_to_string,
    lean_slice_dec_lt, lean_slice_hash, lean_string_append, lean_string_byte_len, lean_string_cstr,
    lean_string_data, lean_string_dec_eq, lean_string_dec_lt, lean_string_eq,
    lean_string_from_utf8_unchecked, lean_string_get_byte_fast, lean_string_hash,
    lean_string_is_valid_pos, lean_string_length, lean_string_lt, lean_string_memcmp,
    lean_string_mk, lean_string_object_size, lean_string_of_usize, lean_string_push,
    lean_string_to_str, lean_string_to_utf8, lean_string_utf8_at_end, lean_string_utf8_byte_size,
    lean_string_utf8_extract, lean_string_utf8_get, lean_string_utf8_get_bang,
    lean_string_utf8_get_fast, lean_string_utf8_get_fast_cold, lean_string_utf8_get_opt,
    lean_string_utf8_len, lean_string_utf8_next, lean_string_utf8_next_fast,
    lean_string_utf8_next_fast_cold, lean_string_utf8_prev, lean_string_utf8_set, LeanString,
};

pub use sint::{
    lean_bool_to_int16, lean_bool_to_int32, lean_bool_to_int64, lean_bool_to_int8,
    lean_bool_to_isize, lean_bool_to_uint16, lean_bool_to_uint32, lean_bool_to_uint64,
    lean_bool_to_uint8, lean_bool_to_usize, lean_int16_abs, lean_int16_add, lean_int16_complement,
    lean_int16_dec_eq, lean_int16_dec_le, lean_int16_dec_lt, lean_int16_div, lean_int16_land,
    lean_int16_lor, lean_int16_mod, lean_int16_mul, lean_int16_neg, lean_int16_of_int,
    lean_int16_of_nat, lean_int16_shift_left, lean_int16_shift_right, lean_int16_sub,
    lean_int16_to_int, lean_int16_to_int32, lean_int16_to_int64, lean_int16_to_int8,
    lean_int16_to_isize, lean_int16_xor, lean_int32_abs, lean_int32_add, lean_int32_complement,
    lean_int32_dec_eq, lean_int32_dec_le, lean_int32_dec_lt, lean_int32_div, lean_int32_land,
    lean_int32_lor, lean_int32_mod, lean_int32_mul, lean_int32_neg, lean_int32_of_int,
    lean_int32_of_nat, lean_int32_shift_left, lean_int32_shift_right, lean_int32_sub,
    lean_int32_to_int, lean_int32_to_int16, lean_int32_to_int64, lean_int32_to_int8,
    lean_int32_to_isize, lean_int32_xor, lean_int64_abs, lean_int64_add, lean_int64_complement,
    lean_int64_dec_eq, lean_int64_dec_le, lean_int64_dec_lt, lean_int64_div, lean_int64_land,
    lean_int64_lor, lean_int64_mod, lean_int64_mul, lean_int64_neg, lean_int64_of_int,
    lean_int64_of_nat, lean_int64_shift_left, lean_int64_shift_right, lean_int64_sub,
    lean_int64_to_int16, lean_int64_to_int32, lean_int64_to_int8, lean_int64_to_isize,
    lean_int64_xor, lean_int8_abs, lean_int8_add, lean_int8_complement, lean_int8_dec_eq,
    lean_int8_dec_le, lean_int8_dec_lt, lean_int8_div, lean_int8_land, lean_int8_lor,
    lean_int8_mod, lean_int8_mul, lean_int8_neg, lean_int8_of_int, lean_int8_of_nat,
    lean_int8_shift_left, lean_int8_shift_right, lean_int8_sub, lean_int8_to_int,
    lean_int8_to_int16, lean_int8_to_int32, lean_int8_to_int64, lean_int8_to_isize, lean_int8_xor,
    lean_isize_abs, lean_isize_add, lean_isize_complement, lean_isize_dec_eq, lean_isize_dec_le,
    lean_isize_dec_lt, lean_isize_div, lean_isize_land, lean_isize_lor, lean_isize_mod,
    lean_isize_mul, lean_isize_neg, lean_isize_of_int, lean_isize_of_nat, lean_isize_shift_left,
    lean_isize_shift_right, lean_isize_sub, lean_isize_to_int, lean_isize_to_int16,
    lean_isize_to_int32, lean_isize_to_int64, lean_isize_to_int8, lean_isize_xor,
};

pub use uint::{
    lean_uint16_add, lean_uint16_complement, lean_uint16_dec_eq, lean_uint16_dec_le,
    lean_uint16_dec_lt, lean_uint16_div, lean_uint16_land, lean_uint16_log2, lean_uint16_lor,
    lean_uint16_lxor, lean_uint16_mod, lean_uint16_mul, lean_uint16_neg, lean_uint16_of_nat,
    lean_uint16_of_nat_mk, lean_uint16_shift_left, lean_uint16_shift_right, lean_uint16_sub,
    lean_uint16_to_nat, lean_uint16_to_uint32, lean_uint16_to_uint64, lean_uint16_to_uint8,
    lean_uint16_to_usize, lean_uint16_xor, lean_uint32_add, lean_uint32_complement,
    lean_uint32_dec_le, lean_uint32_div, lean_uint32_land, lean_uint32_log2, lean_uint32_lor,
    lean_uint32_lxor, lean_uint32_mod, lean_uint32_mul, lean_uint32_neg, lean_uint32_of_nat,
    lean_uint32_of_nat_mk, lean_uint32_shift_left, lean_uint32_shift_right, lean_uint32_sub,
    lean_uint32_to_nat, lean_uint32_to_uint16, lean_uint32_to_uint64, lean_uint32_to_uint8,
    lean_uint32_to_usize, lean_uint32_xor, lean_uint64_add, lean_uint64_complement,
    lean_uint64_dec_eq, lean_uint64_dec_le, lean_uint64_dec_lt, lean_uint64_div, lean_uint64_land,
    lean_uint64_log2, lean_uint64_lor, lean_uint64_lxor, lean_uint64_mix_hash, lean_uint64_mod,
    lean_uint64_mul, lean_uint64_neg, lean_uint64_of_nat, lean_uint64_of_nat_mk,
    lean_uint64_shift_left, lean_uint64_shift_right, lean_uint64_sub, lean_uint64_to_uint16,
    lean_uint64_to_uint32, lean_uint64_to_uint8, lean_uint64_to_usize, lean_uint64_xor,
    lean_uint8_add, lean_uint8_complement, lean_uint8_dec_eq, lean_uint8_dec_le, lean_uint8_dec_lt,
    lean_uint8_div, lean_uint8_land, lean_uint8_log2, lean_uint8_lor, lean_uint8_lxor,
    lean_uint8_mod, lean_uint8_mul, lean_uint8_neg, lean_uint8_of_nat, lean_uint8_of_nat_mk,
    lean_uint8_shift_left, lean_uint8_shift_right, lean_uint8_sub, lean_uint8_to_nat,
    lean_uint8_to_uint16, lean_uint8_to_uint32, lean_uint8_to_uint64, lean_uint8_to_usize,
    lean_uint8_xor, lean_usize_add, lean_usize_complement, lean_usize_dec_eq, lean_usize_dec_le,
    lean_usize_dec_lt, lean_usize_div, lean_usize_land, lean_usize_log2, lean_usize_lor,
    lean_usize_lxor, lean_usize_mix_hash, lean_usize_mod, lean_usize_mul, lean_usize_neg,
    lean_usize_of_nat, lean_usize_of_nat_mk, lean_usize_shift_left, lean_usize_shift_right,
    lean_usize_sub, lean_usize_to_uint16, lean_usize_to_uint32, lean_usize_to_uint64,
    lean_usize_to_uint8, lean_usize_xor,
};

pub use thunk::{
    lean_mk_thunk, lean_thunk_get, lean_thunk_get_core, lean_thunk_get_own, lean_thunk_pure,
    LeanThunk,
};

pub use stref::{
    lean_st_mk_ref, lean_st_ref_get, lean_st_ref_ptr_eq, lean_st_ref_reset, lean_st_ref_set,
    lean_st_ref_swap, lean_st_ref_take, LeanRef,
};

pub use external::{
    lean_alloc_external, lean_get_external_class, lean_get_external_data, lean_set_external_data,
    LeanExternal, LeanExternalClass, LeanExternalFinalize, LeanExternalForeach,
};

pub use platform::{
    lean_dbg_sleep, lean_expr_data, lean_finalize_task_manager, lean_get_githash,
    lean_get_max_ctor_fields, lean_get_max_ctor_scalars_size, lean_get_max_ctor_tag,
    lean_get_usize_size, lean_hashmap_mk_idx, lean_hashset_mk_idx, lean_init_task_manager,
    lean_initialize_runtime_module, lean_internal_has_llvm_backend, lean_internal_is_stage0,
    lean_io_initializing, lean_io_mark_end_initialization, lean_io_result_show_error,
    lean_is_exclusive_obj, lean_manual_get_root, lean_name_hash, lean_nat_div_exact, lean_nat_gcd,
    lean_ptr_addr, lean_runtime_forget, lean_runtime_mark_multi_threaded,
    lean_runtime_mark_persistent, lean_set_panic_messages, lean_strict_and, lean_strict_or,
    lean_system_platform_target, lean_version_get_is_release, lean_version_get_major,
    lean_version_get_minor, lean_version_get_patch, lean_version_get_special_desc, lean_void_mk,
};

// Convenience functions

#[inline(always)]
pub fn lean_unit() -> *mut LeanObject {
    lean_box(0)
}

#[inline(always)]
pub fn lean_bool(b: bool) -> *mut LeanObject {
    lean_box(b as usize)
}

#[inline(always)]
pub fn lean_unbox_bool(o: *mut LeanObject) -> bool {
    lean_unbox(o) != 0
}
