-- Test suite for recursion patterns

-- Tree type for tree recursion test
inductive RTree where
  | leaf : Nat → RTree
  | node : RTree → RTree → RTree

-- Partial functions moved to top level (can't prove termination)
partial def collatz : Nat → Nat
  | 1 => 0
  | n => if n % 2 == 0 then 1 + collatz (n / 2) else 1 + collatz (3 * n + 1)

partial def range (start stop : Nat) : List Nat :=
  if start >= stop then []
  else start :: range (start + 1) stop

partial def gcd (a b : Nat) : Nat :=
  if b == 0 then a else gcd b (a % b)

def main : IO Unit := do
  IO.println "=== Recursion Tests ==="

  -- Test 1: Simple recursion
  let rec sumTo : Nat → Nat
    | 0 => 0
    | n + 1 => (n + 1) + sumTo n
  IO.println s!"sum_to_10: {sumTo 10}"  -- 55

  -- Test 2: Tail recursion with accumulator
  let rec sumToTR (n : Nat) (acc : Nat) : Nat :=
    match n with
    | 0 => acc
    | n + 1 => sumToTR n (acc + n + 1)
  IO.println s!"sum_to_tr: {sumToTR 100 0}"  -- 5050

  -- Test 3: Fibonacci (exponential)
  let rec fib : Nat → Nat
    | 0 => 0
    | 1 => 1
    | n + 2 => fib n + fib (n + 1)
  IO.println s!"fib_10: {fib 10}"  -- 55
  IO.println s!"fib_15: {fib 15}"  -- 610

  -- Test 4: Fibonacci (tail recursive)
  let rec fibTR (n : Nat) (a b : Nat) : Nat :=
    match n with
    | 0 => a
    | n + 1 => fibTR n b (a + b)
  IO.println s!"fib_tr_20: {fibTR 20 0 1}"  -- 6765

  -- Test 5: List length (structural recursion)
  let rec listLen : List Nat → Nat
    | [] => 0
    | _ :: xs => 1 + listLen xs
  IO.println s!"list_len: {listLen [1,2,3,4,5]}"  -- 5

  -- Test 6: List reverse (tail recursive)
  let rec reverseTR (xs : List Nat) (acc : List Nat) : List Nat :=
    match xs with
    | [] => acc
    | x :: rest => reverseTR rest (x :: acc)
  IO.println s!"reverse_tr: {reverseTR [1,2,3,4,5] []}"  -- [5,4,3,2,1]

  -- Test 7: Ackermann function (deeply recursive)
  let rec ack : Nat → Nat → Nat
    | 0, n => n + 1
    | m + 1, 0 => ack m 1
    | m + 1, n + 1 => ack m (ack (m + 1) n)
  IO.println s!"ack_2_3: {ack 2 3}"  -- 9
  IO.println s!"ack_3_2: {ack 3 2}"  -- 29

  -- Test 8: Tree recursion
  let rec rtreeSum : RTree → Nat
    | RTree.leaf n => n
    | RTree.node l r => rtreeSum l + rtreeSum r

  let tree := RTree.node
    (RTree.node (RTree.leaf 1) (RTree.leaf 2))
    (RTree.node (RTree.leaf 3) (RTree.leaf 4))
  IO.println s!"tree_sum: {rtreeSum tree}"  -- 10

  -- Test 9: Multiple recursive calls
  let rec countPaths : Nat → Nat → Nat
    | 0, _ => 1
    | _, 0 => 1
    | m + 1, n + 1 => countPaths m (n + 1) + countPaths (m + 1) n
  IO.println s!"count_paths_3_3: {countPaths 3 3}"  -- 20

  -- Test 10: Recursion with conditions (uses top-level partial def)
  IO.println s!"collatz_27: {collatz 27}"  -- 111

  -- Test 11: Recursion building a list (uses top-level partial def)
  IO.println s!"range: {range 0 5}"  -- [0,1,2,3,4]

  -- Test 12: Recursion with multiple accumulators
  let rec fibPair (n : Nat) : Nat × Nat :=
    match n with
    | 0 => (0, 1)
    | n + 1 =>
      let (a, b) := fibPair n
      (b, a + b)
  IO.println s!"fib_pair_10: {(fibPair 10).1}"  -- 55

  -- Test 13: GCD (Euclidean algorithm, uses top-level partial def)
  IO.println s!"gcd_48_18: {gcd 48 18}"  -- 6
  IO.println s!"gcd_1071_462: {gcd 1071 462}"  -- 21

  -- Test 14: Power function
  let rec power (base exp : Nat) : Nat :=
    match exp with
    | 0 => 1
    | n + 1 => base * power base n
  IO.println s!"power_2_10: {power 2 10}"  -- 1024

  -- Test 15: Fast exponentiation
  let rec fastPow (base exp acc : Nat) : Nat :=
    match exp with
    | 0 => acc
    | n + 1 =>
      if (n + 1) % 2 == 0
      then fastPow (base * base) ((n + 1) / 2) acc
      else fastPow base n (acc * base)
  IO.println s!"fast_pow_2_10: {fastPow 2 10 1}"  -- 1024

  IO.println "=== Recursion Tests Complete ==="
