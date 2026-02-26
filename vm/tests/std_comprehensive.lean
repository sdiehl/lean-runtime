-- Comprehensive test of Std data structures
import Std

def testHashMap : IO Unit := do
  IO.println "=== HashMap Test ==="
  let mut m : Std.HashMap String Nat := {}

  -- Insert many values
  for i in [0:1000] do
    m := m.insert s!"key{i}" i

  IO.println s!"Size after 1000 inserts: {m.size}"

  -- Lookup
  match m.get? "key500" with
  | some v => IO.println s!"get 'key500': {v}"
  | none => IO.println "get 'key500': not found"

  -- Erase half
  for i in [0:500] do
    m := m.erase s!"key{i}"

  IO.println s!"Size after 500 erases: {m.size}"

def testHashSet : IO Unit := do
  IO.println "=== HashSet Test ==="
  let mut s : Std.HashSet Nat := {}

  -- Insert many values
  for i in [0:1000] do
    s := s.insert i

  IO.println s!"Size after 1000 inserts: {s.size}"
  IO.println s!"contains 500: {s.contains 500}"
  IO.println s!"contains 2000: {s.contains 2000}"

def testArray : IO Unit := do
  IO.println "=== Array Test ==="
  let mut arr : Array Nat := #[]

  -- Push many values
  for i in [0:10000] do
    arr := arr.push i

  IO.println s!"Size after 10000 pushes: {arr.size}"
  IO.println s!"arr[5000]: {arr[5000]!}"

  -- Sum all elements
  let mut sum := 0
  for x in arr do
    sum := sum + x
  IO.println s!"Sum of 0..9999: {sum}"

def testList : IO Unit := do
  IO.println "=== List Test ==="
  let mut lst : List Nat := []

  -- Build list
  for i in [0:1000] do
    lst := i :: lst

  IO.println s!"Length: {lst.length}"
  IO.println s!"Head: {lst.head!}"

  -- Reverse
  let rev := lst.reverse
  IO.println s!"After reverse, head: {rev.head!}"

def testNestedStructures : IO Unit := do
  IO.println "=== Nested Structures Test ==="
  -- HashMap of Arrays
  let mut m : Std.HashMap String (Array Nat) := {}

  for i in [0:100] do
    let mut arr : Array Nat := #[]
    for j in [0:100] do
      arr := arr.push (i * 100 + j)
    m := m.insert s!"arr{i}" arr

  IO.println s!"HashMap size: {m.size}"

  match m.get? "arr50" with
  | some arr => IO.println s!"arr50 size: {arr.size}, arr50[50]: {arr[50]!}"
  | none => IO.println "arr50 not found"

def main : IO Unit := do
  IO.println "=== Std Comprehensive Test ==="
  IO.println ""

  testHashMap
  IO.println ""

  testHashSet
  IO.println ""

  testArray
  IO.println ""

  testList
  IO.println ""

  testNestedStructures
  IO.println ""

  IO.println "=== All Tests Complete ==="
