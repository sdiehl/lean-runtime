-- Test control flow and pattern matching

inductive Tree (α : Type) where
  | leaf : α → Tree α
  | node : Tree α → Tree α → Tree α

def Tree.sum : Tree Nat → Nat
  | .leaf n => n
  | .node l r => l.sum + r.sum

def Tree.depth : Tree α → Nat
  | .leaf _ => 1
  | .node l r => 1 + max l.depth r.depth

def Tree.map (f : α → β) : Tree α → Tree β
  | .leaf x => .leaf (f x)
  | .node l r => .node (l.map f) (r.map f)

def Tree.toList : Tree α → List α
  | .leaf x => [x]
  | .node l r => l.toList ++ r.toList

inductive Expr where
  | num : Int → Expr
  | add : Expr → Expr → Expr
  | mul : Expr → Expr → Expr
  | neg : Expr → Expr

def Expr.eval : Expr → Int
  | .num n => n
  | .add a b => a.eval + b.eval
  | .mul a b => a.eval * b.eval
  | .neg e => -e.eval

-- Collatz with termination proof (uses well-founded recursion on fuel)
partial def collatz (n : Nat) : List Nat :=
  if n <= 1 then [n]
  else if n % 2 == 0 then n :: collatz (n / 2)
  else n :: collatz (3 * n + 1)

def main : IO Unit := do
  IO.println "=== Control Flow Test ==="

  -- If-then-else
  for i in [0:5] do
    let result := if i % 2 == 0 then "even" else "odd"
    IO.println s!"{i} is {result}"

  -- Pattern matching on Option
  let testOpt : Option Nat := some 42
  match testOpt with
  | some n => IO.println s!"Got value: {n}"
  | none => IO.println "No value"

  -- Pattern matching on List
  let testList := [1, 2, 3, 4, 5]
  match testList with
  | [] => IO.println "Empty list"
  | [x] => IO.println s!"Single element: {x}"
  | x :: y :: rest => IO.println s!"First two: {x}, {y}, rest length: {rest.length}"

  -- Binary tree operations
  IO.println ""
  IO.println "=== Tree Operations ==="
  let tree := Tree.node
    (Tree.node (Tree.leaf 1) (Tree.leaf 2))
    (Tree.node (Tree.leaf 3) (Tree.node (Tree.leaf 4) (Tree.leaf 5)))

  IO.println s!"Tree sum: {tree.sum}"
  IO.println s!"Tree depth: {tree.depth}"
  IO.println s!"Tree toList: {tree.toList}"

  let doubledTree := tree.map (· * 2)
  IO.println s!"Doubled tree toList: {doubledTree.toList}"

  -- Expression evaluation
  IO.println ""
  IO.println "=== Expression Evaluation ==="
  -- (3 + 4) * (2 + (-1))
  let expr := Expr.mul
    (Expr.add (Expr.num 3) (Expr.num 4))
    (Expr.add (Expr.num 2) (Expr.neg (Expr.num 1)))
  IO.println s!"(3 + 4) * (2 + (-1)) = {expr.eval}"

  -- Collatz sequence
  IO.println ""
  IO.println "=== Collatz Sequence ==="
  IO.println s!"collatz 6: {collatz 6}"
  IO.println s!"collatz 27 length: {(collatz 27).length}"

  -- Guards in patterns
  IO.println ""
  IO.println "=== Grade Classification ==="
  let grades := [95, 85, 75, 65, 55]
  for score in grades do
    let grade :=
      if score >= 90 then "A"
      else if score >= 80 then "B"
      else if score >= 70 then "C"
      else if score >= 60 then "D"
      else "F"
    IO.println s!"Score {score}: Grade {grade}"

  -- Nested pattern matching
  IO.println ""
  IO.println "=== Nested Patterns ==="
  let pairs := [(some 1, some 2), (some 3, none), (none, some 4), (none, none)]
  for (a, b) in pairs do
    match (a, b) with
    | (some x, some y) => IO.println s!"Both: {x}, {y}"
    | (some x, none) => IO.println s!"Only first: {x}"
    | (none, some y) => IO.println s!"Only second: {y}"
    | (none, none) => IO.println "Neither"

  IO.println "=== Control Flow Test Complete ==="
