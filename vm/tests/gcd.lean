-- GCD

def gcdAux : Nat → Nat → Nat → Nat
  | 0, _, b => b
  | _, a, 0 => a
  | fuel + 1, a, b => gcdAux fuel b (a % b)

def gcd (a b : Nat) : Nat := gcdAux 100 a b

def main : IO Unit := do
  IO.println s!"gcd 48 18 = {gcd 48 18}"
  IO.println s!"gcd 56 42 = {gcd 56 42}"
  IO.println s!"gcd 270 192 = {gcd 270 192}"
