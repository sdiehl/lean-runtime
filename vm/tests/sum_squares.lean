-- Sum of squares

def sumSquares : Nat → Nat
  | 0 => 0
  | n + 1 => (n + 1) * (n + 1) + sumSquares n

def printSums : Nat → Nat → IO Unit
  | 0, _ => pure ()
  | fuel + 1, n => do
    IO.println s!"{n}: {sumSquares n}"
    printSums fuel (n + 1)

def main : IO Unit := printSums 11 1
