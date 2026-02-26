-- Test suite for function calling conventions

-- Mutual recursion defined at top level
mutual
  def isEvenMut : Nat → Bool
    | 0 => true
    | n + 1 => isOddMut n
  def isOddMut : Nat → Bool
    | 0 => false
    | n + 1 => isEvenMut n
end

-- Helper function for where clause test
def compute (n : Nat) : Nat := n + 10

-- Structure for default arguments test (must be at top level)
structure Config where
  width : Nat := 80
  height : Nat := 24

def main : IO Unit := do
  IO.println "=== Function Calling Tests ==="

  -- Test 1: Simple function call
  let add := fun x y => x + y
  IO.println s!"simple_call: {add 3 4}"  -- 7

  -- Test 2: Partial application (currying)
  let add3 := add 3
  IO.println s!"partial_app: {add3 7}"  -- 10

  -- Test 3: Multiple partial applications
  let f := fun a b c d => a + b + c + d
  let f1 := f 1
  let f2 := f1 2
  let f3 := f2 3
  IO.println s!"multi_partial: {f3 4}"  -- 10

  -- Test 4: Function with many arguments
  let sum6 := fun a b c d e f => a + b + c + d + e + f
  IO.println s!"many_args: {sum6 1 2 3 4 5 6}"  -- 21

  -- Test 5: Recursive function
  let rec factorial : Nat → Nat
    | 0 => 1
    | n + 1 => (n + 1) * factorial n
  IO.println s!"factorial_5: {factorial 5}"  -- 120
  IO.println s!"factorial_10: {factorial 10}"  -- 3628800

  -- Test 6: Tail recursive function
  let rec factorialTR (n : Nat) (acc : Nat) : Nat :=
    match n with
    | 0 => acc
    | n + 1 => factorialTR n ((n + 1) * acc)
  IO.println s!"factorial_tr: {factorialTR 10 1}"  -- 3628800

  -- Test 7: Mutual recursion
  IO.println s!"mutual_even_10: {isEvenMut 10}"  -- true
  IO.println s!"mutual_odd_7: {isOddMut 7}"  -- true
  IO.println s!"mutual_even_7: {isEvenMut 7}"  -- false

  -- Test 8: Higher-order functions
  let twice := fun (f : Nat → Nat) x => f (f x)
  let inc := fun x => x + 1
  IO.println s!"higher_order: {twice inc 5}"  -- 7

  let thrice := fun (f : Nat → Nat) x => f (f (f x))
  IO.println s!"thrice: {thrice inc 0}"  -- 3

  -- Test 9: Function composition
  let compose := fun (g : Nat → Nat) (f : Nat → Nat) x => g (f x)
  let double := fun x => x * 2
  let square := fun x => x * x
  let doubleThenSquare := compose square double
  IO.println s!"compose: {doubleThenSquare 3}"  -- 36 (3*2=6, 6*6=36)

  -- Test 10: Anonymous functions inline
  IO.println s!"anon_inline: {(fun x => x * x) 7}"  -- 49

  -- Test 11: Let-bound functions
  let localFunc := fun x =>
    let helper := fun y => y + 1
    helper (helper x)
  IO.println s!"let_func: {localFunc 10}"  -- 12

  -- Test 12: Functions returning functions
  let makeMultiplier : Nat → (Nat → Nat) := fun n => fun x => n * x
  let mult5 := makeMultiplier 5
  let mult10 := makeMultiplier 10
  IO.println s!"func_return_func: {mult5 7}, {mult10 7}"  -- 35, 70

  -- Test 13: Polymorphic function (identity)
  let id : {α : Type} → α → α := fun x => x
  IO.println s!"poly_id_nat: {id 42}"  -- 42
  IO.println s!"poly_id_string: {id "hello"}"  -- hello

  -- Test 14: Polymorphic function (const)
  let const : {α β : Type} → α → β → α := fun x _ => x
  IO.println s!"poly_const: {const 1 2}"  -- 1

  -- Test 15: Deeply nested function calls
  let g := fun x => x + 1
  IO.println s!"deep_calls: {g (g (g (g (g (g (g (g (g (g 0)))))))))}"  -- 10

  -- Test 16: Large arity partial application
  let f7 := fun a b c d e f g => a + b + c + d + e + f + g
  let p1 := f7 1
  let p2 := p1 2
  let p3 := p2 3
  let p4 := p3 4
  let p5 := p4 5
  let p6 := p5 6
  IO.println s!"large_arity_partial: {p6 7}"  -- 28

  -- Test 17: Default arguments (via structure)
  let cfg1 : Config := {}
  let cfg2 : Config := { width := 100 }
  IO.println s!"default_args: {cfg1.width}, {cfg2.width}"  -- 80, 100

  -- Test 18: Using helper function
  let result := compute 5 * 2
  IO.println s!"helper_func: {result}"  -- 30

  IO.println "=== Function Calling Tests Complete ==="
