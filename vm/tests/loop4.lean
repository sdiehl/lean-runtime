-- Simple 4-iteration loop test

def loop : Nat → IO Unit
  | 0 => IO.println "Loop end"
  | n + 1 => do
    IO.println "tick"
    loop n

def main : IO Unit := do
  IO.println "Start"
  loop 4
  IO.println "Done"
