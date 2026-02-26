-- Stress test for Std library
import Std

def testLargeArray : IO Unit := do
  IO.println "=== Large Array Test ==="
  let mut arr : Array Nat := #[]
  -- Push 100k elements
  for i in [:100000] do
    arr := arr.push i
  IO.println s!"Size: {arr.size}"
  IO.println s!"Last: {arr[99999]!}"

  -- Sum using fold
  let sum := arr.foldl (· + ·) 0
  IO.println s!"Sum: {sum}"

def testArraySort : IO Unit := do
  IO.println "=== Array Sort Test ==="
  let mut arr : Array Nat := #[]
  -- Create reverse-sorted array
  for i in [:10000] do
    arr := arr.push (10000 - i)
  IO.println s!"Before sort, first 5: {arr[0]!}, {arr[1]!}, {arr[2]!}, {arr[3]!}, {arr[4]!}"

  -- Sort it
  let sorted := arr.qsort (· < ·)
  IO.println s!"After sort, first 5: {sorted[0]!}, {sorted[1]!}, {sorted[2]!}, {sorted[3]!}, {sorted[4]!}"
  IO.println s!"After sort, last: {sorted[9999]!}"

def testLargeHashMap : IO Unit := do
  IO.println "=== Large HashMap Test ==="
  let mut m : Std.HashMap Nat String := {}

  -- Insert 50k entries
  for i in [:50000] do
    m := m.insert i s!"value_{i}"
  IO.println s!"Size after 50k inserts: {m.size}"

  -- Lookup some values
  match m.get? 25000 with
  | some v => IO.println s!"get 25000: {v}"
  | none => IO.println "get 25000: not found"

  -- Erase half
  for i in [:25000] do
    m := m.erase i
  IO.println s!"Size after 25k erases: {m.size}"

def testListOperations : IO Unit := do
  IO.println "=== List Operations Test ==="
  -- Build a list
  let mut lst : List Nat := []
  for i in [:5000] do
    lst := i :: lst
  IO.println s!"Length: {lst.length}"

  -- Reverse
  let rev := lst.reverse
  IO.println s!"Reversed head: {rev.head!}"

  -- Map
  let doubled := lst.map (· * 2)
  IO.println s!"Doubled head: {doubled.head!}"

  -- Filter
  let evens := lst.filter (· % 2 == 0)
  IO.println s!"Evens length: {evens.length}"

def testNestedLoops : IO Unit := do
  IO.println "=== Nested Loops Test ==="
  let mut total := 0
  for i in [:100] do
    for j in [:100] do
      for k in [:100] do
        total := total + 1
  IO.println s!"Total iterations: {total}"

def testStringOperations : IO Unit := do
  IO.println "=== String Operations Test ==="
  let mut s := ""
  for i in [:1000] do
    s := s ++ s!"{i},"
  IO.println s!"String length: {s.length}"
  IO.println s!"First 50 chars: {s.take 50}"

def main : IO Unit := do
  IO.println "=========================================="
  IO.println "         STD LIBRARY STRESS TEST         "
  IO.println "=========================================="
  IO.println ""

  testLargeArray
  IO.println ""

  testArraySort
  IO.println ""

  testLargeHashMap
  IO.println ""

  testListOperations
  IO.println ""

  testNestedLoops
  IO.println ""

  testStringOperations
  IO.println ""

  IO.println "=========================================="
  IO.println "         ALL STRESS TESTS PASSED         "
  IO.println "=========================================="
