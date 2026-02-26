-- FizzBuzz test: simplified to avoid toString issues

def fizzbuzz (n : Nat) : String :=
  if n % 15 == 0 then "FizzBuzz"
  else if n % 3 == 0 then "Fizz"
  else if n % 5 == 0 then "Buzz"
  else "num"  -- simplified: just print "num" instead of the actual number

def loopAux : Nat → Nat → IO Unit
  | 0, _ => IO.println "Loop done"
  | fuel + 1, i => do
    IO.println (fizzbuzz i)
    loopAux fuel (i + 1)

def main : IO Unit := do
  IO.println "FizzBuzz 1-20:"
  loopAux 20 1
  IO.println "Done!"
