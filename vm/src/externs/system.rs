//! System operations (platform, exit, time, random, process)

use super::{io_result_error, io_result_ok_val, ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Platform information
        (
            "lean_system_platform_nbits",
            system_platform_nbits as ExternFn,
        ),
        ("lean_system_platform_target", system_platform_target),
        ("lean_system_platform_windows", system_platform_windows),
        ("lean_system_platform_osx", system_platform_osx),
        (
            "lean_system_platform_emscripten",
            system_platform_emscripten,
        ),
        // Version information
        ("lean_version_get_major", version_get_major),
        ("lean_version_get_minor", version_get_minor),
        ("lean_version_get_patch", version_get_patch),
        ("lean_version_get_is_release", version_get_is_release),
        ("lean_version_get_special_desc", version_get_special_desc),
        // Note: l_System_Platform_numBits comes from Init bytecode
        ("lean_io_exit", io_exit),
        ("lean_io_get_num_heartbeats", io_get_num_heartbeats),
        ("lean_io_mono_nanos_now", io_mono_nanos_now),
        ("lean_io_get_random_bytes", io_get_random_bytes),
        // Command line
        ("lean_io_get_args", io_get_args),
        // Process operations
        ("lean_io_process_spawn", io_process_spawn),
        ("lean_io_process_wait", io_process_wait),
        ("lean_io_process_child_take_stdin", io_process_take_stdin),
        // Timing
        ("lean_io_getenv_arg", io_getenv_arg),
        ("lean_io_sleep", io_sleep),
        ("lean_get_current_time", get_current_time),
    ]
}

fn system_platform_nbits(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_small_nat(std::mem::size_of::<usize>() * 8))
}

fn system_platform_target(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return target triple as string
    #[cfg(target_os = "macos")]
    let target = "aarch64-apple-darwin";
    #[cfg(target_os = "linux")]
    let target = "x86_64-unknown-linux-gnu";
    #[cfg(target_os = "windows")]
    let target = "x86_64-pc-windows-msvc";
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let target = "unknown";
    Ok(LeanValue::from_string(target))
}

fn system_platform_windows(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(cfg!(target_os = "windows")))
}

fn system_platform_osx(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(cfg!(target_os = "macos")))
}

fn system_platform_emscripten(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(cfg!(target_os = "emscripten")))
}

fn version_get_major(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return Lean 4 version major (currently 4)
    Ok(LeanValue::from_small_nat(4))
}

fn version_get_minor(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return Lean 4 version minor
    Ok(LeanValue::from_small_nat(0))
}

fn version_get_patch(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return Lean 4 version patch
    Ok(LeanValue::from_small_nat(0))
}

fn version_get_is_release(_args: &[LeanValue]) -> Result<LeanValue> {
    Ok(LeanValue::from_bool(false))
}

fn version_get_special_desc(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return special description (empty for release, "nightly" for dev builds)
    Ok(LeanValue::from_string("bytecode-vm"))
}

fn io_exit(args: &[LeanValue]) -> Result<LeanValue> {
    let code = lean_unbox(args[0].as_ptr()) as i32;
    std::process::exit(code);
}

fn io_get_num_heartbeats(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return 0 for now (used for profiling)
    Ok(io_result_ok_val(LeanValue::from_small_nat(0)))
}

fn io_mono_nanos_now(_args: &[LeanValue]) -> Result<LeanValue> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    unsafe {
        Ok(io_result_ok_val(LeanValue::from_raw(lean_box(
            nanos as usize,
        ))))
    }
}

fn io_get_random_bytes(args: &[LeanValue]) -> Result<LeanValue> {
    let n = lean_unbox(args[0].as_ptr());
    unsafe {
        let arr = lean_mk_empty_byte_array(lean_box(n));
        // Fill with pseudo-random bytes
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);

        for _ in 0..n {
            // Simple LCG
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let byte = (seed >> 32) as u8;
            lean_byte_array_push(arr, byte);
        }
        Ok(io_result_ok_val(LeanValue::from_raw(arr)))
    }
}

