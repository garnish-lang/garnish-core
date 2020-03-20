# Operations

## Arithmetic
Arithmetic operations expect Number values. If any other value is
provided, the result will be a Unit.

Supported arithmetic operations are: 

`+` Addition / Absolute Value

`-` Subtraction / Negation

`*` Multiplication

`/` Division

`//` Integer division

`%` Modulo

`**` Exponential
```
5 + 4

5 - 4

5 * 4

5 / 4

5 // 4

5 % 4

5 ** 4

-5

+4
```

## Type Cast
Values may be cast into another type with the `#>` operator. The left
side is the value to cast and the right is a value whose type is the one
to cast to.

Supported casts:

Number -> String

Number -> Symbol - If numeric value is not in symbol table, Unit is the
result.

String -> Number - If string value cannot be represented as a number,
Unit is the result

String -> Symbol - If string value is not in symbol table, Unit is the
result.

Symbol -> Number

Symbol -> String

Pair -> String

Range -> String

List -> String

```
100 #> ""

1 #> :

"500" #> 0

"some_symbol" #> :

:some_symbol #> ""

:some_symbol #> 0

:name = "John" #> ""

10..100 #> ""

10..100 #> (,)

10 20 30 #> ""
```

## Logical
Four logical operators are supported.

Supported logical operations are: 

`&&` AND

`||` OR

`^^` XOR

`!` NOT

Logical operation expect either symbols `:true` or `:false` but will
also evaluate if given a Unit value. Unit values are logical equivalent
to `:false`. 

Any other value provided is logically equivalent to `:true` (except for
the `!` operator. See Bitwise).
```
:true && :false

:true || :false

:true ^^ :false

!:true

!()

:true && ()
```

## Comparative
All comparative operations will result in either `:true` or `:false`.

Supported comparative operations are:

`==` Equality

`!=` Inequality

`>` Greater than

`>=` Greater than or equal

`<` Less than

`<=` Less than or equal

`#=` Type equals

Equality and inequality are performed by value not reference, even with
List type values.
```
10 == 20

10 != 20

10 > 20

10 >= 20

10 < 20

10 <= 20

10 #= 20
```

## Bitwise
Bitwise operations expect a Number with no decimal part. If any other
value is provided, the result will be a Unit.

Supported bitwise operations are: 

`&` and

`|` or

`^` xor

`!` not

`<<` left shift

`>>` right shift
```
10 & 30

10 | 30

10 ^ 30

!10

10 << 2

10 >> 2
```

## Linking
Certain types of values may be "linked" together resulting in a special
linked value.

Types that can be linked:
- String
- List

Only one linking operators is supported:

`->` Connect

The connect operation concatenates the values together if same type, or
appends if different type. If making a list of lists, provide a list
with single element to append another list
```
"Hello, " -> "World"

10, 20, 30, 40, 50 -> 60

10, 20, 30 -> 40, 50, 60

(10, 20), (30, 40) -> ((50, 60),)
```

## Access
Sub-values may be obtained by one of two ways:

### Dot Access
Performed with the `.` operator. It expects any value on the left and an
identifier on the right. The identifier is treated as a symbol or an
index if a number.

```
(:name = "John", :age = 54).name

(10, 20, 30).2
```

### Built in access

Most types have sub-values that may be accessed.

#### String

`length` - Number of characters in the string

```
"The quick brown fox.".length
```

#### Range

`length` - Number of values in the range

`min` - Minimum value of the range

`max` - Maximum value of the range

```
(10..100).length

(10..100).min

(10..100).max
```

#### Pair

`left` - Left side value of the range

`right` - Right side value of the range

```
(:name = "John").left

(:name = "John").right
```

#### List

`length` - Number of items in the list

```
(10 20 30).length
```

## Conditional
Conditional logic flow is performed with conditional operators.

Supported conditional operators are:

`=>` True Check - Executes right expression if left value is 'true'

`!>` Else / False Check - Executes right expression if previous 'True
Check' failed. If this doesn't follow a 'True Check' then right
expression is executed if left is 'false'.

`=?>` Result Check - Executes right expression if left value is equal to
current result.

Conditional operations expect any value on the left and an expression on
the right.

Values that are considered 'false' are:

- the symbol `:false`
- a Unit value

All other values are considered 'true'

```
:true => { "True statement" }

:false !> { "False statement" }

:false => { "True statement" } !> { "False statement" }

:false => { 
    "True statement" 
} !> { 
    "False statement" 
}

100 =?> "Perfect Score"
```

