-- Test suite for arithmetic edge cases

def main : IO Unit := do
  IO.println "=== Arithmetic Tests ==="

  -- Test 1: Basic Nat operations
  IO.println s!"nat_add: {5 + 3}"  -- 8
  IO.println s!"nat_sub: {10 - 3}"  -- 7
  IO.println s!"nat_sub_underflow: {3 - 10}"  -- 0 (saturating)
  IO.println s!"nat_mul: {7 * 6}"  -- 42
  IO.println s!"nat_div: {17 / 5}"  -- 3
  IO.println s!"nat_mod: {17 % 5}"  -- 2
  IO.println s!"nat_div_zero: {5 / 0}"  -- 0

  -- Test 2: Large Nat values
  let big1 := 1000000000
  let big2 := 2000000000
  IO.println s!"nat_big_add: {big1 + big2}"  -- 3000000000
  IO.println s!"nat_big_mul: {big1 * 1000}"  -- 1000000000000

  -- Test 3: Int operations
  let i1 : Int := 10
  let i2 : Int := -7
  IO.println s!"int_add: {i1 + i2}"  -- 3
  IO.println s!"int_sub: {i1 - i2}"  -- 17
  IO.println s!"int_mul: {i1 * i2}"  -- -70
  IO.println s!"int_neg: {-i1}"  -- -10
  IO.println s!"int_neg_neg: {-i2}"  -- 7

  -- Test 4: Int division (truncating toward zero)
  IO.println s!"int_div_pos_pos: {(17 : Int) / 5}"  -- 3
  IO.println s!"int_div_neg_pos: {(-17 : Int) / 5}"  -- -3
  IO.println s!"int_div_pos_neg: {(17 : Int) / (-5)}"  -- -3
  IO.println s!"int_div_neg_neg: {(-17 : Int) / (-5)}"  -- 3

  -- Test 5: Int modulo
  IO.println s!"int_mod_pos_pos: {(17 : Int) % 5}"  -- 2
  IO.println s!"int_mod_neg_pos: {(-17 : Int) % 5}"  -- -2
  IO.println s!"int_mod_pos_neg: {(17 : Int) % (-5)}"  -- 2
  IO.println s!"int_mod_neg_neg: {(-17 : Int) % (-5)}"  -- -2

  -- Test 6: UInt8 operations
  let u8_a : UInt8 := 200
  let u8_b : UInt8 := 100
  IO.println s!"uint8_add: {u8_a + u8_b}"  -- 44 (overflow wraps)
  IO.println s!"uint8_sub: {u8_a - u8_b}"  -- 100
  IO.println s!"uint8_sub_wrap: {u8_b - u8_a}"  -- 156 (underflow wraps)
  IO.println s!"uint8_mul: {(15 : UInt8) * 17}"  -- 255
  IO.println s!"uint8_max: {(255 : UInt8)}"  -- 255

  -- Test 7: UInt32 operations
  let u32_a : UInt32 := 3000000000
  let u32_b : UInt32 := 2000000000
  IO.println s!"uint32_add: {u32_a + u32_b}"  -- wraps
  IO.println s!"uint32_sub: {u32_a - u32_b}"  -- 1000000000

  -- Test 8: UInt64 operations
  let u64_a : UInt64 := 10000000000000000000
  let u64_b : UInt64 := 5000000000000000000
  IO.println s!"uint64_add: {u64_a + u64_b}"  -- wraps
  IO.println s!"uint64_sub: {u64_a - u64_b}"  -- 5000000000000000000

  -- Test 9: Bitwise operations
  IO.println s!"nat_land: {12 &&& 10}"  -- 8
  IO.println s!"nat_lor: {12 ||| 10}"  -- 14
  IO.println s!"nat_xor: {12 ^^^ 10}"  -- 6

  let u32_x : UInt32 := 0xFF00FF00
  let u32_y : UInt32 := 0x0F0F0F0F
  IO.println s!"uint32_land: {u32_x &&& u32_y}"  -- 0x0F000F00
  IO.println s!"uint32_lor: {u32_x ||| u32_y}"  -- 0xFF0FFF0F
  IO.println s!"uint32_xor: {u32_x ^^^ u32_y}"  -- 0xF00FF00F

  -- Test 10: Shift operations
  IO.println s!"nat_shl: {1 <<< 10}"  -- 1024
  IO.println s!"nat_shr: {1024 >>> 5}"  -- 32
  IO.println s!"uint32_shl: {(1 : UInt32) <<< 31}"  -- 2147483648
  IO.println s!"uint32_shr: {(0x80000000 : UInt32) >>> 16}"  -- 32768

  -- Test 11: Comparison operations (as Bool)
  IO.println s!"nat_lt: {decide (5 < 10)}"  -- true
  IO.println s!"nat_le: {decide (5 <= 5)}"  -- true
  IO.println s!"nat_gt: {decide (10 > 5)}"  -- true
  IO.println s!"nat_ge: {decide (5 >= 5)}"  -- true
  IO.println s!"nat_eq: {decide (5 = 5)}"  -- true
  IO.println s!"nat_ne: {decide (5 ≠ 6)}"  -- true

  IO.println s!"int_lt: {decide ((-5 : Int) < 5)}"  -- true
  IO.println s!"int_le: {decide ((-5 : Int) <= (-5))}"  -- true

  -- Test 12: Float operations
  let f1 : Float := 3.14159
  let f2 : Float := 2.71828
  IO.println s!"float_add: {f1 + f2}"
  IO.println s!"float_sub: {f1 - f2}"
  IO.println s!"float_mul: {f1 * f2}"
  IO.println s!"float_div: {f1 / f2}"
  IO.println s!"float_neg: {-f1}"

  -- Test 13: Float special values
  IO.println s!"float_div_zero: {(1.0 : Float) / 0.0}"  -- inf
  IO.println s!"float_neg_div_zero: {(-1.0 : Float) / 0.0}"  -- -inf
  IO.println s!"float_zero_div_zero: {(0.0 : Float) / 0.0}"  -- nan

  -- Test 14: Float functions
  IO.println s!"float_sqrt: {Float.sqrt 2.0}"
  IO.println s!"float_abs: {Float.abs (-5.5)}"  -- 5.5
  IO.println s!"float_floor: {Float.floor 3.7}"  -- 3.0
  IO.println s!"float_ceil: {Float.ceil 3.2}"  -- 4.0

  -- Test 15: Nat power
  IO.println s!"nat_pow: {2 ^ 10}"  -- 1024
  IO.println s!"nat_pow_0: {5 ^ 0}"  -- 1
  IO.println s!"nat_pow_1: {5 ^ 1}"  -- 5

  -- Test 16: Mixed type operations (via coercion)
  let n : Nat := 5
  let i : Int := -3
  IO.println s!"nat_to_int: {(n : Int) + i}"  -- 2

  -- Test 17: Min/Max
  IO.println s!"nat_min: {min 5 3}"  -- 3
  IO.println s!"nat_max: {max 5 3}"  -- 5
  IO.println s!"int_min: {min (-5 : Int) 3}"  -- -5
  IO.println s!"int_max: {max (-5 : Int) 3}"  -- 3

  IO.println "=== Arithmetic Tests Complete ==="
