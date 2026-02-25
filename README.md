# Lean 4 Rust Runtime

Runtime library for Lean 4 programs compiled via the `--rust` backend. Implements the same ABI as the C runtime (`lean.h`), providing reference-counted object management, boxed/unboxed value representations, and all `@[extern]` functions required by compiled Lean IR.

```bash
cargo build          # debug build
cargo test           # unit tests
cargo clippy         # lint
```

## Modules

| Module       | Description                                                                                                   |
| ------------ | ------------------------------------------------------------------------------------------------------------- |
| `object`     | Core `LeanObject` header layout, heap allocation, tag/field constants, and pointer arithmetic.                |
| `rc`         | Reference counting: `lean_inc`, `lean_dec`, exclusive checks, persistent marking, and recursive freeing.      |
| `box`        | Tagged pointer boxing/unboxing for scalars, UInt32, UInt64, USize, and Float64.                               |
| `ctor`       | Constructor allocation (`lean_alloc_ctor`) and field access (`lean_ctor_get`/`lean_ctor_set`).                |
| `closure`    | Closure allocation, fixed-argument storage, and `lean_apply_N` multi-arity application.                       |
| `array`      | Boxed object arrays: alloc, push, get/set, `List <-> Array` conversion.                                       |
| `sarray`     | Scalar (byte) arrays: `ByteArray` alloc, push, get/set, UTF-8 validation.                                     |
| `string`     | UTF-8 string objects: creation, append, comparison, character iteration, `Nat`/`List Char` conversion.        |
| `nat`        | Natural number arithmetic with automatic promotion to big integers on overflow.                               |
| `bignat`     | Arbitrary-precision natural numbers backed by `num-bigint`, stored as tag-250 heap objects.                   |
| `int`        | Signed arbitrary-precision integer arithmetic (`Int`), using big nat storage for large values.                |
| `uint`       | Fixed-width unsigned integer operations (`UInt8`/`16`/`32`/`64`/`USize`): truncation, conversion, comparison. |
| `sint`       | Fixed-width signed integer operations (`Int8`/`16`/`32`/`64`/`ISize`): wrapping arithmetic, bitwise ops.      |
| `float`      | `Float` (f64) and `Float32` (f32) arithmetic, classification, string conversion, and bit-level access.        |
| `floatarray` | `FloatArray`: scalar array of `f64` values with push/get/set.                                                 |
| `io`         | IO monad primitives: result construction, `IO.println`, `IO.getStdin`, file handles, process exit.            |
| `stref`      | `ST.Ref` / `IO.Ref` primitives: alloc, get, set, swap, take.                                                  |
| `thunk`      | Lazy thunk allocation and forcing (`Thunk.get`).                                                              |
| `external`   | External (opaque) object support with custom finalizers.                                                      |
| `panic`      | Panic and error handling: `lean_panic_fn`, `lean_internal_panic`.                                             |
| `platform`   | Version info, platform target, constructor limits, runtime initialization stubs.                              |
| `misc`       | `Name` structural equality, `sorry` axiom stub, `dbg_trace`, platform nbits query.                            |
| `debug`      | Debug-mode instrumentation: pointer validation, use-after-free detection, `lean_debug_dump`.                  |
| `owned`      | Safe RAII wrapper (`LeanOwnedValue`) for `*mut LeanObject` with automatic reference counting.                 |

## Safe Wrapper (`LeanOwnedValue`)

The `owned` module provides `LeanOwnedValue`, an RAII wrapper around raw `*mut LeanObject` pointers. It automatically increments the reference count on `Clone` and decrements on `Drop`, making it safe to use from hand-written Rust code that interacts with the Lean runtime. Generated code continues to use explicit `lean_inc`/`lean_dec` calls and is unaffected.
