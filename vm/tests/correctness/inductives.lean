-- Test suite for inductive types and pattern matching

-- Custom inductive: Binary tree
inductive Tree (α : Type) where
  | leaf : Tree α
  | node : α → Tree α → Tree α → Tree α
deriving Repr

-- Custom inductive: Simple enum
inductive Color where
  | red | green | blue
deriving Repr

-- Custom inductive: With multiple fields
inductive Person where
  | mk : String → Nat → Person
deriving Repr

-- Recursive inductive: Peano naturals
inductive Peano where
  | zero : Peano
  | succ : Peano → Peano
deriving Repr

-- Expression tree for evaluation test
inductive Expr where
  | num : Nat → Expr
  | add : Expr → Expr → Expr
  | mul : Expr → Expr → Expr
  | neg : Expr → Expr

def eval : Expr → Int
  | Expr.num n => n
  | Expr.add a b => eval a + eval b
  | Expr.mul a b => eval a * eval b
  | Expr.neg e => -(eval e)

def main : IO Unit := do
  IO.println "=== Inductive Type Tests ==="

  -- Test 1: Option type (built-in)
  let opt1 : Option Nat := some 42
  let opt2 : Option Nat := none
  IO.println s!"option_some: {opt1}"
  IO.println s!"option_none: {opt2}"

  -- Test 2: Pattern match on Option
  let getValue : Option Nat → Nat
    | some x => x
    | none => 0
  IO.println s!"option_match_some: {getValue opt1}"
  IO.println s!"option_match_none: {getValue opt2}"

  -- Test 3: List constructors
  let list1 : List Nat := []
  let list2 : List Nat := [1, 2, 3]
  let list3 : List Nat := 0 :: list2
  IO.println s!"list_empty: {list1}"
  IO.println s!"list_literal: {list2}"
  IO.println s!"list_cons: {list3}"

  -- Test 4: Pattern match on List
  let head : List Nat → Nat
    | [] => 0
    | x :: _ => x
  let tail : List Nat → List Nat
    | [] => []
    | _ :: xs => xs
  IO.println s!"list_head: {head list2}"
  IO.println s!"list_tail: {tail list2}"

  -- Test 5: Nested pattern match
  let secondOrZero : List Nat → Nat
    | _ :: x :: _ => x
    | _ => 0
  IO.println s!"nested_pattern: {secondOrZero [1,2,3]}"
  IO.println s!"nested_pattern_short: {secondOrZero [1]}"

  -- Test 6: Custom enum
  let c1 := Color.red
  let c2 := Color.green
  let colorToNat : Color → Nat
    | Color.red => 0
    | Color.green => 1
    | Color.blue => 2
  IO.println s!"enum_red: {colorToNat c1}"
  IO.println s!"enum_green: {colorToNat c2}"

  -- Test 7: Custom struct-like inductive
  let p := Person.mk "Alice" 30
  let getName : Person → String
    | Person.mk name _ => name
  let getAge : Person → Nat
    | Person.mk _ age => age
  IO.println s!"struct_name: {getName p}"
  IO.println s!"struct_age: {getAge p}"

  -- Test 8: Binary tree
  let tree := Tree.node 1
    (Tree.node 2 Tree.leaf Tree.leaf)
    (Tree.node 3 Tree.leaf Tree.leaf)

  let rec treeSum : Tree Nat → Nat
    | Tree.leaf => 0
    | Tree.node v l r => v + treeSum l + treeSum r
  IO.println s!"tree_sum: {treeSum tree}"  -- Should be 6

  let rec treeSize : Tree Nat → Nat
    | Tree.leaf => 0
    | Tree.node _ l r => 1 + treeSize l + treeSize r
  IO.println s!"tree_size: {treeSize tree}"  -- Should be 3

  -- Test 9: Peano naturals
  let p0 := Peano.zero
  let p1 := Peano.succ p0
  let p2 := Peano.succ p1
  let p3 := Peano.succ p2

  let rec peanoToNat : Peano → Nat
    | Peano.zero => 0
    | Peano.succ n => 1 + peanoToNat n
  IO.println s!"peano_0: {peanoToNat p0}"
  IO.println s!"peano_3: {peanoToNat p3}"

  let rec peanoAdd : Peano → Peano → Peano
    | Peano.zero, n => n
    | Peano.succ m, n => Peano.succ (peanoAdd m n)
  IO.println s!"peano_add: {peanoToNat (peanoAdd p2 p3)}"  -- Should be 5

  -- Test 10: Expression evaluation
  let expr := Expr.add (Expr.num 3) (Expr.mul (Expr.num 4) (Expr.num 5))
  IO.println s!"expr_eval: {eval expr}"  -- Should be 23

  -- Test 11: Bool (built-in inductive)
  let b1 := true
  let b2 := false
  let boolToNat : Bool → Nat
    | true => 1
    | false => 0
  IO.println s!"bool_true: {boolToNat b1}"
  IO.println s!"bool_false: {boolToNat b2}"

  -- Test 12: Prod (pairs)
  let pair := (42, "hello")
  IO.println s!"pair_fst: {pair.1}"
  IO.println s!"pair_snd: {pair.2}"

  -- Test 13: Sum type
  let sum1 : Sum Nat String := Sum.inl 42
  let sum2 : Sum Nat String := Sum.inr "hello"
  let sumToString : Sum Nat String → String
    | Sum.inl n => s!"Left: {n}"
    | Sum.inr s => s!"Right: {s}"
  IO.println s!"sum_left: {sumToString sum1}"
  IO.println s!"sum_right: {sumToString sum2}"

  IO.println "=== Inductive Type Tests Complete ==="
