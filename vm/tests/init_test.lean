-- Test using Init

def factorial : Nat → Nat
  | 0 => 1
  | n + 1 => (n + 1) * factorial n

def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib (n + 1) + fib n

-- Test List operations
def sumList : List Nat → Nat
  | [] => 0
  | x :: xs => x + sumList xs

-- Test Option
def optAdd : Option Nat → Nat → Nat
  | some x, y => x + y
  | none, y => y

def main : IO Unit := do
  -- Compute some values (no printing since that requires more externs)
  let _ := factorial 10  -- 3628800
  let _ := fib 15        -- 610
  let _ := sumList [1, 2, 3, 4, 5]  -- 15
  let _ := optAdd (some 10) 5  -- 15
  pure ()
