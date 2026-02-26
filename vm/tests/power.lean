-- Power

def power : Nat → Nat → Nat
  | _, 0 => 1
  | base, exp + 1 => base * power base exp

def main : IO Unit := do
  IO.println s!"{power 10 3}"
  IO.println s!"{power 10 4}"
