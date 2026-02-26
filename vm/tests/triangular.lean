-- Triangular numbers

def triangular : Nat → Nat
  | 0 => 0
  | n + 1 => (n + 1) + triangular n

def printT : Nat → Nat → IO Unit
  | 0, _ => pure ()
  | fuel + 1, n => do
    IO.println s!"{n}: {triangular n}"
    printT fuel (n + 1)

def main : IO Unit := printT 16 0
