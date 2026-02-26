-- Minimal test

def add (x y : Nat) : Nat := x + y

def main : IO Unit := do
  let _ := add 2 3
  pure ()
