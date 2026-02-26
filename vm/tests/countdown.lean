-- Countdown

def countdown : Nat → IO Unit
  | 0 => IO.println "done"
  | n + 1 => do
    IO.println s!"{n + 1}"
    countdown n

def main : IO Unit := countdown 10
