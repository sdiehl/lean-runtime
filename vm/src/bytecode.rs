//! Bytecode format for Lean 4 VM
//!
//! File format (.leanbc):
//! ```text
//! Header:
//!   magic: [u8; 4] = "LNBC"
//!   version: u32
//!   num_strings: u32
//!   num_functions: u32
//!   num_externs: u32
//!   num_constants: u32
//!   entry_function: u32
//!   init_function: u32 (0xFFFFFFFF if none)
//!
//! String pool:
//!   strings: [LenPrefixedString; num_strings]
//!
//! Extern table:
//!   externs: [ExternDecl; num_externs]
//!
//! Constant declarations (module-level closed constants):
//!   constants: [ConstantDecl; num_constants]
//!     name_length: u32
//!     name: [u8; name_length]
//!     init_func: u32 (index of _init_ function)
//!
//! Functions:
//!   functions: [Function; num_functions]
//! ```

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

pub const MAGIC: &[u8; 4] = b"LNBC";
pub const VERSION: u32 = 1;

/// Bytecode opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Stack/locals
    LoadLocal = 0x01,       // Load local variable onto stack
    StoreLocal = 0x02,      // Store TOS into local variable
    LoadConst = 0x03,       // Load constant (string/nat literal)
    Pop = 0x04,             // Discard TOS
    Dup = 0x05,             // Duplicate TOS
    LoadModuleConst = 0x06, // Load pre-initialized module constant

    // Constructors
    AllocCtor = 0x10,  // Allocate constructor: tag, num_fields
    CtorGet = 0x11,    // Get field: field_index
    CtorSet = 0x12,    // Set field: field_index (obj, val on stack)
    CtorSetTag = 0x13, // Set tag: new_tag
    GetTag = 0x14,     // Push tag of TOS

    // Closures
    AllocClosure = 0x20, // Allocate closure: func_id, arity, num_captured
    ClosureGet = 0x21,   // Get captured variable
    ClosureSet = 0x22,   // Set captured variable

    // Reference counting
    Inc = 0x30,         // Increment refcount
    Dec = 0x31,         // Decrement refcount
    IsShared = 0x32,    // Push 1 if refcount > 1, else 0
    IsExclusive = 0x33, // Push 1 if refcount == 1, else 0

    // Function calls
    Call = 0x40,       // Call function: func_id, num_args
    TailCall = 0x41,   // Tail call: func_id, num_args
    Apply = 0x42,      // Apply closure: num_args
    TailApply = 0x43,  // Tail apply closure: num_args
    PartialApp = 0x44, // Partial application: func_id, num_args, arity
    CallExtern = 0x45, // Call external/syscall: extern_id, num_args
    CallImport = 0x46, // Call imported bytecode function: name_string_id, num_args

    // Control flow
    Jump = 0x50,        // Unconditional jump: offset (i32)
    JumpIf = 0x51,      // Jump if TOS is true (tag != 0): offset
    JumpIfNot = 0x52,   // Jump if TOS is false (tag == 0): offset
    Switch = 0x53,      // Switch on tag: num_cases, [offset; num_cases], default_offset
    Ret = 0x54,         // Return TOS
    Unreachable = 0x55, // Panic - unreachable code

    // Scalars/boxing
    Box = 0x60,      // Box scalar value
    Unbox = 0x61,    // Unbox to scalar
    IsScalar = 0x62, // Push 1 if TOS is scalar

    // Primitives - Nat
    NatLit = 0x70, // Push nat literal: value (u64)
    NatAdd = 0x71,
    NatSub = 0x72,
    NatMul = 0x73,
    NatDiv = 0x74,
    NatMod = 0x75,
    NatLt = 0x76,
    NatLe = 0x77,
    NatEq = 0x78,
    NatSucc = 0x79,

    // Primitives - String
    StringLit = 0x80, // Push string from pool: string_id
    StringAppend = 0x81,
    StringLength = 0x82,
    StringEq = 0x83,

    // Primitives - other
    UnitLit = 0x90, // Push unit value
    BoolLit = 0x91, // Push bool: 0 or 1

    // Projections (for scalar fields in constructors)
    ScalarProj = 0xA0, // Get scalar field: num_objs, offset, size
    ScalarSet = 0xA1,  // Set scalar field: num_objs, offset, size

    // Debug
    Trace = 0xF0, // Print debug info (if trace feature enabled)
}

