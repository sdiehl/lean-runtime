-- Test suite for monads and do-notation

def main : IO Unit := do
  IO.println "=== Monad Tests ==="

  -- Test 1: Option monad - some path
  let opt1 : Option Nat := do
    let x ← some 5
    let y ← some 3
    return x + y
  IO.println s!"option_some_do: {opt1}"  -- some 8

  -- Test 2: Option monad - none short-circuit
  let opt2 : Option Nat := do
    let x ← some 5
    let _ ← (none : Option Nat)
    let y ← some 3
    return x + y
  IO.println s!"option_none_do: {opt2}"  -- none

  -- Test 3: Option bind explicitly
  let opt3 := Option.bind (some 10) (fun x => some (x * 2))
  IO.println s!"option_bind: {opt3}"  -- some 20

  -- Test 4: Option map
  let opt4 := Option.map (· + 1) (some 5)
  IO.println s!"option_map: {opt4}"  -- some 6

  -- Test 5: List monad (use flatMap)
  let list1 : List Nat := [1, 2, 3].flatMap (fun x =>
    [10, 20].flatMap (fun y =>
      [x + y]))
  IO.println s!"list_bind: {list1}"  -- [11, 21, 12, 22, 13, 23]

  -- Test 6: List flatMap with filter
  let list2 : List Nat := [1, 2, 3, 4, 5].flatMap (fun x =>
    if x % 2 == 0 then [x] else [])
  IO.println s!"list_filter_bind: {list2}"  -- [2, 4]

  -- Test 7: Except monad - success path
  let exc1 : Except String Nat := do
    let x ← Except.ok 5
    let y ← Except.ok 3
    return x + y
  IO.println s!"except_ok: {exc1}"  -- ok 8

  -- Test 8: Except monad - error short-circuit
  let exc2 : Except String Nat := do
    let x ← Except.ok 5
    let _ ← (Except.error "oops" : Except String Nat)
    let y ← Except.ok 3
    return x + y
  IO.println s!"except_error: {exc2}"  -- error "oops"

  -- Test 9: Nested do blocks
  let nested : Option Nat := do
    let x ← some 10
    let y ← do
      let a ← some 5
      let b ← some 3
      pure (a + b)
    pure (x + y)
  IO.println s!"nested_do: {nested}"  -- some 18

  -- Test 10: Do with let (non-monadic)
  let withLet : Option Nat := do
    let x ← some 10
    let y := x * 2  -- plain let, not bind
    let z ← some 5
    return y + z
  IO.println s!"do_with_let: {withLet}"  -- some 25

  -- Test 11: Do with if
  let withIf : Option Nat := do
    let x ← some 10
    if x > 5 then
      return x * 2
    else
      return x
  IO.println s!"do_with_if: {withIf}"  -- some 20

  -- Test 12: Do with match
  let withMatch : Option Nat := do
    let x ← some (some 5)
    match x with
    | some n => return n * 2
    | none => return 0
  IO.println s!"do_with_match: {withMatch}"  -- some 10

  -- Test 13: IO monad sequencing
  IO.println "io_sequence_1"
  IO.println "io_sequence_2"
  IO.println "io_sequence_3"

  -- Test 14: IO with variable binding
  let msg := "hello"
  let upper := msg.toUpper
  IO.println s!"io_binding: {upper}"  -- HELLO

  -- Test 15: StateM monad simulation (using pairs)
  let stateLike : Nat → Nat × Nat := fun s =>
    let s1 := s + 1
    let s2 := s1 * 2
    (s2, s2)  -- (result, newState)
  let (result, finalState) := stateLike 5
  IO.println s!"state_like: result={result}, state={finalState}"  -- 12, 12

  -- Test 16: Functor map
  let mapped := Functor.map (· * 2) (some 21)
  IO.println s!"functor_map: {mapped}"  -- some 42

  -- Test 17: <$> operator
  let mapped2 := (· + 10) <$> (some 5)
  IO.println s!"map_operator: {mapped2}"  -- some 15

  -- Test 18: pure
  let pureSome : Option Nat := pure 42
  IO.println s!"pure_option: {pureSome}"  -- some 42

  -- Test 19: Applicative <*>
  let applied : Option Nat := some (· + 10) <*> some 5
  IO.println s!"applicative: {applied}"  -- some 15

  -- Test 20: Complex option chain
  let complex : Option (List Nat) := do
    let base ← some [1, 2, 3]
    let factor ← some 10
    let mapped := base.map (· * factor)
    let filtered := mapped.filter (· > 15)
    return filtered
  IO.println s!"complex_do: {complex}"  -- some [20, 30]

  IO.println "=== Monad Tests Complete ==="
