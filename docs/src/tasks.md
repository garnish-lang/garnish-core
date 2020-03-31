# Tasks

List of features to implement.

## ✅ Phase 1
Clean up runtime module.

- ✅ Merge AssociativeList and List types
- ✅ Convert usage of `Vec<u32>` type to bytes `Vec<u8>`
- ✅ Change list creation to use a reference stack instead of putting value
  in the data pool
- ✅ Change Expression to use symbol for value instead of String
- ✅ Change ExternalMethod to use symbol for value instead of String
- ✅ Change Resolve to use symbol for value instead of String
- ❓ Make `:true` and `:false` symbols first in symbol table
- ✅ Support for referencing expressions that haven't been made yet
- ✅ Replace all `panic!`s with Results or have operation yield a Unit
  where appropriate
- ✅ Move ExpressionValueRef to be in same module as ExpressionValue
- ✅ Remove 'fmt' module
- ✅ Replace 'unit_if_not_enough_refs' with error Result
- ✅ Move 'make_range' and 'make_pair' into own modules
- ✅ Implement frames for 'put_result' instruction to scope results to
  individual expression invocations
- ✅ Simplify enum translation to numeric value
- ✅ Separate instructions from data
- ✅ Separate registers from data
- ❓ Re-implement data copying by walking references instead of iteration
- ✅ Make range serialization consistent

## ✅ Phase 2
General opinionated refactoring and making code idiomatic.

## Phase 3
Finalize implementation of all types.

### Character
- ✅ Created
- ✅ As Grapheme Cluster

### Character List

- ✅ Created
- ✅ Replace Strings

### Number

- ✅ Integer
- ✅ Float

### Symbol

- ✅ Empty Symbols

### Range

- ✅ Character Ranges
- ✅ Float Ranges
- ✅ Exclusive
- ✅ Inclusive
- ✅ Open Ranges
- ✅ Steps

### List

- ✅ Normal List
- ✅ Associations

## Phase 4
Implement all operations.

### Arithmetic

- ✅ Addition
- ✅ Subtraction
- ✅ Multiplication
- ✅ Division
- ✅ Integer Division
- ✅ Modulo
- ✅ Negation
- ✅ Absolute Value
- ✅ Exponential

### Type Cast

- ✅ Unit -> CharacterList
- ✅ Unit -> Unit
- ✅ Char -> CharacterList
- ✅ Char -> Integer
- ✅ Char -> Char
- ✅ Integer -> Char
- ✅ Integer -> CharacterList
- ✅ Integer -> Symbol
- ✅ Integer -> Integer
- ✅ Integer -> Float
- ✅ Float -> CharacterList
- ✅ Float -> Integer
- ✅ Float -> Float
- ✅ CharacterList -> Integer
- ✅ CharacterList -> Float
- ✅ CharacterList -> Symbol
- ✅ CharacterList -> CharacterList
- ✅ Symbol -> Integer
- ✅ Symbol -> CharacterList
- ✅ Symbol -> Symbol
- ✅ Pair -> CharacterList
- ✅ Pair -> Pair
- ✅ Range -> CharacterList
- ✅ Range -> Range
- ✅ List -> CharacterList
- ✅ List -> List

### Logical

- ✅ AND
- ✅ OR
- ✅ XOR
- ✅ NOT

### Comparative

- ✅ Equality
- ✅ Inequality
  - ✅ Unit == Unit
  - ✅ Character == Character
  - ✅ Character == CharacterList
  - ✅ CharacterList == Character
  - ✅ CharacterList == CharacterList
  - ✅ Symbol == Symbol
  - ✅ Integer == Integer
  - ✅ Integer == Float
  - ✅ Float == Float
  - ✅ Float == Integer
  - ✅ Range == Range
  - ✅ Pair == Pair
  - ✅ List == List
- ✅ Greater Than
- ✅ Greater Than or Equal
- ✅ Less Than
- ✅ Less Than or Equal 
  - ✅ Unit <> Any
  - ✅ Character <> Character
  - ✅ Character <> CharacterList
  - ✅ CharacterList <> Character
  - ✅ CharacterList <> CharacterList
  - ✅ Symbol <> Symbol
  - ✅ Integer <> Integer
  - ✅ Integer <> Float
  - ✅ Float <> Float
  - ✅ Float <> Integer
  - ✅ Range <> Any
  - ✅ Pair <> Any
  - ✅ List <> Any
  
- ✅ Type Equal

### Bitwise

