-- Find threshold where array push fails
def main : IO Unit := do
  let mut arr : Array Nat := #[]
  for i in [:43] do
    arr := arr.push i
  IO.println s!"Done: {arr.size}"