impl Opcode {
    pub fn from_u8(byte: u8) -> Option<Opcode> {
        match byte {
            0x01..=0x06
            | 0x10..=0x14
            | 0x20..=0x22
            | 0x30..=0x33
            | 0x40..=0x46
            | 0x50..=0x55
            | 0x60..=0x62
            | 0x70..=0x79
            | 0x80..=0x83
            | 0x90..=0x91
            | 0xA0..=0xA1
            | 0xF0 => {
                // SAFETY: we check the value is valid above
                Some(unsafe { std::mem::transmute::<u8, Opcode>(byte) })
            }
            _ => None,
        }
    }
}

/// External function declaration
#[derive(Debug, Clone)]
pub struct ExternDecl {
    pub name: String,
    pub arity: u8,
}

/// Module-level constant declaration (closed constant)
#[derive(Debug, Clone)]
pub struct ConstantDecl {
    pub name: String,
    pub init_func: u32,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: u8,
    pub num_locals: u16,
    pub code: Vec<u8>,
}

/// Constant in the constant pool
#[derive(Debug, Clone)]
pub enum Constant {
    String(String),
    Nat(u64),
}

/// Complete bytecode module
#[derive(Debug, Clone, Default)]
pub struct Module {
    pub strings: Vec<String>,
    pub constants: Vec<Constant>,
    pub externs: Vec<ExternDecl>,
    pub const_decls: Vec<ConstantDecl>,
    pub functions: Vec<Function>,
    pub entry: u32,
    pub init_func: Option<u32>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize module to bytes
    pub fn serialize<W: Write>(&self, w: &mut W) -> io::Result<()> {
        // Header
        w.write_all(MAGIC)?;
        w.write_u32::<LittleEndian>(VERSION)?;
        w.write_u32::<LittleEndian>(self.strings.len() as u32)?;
        w.write_u32::<LittleEndian>(self.functions.len() as u32)?;
        w.write_u32::<LittleEndian>(self.externs.len() as u32)?;
        w.write_u32::<LittleEndian>(self.const_decls.len() as u32)?;
        w.write_u32::<LittleEndian>(self.entry)?;
        w.write_u32::<LittleEndian>(self.init_func.unwrap_or(0xFFFFFFFF))?;

        // String pool
        for s in &self.strings {
            w.write_u32::<LittleEndian>(s.len() as u32)?;
            w.write_all(s.as_bytes())?;
        }

        // Externs
        for ext in &self.externs {
            w.write_u32::<LittleEndian>(ext.name.len() as u32)?;
            w.write_all(ext.name.as_bytes())?;
            w.write_u8(ext.arity)?;
        }

        // Constant declarations
        for c in &self.const_decls {
            w.write_u32::<LittleEndian>(c.name.len() as u32)?;
            w.write_all(c.name.as_bytes())?;
            w.write_u32::<LittleEndian>(c.init_func)?;
        }

        // Functions
        for func in &self.functions {
            w.write_u32::<LittleEndian>(func.name.len() as u32)?;
            w.write_all(func.name.as_bytes())?;
            w.write_u8(func.arity)?;
            w.write_u16::<LittleEndian>(func.num_locals)?;
            w.write_u32::<LittleEndian>(func.code.len() as u32)?;
            w.write_all(&func.code)?;
        }

        Ok(())
    }

    /// Deserialize module from bytes
    pub fn deserialize<R: Read>(r: &mut R) -> io::Result<Self> {
        // Header
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid magic"));
        }

