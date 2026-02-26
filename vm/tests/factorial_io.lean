-- Factorial with IO output

def factorial : Nat → Nat
  | 0 => 1
  | n + 1 => (n + 1) * factorial n

def printFactorials : Nat → Nat → IO Unit
  | 0, _ => pure ()
  | fuel + 1, n => do
    IO.println s!"{n}! = {factorial n}"
    printFactorials fuel (n + 1)

def main : IO Unit := do
  printFactorials 11 0
