-- Tiny sort test
def main : IO Unit := do
  let arr : Array Nat := #[3, 1, 2]
  IO.println s!"Before: {arr.toList}"
  let sorted := arr.qsort (· < ·)
  IO.println s!"After: {sorted.toList}"
