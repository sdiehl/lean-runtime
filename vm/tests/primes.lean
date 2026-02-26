-- Primes

def isPrimeAux : Nat → Nat → Nat → Bool
  | 0, _, _ => true
  | _, _, 1 => true
  | fuel + 1, n, d =>
    if d * d > n then true
    else if n % d == 0 then false
    else isPrimeAux fuel n (d + 1)

def isPrime (n : Nat) : Bool :=
  if n < 2 then false else isPrimeAux 100 n 2

def printPrimes : Nat → Nat → IO Unit
  | 0, _ => pure ()
  | fuel + 1, n => do
    if isPrime n then IO.println s!"{n}"
    printPrimes fuel (n + 1)

def main : IO Unit := printPrimes 30 2
