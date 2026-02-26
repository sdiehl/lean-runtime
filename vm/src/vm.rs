//! Virtual machine for executing Lean bytecode

use crate::bytecode::{Module, Opcode};
use crate::externs::{self, ExternFn};
use crate::linker::FUNC_ID_RESOLVED_BIT;
use crate::value::{alloc_ctor, LeanValue};
use lean_runtime::*;
use smallvec::SmallVec;
use std::collections::HashMap;

/// Inline capacity for local variables (avoids heap allocation for most functions)
const LOCALS_INLINE_CAP: usize = 16;

/// Inline capacity for the value stack (avoids heap allocation for shallow stacks)
const STACK_INLINE_CAP: usize = 32;

/// Inline capacity for the call stack (avoids heap allocation for shallow call depths)
const FRAMES_INLINE_CAP: usize = 16;

/// Type alias for locals storage
type Locals = SmallVec<[LeanValue; LOCALS_INLINE_CAP]>;

/// Type alias for value stack
type Stack = SmallVec<[LeanValue; STACK_INLINE_CAP]>;

/// Type alias for call stack
type Frames = SmallVec<[Frame; FRAMES_INLINE_CAP]>;

/// VM execution error
#[derive(Debug)]
pub enum VMError {
    StackUnderflow,
    InvalidOpcode(u8),
    InvalidFunctionId(u32),
    InvalidLocalIndex(u16),
    InvalidExternId(u32),
    TypeMismatch(&'static str),
    Unreachable,
    IOError(String),
}

impl std::fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "stack underflow"),
            VMError::InvalidOpcode(op) => write!(f, "invalid opcode: 0x{:02x}", op),
            VMError::InvalidFunctionId(id) => write!(f, "invalid function id: {}", id),
            VMError::InvalidLocalIndex(idx) => write!(f, "invalid local index: {}", idx),
            VMError::InvalidExternId(id) => write!(f, "invalid extern id: {}", id),
            VMError::TypeMismatch(msg) => write!(f, "type mismatch: {}", msg),
            VMError::Unreachable => write!(f, "reached unreachable code"),
            VMError::IOError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for VMError {}

type Result<T> = std::result::Result<T, VMError>;

/// Call frame
struct Frame {
    func_id: u32,
    pc: usize,
    locals: Locals,
    #[allow(dead_code)]
    stack_base: usize,
}

