-- Full Init test

-- Array operations
def testArray : IO Unit := do
  let arr := #[1, 2, 3, 4, 5]
  IO.println s!"Array size: {arr.size}"
  IO.println s!"Array[2]: {arr[2]!}"
  let arr2 := arr.push 6
  IO.println s!"After push: {arr2.size}"

-- More List operations
def testListOps : IO Unit := do
  let xs := [10, 20, 30]
  let ys := [40, 50]
  let zs := xs ++ ys
  IO.println s!"List append length: {zs.length}"
  match xs.head? with
  | some h => IO.println s!"List head: {h}"
  | none => IO.println "empty"
  IO.println s!"List reverse length: {xs.reverse.length}"

-- Int operations
def testInt : IO Unit := do
  let a : Int := 10
  let b : Int := -5
  IO.println s!"Int add: {a + b}"
  IO.println s!"Int sub: {a - b}"
  IO.println s!"Int mul: {a * b}"

-- More String operations
def testStringOps : IO Unit := do
  let s := "Hello, World!"
  IO.println s!"String: {s}"
  IO.println s!"Uppercase first: {s.take 1}"

-- Prod/pair
def testProd : IO Unit := do
  let p := (1, "hello")
  IO.println s!"Pair fst: {p.1}"
  IO.println s!"Pair snd: {p.2}"

-- If-then-else chains
def testIfElse : IO Unit := do
  let x := 15
  if x < 10 then
    IO.println "x < 10"
  else if x < 20 then
    IO.println "10 <= x < 20"
  else
    IO.println "x >= 20"

def main : IO Unit := do
  IO.println "=== Full Init Test ==="
  testArray
  testListOps
  testInt
  testStringOps
  testProd
  testIfElse
  IO.println "=== All Done ==="
