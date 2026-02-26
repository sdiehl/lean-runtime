-- Test basic array operations
def main : IO Unit := do
  IO.println "=== Array Test ==="

  -- Create an array
  let arr := #[1, 2, 3, 4, 5]
  IO.println s!"Array size: {arr.size}"
  IO.println s!"arr[0]: {arr[0]!}"
  IO.println s!"arr[2]: {arr[2]!}"

  -- Push
  let arr2 := arr.push 6
  IO.println s!"After push: {arr2.size}"

  -- Array with default
  let arr3 : Array Nat := #[]
  IO.println s!"Empty array size: {arr3.size}"

  IO.println "=== Done ==="
