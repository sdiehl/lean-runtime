-- Collatz

def collatzStep (n : Nat) : Nat :=
  if n % 2 == 0 then n / 2 else 3 * n + 1

def collatzLen : Nat → Nat → Nat
  | 0, _ => 0
  | _, 1 => 1
  | fuel + 1, n => 1 + collatzLen fuel (collatzStep n)

def printCollatz : Nat → Nat → IO Unit
  | 0, _ => pure ()
  | fuel + 1, n => do
    IO.println s!"{n}: {collatzLen 1000 n}"
    printCollatz fuel (n + 1)

def main : IO Unit := printCollatz 20 1
