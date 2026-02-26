-- Test monadic operations and do notation

def safeDivide (a b : Nat) : Option Nat :=
  if b == 0 then none else some (a / b)

def safeIndex (arr : Array Nat) (i : Nat) : Option Nat := do
  if i < arr.size then
    return arr[i]!
  else
    none

def processData (data : List Nat) : Option Nat := do
  guard !data.isEmpty
  let first ← data.head?
  let last ← data.getLast?
  return first + last

def main : IO Unit := do
  IO.println "=== Monad Operations Test ==="

  -- Option monad
  let result1 := safeDivide 20 5
  IO.println s!"safeDivide 20 5 = {result1}"

  let result2 := safeDivide 20 0
  IO.println s!"safeDivide 20 0 = {result2}"

  -- Safe indexing
  let arr := #[10, 20, 30, 40, 50]
  IO.println s!"safeIndex arr 2 = {safeIndex arr 2}"
  IO.println s!"safeIndex arr 10 = {safeIndex arr 10}"

  -- Guard in Option
  IO.println s!"processData [1,2,3,4,5] = {processData [1,2,3,4,5]}"
  IO.println s!"processData [] = {processData []}"

  -- IO monad with mutable state
  let mut counter := 0
  for _ in [0:10] do
    counter := counter + 1
  IO.println s!"Counter after 10 iterations: {counter}"

  -- Nested do blocks
  let result3 := do
    let x ← some 10
    let y ← some 20
    let z ← if x > 5 then some (x + y) else none
    return z * 2
  IO.println s!"Nested do result: {result3}"

  -- Chained Option operations
  let chain := (some 5).bind (fun x =>
    (some 3).bind (fun y =>
      some (x * y)))
  IO.println s!"Chained bind: {chain}"

  -- Using <$> (map)
  let mapped := (· + 1) <$> some 10
  IO.println s!"(+1) <$> some 10 = {mapped}"

  -- OrElse for Options
  let opt1 : Option Nat := none
  let opt2 : Option Nat := some 42
  let result4 := opt1 <|> opt2
  IO.println s!"none <|> some 42 = {result4}"

  -- Sequence of Options
  let opts := [some 1, some 2, some 3]
  let sequenced := opts.mapM id
  IO.println s!"sequence [some 1, some 2, some 3] = {sequenced}"

  let opts2 := [some 1, none, some 3]
  let sequenced2 := opts2.mapM id
  IO.println s!"sequence [some 1, none, some 3] = {sequenced2}"

  IO.println "=== Monad Test Complete ==="
