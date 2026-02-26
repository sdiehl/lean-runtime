-- Test algorithmic operations with stdlib

-- Merge sort implementation
def merge (xs ys : List Nat) : List Nat :=
  match xs, ys with
  | [], ys => ys
  | xs, [] => xs
  | x :: xs', y :: ys' =>
    if x <= y then x :: merge xs' (y :: ys')
    else y :: merge (x :: xs') ys'

-- partial is valid for VM execution - termination is obvious but hard to prove
partial def mergeSort (xs : List Nat) : List Nat :=
  match xs with
  | [] => []
  | [x] => [x]
  | _ =>
    let mid := xs.length / 2
    merge (mergeSort (xs.take mid)) (mergeSort (xs.drop mid))

-- Binary search with termination proof
def binarySearch (arr : Array Nat) (target : Nat) : Option Nat :=
  let rec search (lo hi : Nat) : Option Nat :=
    if h : lo >= hi then none
    else
      let mid := (lo + hi) / 2
      if arr[mid]! == target then some mid
      else if arr[mid]! < target then search (mid + 1) hi
      else search lo mid
  termination_by hi - lo
  decreasing_by
    all_goals simp_wf
    all_goals omega
  search 0 arr.size

-- Sieve of Eratosthenes
def sieve (n : Nat) : List Nat := Id.run do
  let mut isPrime := Array.replicate (n + 1) true
  if n >= 0 then isPrime := isPrime.set! 0 false
  if n >= 1 then isPrime := isPrime.set! 1 false
  for i in [2:n+1] do
    if isPrime[i]! then
      let mut j := i * 2
      while j <= n do
        isPrime := isPrime.set! j false
        j := j + i
  let mut result : List Nat := []
  for i in [2:n+1] do
    if isPrime[i]! then
      result := result ++ [i]
  return result

-- Edit distance (Levenshtein) with iterative DP
def editDistance (s t : String) : Nat := Id.run do
  let sChars := s.toList.toArray
  let tChars := t.toList.toArray
  let m := sChars.size
  let n := tChars.size

  -- Build DP table iteratively
  let mut prev : Array Nat := #[]
  for j in [0:n+1] do
    prev := prev.push j

  for i in [1:m+1] do
    let mut curr : Array Nat := #[i]
    for j in [1:n+1] do
      let cost := if sChars[i-1]! == tChars[j-1]! then 0 else 1
      let delete := prev[j]! + 1
      let insert := curr[j-1]! + 1
      let replace := prev[j-1]! + cost
      curr := curr.push (min delete (min insert replace))
    prev := curr

  return prev[n]!

def main : IO Unit := do
  IO.println "=== Algorithm Test ==="

  -- Merge sort
  IO.println ""
  IO.println "=== Merge Sort ==="
  let unsorted := [64, 34, 25, 12, 22, 11, 90]
  let sorted := mergeSort unsorted
  IO.println s!"Unsorted: {unsorted}"
  IO.println s!"Sorted: {sorted}"

  -- Large sort test
  let large := List.range 100 |>.reverse
  let sortedLarge := mergeSort large
  IO.println s!"Sorted range 0-99 first 10: {sortedLarge.take 10}"
  IO.println s!"Sorted range 0-99 last 10: {sortedLarge.drop 90}"

  -- Binary search
  IO.println ""
  IO.println "=== Binary Search ==="
  let searchArr := #[1, 3, 5, 7, 9, 11, 13, 15, 17, 19]
  IO.println s!"Array: {searchArr.toList}"
  IO.println s!"Search for 7: {binarySearch searchArr 7}"
  IO.println s!"Search for 15: {binarySearch searchArr 15}"
  IO.println s!"Search for 8: {binarySearch searchArr 8}"
  IO.println s!"Search for 1: {binarySearch searchArr 1}"
  IO.println s!"Search for 19: {binarySearch searchArr 19}"

  -- Sieve of Eratosthenes
  IO.println ""
  IO.println "=== Sieve of Eratosthenes ==="
  let primes := sieve 50
  IO.println s!"Primes up to 50: {primes}"
  IO.println s!"Count: {primes.length}"

  let primes100 := sieve 100
  IO.println s!"Primes up to 100 count: {primes100.length}"

  -- Edit distance
  IO.println ""
  IO.println "=== Edit Distance ==="
  IO.println s!"editDistance 'kitten' 'sitting': {editDistance "kitten" "sitting"}"
  IO.println s!"editDistance 'saturday' 'sunday': {editDistance "saturday" "sunday"}"
  IO.println s!"editDistance 'hello' 'hello': {editDistance "hello" "hello"}"
  IO.println s!"editDistance '' 'abc': {editDistance "" "abc"}"

  IO.println "=== Algorithm Test Complete ==="
