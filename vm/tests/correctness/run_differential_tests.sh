#!/bin/bash
# Differential testing: compare native Lean vs bytecode VM

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VM_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT_DIR="$(cd "$VM_DIR/.." && pwd)"

# Find the modified Lean compiler with bytecode support
if [ -f "$ROOT_DIR/../lean4-src/build/release/stage1/bin/lean" ]; then
    LEAN4_SRC="$ROOT_DIR/../lean4-src"
elif [ -f "/Users/sdiehl/Git/lean-compiler/lean4-src/build/release/stage1/bin/lean" ]; then
    LEAN4_SRC="/Users/sdiehl/Git/lean-compiler/lean4-src"
else
    echo "ERROR: Cannot find modified Lean compiler with bytecode support"
    exit 1
fi

LEAN_COMPILER="$LEAN4_SRC/build/release/stage1/bin/lean"
VM="cargo run --release -q --manifest-path=$ROOT_DIR/Cargo.toml --"
INIT_DIR="$ROOT_DIR/init"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
SKIP=0

echo "============================================"
echo "  Differential Testing: Native vs Bytecode"
echo "============================================"
echo ""
echo "Lean compiler: $LEAN_COMPILER"
echo "VM root: $ROOT_DIR"
echo "Init dir: $INIT_DIR"
echo ""

# Create temp directory
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

for lean_file in "$SCRIPT_DIR"/*.lean; do
    if [ ! -f "$lean_file" ]; then
        continue
    fi

    name=$(basename "$lean_file" .lean)

    # Skip non-test files
    if [ "$name" == "run_differential_tests" ]; then
        continue
    fi

    echo -n "Testing $name... "

    # Compile to bytecode
    bc_file="$TMPDIR/${name}.leanbc"
    if ! "$LEAN_COMPILER" --bytecode="$bc_file" "$lean_file" 2>"$TMPDIR/${name}_compile_err.txt"; then
        echo -e "${YELLOW}SKIP${NC} (bytecode compile error)"
        head -3 "$TMPDIR/${name}_compile_err.txt"
        ((SKIP++))
        continue
    fi

    # Run native Lean (compile and execute)
    native_out="$TMPDIR/${name}_native.out"
    if ! "$LEAN_COMPILER" --run "$lean_file" > "$native_out" 2>&1; then
        echo -e "${YELLOW}SKIP${NC} (native run error)"
        head -3 "$native_out"
        ((SKIP++))
        continue
    fi

    # Run bytecode VM
    vm_out="$TMPDIR/${name}_vm.out"
    if ! $VM -L "$INIT_DIR" "$bc_file" > "$vm_out" 2>&1; then
        echo -e "${RED}FAIL${NC} (VM error)"
        echo "  VM output:"
        head -10 "$vm_out"
        ((FAIL++))
        continue
    fi

    # Compare outputs
    if diff -q "$native_out" "$vm_out" > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        ((PASS++))
    else
        echo -e "${RED}FAIL${NC} (output mismatch)"
        echo ""
        echo "  === Native output ==="
        head -20 "$native_out"
        echo ""
        echo "  === VM output ==="
        head -20 "$vm_out"
        echo ""
        echo "  === Diff (first 30 lines) ==="
        diff "$native_out" "$vm_out" | head -30
        echo ""
        ((FAIL++))
    fi
done

echo ""
echo "============================================"
echo "  Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo "============================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
