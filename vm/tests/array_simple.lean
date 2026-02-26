-- Simple array test to debug allocation issue
def main : IO Unit := do
  IO.println "=== Array Simple Test ==="

  -- Basic push
  let mut arr : Array Nat := #[]
  IO.println s!"Empty size: {arr.size}"

  arr := arr.push 1
  IO.println s!"After push 1: {arr.size}"

  arr := arr.push 2
  IO.println s!"After push 2: {arr.size}"

  arr := arr.push 3
  IO.println s!"After push 3: {arr.size}"

  IO.println s!"arr[0]: {arr[0]!}"
  IO.println s!"arr[1]: {arr[1]!}"
  IO.println s!"arr[2]: {arr[2]!}"

  -- Small loop
  let mut arr2 : Array Nat := #[]
  for i in [:10] do
    arr2 := arr2.push i
  IO.println s!"After 10 pushes: {arr2.size}"

  IO.println "=== Done ==="
