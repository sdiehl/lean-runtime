-- Nested loops

def inner : Nat → Nat → Nat → IO Unit
  | 0, _, _ => pure ()
  | fuel + 1, i, j => do
    IO.println s!"{i}*{j}={i * j}"
    inner fuel i (j + 1)

def outer : Nat → Nat → Nat → IO Unit
  | 0, _, _ => pure ()
  | fuel + 1, i, max => do
    inner max i 1
    outer fuel (i + 1) max

def main : IO Unit := outer 5 1 5
