-- Simple list test

def sumList : List Nat → Nat
  | [] => 0
  | x :: xs => x + sumList xs

def main : IO Unit := do
  IO.println "Start"
  let myList := [1, 2, 3, 4, 5]
  let s := sumList myList
  IO.println "Sum computed"
  if s == 15 then
    IO.println "PASS"
  else
    IO.println "FAIL"
  IO.println "End"
