-- Test Nat operations from stdlib

def factorial : Nat → Nat
  | 0 => 1
  | n + 1 => (n + 1) * factorial n

def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib (n + 1) + fib n

def gcd (a b : Nat) : Nat :=
  if h : b == 0 then a
  else gcd b (a % b)
termination_by b
decreasing_by
  simp_wf
  have : b ≠ 0 := by intro h'; simp [h'] at h
  exact Nat.mod_lt a (Nat.pos_of_ne_zero this)

-- Use imperative style to avoid termination proof complexity
def isPrime (n : Nat) : Bool :=
  if n < 2 then false
  else Id.run do
    let mut d := 2
    while d * d <= n do
      if n % d == 0 then
        return false
      d := d + 1
    return true

def main : IO Unit := do
  IO.println "=== Nat Operations Test ==="

  -- Basic arithmetic
  IO.println s!"5 + 3 = {5 + 3}"
  IO.println s!"10 - 3 = {10 - 3}"
  IO.println s!"7 * 6 = {7 * 6}"
  IO.println s!"20 / 3 = {20 / 3}"
  IO.println s!"20 % 3 = {20 % 3}"

  -- Comparison (use decide for Prop -> Bool)
  IO.println s!"5 < 10: {decide ((5 : Nat) < 10)}"
  IO.println s!"10 <= 10: {decide ((10 : Nat) <= 10)}"
  IO.println s!"5 == 5: {(5 : Nat) == 5}"
  IO.println s!"5 != 6: {(5 : Nat) != 6}"

  -- Power
  IO.println s!"2^10 = {2^10}"
  IO.println s!"3^5 = {3^5}"

  -- Min/Max
  IO.println s!"min 5 10 = {min 5 10}"
  IO.println s!"max 5 10 = {max 5 10}"

  -- Factorial
  IO.println s!"factorial 0 = {factorial 0}"
  IO.println s!"factorial 5 = {factorial 5}"
  IO.println s!"factorial 10 = {factorial 10}"

  -- Fibonacci
  IO.println s!"fib 0 = {fib 0}"
  IO.println s!"fib 10 = {fib 10}"
  IO.println s!"fib 20 = {fib 20}"

  -- GCD
  IO.println s!"gcd 48 18 = {gcd 48 18}"
  IO.println s!"gcd 100 35 = {gcd 100 35}"

  -- Prime testing
  IO.println s!"isPrime 2 = {isPrime 2}"
  IO.println s!"isPrime 17 = {isPrime 17}"
  IO.println s!"isPrime 18 = {isPrime 18}"
  IO.println s!"isPrime 97 = {isPrime 97}"

  -- Range operations
  let range10 := List.range 10
  IO.println s!"List.range 10 = {range10}"
  IO.println s!"Sum of range 10 = {range10.foldl (· + ·) 0}"

  -- Bitwise operations
  IO.println s!"5 &&& 3 = {5 &&& 3}"
  IO.println s!"5 ||| 3 = {5 ||| 3}"
  IO.println s!"5 ^^^ 3 = {5 ^^^ 3}"
  IO.println s!"8 >>> 2 = {8 >>> 2}"
  IO.println s!"2 <<< 3 = {2 <<< 3}"

  IO.println "=== Nat Test Complete ==="
