-- Debug sort test
import Std

def main : IO Unit := do
  IO.println "Creating array..."
  let arr : Array Nat := #[5, 3, 8, 1, 9, 2, 7, 4, 6, 0]
  IO.println s!"Before: {arr.toList}"

  IO.println "Sorting..."
  let sorted := arr.qsort (· < ·)
  IO.println s!"After: {sorted.toList}"

  IO.println "Done!"
