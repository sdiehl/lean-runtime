//! Task/Thread concurrency operations

use super::{io_result_ok_val, ExternFn, Result};
use crate::value::LeanValue;
use lean_runtime::*;

pub fn get_builtins() -> Vec<(&'static str, ExternFn)> {
    vec![
        // Task operations
        ("lean_task_spawn", task_spawn as ExternFn),
        ("lean_task_pure", task_pure),
        ("lean_task_get_own", task_get_own),
        ("lean_task_bind", task_bind),
        ("lean_task_map", task_map),
        // Promise operations
        ("lean_io_promise_new", promise_new),
        ("lean_io_promise_resolve", promise_resolve),
        ("lean_io_promise_result", promise_result),
        // Mutex operations (stubs)
        ("lean_io_mutex_new", mutex_new),
        ("lean_io_mutex_lock", mutex_lock),
        ("lean_io_mutex_unlock", mutex_unlock),
        // Condition variable (stubs)
        ("lean_io_condvar_new", condvar_new),
        ("lean_io_condvar_wait", condvar_wait),
        ("lean_io_condvar_notify_one", condvar_notify_one),
        ("lean_io_condvar_notify_all", condvar_notify_all),
        // Channel operations (stubs)
        ("lean_io_channel_new", channel_new),
        ("lean_io_channel_send", channel_send),
        ("lean_io_channel_recv", channel_recv),
    ]
}

// Task: represented as a ctor with tag 1 (completed) holding the value
// or tag 0 (pending) holding a thunk

fn task_spawn(args: &[LeanValue]) -> Result<LeanValue> {
    // For the bytecode VM, we don't actually spawn threads
    // Instead, we immediately evaluate the thunk and return a completed task
    // The thunk is a closure that should be called
    unsafe {
        // args[0] is priority (ignored), args[1] is the thunk
        let thunk = if args.len() > 1 {
            args[1].as_ptr()
        } else {
            args[0].as_ptr()
        };

        // For now, just wrap the thunk as a "pending" task
        // Real implementation would evaluate it
        lean_inc(thunk);
        let task = lean_alloc_ctor(0, 1, 0); // tag 0 = pending
        lean_ctor_set(task, 0, thunk);
        Ok(io_result_ok_val(LeanValue::from_raw(task)))
    }
}

fn task_pure(args: &[LeanValue]) -> Result<LeanValue> {
    // Create a completed task with the given value
    unsafe {
        lean_inc(args[0].as_ptr());
        let task = lean_alloc_ctor(1, 1, 0); // tag 1 = completed
        lean_ctor_set(task, 0, args[0].as_ptr());
        Ok(LeanValue::from_raw(task))
    }
}

fn task_get_own(args: &[LeanValue]) -> Result<LeanValue> {
    // Get the value from a task (blocks if pending)
    unsafe {
        let task = args[0].as_ptr();
        let tag = lean_obj_tag(task);
        if tag == 1 {
            // Completed - return the value
            let val = lean_ctor_get(task, 0);
            lean_inc(val);
            Ok(LeanValue::from_raw(val))
        } else {
            // Pending - for the VM, we'd need to evaluate the thunk
            // For now, return unit
            Ok(LeanValue::unit())
        }
    }
}

fn task_bind(args: &[LeanValue]) -> Result<LeanValue> {
    // Bind operation on tasks (stub)
    let _ = args;
    task_pure(&[LeanValue::unit()])
}

fn task_map(args: &[LeanValue]) -> Result<LeanValue> {
    // Map a function over a task (stub)
    let _ = args;
    task_pure(&[LeanValue::unit()])
}

// Promise: like a one-shot channel or future
fn promise_new(_args: &[LeanValue]) -> Result<LeanValue> {
    // Create a new unfulfilled promise
    // Represented as ctor with tag 0 (unfulfilled) and no value
    unsafe {
        let promise = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(promise, 0, lean_box(0)); // placeholder
        Ok(io_result_ok_val(LeanValue::from_raw(promise)))
    }
}

fn promise_resolve(args: &[LeanValue]) -> Result<LeanValue> {
    // Resolve a promise with a value
    unsafe {
        let promise = args[0].as_ptr();
        let val = args[1].as_ptr();
        lean_inc(val);
        lean_ctor_set(promise, 0, val);
        // Change tag to 1 (fulfilled) - we can't actually change tag, so this is approximate
        Ok(io_result_ok_val(LeanValue::unit()))
    }
}

fn promise_result(args: &[LeanValue]) -> Result<LeanValue> {
    // Get the promise's associated task/result
    unsafe {
        let promise = args[0].as_ptr();
        let val = lean_ctor_get(promise, 0);
        lean_inc(val);
        Ok(io_result_ok_val(LeanValue::from_raw(val)))
    }
}

// Mutex stubs
fn mutex_new(_args: &[LeanValue]) -> Result<LeanValue> {
    // Create a new mutex (stub - single-threaded VM doesn't need real mutex)
    unsafe {
        let mutex = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(mutex, 0, lean_box(0)); // unlocked
        Ok(io_result_ok_val(LeanValue::from_raw(mutex)))
    }
}

fn mutex_lock(args: &[LeanValue]) -> Result<LeanValue> {
    // Lock a mutex (stub - always succeeds)
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn mutex_unlock(args: &[LeanValue]) -> Result<LeanValue> {
    // Unlock a mutex (stub)
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}

// Condition variable stubs
fn condvar_new(_args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let cv = lean_alloc_ctor(0, 0, 0);
        Ok(io_result_ok_val(LeanValue::from_raw(cv)))
    }
}

fn condvar_wait(args: &[LeanValue]) -> Result<LeanValue> {
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn condvar_notify_one(args: &[LeanValue]) -> Result<LeanValue> {
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn condvar_notify_all(args: &[LeanValue]) -> Result<LeanValue> {
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}

// Channel stubs
fn channel_new(_args: &[LeanValue]) -> Result<LeanValue> {
    unsafe {
        let ch = lean_alloc_ctor(0, 1, 0);
        lean_ctor_set(ch, 0, lean_box(0));
        Ok(io_result_ok_val(LeanValue::from_raw(ch)))
    }
}

fn channel_send(args: &[LeanValue]) -> Result<LeanValue> {
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}

fn channel_recv(args: &[LeanValue]) -> Result<LeanValue> {
    let _ = args;
    Ok(io_result_ok_val(LeanValue::unit()))
}
