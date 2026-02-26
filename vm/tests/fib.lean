-- Fibonacci test (simplified - no IO)

def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib (n + 1) + fib n

def main : IO Unit := do
  let _ := fib 10  -- should be 55
  pure ()
