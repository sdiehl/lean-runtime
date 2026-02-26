-- Test array push in a loop
def main : IO Unit := do
  IO.println "=== Array Loop Test ==="

  -- Test with increasing sizes
  for size in [10, 50, 100, 500, 1000] do
    let mut arr : Array Nat := #[]
    for i in [0:size] do
      arr := arr.push i
    IO.println s!"Size {size}: arr.size = {arr.size}, last = {arr[arr.size - 1]!}"

  IO.println "=== Done ==="