/// The virtual machine
pub struct VM {
    stack: Stack,
    frames: Frames,
    modules: Vec<Module>,
    func_table: HashMap<String, (usize, usize)>,
    externs: HashMap<String, ExternFn>,
    #[allow(dead_code)]
    globals: HashMap<String, LeanValue>,
    /// Cache for closed constants (___closed__N functions).
    /// Key is (module_idx, func_idx), value is the cached result.
    /// This prevents re-initialization of closed constants on every reference.
    init_cache: HashMap<(usize, usize), LeanValue>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            stack: SmallVec::new(),
            frames: SmallVec::new(),
            modules: Vec::new(),
            func_table: HashMap::new(),
            externs: HashMap::new(),
            globals: HashMap::new(),
            init_cache: HashMap::new(),
        };
        vm.register_builtins();
        vm
    }

    fn register_builtins(&mut self) {
        for (name, func) in externs::get_builtins() {
            self.externs.insert(name.to_string(), func);
        }
    }

    pub fn register_extern(&mut self, name: &str, func: ExternFn) {
        self.externs.insert(name.to_string(), func);
    }

    pub fn load_module(&mut self, module: Module) {
        let mod_idx = self.modules.len();
        for (func_idx, func) in module.functions.iter().enumerate() {
            self.func_table
                .insert(func.name.clone(), (mod_idx, func_idx));
        }
        self.modules.push(module);
    }

    pub fn run(&mut self) -> Result<LeanValue> {
        if self.modules.is_empty() {
            return Err(VMError::InvalidFunctionId(0));
        }
        // Run from the LAST loaded module (the main program)
        // Earlier modules are includes (Init, etc.)
        let main_mod_idx = self.modules.len() - 1;
        let entry = self.modules[main_mod_idx].entry as usize;
        self.call_function(main_mod_idx, entry, vec![])?;
        self.execute()
    }

    pub fn run_function(&mut self, name: &str, args: &[LeanValue]) -> Result<LeanValue> {
        let (mod_idx, func_idx) = self
            .func_table
            .get(name)
            .copied()
            .ok_or(VMError::InvalidFunctionId(0))?;
        // Clone args to transfer ownership to the function
        let owned_args: Vec<_> = args.to_vec();
        self.call_function(mod_idx, func_idx, owned_args)?;
        self.execute()
    }

    fn call_function(
        &mut self,
        mod_idx: usize,
        func_idx: usize,
        args: Vec<LeanValue>,
    ) -> Result<()> {
        let func = &self.modules[mod_idx].functions[func_idx];
        #[cfg(feature = "trace")]
        eprintln!(
            "    [DEBUG] call_function: mod={}, func={}, name={}, code_len={}",
            mod_idx,
            func_idx,
            func.name,
            func.code.len()
        );
        let mut locals: Locals = SmallVec::with_capacity(func.num_locals as usize);
        // Move args directly into locals - ownership transferred from caller
        for arg in args {
            locals.push(arg);
        }
        while locals.len() < func.num_locals as usize {
            locals.push(LeanValue::unit());
        }
        self.frames.push(Frame {
            func_id: (mod_idx as u32) << 16 | (func_idx as u32),
            pc: 0,
            locals,
            stack_base: self.stack.len(),
        });
        Ok(())
    }

    /// Apply arguments to a bytecode function closure.
    /// Handles partial application, exact application, and over-application.
    fn apply_bytecode_closure(
        &mut self,
        closure: LeanValue,
        new_args: Vec<LeanValue>,
    ) -> Result<LeanValue> {
        unsafe {
            let closure_ptr = closure.as_ptr() as *const LeanClosure;
            let arity = (*closure_ptr).arity as usize;
            let num_fixed = (*closure_ptr).num_fixed as usize;
            let func_id = (*closure_ptr).fun as usize as u32;

            // Handle pre-resolved func_id (high bit set)
            let (mod_idx, func_idx) = if func_id & FUNC_ID_RESOLVED_BIT != 0 {
                (0usize, (func_id & !FUNC_ID_RESOLVED_BIT) as usize)
            } else {
                ((func_id >> 16) as usize, (func_id & 0xFFFF) as usize)
            };

            #[cfg(feature = "trace")]
            eprintln!(
                "[DEBUG] apply_bytecode_closure: arity={}, num_fixed={}, new_args={}, func_id=0x{:x}",
                arity, num_fixed, new_args.len(), func_id
            );

            let total_args = num_fixed + new_args.len();

            if total_args < arity {
                // Partial application: create a new closure with more fixed args
                let new_closure = lean_alloc_closure(
                    func_id as usize as *const (),
                    arity as u32,
                    total_args as u32,
                );
                // Copy existing fixed args
                for i in 0..num_fixed {
                    let arg = lean_closure_get(closure.as_ptr(), i as u32);
                    lean_inc(arg);
                    lean_closure_set(new_closure, i as u32, arg);
                }
                // Add new args
                for (i, arg) in new_args.into_iter().enumerate() {
                    lean_closure_set(new_closure, (num_fixed + i) as u32, arg.into_raw());
                }
                // Drop the original closure
                std::mem::drop(closure);
                Ok(LeanValue::from_raw(new_closure))
            } else if total_args == arity {
                // Exact application: collect all args and call the function
                let mut all_args = Vec::with_capacity(arity);
                for i in 0..num_fixed {
                    let arg = lean_closure_get(closure.as_ptr(), i as u32);
                    lean_inc(arg);
                    all_args.push(LeanValue::from_raw(arg));
                }
                for arg in new_args {
                    all_args.push(arg);
                }
                // Drop the closure (no longer needed)
                std::mem::drop(closure);
                // Call the bytecode function
                self.call_function(mod_idx, func_idx, all_args)?;
                // The function will push its result when it returns
                // We need to run until it returns
                self.execute_until_return()
            } else {
                // Over-application: saturate first, then apply remaining args
                let needed = arity - num_fixed;
                let (first, rest): (Vec<_>, Vec<_>) = new_args
                    .into_iter()
                    .enumerate()
                    .partition(|(i, _)| *i < needed);
                let first_args: Vec<_> = first.into_iter().map(|(_, v)| v).collect();
                let rest_args: Vec<_> = rest.into_iter().map(|(_, v)| v).collect();

                // First, fully saturate
                let mut all_args = Vec::with_capacity(arity);
                for i in 0..num_fixed {
                    let arg = lean_closure_get(closure.as_ptr(), i as u32);
                    lean_inc(arg);
                    all_args.push(LeanValue::from_raw(arg));
                }
                for arg in first_args {
                    all_args.push(arg);
                }
                // Drop the closure
                std::mem::drop(closure);
                // Call the function
                self.call_function(mod_idx, func_idx, all_args)?;
                let result = self.execute_until_return()?;

                // Now apply remaining args to the result (which should be a closure)
                self.apply_bytecode_closure(result, rest_args)
            }
        }
    }

    /// Execute until the current call returns, returning the result.
    /// This is used for bytecode closure application where we need the result synchronously.
    fn execute_until_return(&mut self) -> Result<LeanValue> {
        let target_depth = self.frames.len() - 1;
        self.execute_inner(Some(target_depth))
    }

    fn execute(&mut self) -> Result<LeanValue> {
        self.execute_inner(None)
    }

    /// Execute VM instructions. If target_depth is Some(n), stop when frame depth <= n.
    fn execute_inner(&mut self, target_depth: Option<usize>) -> Result<LeanValue> {
        loop {
            // Check termination conditions
            if self.frames.is_empty() {
                return self.stack.pop().ok_or(VMError::StackUnderflow);
            }
            if let Some(depth) = target_depth {
                if self.frames.len() <= depth {
                    return self.stack.pop().ok_or(VMError::StackUnderflow);
                }
            }

            let (func_id, pc) = {
                let frame = self.frames.last().unwrap();
                (frame.func_id, frame.pc)
            };

            let mod_idx = (func_id >> 16) as usize;
            let func_idx = (func_id & 0xFFFF) as usize;
            let func = &self.modules[mod_idx].functions[func_idx];

            if pc >= func.code.len() {
                let result = self.stack.pop().unwrap_or_else(LeanValue::unit);
                self.frames.pop();
                // Check if we should return due to target depth
                if let Some(depth) = target_depth {
                    if self.frames.len() <= depth {
                        return Ok(result);
                    }
                }
                if !self.frames.is_empty() {
                    self.stack.push(result);
                } else {
                    return Ok(result);
                }
                continue;
            }

            let op_byte = func.code[pc];
            self.frames.last_mut().unwrap().pc += 1;
            let op = Opcode::from_u8(op_byte).ok_or(VMError::InvalidOpcode(op_byte))?;

            #[cfg(feature = "trace")]
            eprintln!("  [{:04}] {:?}", pc, op);

            match op {
                Opcode::LoadLocal => {
                    let idx = self.read_u16()? as usize;
                    let frame = self.frames.last().unwrap();
                    if idx >= frame.locals.len() {
                        return Err(VMError::InvalidLocalIndex(idx as u16));
                    }
                    // Clone to increment ref count - this handles the "borrow" semantics
                    // of LoadLocal. The IR emits Inc after LoadLocal when it needs an
                    // extra reference, but since we clone here, Inc becomes a no-op.
                    let val = frame.locals[idx].clone();

                    #[cfg(feature = "trace")]
                    unsafe {
                        let ptr = val.as_ptr();
                        if lean_is_scalar(ptr) {
                            eprintln!(
                                "    LoadLocal[{}] = scalar 0x{:x} (value={})",
                                idx,
                                ptr as usize,
                                lean_unbox(ptr)
                            );
                        } else {
                            eprintln!(
                                "    LoadLocal[{}] = ptr 0x{:x} tag={}",
                                idx,
                                ptr as usize,
                                lean_obj_tag(ptr)
                            );
                        }
                    }

                    self.stack.push(val);
                }

                Opcode::StoreLocal => {
                    let idx = self.read_u16()? as usize;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let frame = self.frames.last_mut().unwrap();
                    if idx >= frame.locals.len() {
                        return Err(VMError::InvalidLocalIndex(idx as u16));
                    }
                    // Replace old value - Rust's Drop will decrement its ref count
                    frame.locals[idx] = val;
                }

                Opcode::Pop => {
                    // Pop and let Drop decrement ref count
                    self.stack.pop().ok_or(VMError::StackUnderflow)?;
                }

                Opcode::Dup => {
                    // Duplicate with clone to increment ref count
                    let val = self.stack.last().ok_or(VMError::StackUnderflow)?;
                    self.stack.push(val.clone());
                }

                Opcode::LoadConst => {
                    let _const_id = self.read_u32()?;
                    self.stack.push(LeanValue::unit());
                }

                Opcode::AllocCtor => {
                    let tag = self.read_u8()?;
                    let num_fields = self.read_u8()? as u32;
                    let obj = alloc_ctor(tag, num_fields, 0);
                    for i in (0..num_fields).rev() {
                        let field = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                        unsafe {
                            lean_ctor_set(obj.as_ptr(), i, field.into_raw());
                        }
                    }
                    self.stack.push(obj);
                }

                Opcode::CtorGet => {
                    let idx = self.read_u8()? as u32;
                    let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let field = obj.ctor_get(idx);
                    // obj is dropped here, decrementing its ref count
                    self.stack.push(field);
                }

                Opcode::CtorSet => {
                    let idx = self.read_u8()? as u32;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let mut obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    obj.ctor_set(idx, val);
                    self.stack.push(obj);
                }

                Opcode::CtorSetTag => {
                    let _tag = self.read_u8()?;
                }

                Opcode::GetTag => {
                    let obj = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let tag = obj.tag();
                    #[cfg(feature = "trace")]
                    eprintln!("  GetTag: {:?} -> tag={}", obj, tag);
                    // obj is dropped here, decrementing its ref count
                    self.stack.push(LeanValue::from_small_nat(tag));
                }

                Opcode::AllocClosure => {
                    let func_id = self.read_u32()?;
                    let arity = self.read_u8()? as u32;
                    let num_captured = self.read_u8()? as usize;

                    // Resolve func_id to corrected form for closure storage
                    let corrected_func_id = if func_id & FUNC_ID_RESOLVED_BIT != 0 {
                        // Pre-resolved: keep as-is (includes resolved bit for later Apply)
                        func_id
                    } else {
                        // Unresolved: mod_idx=0 means "same module as caller"
                        let mut mod_idx = (func_id >> 16) as usize;
                        let func_idx = (func_id & 0xFFFF) as usize;
                        if mod_idx == 0 {
                            let caller_func_id = self.frames.last().unwrap().func_id;
                            mod_idx = (caller_func_id >> 16) as usize;
                        }
                        ((mod_idx as u32) << 16) | (func_idx as u32)
                    };

                    let stack_len = self.stack.len();
                    if stack_len < num_captured {
                        return Err(VMError::StackUnderflow);
                    }
                    let captured: Vec<_> = self.stack.drain(stack_len - num_captured..).collect();
                    unsafe {
                        let closure = lean_alloc_closure(
                            corrected_func_id as usize as *const (),
                            arity,
                            num_captured as u32,
                        );
                        for (i, val) in captured.into_iter().enumerate() {
                            lean_closure_set(closure, i as u32, val.into_raw());
                        }
                        self.stack.push(LeanValue::from_raw(closure));
                    }
                }

                Opcode::ClosureGet => {
                    let idx = self.read_u8()? as u32;
                    let closure = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    unsafe {
                        let val = lean_closure_get(closure.as_ptr(), idx);
                        lean_inc(val);
                        // closure is dropped here, decrementing its ref count
                        self.stack.push(LeanValue::from_raw(val));
                    }
                }

                Opcode::ClosureSet => {
                    let idx = self.read_u8()? as u32;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let closure = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    unsafe {
                        lean_closure_set(closure.as_ptr(), idx, val.into_raw());
                    }
                    self.stack.push(closure);
                }

                Opcode::Inc => {
                    // No-op: Rust's RAII handles reference counting automatically.
                    // LoadLocal clones (rc+1), Drop decrements (rc-1).
                    // The IR's Inc/Dec are not needed with RAII semantics.
                }

                Opcode::Dec => {
                    // No-op: Rust's RAII handles reference counting automatically.
                    // See comment on Inc above.
                }

                Opcode::IsShared => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    self.stack.push(LeanValue::from_bool(val.is_shared()));
                }

                Opcode::IsExclusive => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let exclusive = val.is_exclusive();
                    // val is dropped here, decrementing its ref count
                    self.stack.push(LeanValue::from_bool(exclusive));
                }

                Opcode::Call => {
                    let func_id = self.read_u32()?;
                    let num_args = self.read_u8()? as usize;
                    let stack_len = self.stack.len();
                    if stack_len < num_args {
                        return Err(VMError::StackUnderflow);
                    }
                    let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();

                    // Check if func_id is pre-resolved (high bit set by linker)
                    let (mod_idx, func_idx) = if func_id & FUNC_ID_RESOLVED_BIT != 0 {
                        // Pre-resolved: lower 31 bits are absolute func_idx, module is 0
                        (0usize, (func_id & !FUNC_ID_RESOLVED_BIT) as usize)
                    } else {
                        // Unresolved: mod_idx=0 means "same module as caller"
                        let mut mod_idx = (func_id >> 16) as usize;
                        let func_idx = (func_id & 0xFFFF) as usize;
                        if mod_idx == 0 {
                            let caller_func_id = self.frames.last().unwrap().func_id;
                            #[cfg(feature = "trace")]
                            eprintln!(
                                "    [DEBUG] Call: mod_idx=0, resolving from caller_func_id=0x{:08x}",
                                caller_func_id
                            );
                            mod_idx = (caller_func_id >> 16) as usize;
                        }
                        (mod_idx, func_idx)
                    };

                    #[cfg(feature = "trace")]
                    {
                        let func_name = &self.modules[mod_idx].functions[func_idx].name;
                        eprintln!(
                            "  -> Call {}::{} (mod={}, func={})",
                            mod_idx, func_name, mod_idx, func_idx
                        );
                    }

                    // Args are moved into the function - ownership transferred
                    self.call_function(mod_idx, func_idx, args)?;
                }

                Opcode::TailCall => {
                    let func_id = self.read_u32()?;
                    let num_args = self.read_u8()? as usize;
                    let stack_len = self.stack.len();
                    if stack_len < num_args {
                        return Err(VMError::StackUnderflow);
                    }
                    let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();

                    // Check if func_id is pre-resolved (high bit set by linker)
                    let (mod_idx, func_idx) = if func_id & FUNC_ID_RESOLVED_BIT != 0 {
                        (0usize, (func_id & !FUNC_ID_RESOLVED_BIT) as usize)
                    } else {
                        let mut mod_idx = (func_id >> 16) as usize;
                        let func_idx = (func_id & 0xFFFF) as usize;
                        if mod_idx == 0 {
                            let caller_func_id = self.frames.last().unwrap().func_id;
                            mod_idx = (caller_func_id >> 16) as usize;
                        }
                        (mod_idx, func_idx)
                    };

                    // Pop frame - locals are dropped, decrementing ref counts
                    self.frames.pop();
                    // Args are moved into the function - ownership transferred
                    self.call_function(mod_idx, func_idx, args)?;
                }

                Opcode::Apply => {
                    let num_args = self.read_u8()? as usize;
                    let stack_len = self.stack.len();
                    if stack_len < num_args + 1 {
                        return Err(VMError::StackUnderflow);
                    }
                    let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                    let closure = self.stack.pop().ok_or(VMError::StackUnderflow)?;

                    #[cfg(feature = "trace")]
                    unsafe {
                        let ptr = closure.as_ptr();
                        if lean_is_scalar(ptr) {
                            eprintln!(
                                "[DEBUG] Apply: closure is SCALAR 0x{:x} (value={}), num_args={}",
                                ptr as usize,
                                lean_unbox(ptr),
                                num_args
                            );
                            eprintln!("  Stack before Apply had {} elements", stack_len);
                        } else {
                            let tag = lean_obj_tag(ptr);
                            eprintln!(
                                "[DEBUG] Apply: closure ptr=0x{:x} tag={}, num_args={}",
                                ptr as usize, tag, num_args
                            );
                        }
                    }

                    // Check if this is a bytecode function closure
                    let raw_closure = closure.as_ptr();
                    let is_bytecode_closure = unsafe {
                        if lean_is_scalar(raw_closure) || lean_obj_tag(raw_closure) != 245 {
                            false
                        } else {
                            let closure_obj = raw_closure as *const LeanClosure;
                            let fun_ptr = (*closure_obj).fun as usize;
                            // Bytecode function IDs are small (<= 0xFFFFFFFF).
                            // Real function pointers on 64-bit are much larger.
                            fun_ptr < 0x100000000
                        }
                    };

                    if is_bytecode_closure {
                        // Handle bytecode function closure
                        let result = self.apply_bytecode_closure(closure, args)?;
                        self.stack.push(result);
                    } else {
                        // Transfer ownership to lean_apply_m (it consumes closure and args)
                        let raw_args: Vec<*mut LeanObject> =
                            args.into_iter().map(|v| v.into_raw()).collect();
                        let raw_closure = closure.into_raw();
                        unsafe {
                            let result =
                                lean_apply_m(raw_closure, num_args as u32, raw_args.as_ptr());
                            self.stack.push(LeanValue::from_raw(result));
                        }
                    }
                }

                Opcode::TailApply => {
                    let num_args = self.read_u8()? as usize;
                    let stack_len = self.stack.len();
                    if stack_len < num_args + 1 {
                        return Err(VMError::StackUnderflow);
                    }
                    let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                    let closure = self.stack.pop().ok_or(VMError::StackUnderflow)?;

                    // Check if this is a bytecode function closure
                    let raw_closure = closure.as_ptr();
                    let is_bytecode_closure = unsafe {
                        if lean_is_scalar(raw_closure) || lean_obj_tag(raw_closure) != 245 {
                            false
                        } else {
                            let closure_obj = raw_closure as *const LeanClosure;
                            let fun_ptr = (*closure_obj).fun as usize;
                            // Bytecode function IDs are small (<= 0xFFFFFFFF).
                            fun_ptr < 0x100000000
                        }
                    };

                    if is_bytecode_closure {
                        // Pop frame first for tail call semantics
                        self.frames.pop();
                        // Handle bytecode function closure
                        let result = self.apply_bytecode_closure(closure, args)?;
                        if self.frames.is_empty() {
                            return Ok(result);
                        }
                        self.stack.push(result);
                    } else {
                        // Transfer ownership to lean_apply_m (it consumes closure and args)
                        let raw_args: Vec<*mut LeanObject> =
                            args.into_iter().map(|v| v.into_raw()).collect();
                        let raw_closure = closure.into_raw();
                        self.frames.pop();
                        unsafe {
                            let result =
                                lean_apply_m(raw_closure, num_args as u32, raw_args.as_ptr());
                            if self.frames.is_empty() {
                                return Ok(LeanValue::from_raw(result));
                            }
                            self.stack.push(LeanValue::from_raw(result));
                        }
                    }
                }

                Opcode::PartialApp => {
                    // Format: func_id (u32), arity (u8), num_args (u8)
                    let func_id = self.read_u32()?;
                    let arity = self.read_u8()? as u32;
                    let num_args = self.read_u8()? as usize;

                    // Resolve func_id to corrected form for closure storage
                    let corrected_func_id = if func_id & FUNC_ID_RESOLVED_BIT != 0 {
                        // Pre-resolved: keep as-is
                        func_id
                    } else {
                        let mut mod_idx = (func_id >> 16) as usize;
                        let func_idx = (func_id & 0xFFFF) as usize;
                        if mod_idx == 0 {
                            let caller_func_id = self.frames.last().unwrap().func_id;
                            mod_idx = (caller_func_id >> 16) as usize;
                        }
                        ((mod_idx as u32) << 16) | (func_idx as u32)
                    };

                    let stack_len = self.stack.len();
                    if stack_len < num_args {
                        return Err(VMError::StackUnderflow);
                    }
                    let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                    unsafe {
                        let closure = lean_alloc_closure(
                            corrected_func_id as usize as *const (),
                            arity,
                            num_args as u32,
                        );
                        for (i, arg) in args.into_iter().enumerate() {
                            lean_closure_set(closure, i as u32, arg.into_raw());
                        }
                        self.stack.push(LeanValue::from_raw(closure));
                    }
                }

                Opcode::CallExtern => {
                    // CallExtern is for syscalls/primitives.
                    // For backwards compat with old bytecode, also check func_table.
                    let extern_id = self.read_u32()? as usize;
                    let num_args = self.read_u8()? as usize;
                    let func_id = self.frames.last().unwrap().func_id;
                    let mod_idx = (func_id >> 16) as usize;
                    let ext = &self.modules[mod_idx].externs[extern_id];
                    let ext_name = ext.name.clone();
                    #[cfg(feature = "trace")]
                    eprintln!("  CallExtern: {}", ext_name);

                    let stack_len = self.stack.len();
                    if stack_len < num_args {
                        return Err(VMError::StackUnderflow);
                    }

                    // Check syscall table first (preferred for new bytecode)
                    if let Some(&func) = self.externs.get(&ext_name) {
                        let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                        let result = func(&args)?;
                        for arg in args {
                            std::mem::forget(arg);
                        }
                        self.stack.push(result);
                    } else if let Some(&(bc_mod_idx, bc_func_idx)) = self.func_table.get(&ext_name)
                    {
                        // Backwards compat: old bytecode uses CallExtern for imports
                        let bc_arity =
                            self.modules[bc_mod_idx].functions[bc_func_idx].arity as usize;
                        if num_args < bc_arity {
                            #[cfg(feature = "trace")]
                            eprintln!(
                                "    -> partial application: {} args, arity {}",
                                num_args, bc_arity
                            );
                            let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                            let bc_func_id = ((bc_mod_idx as u32) << 16) | (bc_func_idx as u32);
                            unsafe {
                                let closure = lean_alloc_closure(
                                    bc_func_id as usize as *const (),
                                    bc_arity as u32,
                                    num_args as u32,
                                );
                                for (i, arg) in args.into_iter().enumerate() {
                                    lean_closure_set(closure, i as u32, arg.into_raw());
                                }
                                self.stack.push(LeanValue::from_raw(closure));
                            }
                        } else {
                            #[cfg(feature = "trace")]
                            eprintln!(
                                "    -> bytecode function (mod={}, func={})",
                                bc_mod_idx, bc_func_idx
                            );
                            let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                            self.call_function(bc_mod_idx, bc_func_idx, args)?;
                        }
                    } else {
                        eprintln!(
                            "Missing extern: {} (extern_id={} in module {})",
                            ext_name, extern_id, mod_idx
                        );
                        return Err(VMError::InvalidExternId(extern_id as u32));
                    }
                }

                Opcode::CallImport => {
                    // CallImport is for imported bytecode functions (looked up by name)
                    let name_id = self.read_u32()? as usize;
                    let num_args = self.read_u8()? as usize;
                    let func_id = self.frames.last().unwrap().func_id;
                    let mod_idx = (func_id >> 16) as usize;
                    let func_name = &self.modules[mod_idx].strings[name_id];
                    #[cfg(feature = "trace")]
                    eprintln!("  CallImport: {} (num_args={})", func_name, num_args);

                    let stack_len = self.stack.len();
                    if stack_len < num_args {
                        return Err(VMError::StackUnderflow);
                    }

                    if let Some(&(bc_mod_idx, bc_func_idx)) = self.func_table.get(func_name) {
                        let bc_arity =
                            self.modules[bc_mod_idx].functions[bc_func_idx].arity as usize;
                        if num_args < bc_arity {
                            // Partial application: create a closure
                            #[cfg(feature = "trace")]
                            eprintln!(
                                "    -> partial application: {} args, arity {}",
                                num_args, bc_arity
                            );
                            let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                            let bc_func_id = ((bc_mod_idx as u32) << 16) | (bc_func_idx as u32);
                            unsafe {
                                let closure = lean_alloc_closure(
                                    bc_func_id as usize as *const (),
                                    bc_arity as u32,
                                    num_args as u32,
                                );
                                for (i, arg) in args.into_iter().enumerate() {
                                    lean_closure_set(closure, i as u32, arg.into_raw());
                                }
                                self.stack.push(LeanValue::from_raw(closure));
                            }
                        } else {
                            // Full application: call the bytecode function
                            #[cfg(feature = "trace")]
                            eprintln!(
                                "    -> bytecode function (mod={}, func={})",
                                bc_mod_idx, bc_func_idx
                            );
                            let args: Vec<_> = self.stack.drain(stack_len - num_args..).collect();
                            self.call_function(bc_mod_idx, bc_func_idx, args)?;
                        }
                    } else {
                        eprintln!(
                            "Missing import: {} (name_id={} in module {})",
                            func_name, name_id, mod_idx
                        );
                        return Err(VMError::InvalidFunctionId(name_id as u32));
                    }
                }

                Opcode::Jump => {
                    let offset = self.read_i32()?;
                    let frame = self.frames.last_mut().unwrap();
                    frame.pc = (frame.pc as i32 + offset) as usize;
                }

                Opcode::JumpIf => {
                    let offset = self.read_i32()?;
                    let cond = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    if cond.tag() != 0 {
                        let frame = self.frames.last_mut().unwrap();
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }

                Opcode::JumpIfNot => {
                    let offset = self.read_i32()?;
                    let cond = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    if cond.tag() == 0 {
                        let frame = self.frames.last_mut().unwrap();
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }

                Opcode::Switch => {
                    let num_cases = self.read_u16()? as usize;
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let tag = val.tag();
                    let mut offsets = Vec::with_capacity(num_cases);
                    for _ in 0..num_cases {
                        offsets.push(self.read_i32()?);
                    }
                    let default_offset = self.read_i32()?;
                    let offset = if tag < num_cases {
                        offsets[tag]
                    } else {
                        default_offset
                    };
                    let frame = self.frames.last_mut().unwrap();
                    frame.pc = (frame.pc as i32 + offset) as usize;
                    // val is dropped here, decrementing its ref count
                }

                Opcode::Ret => {
                    let result = self.stack.pop().ok_or(VMError::StackUnderflow)?;

                    // Cache closed constants on first return and mark persistent
                    let frame = self.frames.last().unwrap();
                    let func_id = frame.func_id;
                    let mod_idx = (func_id >> 16) as usize;
                    let func_idx = (func_id & 0xFFFF) as usize;
                    let _func = &self.modules[mod_idx].functions[func_idx];

                    // NOTE: Caching disabled - causes ref count inflation
                    // if func.arity == 0 && is_closed_constant_name(&func.name) {
                    //     self.init_cache
                    //         .entry((mod_idx, func_idx))
                    //         .or_insert_with(|| result.clone());
                    // }

                    // Pop frame - locals are dropped, decrementing ref counts
                    self.frames.pop();
                    #[cfg(feature = "trace")]
                    eprintln!(
                        "  Ret: frames_left={}, result={:?}",
                        self.frames.len(),
                        result
                    );
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    self.stack.push(result);
                }

                Opcode::Unreachable => {
                    return Err(VMError::Unreachable);
                }

                Opcode::Box | Opcode::Unbox => {}

                Opcode::IsScalar => {
                    let val = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let scalar = val.is_scalar();
                    // val is dropped here, decrementing its ref count
                    self.stack.push(LeanValue::from_bool(scalar));
                }

                Opcode::NatLit => {
                    let val = self.read_u64()?;
                    self.stack.push(LeanValue::from_nat(val));
                }

                Opcode::NatAdd => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: both scalars and no overflow
                    if a.is_scalar() && b.is_scalar() {
                        let av = a.unbox();
                        let bv = b.unbox();
                        if let Some(sum) = av.checked_add(bv) {
                            if sum <= LEAN_MAX_SMALL_NAT {
                                self.stack.push(LeanValue::from_small_nat(sum));
                                continue;
                            }
                        }
                    }
                    // Slow path: delegate to runtime
                    let result = unsafe { lean_nat_add(a.into_raw(), b.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::NatSub => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: both scalars (saturating subtraction)
                    if a.is_scalar() && b.is_scalar() {
                        let av = a.unbox();
                        let bv = b.unbox();
                        let diff = av.saturating_sub(bv);
                        self.stack.push(LeanValue::from_small_nat(diff));
                        continue;
                    }
                    // Slow path: delegate to runtime
                    let result = unsafe { lean_nat_sub(a.into_raw(), b.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::NatMul => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: both scalars and no overflow
                    if a.is_scalar() && b.is_scalar() {
                        let av = a.unbox();
                        let bv = b.unbox();
                        if let Some(prod) = av.checked_mul(bv) {
                            if prod <= LEAN_MAX_SMALL_NAT {
                                self.stack.push(LeanValue::from_small_nat(prod));
                                continue;
                            }
                        }
                    }
                    // Slow path: delegate to runtime
                    let result = unsafe { lean_nat_mul(a.into_raw(), b.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::NatDiv => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let result = unsafe { lean_nat_div(a.into_raw(), b.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::NatMod => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let result = unsafe { lean_nat_mod(a.into_raw(), b.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::NatLt => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: both scalars
                    if a.is_scalar() && b.is_scalar() {
                        self.stack.push(LeanValue::from_bool(a.unbox() < b.unbox()));
                        continue;
                    }
                    // Slow path: delegate to runtime
                    let result = unsafe { lean_nat_dec_lt(a.into_raw(), b.into_raw()) };
                    self.stack.push(LeanValue::from_bool(result != 0));
                }

                Opcode::NatLe => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: both scalars
                    if a.is_scalar() && b.is_scalar() {
                        self.stack
                            .push(LeanValue::from_bool(a.unbox() <= b.unbox()));
                        continue;
                    }
                    // Slow path: delegate to runtime
                    let result = unsafe { lean_nat_dec_le(a.into_raw(), b.into_raw()) };
                    self.stack.push(LeanValue::from_bool(result != 0));
                }

                Opcode::NatEq => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: both scalars (pointer equality suffices)
                    if a.is_scalar() && b.is_scalar() {
                        self.stack
                            .push(LeanValue::from_bool(a.as_ptr() == b.as_ptr()));
                        continue;
                    }
                    // Slow path: delegate to runtime
                    let result = unsafe { lean_nat_dec_eq(a.into_raw(), b.into_raw()) };
                    self.stack.push(LeanValue::from_bool(result != 0));
                }

                Opcode::NatSucc => {
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Fast path: scalar with no overflow
                    if a.is_scalar() {
                        let av = a.unbox();
                        if av < LEAN_MAX_SMALL_NAT {
                            self.stack.push(LeanValue::from_small_nat(av + 1));
                            continue;
                        }
                    }
                    // Slow path: delegate to runtime
                    let one = LeanValue::from_small_nat(1);
                    let result = unsafe { lean_nat_add(a.into_raw(), one.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::StringLit => {
                    let str_id = self.read_u32()? as usize;
                    let func_id = self.frames.last().unwrap().func_id;
                    let mod_idx = (func_id >> 16) as usize;
                    let s = self.modules[mod_idx].strings[str_id].clone();
                    self.stack.push(LeanValue::from_string(&s));
                }

                Opcode::StringAppend => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let result = unsafe { lean_string_append(a.into_raw(), b.into_raw()) };
                    self.stack.push(unsafe { LeanValue::from_raw(result) });
                }

                Opcode::StringLength => {
                    let s = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    // Note: lean_string_length does NOT consume s, it just reads
                    // So we use as_ptr() and let Drop handle cleanup
                    let len = unsafe { lean_string_length(s.as_ptr()) };
                    self.stack.push(unsafe { LeanValue::from_raw(len) });
                }

                Opcode::StringEq => {
                    let b = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(VMError::StackUnderflow)?;
                    let result = unsafe { lean_string_dec_eq(a.into_raw(), b.into_raw()) };
                    self.stack.push(LeanValue::from_bool(result != 0));
                }

                Opcode::UnitLit => {
                    self.stack.push(LeanValue::unit());
                }

                Opcode::BoolLit => {
                    let val = self.read_u8()?;
                    self.stack.push(LeanValue::from_bool(val != 0));
                }

                Opcode::LoadModuleConst => {
                    // Load pre-initialized module constant by index
                    // This opcode is for future use when the emitter generates it
                    let const_id = self.read_u32()? as usize;
                    let func_id = self.frames.last().unwrap().func_id;
                    let mod_idx = (func_id >> 16) as usize;

                    // The const_id maps to a function index
                    if let Some(cached) = self.init_cache.get(&(mod_idx, const_id)) {
                        self.stack.push(cached.clone());
                    } else {
                        // Not yet initialized - this shouldn't happen with proper init
                        return Err(VMError::InvalidOpcode(op as u8));
                    }
                }

                Opcode::ScalarProj | Opcode::ScalarSet | Opcode::Trace => {
                    return Err(VMError::InvalidOpcode(op as u8));
                }
            }
        }
    }

    fn read_u8(&mut self) -> Result<u8> {
        let frame = self.frames.last_mut().unwrap();
        let func_id = frame.func_id;
        let pc = frame.pc;
        let mod_idx = (func_id >> 16) as usize;
        let func_idx = (func_id & 0xFFFF) as usize;
        let func = &self.modules[mod_idx].functions[func_idx];
        if pc >= func.code.len() {
            #[cfg(feature = "trace")]
            eprintln!(
                "  [DEBUG read_u8] pc={} >= code_len={} for func_id=0x{:08x} (mod={}, func={})",
                pc,
                func.code.len(),
                func_id,
                mod_idx,
                func_idx
            );
            return Err(VMError::InvalidOpcode(0));
        }
        let val = func.code[pc];
        self.frames.last_mut().unwrap().pc += 1;
        Ok(val)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let b0 = self.read_u8()? as u16;
        let b1 = self.read_u8()? as u16;
        Ok(b0 | (b1 << 8))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let b0 = self.read_u8()? as u32;
        let b1 = self.read_u8()? as u32;
        let b2 = self.read_u8()? as u32;
        let b3 = self.read_u8()? as u32;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let lo = self.read_u32()? as u64;
        let hi = self.read_u32()? as u64;
        Ok(lo | (hi << 32))
    }
}
