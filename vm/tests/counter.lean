-- Simple counter test - no string interpolation

def countDown : Nat → IO Unit
  | 0 => IO.println "Done counting!"
  | n + 1 => do
    IO.println "tick"
    countDown n

def main : IO Unit := do
  IO.println "Starting count:"
  countDown 5
  IO.println "Finished!"