        let version = r.read_u32::<LittleEndian>()?;
        if version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported version: {}", version),
            ));
        }

        let num_strings = r.read_u32::<LittleEndian>()? as usize;
        let num_functions = r.read_u32::<LittleEndian>()? as usize;
        let num_externs = r.read_u32::<LittleEndian>()? as usize;
        let num_constants = r.read_u32::<LittleEndian>()? as usize;
        let entry = r.read_u32::<LittleEndian>()?;
        let init_raw = r.read_u32::<LittleEndian>()?;
        let init_func = if init_raw == 0xFFFFFFFF {
            None
        } else {
            Some(init_raw)
        };

        // String pool
        let mut strings = Vec::with_capacity(num_strings);
        for _ in 0..num_strings {
            let len = r.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)?;
            strings.push(
                String::from_utf8(buf)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }

        // Externs
        let mut externs = Vec::with_capacity(num_externs);
        for _ in 0..num_externs {
            let len = r.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)?;
            let name = String::from_utf8(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let arity = r.read_u8()?;
            externs.push(ExternDecl { name, arity });
        }

        // Constant declarations
        let mut const_decls = Vec::with_capacity(num_constants);
        for _ in 0..num_constants {
            let len = r.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)?;
            let name = String::from_utf8(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let init_fn = r.read_u32::<LittleEndian>()?;
            const_decls.push(ConstantDecl {
                name,
                init_func: init_fn,
            });
        }

        // Functions
        let mut functions = Vec::with_capacity(num_functions);
        for _ in 0..num_functions {
            let len = r.read_u32::<LittleEndian>()? as usize;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)?;
            let name = String::from_utf8(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let arity = r.read_u8()?;
            let num_locals = r.read_u16::<LittleEndian>()?;
            let code_len = r.read_u32::<LittleEndian>()? as usize;
            let mut code = vec![0u8; code_len];
            r.read_exact(&mut code)?;
            functions.push(Function {
                name,
                arity,
                num_locals,
                code,
            });
        }

        Ok(Module {
            strings,
            constants: Vec::new(),
            externs,
            const_decls,
            functions,
            entry,
            init_func,
        })
    }
}

/// Bytecode builder for constructing function bodies
#[derive(Default)]
pub struct BytecodeBuilder {
    code: Vec<u8>,
    labels: Vec<Option<usize>>,   // label -> offset
    patches: Vec<(usize, usize)>, // (patch_offset, label_id)
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, op: Opcode) {
        self.code.push(op as u8);
    }

    pub fn emit_u8(&mut self, val: u8) {
        self.code.push(val);
    }

    pub fn emit_u16(&mut self, val: u16) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    pub fn emit_u32(&mut self, val: u32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    pub fn emit_i32(&mut self, val: i32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    pub fn emit_u64(&mut self, val: u64) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    /// Create a new label, returns label id
    pub fn new_label(&mut self) -> usize {
        let id = self.labels.len();
        self.labels.push(None);
        id
    }

    /// Mark current position as label target
    pub fn mark_label(&mut self, label: usize) {
        self.labels[label] = Some(self.code.len());
    }

    /// Emit jump to label (will be patched later)
    pub fn emit_jump(&mut self, op: Opcode, label: usize) {
        self.emit(op);
        let patch_offset = self.code.len();
        self.emit_i32(0); // placeholder
        self.patches.push((patch_offset, label));
    }

    /// Finalize bytecode, resolving all label references
    pub fn finish(mut self) -> Vec<u8> {
        for (patch_offset, label_id) in self.patches {
            let target = self.labels[label_id].expect("unresolved label");
            let offset = (target as i32) - (patch_offset as i32 + 4);
            self.code[patch_offset..patch_offset + 4].copy_from_slice(&offset.to_le_bytes());
        }
        self.code
    }

    pub fn current_offset(&self) -> usize {
        self.code.len()
    }
}
