-- Compare different range syntaxes
def main : IO Unit := do
  IO.println "=== Range Syntax Comparison ==="

  -- This syntax uses a simpler iterator
  let mut arr1 : Array Nat := #[]
  for i in [:20] do
    arr1 := arr1.push i
  IO.println s!"[:20] syntax: size = {arr1.size}"

  -- This syntax might use more complex structure
  let mut arr2 : Array Nat := #[]
  let n := 20
  for i in [0:n] do
    arr2 := arr2.push i
  IO.println s!"[0:n] syntax: size = {arr2.size}"

  IO.println "=== Done ==="