- ✅ and
- ✅ or
- ✅ xor
- ✅ not
- ✅ left shift
- ✅ right shift

### ✅ Linking

- ✅ String
- ✅ List

### Conditional

- ✅ True Check
- ✅ False Check
- ✅ Result Check
- ✅ Conditional Chains

### Access
- ✅ Character List
  - ✅ length
  - ✅ integer index 
  - ✅ Range (slice)
- ✅ Range
  - ✅ length
  - ✅ start
  - ✅ end
  - ✅ step
  - ✅ is_start_exclusive
  - ✅ is_end_exclusive
  - ✅ is_start_open
  - ✅ is_end_open
- ✅ Pair
  - ✅ left
  - ✅ right
- ✅ Partial
  - ✅ base
  - ✅ value
- Slice
  - ✅ length
  - ❓ key_count
  - ❓ integer index
  - ❓ symbol/character list access
  - ✅ Range (slice)
- Link
  - ✅ length
  - ❓ key_count
  - ❓ integer index
  - ❓ symbol/character list access
  - ✅ Range (slice)
- ✅ List
  - ✅ length
  - ✅ key_count
  - ✅ integer index
  - ✅ symbol/character list access
  - ✅ Range (slice)

### Functional

- ✅ Apply
  - ✅ Symbol application on Any except Expression, ExternalMethod and
    Partial will defer to access
  - ✅ Character List, defer to access
  - ✅ Range
    - Range (slice), defer to access
    - Integer, Float, Character (add step)
  - ✅ List
    - Integer, Range defer to access
    - Symbol, String associative access 
  - ✅ Expression & External Method
    - Any to invoke expression or method with value as input
  - ✅ Partial
    - Apply partial rules with partial value and applied value and then
      invoke base expression or method
  
- ✅ Partially Apply 

### Iteration

- ✅ CharacterList
- Range
  - ✅ Integer
  - ✅ Float
  - ✅ Character
  - ✅ inclusive vs exclusive
  - ✅ with steps
  - ❓ open ranges
- ✅ List
- ❓ Slice
- ❓ Link
- ✅ Expression
- ✅ Iteration Input
  - ✅ current
  - ✅ result
  - ✅ initial result
- ✅ Single value
  - To String
  - To List
- ❓ Reverse iterate
- ❓ Reverse iterate single value
- ✅ Iteration Instructions
  - ✅ output
  - ✅ continue
  - ✅ skip
  - ✅ complete
  - ❓ exit
- ❓ Multi-iteration

# Phase 5
Compiler

## Lexer

### Tokens

- ✅ \+
- ✅ \-
- ✅ \*
- ✅ /
- ✅ //
- ✅ %
- ✅ **
- ✅ \#>
- ✅ &&
- ✅ ||
- ✅ ^^
- ✅ !
- ✅ ==
- ✅ !=
- ✅ \>
- ✅ \>=
- ✅ <
- ✅ <=
- ✅ \#=
- ✅ &
- ✅ |
- ✅ ^
- ✅ <<
- ✅ \>>
- ✅ =
- ✅ ->
- ✅ .
- ✅ ..
- ✅ \>..
- ✅ ..<
- ✅ \>..<
- ✅ =>
- ✅ !>
- ✅ =?>
- ✅ :
- ✅ {
- ✅ }
- ✅ (
- ✅ )
- ✅ ~
- ✅ ~~
- ✅ ~>
- ✅ \`
- ✅ \>>>
- ✅ \>>|
- ✅ |>>
- ✅ |>|
- ✅ <>>
- ✅ |>output
- ✅ |>continue
- ✅ |>skip
- ✅ |>complete
- ✅ ?
- ✅ $
- ✅ ()
- ✅ ,

### Literals

- ✅ Character
- ✅ CharacterList
- ✅ Number
- ✅ Symbol/Identifiers
- ✅ Whitespace
  - ✅ spaces/tabs
  - ✅ newlines

# Phase 6
Parser

## 6.1
Identifying groupings from parenthesis, braces and newlines.

Separate groups from braces.

Reassign operations. 
	- ✅ Minus sign to subtraction or negation
	- dot to access or decimal
	- dot chains etc.
	- commas to list separators or conditional separators
	- ✅ spaces to list separators
	- ✅ \*fix reasignment
	- ✅ Plus sign to addition or absolute value

Identify sub expressions from terminability, double newlines and conditionals

## 6.2
Create AST

## 6.3
Create instruction set from AST