### Chaining conditionals
Conditional statements may be chained together with a `,`. For a set of
chained conditional operations, only the first one to execute its right
side expression will do so, after which the rest of the chain is
skipped.

A default or 'else' conditional may be provided as the last operation
using the `!>` operator.

```
? % 15 == 0 => { "fizzbuzz" },
? % 3 == 0 => { "fizz" },
? % 5 == 0 => { "buzz" },
!> { "Neither fizz nor buzz" }

:high =?> { "High" },
:medium =?> { "Medium" },
:low =?> { "Low" },
!> { "Ungraded" }
```

## Functional

### Apply
The traditional invoke is performed with the `~` operator. On the left
is the expression to invoke and on the right the arguments to pass into
it in list form

Assuming we have an expression called `lerp` that expecting three number
arguments.

```
lerp ~ 10 20 0.5
```

### Partially Apply
You may partially apply arguments to an expression with the `~~`
operator. This results in a new expression value. When this new
expression is invoked the arguments are injected with any additional
arguments.

Arguments may be skipped during partial application by specifying a Unit
it their place.

The new expression will now only expect arguments that were not
specified by the partial application.

```
lerp ~~ 10, 20

lerp ~~ (), (), 0.5
```

### Piping
Arguments may be injected into an expressions argument list before
invoking the expression with pipe operator `->`.

```
10, 20, 0.5 ~> lerp

10, 20 ~> lerp ~~ (), (), 0.5

0.5 ~> lerp ~~ 10, 20
```

### *fix Invocation
Expressions may also be invoke in one of three *fix ways with the `` `
`` mark. Invoking in this way restricts the number of arguments to 1 or
2 depending on which is used.

Assuming we have an expression called `sqrt` that expecting one number
argument.

#### Prefix - mark goes before expression name. 

Single argument on right.
```
`sqrt 25
```

#### Postfix - mark goes after expression name. 

Single argument on left.
```
25 sqrt`
```

#### Infix - mark goes before and after expression name. 

Two arguments, one on each side.

Assuming we have an expression called `log` that expecting two number
arguments.
```
10 `log` 1000
```

# Iteration
Collections may be iterated over with the `>>>` operator. The left side
is the collection to iterate over and the right side is an expression to
invoke for each element of the collection. The right side expression
will receive input with the following fields.

`current` - The current value in the iteration

`result` - The result from the previous iteration. Unit for first
iteration for default behavior.

`index` - Current iteration count starting at 0.

Types that support iteration are:
- String
- Range
- List
- Expression

```
"Expressions" >>> { $.current }

10..100 >>> { $.current + 5 }

10 20 30 >>> { $.current + 5 }

generator_expr >>> { $.current + 5 }
```

## Initial result
An initial value, other than Unit, can be specified by partially
applying a single value.

```
10 20 30 >>> { $.current + $.result } ~~ 0
```

## Iterate to Value
The default iteration creates a new list containing the result of each
iteration. In order to yield a non list from iteration use the value
iterate operator `>>|`

```
10 20 30 >>| { $.current + $.result } ~~ 0

"one" "two" "three >>| { $.result <-> ", " <-> $.current } ~~ ""
```

## Reverse Iterate
Collections may also be iterated in reverse with the two operators.

`|>>` Reverse iterate to list

`|>|` Reverse iterate to value

```
10 20 30 |>> { $.current + 5 }

10 20 30 |>| { $.current + $.result } ~~ 0
```

## Iteration Instructions
Their are four instructions that allow more control over which values
are used during iteration.

`:>output` - Explicit output of last result. Allows multiple values to
be output per iteration.

`:>continue` - Output iterations result and move to the next iteration
immediately.

`:>skip` - Ignore this iteration's result and move to the next iteration
immediately.

`:>complete` - Output value and end iteration.

`:>exit` - Do not output value and end iteration.

## Multi Iteration
Multiple collections may be iterated at the same time by using the `<>>`
operator. The left side should be a list of collection values and the
right side the expression to execute per value.

The right side expression will still receive the `result` and `index` as
part of the input. The `current` value will now be a list containing one
item from each input collection.

The iteration lasts until each item in all collections have been
iterated over. A Unit value is used if one collection has no more items
to iterate over.

```
(10, 20, 30), (40, 50, 60) <>> { $.current.1, $.current.2 }
```