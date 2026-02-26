-- Test Std.HashMap operations
import Std

def main : IO Unit := do
  IO.println "=== HashMap Test ==="

  -- Create empty HashMap
  let mut m : Std.HashMap String Nat := {}

  -- Insert some values
  m := m.insert "one" 1
  m := m.insert "two" 2
  m := m.insert "three" 3

  -- Check size
  IO.println s!"Size: {m.size}"

  -- Lookup values
  match m.get? "two" with
  | some v => IO.println s!"get 'two': {v}"
  | none => IO.println "get 'two': not found"

  match m.get? "four" with
  | some v => IO.println s!"get 'four': {v}"
  | none => IO.println "get 'four': not found"

  -- Contains check
  IO.println s!"contains 'one': {m.contains "one"}"
  IO.println s!"contains 'five': {m.contains "five"}"

  -- Remove a key
  m := m.erase "two"
  IO.println s!"After erase 'two', size: {m.size}"

  IO.println "=== Done ==="