fn io_get_args(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return command line arguments as Array String
    unsafe {
        let args: Vec<String> = std::env::args().collect();
        let arr = lean_mk_empty_array_with_capacity(lean_box(args.len()));
        let mut arr = arr;
        for arg in args {
            let s = LeanValue::from_string(&arg).into_raw();
            arr = lean_array_push(arr, s);
        }
        Ok(io_result_ok_val(LeanValue::from_raw(arr)))
    }
}

fn io_process_spawn(args: &[LeanValue]) -> Result<LeanValue> {
    // Spawn a child process
    // Args: (config : Process.SpawnArgs, world)
    // SpawnArgs contains: cmd, args, cwd, env, stdin, stdout, stderr
    unsafe {
        // For simplicity, extract cmd as first string field
        let config = args[0].as_ptr();
        if lean_is_scalar(config) {
            return Ok(io_result_error("process_spawn: invalid config"));
        }

        // Extract command (field 0)
        let cmd_obj = lean_ctor_get(config, 0);
        let cmd_cstr = lean_string_cstr(cmd_obj);
        let cmd_len = lean_string_byte_len(cmd_obj);
        let cmd = std::str::from_utf8(std::slice::from_raw_parts(cmd_cstr, cmd_len)).unwrap_or("");

        // Extract args array (field 1)
        let args_arr = lean_ctor_get(config, 1);
        let args_size = lean_unbox(lean_array_get_size(args_arr));

        let mut cmd_args: Vec<String> = Vec::new();
        for i in 0..args_size {
            let arg_obj = lean_array_uget(args_arr, i);
            let arg_cstr = lean_string_cstr(arg_obj);
            let arg_len = lean_string_byte_len(arg_obj);
            if let Ok(arg) = std::str::from_utf8(std::slice::from_raw_parts(arg_cstr, arg_len)) {
                cmd_args.push(arg.to_string());
            }
        }

        match std::process::Command::new(cmd).args(&cmd_args).spawn() {
            Ok(child) => {
                // Return child process handle (store pid as boxed value)
                let pid = child.id();
                // Store the child in a global map for later wait
                // For now, just return the pid as a handle
                let handle = lean_alloc_ctor(0, 1, 0);
                lean_ctor_set(handle, 0, lean_box(pid as usize));
                Ok(io_result_ok_val(LeanValue::from_raw(handle)))
            }
            Err(e) => Ok(io_result_error(&format!("process_spawn: {}", e))),
        }
    }
}

fn io_process_wait(args: &[LeanValue]) -> Result<LeanValue> {
    // Wait for a child process (stub - would need process handle storage)
    let _ = args;
    // Return exit code 0 as stub
    Ok(io_result_ok_val(LeanValue::from_small_nat(0)))
}

fn io_process_take_stdin(_args: &[LeanValue]) -> Result<LeanValue> {
    // Take stdin handle from child process (stub)
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn io_getenv_arg(_args: &[LeanValue]) -> Result<LeanValue> {
    // Get specific env variable for Lean (stub)
    unsafe { Ok(io_result_ok_val(LeanValue::from_raw(lean_box(0)))) }
}

fn io_sleep(args: &[LeanValue]) -> Result<LeanValue> {
    // Sleep for specified milliseconds
    let ms = lean_unbox(args[0].as_ptr()) as u64;
    std::thread::sleep(std::time::Duration::from_millis(ms));
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn get_current_time(_args: &[LeanValue]) -> Result<LeanValue> {
    // Return current UNIX timestamp as a pair (seconds, nanoseconds)
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();

    unsafe {
        // Create a pair (secs, nanos)
        let pair = lean_alloc_ctor(0, 2, 0);
        lean_ctor_set(pair, 0, lean_box(secs as usize));
        lean_ctor_set(pair, 1, lean_box(nanos as usize));
        Ok(io_result_ok_val(LeanValue::from_raw(pair)))
    }
}
