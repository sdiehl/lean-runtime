-- Test larger array sizes
def main : IO Unit := do
  IO.println "=== Array Size Test ==="

  let mut arr : Array Nat := #[]
  for i in [:100] do
    arr := arr.push i
  IO.println s!"100 elements: size = {arr.size}, last = {arr[99]!}"

  let mut arr2 : Array Nat := #[]
  for i in [:500] do
    arr2 := arr2.push i
  IO.println s!"500 elements: size = {arr2.size}, last = {arr2[499]!}"

  let mut arr3 : Array Nat := #[]
  for i in [:1000] do
    arr3 := arr3.push i
  IO.println s!"1000 elements: size = {arr3.size}, last = {arr3[999]!}"

  IO.println "=== Done ==="
