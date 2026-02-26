-- Minimal array test
def main : IO Unit := do
  let mut arr : Array Nat := #[]
  for i in [:30] do
    arr := arr.push i
  IO.println s!"Done: {arr.size}"
