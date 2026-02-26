-- Test List operations from stdlib

def main : IO Unit := do
  IO.println "=== List Operations Test ==="

  -- Basic list construction
  let list1 := [1, 2, 3, 4, 5]
  let list2 := [6, 7, 8, 9, 10]
  IO.println s!"list1 = {list1}"
  IO.println s!"list2 = {list2}"

  -- Length
  IO.println s!"list1.length = {list1.length}"

  -- Append
  IO.println s!"list1 ++ list2 = {list1 ++ list2}"

  -- Head and tail
  IO.println s!"list1.head! = {list1.head!}"
  IO.println s!"list1.tail! = {list1.tail!}"
  IO.println s!"list1.getLast! = {list1.getLast!}"

  -- Indexing
  IO.println s!"list1[0]! = {list1[0]!}"
  IO.println s!"list1[2]! = {list1[2]!}"
  IO.println s!"list1[4]! = {list1[4]!}"

  -- Take and drop
  IO.println s!"list1.take 3 = {list1.take 3}"
  IO.println s!"list1.drop 3 = {list1.drop 3}"

  -- Reverse
  IO.println s!"list1.reverse = {list1.reverse}"

  -- Map
  let doubled := list1.map (· * 2)
  IO.println s!"list1.map (*2) = {doubled}"

  -- Filter
  let evens := list1.filter (· % 2 == 0)
  IO.println s!"list1.filter (even) = {evens}"
  let odds := list1.filter (· % 2 == 1)
  IO.println s!"list1.filter (odd) = {odds}"

  -- Foldl and Foldr
  let sum := list1.foldl (· + ·) 0
  IO.println s!"list1.foldl (+) 0 = {sum}"
  let product := list1.foldl (· * ·) 1
  IO.println s!"list1.foldl (*) 1 = {product}"

  -- Any and All
  IO.println s!"list1.any (> 3) = {list1.any (· > 3)}"
  IO.println s!"list1.all (> 0) = {list1.all (· > 0)}"
  IO.println s!"list1.all (> 3) = {list1.all (· > 3)}"

  -- Find
  IO.println s!"list1.find? (> 3) = {list1.find? (· > 3)}"
  IO.println s!"list1.find? (> 10) = {list1.find? (· > 10)}"

  -- Contains
  IO.println s!"list1.contains 3 = {list1.contains 3}"
  IO.println s!"list1.contains 10 = {list1.contains 10}"

  -- Zip
  let zipped := list1.zip list2
  IO.println s!"list1.zip list2 = {zipped}"

  -- Range
  IO.println s!"List.range 5 = {List.range 5}"
  IO.println s!"List.replicate 3 'x' = {List.replicate 3 'x'}"

  -- Empty list operations
  let empty : List Nat := []
  IO.println s!"[].isEmpty = {empty.isEmpty}"
  IO.println s!"list1.isEmpty = {list1.isEmpty}"

  -- Nested lists
  let nested := [[1, 2], [3, 4], [5, 6]]
  IO.println s!"nested = {nested}"
  let flattened := nested.flatten
  IO.println s!"nested.flatten = {flattened}"

  -- List of strings
  let words := ["hello", "world", "lean", "is", "cool"]
  IO.println s!"words = {words}"
  IO.println s!"words.length = {words.length}"
  let longWords := words.filter (·.length > 3)
  IO.println s!"words with length > 3 = {longWords}"

  IO.println "=== List Test Complete ==="
