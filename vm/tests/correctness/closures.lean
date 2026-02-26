-- Test suite for closure correctness
-- Each test prints a result that can be compared against native Lean

def main : IO Unit := do
  IO.println "=== Closure Tests ==="

  -- Test 1: Simple closure capturing one variable
  let x := 10
  let f := fun y => x + y
  IO.println s!"capture_one: {f 5}"  -- Should be 15

  -- Test 2: Closure capturing multiple variables
  let a := 1
  let b := 2
  let c := 3
  let g := fun z => a + b + c + z
  IO.println s!"capture_multi: {g 4}"  -- Should be 10

  -- Test 3: Nested closures
  let outer := 100
  let makeAdder := fun x => fun y => outer + x + y
  let add5 := makeAdder 5
  IO.println s!"nested_closure: {add5 3}"  -- Should be 108

  -- Test 4: Closure returned from function
  let mkCounter : Nat → (Nat → Nat) := fun start => fun n => start + n
  let counter := mkCounter 50
  IO.println s!"returned_closure: {counter 7}"  -- Should be 57

  -- Test 5: Closure passed as argument
  let apply := fun (f : Nat → Nat) (x : Nat) => f x
  let double := fun n => n * 2
  IO.println s!"closure_as_arg: {apply double 21}"  -- Should be 42

  -- Test 6: Closure capturing mutable-like state via recursion
  let rec countdown : Nat → List Nat
    | 0 => [0]
    | n + 1 => (n + 1) :: countdown n
  IO.println s!"recursive_capture: {countdown 5}"  -- Should be [5, 4, 3, 2, 1, 0]

  -- Test 7: Higher-order function with closure
  let nums := [1, 2, 3, 4, 5]
  let factor := 10
  let scaled := nums.map (fun x => x * factor)
  IO.println s!"map_with_closure: {scaled}"  -- Should be [10, 20, 30, 40, 50]

  -- Test 8: Closure in fold
  let base := 100
  let sum := nums.foldl (fun acc x => acc + x + base) 0
  IO.println s!"fold_with_closure: {sum}"  -- Should be 515 (0 + 101 + 102 + 103 + 104 + 105)

  -- Test 9: Multiple closures sharing captured variable
  let shared := 7
  let f1 := fun x => shared + x
  let f2 := fun x => shared * x
  IO.println s!"shared_capture: {f1 3}, {f2 3}"  -- Should be 10, 21

  -- Test 10: Deeply nested closure
  let a2 := 1
  let f3 := fun b =>
    let c := a2 + b
    fun d =>
      let e := c + d
      fun f => e + f + a2
  IO.println s!"deep_nesting: {f3 2 3 4}"  -- Should be 11 (1+2=3, 3+3=6, 6+4+1=11)

  IO.println "=== Closure Tests Complete ==="
