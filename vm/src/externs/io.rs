//! IO operations (file I/O, stdin/stdout, environment)

use super::{io_result_error, io_result_ok_val, ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Basic IO
        ("lean_io_println", io_println as ExternFn),
        ("lean_io_prim_println", io_println),
        ("lean_get_stdout", get_stdout),
        ("lean_io_result_mk_ok", io_result_mk_ok),
        // File I/O
        ("lean_io_prim_read_text_file", io_read_text_file),
        ("lean_io_prim_write_text_file", io_write_text_file),
        ("lean_io_prim_handle_mk", io_handle_mk),
        ("lean_io_prim_handle_is_eof", io_handle_is_eof),
        ("lean_io_prim_handle_flush", io_handle_flush),
        ("lean_io_prim_handle_get_line", io_handle_get_line),
        ("lean_io_prim_handle_put_str", io_handle_put_str),
        ("lean_io_prim_handle_read_bytes", io_handle_read_bytes),
        ("lean_io_prim_handle_write", io_handle_write),
        ("lean_io_stdin", io_stdin),
        ("lean_io_stderr", io_stderr),
        // Environment
        ("lean_io_getenv", io_getenv),
        ("lean_io_setenv", io_setenv),
        ("lean_io_remove_file", io_remove_file),
        ("lean_io_current_dir", io_current_dir),
        ("lean_io_app_path", io_app_path),
        ("lean_io_realpath", io_realpath),
        ("lean_io_is_dir", io_is_dir),
        ("lean_io_file_exists", io_file_exists),
        ("lean_io_create_dir", io_create_dir),
        ("lean_io_read_dir", io_read_dir),
    ]
}

fn io_println(args: &[LeanValue]) -> Result<LeanValue> {
    let s = &args[0];
    if s.is_scalar() {
        return Err(crate::VMError::TypeMismatch("println expects string"));
    }
    unsafe {
        let cstr = lean_string_cstr(s.as_ptr());
        let len = lean_string_byte_len(s.as_ptr());
        let slice = std::slice::from_raw_parts(cstr, len);
        if let Ok(string) = std::str::from_utf8(slice) {
            println!("{}", string);
        }
    }
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn get_stdout(_args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let stdout = lean_get_stdout();
        Ok(LeanValue::from_raw(stdout))
    }
}

fn io_result_mk_ok(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let result = lean_io_result_mk_ok(args[0].as_ptr());
        Ok(LeanValue::from_raw(result))
    }
}

fn io_read_text_file(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let fname_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(fname_ptr);
        let len = lean_string_byte_len(fname_ptr);
        let fname = std::str::from_utf8(std::slice::from_raw_parts(cstr, len))
            .map_err(|_| crate::VMError::TypeMismatch("invalid utf8 filename"))?;

        match std::fs::read_to_string(fname) {
            Ok(content) => {
                let result = LeanValue::from_string(&content);
                Ok(io_result_ok_val(result))
            }
            Err(e) => Ok(io_result_error(&format!("read_text_file: {}", e))),
        }
    }
}

fn io_write_text_file(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let fname_ptr = args[0].as_ptr();
        let content_ptr = args[1].as_ptr();

        let fname_cstr = lean_string_cstr(fname_ptr);
        let fname_len = lean_string_byte_len(fname_ptr);
        let fname = std::str::from_utf8(std::slice::from_raw_parts(fname_cstr, fname_len))
            .map_err(|_| crate::VMError::TypeMismatch("invalid utf8 filename"))?;

        let content_cstr = lean_string_cstr(content_ptr);
        let content_len = lean_string_byte_len(content_ptr);
        let content = std::str::from_utf8(std::slice::from_raw_parts(content_cstr, content_len))
            .map_err(|_| crate::VMError::TypeMismatch("invalid utf8 content"))?;

        match std::fs::write(fname, content) {
            Ok(()) => Ok(io_result_ok_val(LeanValue::unit())),
            Err(e) => Ok(io_result_error(&format!("write_text_file: {}", e))),
        }
    }
}

fn io_handle_mk(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn io_handle_is_eof(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(io_result_ok_val(LeanValue::from_bool(false)))
}

fn io_handle_flush(_args: &[LeanValue]) -> Result<LeanValue> {
    use std::io::Write;
    let _ = std::io::stdout().flush();
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn io_handle_get_line(_args: &[LeanValue]) -> Result<LeanValue> {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            Ok(io_result_ok_val(LeanValue::from_string(&line)))
        }
        Err(e) => Ok(io_result_error(&format!("get_line: {}", e))),
    }
}

fn io_handle_put_str(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let str_ptr = args[1].as_ptr();
        let cstr = lean_string_cstr(str_ptr);
        let len = lean_string_byte_len(str_ptr);
        let s = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");
        print!("{}", s);
        Ok(io_result_ok_val(LeanValue::unit()))
    }
}

