-- Test Option and Result operations from stdlib

def safeDivide (a b : Nat) : Option Nat :=
  if b == 0 then none else some (a / b)

def safeHead (xs : List α) : Option α :=
  match xs with
  | [] => none
  | x :: _ => some x

def main : IO Unit := do
  IO.println "=== Option Operations Test ==="

  -- Basic Option
  let some5 : Option Nat := some 5
  let none0 : Option Nat := none
  IO.println s!"some 5: {some5}"
  IO.println s!"none: {none0}"

  -- Option.isSome/isNone
  IO.println s!"(some 5).isSome: {some5.isSome}"
  IO.println s!"(some 5).isNone: {some5.isNone}"
  IO.println s!"none.isSome: {none0.isSome}"
  IO.println s!"none.isNone: {none0.isNone}"

  -- Option.getD (get with default)
  IO.println s!"(some 5).getD 0: {some5.getD 0}"
  IO.println s!"none.getD 0: {none0.getD 0}"

  -- Safe division
  IO.println s!"safeDivide 10 2: {safeDivide 10 2}"
  IO.println s!"safeDivide 10 0: {safeDivide 10 0}"

  -- Option.map
  let doubled := some5.map (· * 2)
  IO.println s!"(some 5).map (*2): {doubled}"
  let noneDoubled := none0.map (· * 2)
  IO.println s!"none.map (*2): {noneDoubled}"

  -- Option.bind (flatMap)
  let chainResult := (some 10).bind (fun x => safeDivide x 2)
  IO.println s!"(some 10).bind (safeDivide · 2): {chainResult}"
  let chainNone := (some 10).bind (fun x => safeDivide x 0)
  IO.println s!"(some 10).bind (safeDivide · 0): {chainNone}"

  -- Option in list context
  let list1 := [1, 2, 3, 4, 5]
  let list2 : List Nat := []
  IO.println s!"safeHead [1,2,3,4,5]: {safeHead list1}"
  IO.println s!"safeHead []: {safeHead list2}"

  -- Option.orElse
  let result := none0 <|> some 42
  IO.println s!"none <|> some 42: {result}"

  -- Nested Option
  let nested : Option (Option Nat) := some (some 7)
  IO.println s!"some (some 7): {nested}"
  let flattened := nested.bind id
  IO.println s!"Flattened: {flattened}"

  IO.println "=== Option Test Complete ==="
