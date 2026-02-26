-- Debug swap test
def main : IO Unit := do
  IO.println "Creating array..."
  let arr : Array Nat := #[1, 2, 3, 4, 5]
  IO.println s!"Before: {arr.toList}"

  IO.println "Swapping indices 1 and 3..."
  let swapped := arr.swap 1 3
  IO.println s!"After: {swapped.toList}"

  IO.println "Done!"