fn io_handle_read_bytes(_args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let arr = lean_mk_empty_byte_array(lean_box(0));
        Ok(io_result_ok_val(LeanValue::from_raw(arr)))
    }
}

fn io_handle_write(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(io_result_ok_val(LeanValue::from_small_nat(0)))
}

fn io_stdin(_args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let handle = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(handle, 0, lean_box(0)); // fd = 0
        Ok(LeanValue::from_raw(handle))
    }
}

fn io_stderr(_args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let handle = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(handle, 0, lean_box(2)); // fd = 2
        Ok(LeanValue::from_raw(handle))
    }
}

fn io_getenv(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let name_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(name_ptr);
        let len = lean_string_byte_len(name_ptr);
        let name = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        match std::env::var(name) {
            Ok(val) => {
                let str_val = LeanValue::from_string(&val);
                let some = lean_alloc_ctor(1, 1, 0);
                lean_ctor_set(some, 0, str_val.into_raw());
                Ok(io_result_ok_val(LeanValue::from_raw(some)))
            }
            Err(_) => Ok(io_result_ok_val(LeanValue::from_raw(lean_box(0)))),
        }
    }
}

fn io_setenv(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let name_ptr = args[0].as_ptr();
        let val_ptr = args[1].as_ptr();

        let name_cstr = lean_string_cstr(name_ptr);
        let name_len = lean_string_byte_len(name_ptr);
        let name =
            std::str::from_utf8(std::slice::from_raw_parts(name_cstr, name_len)).unwrap_or("");

        let val_cstr = lean_string_cstr(val_ptr);
        let val_len = lean_string_byte_len(val_ptr);
        let val = std::str::from_utf8(std::slice::from_raw_parts(val_cstr, val_len)).unwrap_or("");

        std::env::set_var(name, val);
        Ok(io_result_ok_val(LeanValue::unit()))
    }
}

fn io_remove_file(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let fname_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(fname_ptr);
        let len = lean_string_byte_len(fname_ptr);
        let fname = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        match std::fs::remove_file(fname) {
            Ok(()) => Ok(io_result_ok_val(LeanValue::unit())),
            Err(e) => Ok(io_result_error(&format!("remove_file: {}", e))),
        }
    }
}

fn io_current_dir(_args: &[LeanValue]) -> Result<LeanValue> {
    match std::env::current_dir() {
        Ok(path) => {
            let s = path.to_string_lossy().to_string();
            Ok(io_result_ok_val(LeanValue::from_string(&s)))
        }
        Err(e) => Ok(io_result_error(&format!("current_dir: {}", e))),
    }
}

fn io_app_path(_args: &[LeanValue]) -> Result<LeanValue> {
    match std::env::current_exe() {
        Ok(path) => {
            let s = path.to_string_lossy().to_string();
            Ok(io_result_ok_val(LeanValue::from_string(&s)))
        }
        Err(e) => Ok(io_result_error(&format!("app_path: {}", e))),
    }
}

fn io_realpath(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let path_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(path_ptr);
        let len = lean_string_byte_len(path_ptr);
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        match std::fs::canonicalize(path_str) {
            Ok(path) => {
                let s = path.to_string_lossy().to_string();
                Ok(io_result_ok_val(LeanValue::from_string(&s)))
            }
            Err(e) => Ok(io_result_error(&format!("realpath: {}", e))),
        }
    }
}

fn io_is_dir(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let path_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(path_ptr);
        let len = lean_string_byte_len(path_ptr);
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        let is_dir = std::path::Path::new(path_str).is_dir();
        Ok(io_result_ok_val(LeanValue::from_bool(is_dir)))
    }
}

fn io_file_exists(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let path_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(path_ptr);
        let len = lean_string_byte_len(path_ptr);
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        let exists = std::path::Path::new(path_str).exists();
        Ok(io_result_ok_val(LeanValue::from_bool(exists)))
    }
}

fn io_create_dir(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let path_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(path_ptr);
        let len = lean_string_byte_len(path_ptr);
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        match std::fs::create_dir_all(path_str) {
            Ok(()) => Ok(io_result_ok_val(LeanValue::unit())),
            Err(e) => Ok(io_result_error(&format!("create_dir: {}", e))),
        }
    }
}

fn io_read_dir(args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let path_ptr = args[0].as_ptr();
        let cstr = lean_string_cstr(path_ptr);
        let len = lean_string_byte_len(path_ptr);
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(cstr, len)).unwrap_or("");

        match std::fs::read_dir(path_str) {
            Ok(entries) => {
                let arr = lean_mk_empty_array_with_capacity(lean_box(16));
                let mut arr = arr;
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_obj = LeanValue::from_string(&name).into_raw();
                    arr = lean_array_push(arr, name_obj);
                }
                Ok(io_result_ok_val(LeanValue::from_raw(arr)))
            }
            Err(e) => Ok(io_result_error(&format!("read_dir: {}", e))),
        }
    }
}
