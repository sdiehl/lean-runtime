-- Comprehensive Init test

-- Test Option
def testOption : IO Unit := do
  let some_val : Option Nat := some 42
  let none_val : Option Nat := none
  match some_val with
  | some n => IO.println s!"Option some: {n}"
  | none => IO.println "Option none"
  match none_val with
  | some _ => IO.println "unexpected"
  | none => IO.println "Option none: ok"

-- Test List operations
def testList : IO Unit := do
  let xs := [1, 2, 3, 4, 5]
  IO.println s!"List length: {xs.length}"
  IO.println s!"List sum: {xs.foldl (· + ·) 0}"

-- Test Bool
def testBool : IO Unit := do
  let t := true
  let f := false
  if t && f then IO.println "t&&f = true" else IO.println "t&&f = false"
  if t || f then IO.println "t||f = true" else IO.println "t||f = false"
  if !t then IO.println "!t = true" else IO.println "!t = false"

-- Test comparison
def testCmp : IO Unit := do
  if 5 < 10 then IO.println "5 < 10" else IO.println "5 >= 10"
  if 10 < 5 then IO.println "10 < 5" else IO.println "10 >= 5"
  if 5 == 5 then IO.println "5 == 5" else IO.println "5 != 5"

-- Test String operations
def testString : IO Unit := do
  let s := "hello"
  IO.println s!"String length: {s.length}"
  let s2 := s ++ " world"
  IO.println s!"Concat: {s2}"

-- Test Char
def testChar : IO Unit := do
  let c := 'A'
  IO.println s!"Char: {c}"

def main : IO Unit := do
  IO.println "=== Init Test ==="
  testOption
  testList
  testBool
  testCmp
  testString
  testChar
  IO.println "=== Done ==="
