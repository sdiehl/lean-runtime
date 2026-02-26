-- Test advanced Array operations from stdlib

def main : IO Unit := do
  IO.println "=== Advanced Array Operations Test ==="

  -- Array construction
  let arr1 := #[1, 2, 3, 4, 5]
  let arr2 := Array.replicate 5 0  -- [0, 0, 0, 0, 0]
  IO.println s!"arr1 = {arr1.toList}"
  IO.println s!"Array.replicate 5 0 = {arr2.toList}"

  -- Size
  IO.println s!"arr1.size = {arr1.size}"

  -- Push and pop
  let arr3 := arr1.push 6
  IO.println s!"arr1.push 6 = {arr3.toList}"
  let arr4 := arr3.pop
  IO.println s!"(arr1.push 6).pop = {arr4.toList}"

  -- Get and set
  IO.println s!"arr1[2]! = {arr1[2]!}"
  let arr5 := arr1.set! 2 100
  IO.println s!"arr1.set! 2 100 = {arr5.toList}"

  -- Swap (using Array.swap with bounds proofs via !)
  let arr6 := (arr1.set! 0 arr1[4]!).set! 4 arr1[0]!
  IO.println s!"arr1 with 0,4 swapped = {arr6.toList}"

  -- Map
  let doubled := arr1.map (· * 2)
  IO.println s!"arr1.map (*2) = {doubled.toList}"

  -- Filter
  let evens := arr1.filter (· % 2 == 0)
  IO.println s!"arr1.filter (even) = {evens.toList}"

  -- Foldl
  let sum := arr1.foldl (· + ·) 0
  IO.println s!"arr1.foldl (+) 0 = {sum}"

  -- Any and All
  IO.println s!"arr1.any (> 3) = {arr1.any (· > 3)}"
  IO.println s!"arr1.all (> 0) = {arr1.all (· > 0)}"

  -- Contains
  IO.println s!"arr1.contains 3 = {arr1.contains 3}"
  IO.println s!"arr1.contains 10 = {arr1.contains 10}"

  -- Reverse
  IO.println s!"arr1.reverse = {arr1.reverse.toList}"

  -- Append
  let arr7 := #[6, 7, 8]
  IO.println s!"arr1 ++ #[6,7,8] = {(arr1 ++ arr7).toList}"

  -- Extract (slice)
  IO.println s!"arr1.extract 1 4 = {(arr1.extract 1 4).toList}"

  -- Sort
  let unsorted := #[3, 1, 4, 1, 5, 9, 2, 6]
  let sorted := unsorted.qsort (· < ·)
  IO.println s!"#[3,1,4,1,5,9,2,6] sorted = {sorted.toList}"

  -- Sort descending
  let sortedDesc := unsorted.qsort (· > ·)
  IO.println s!"sorted descending = {sortedDesc.toList}"

  -- Large array operations
  let mut bigArr : Array Nat := #[]
  for i in [0:1000] do
    bigArr := bigArr.push i

  IO.println s!"Big array size = {bigArr.size}"
  IO.println s!"Big array sum = {bigArr.foldl (· + ·) 0}"
  IO.println s!"Big array[500] = {bigArr[500]!}"

  -- Array of arrays
  let matrix := #[#[1, 2, 3], #[4, 5, 6], #[7, 8, 9]]
  IO.println s!"Matrix[1][1] = {matrix[1]![1]!}"

  -- findIdx
  match arr1.findIdx? (· == 3) with
  | some idx => IO.println s!"Index of 3 in arr1: {idx}"
  | none => IO.println "3 not found"

  IO.println "=== Advanced Array Test Complete ==="
