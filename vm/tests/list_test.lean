-- Test using Init with List operations

def sumList : List Nat → Nat
  | [] => 0
  | x :: xs => x + sumList xs

def lengthList : List Nat → Nat
  | [] => 0
  | _ :: xs => 1 + lengthList xs

def main : IO Unit := do
  IO.println "Testing List operations..."

  -- Create a list
  let myList := [1, 2, 3, 4, 5]

  -- Test sum
  IO.println "Sum of [1,2,3,4,5]:"
  let s := sumList myList
  if s == 15 then
    IO.println "  PASS: sum = 15"
  else
    IO.println "  FAIL"

  -- Test length
  IO.println "Length of [1,2,3,4,5]:"
  let len := lengthList myList
  if len == 5 then
    IO.println "  PASS: length = 5"
  else
    IO.println "  FAIL"

  IO.println "Done!"
