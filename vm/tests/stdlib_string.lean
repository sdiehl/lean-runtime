-- Test string operations from stdlib

def main : IO Unit := do
  IO.println "=== String Operations Test ==="

  -- Basic string operations
  let s1 := "Hello"
  let s2 := "World"
  let s3 := s1 ++ ", " ++ s2 ++ "!"
  IO.println s!"Concatenation: {s3}"
  IO.println s!"Length of '{s3}': {s3.length}"

  -- String comparison (use decide for Prop -> Bool)
  IO.println s!"'abc' < 'abd': {decide ("abc" < "abd")}"
  IO.println s!"'hello' == 'hello': {"hello" == "hello"}"
  IO.println s!"'Hello' == 'hello': {"Hello" == "hello"}"

  -- String splitting and joining
  let words := "one,two,three".splitOn ","
  IO.println s!"Split 'one,two,three' by ',': {words}"
  IO.println s!"Join with '-': {String.intercalate "-" words}"

  -- String predicates
  IO.println s!"'hello' starts with 'he': {"hello".startsWith "he"}"
  IO.println s!"'hello' ends with 'lo': {"hello".endsWith "lo"}"

  -- String conversion
  IO.println s!"toString 42: {toString 42}"
  IO.println s!"toString true: {toString true}"

  -- UTF-8 handling
  let utf8Str := "Hello, World!"
  IO.println s!"String: {utf8Str}"
  IO.println s!"Length: {utf8Str.length}"
  IO.println s!"Byte size: {utf8Str.utf8ByteSize}"

  -- Character iteration
  let testStr := "Lean"
  IO.println s!"Characters in '{testStr}':"
  for c in testStr.toList do
    IO.println s!"  '{c}'"

  -- Substring via list operations
  let longStr := "The quick brown fox"
  let chars := longStr.toList
  let slice := (chars.drop 4).take 5
  IO.println s!"Substring [4:9] of '{longStr}': {String.ofList slice}"

  -- isEmpty
  IO.println s!"''.isEmpty: {"".isEmpty}"
  IO.println s!"'hello'.isEmpty: {"hello".isEmpty}"

  -- String to list and back
  let charList := "hello".toList
  IO.println s!"'hello'.toList: {charList}"
  let backToStr := String.ofList charList
  IO.println s!"String.ofList chars: {backToStr}"

  -- Case operations
  IO.println s!"'Hello'.toLower: {"Hello".toLower}"
  IO.println s!"'Hello'.toUpper: {"Hello".toUpper}"

  IO.println "=== String Test Complete ==="
