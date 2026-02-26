-- Fibonacci

def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib (n + 1) + fib n

def printFibs : Nat → Nat → IO Unit
  | 0, _ => pure ()
  | fuel + 1, n => do
    IO.println s!"fib {n} = {fib n}"
    printFibs fuel (n + 1)

def main : IO Unit := printFibs 16 0
