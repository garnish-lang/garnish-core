# Types

## Unit
Representation of an uninitialized value or nothing.
```
()
```

## Number
Any numeric value.
```
10

-55
```

## Character
A single unicode character value. 

```
'a'

'Z'

'5'

'🧩'
```

## String
Any type of text value. Can span multiply lines and retain formatting
with triple quote marks `"""`.
```
"Expression Lang"

"x"

"""
This is a multi-line string.

With formatting.
"""
```

## Symbol
A kind of constant value. Any empty symbol can also be used, mainly for
type check / conversions.

```
:key

:flag

:
```

### True and False
Boolean values 'true' and 'false' are represented by symbols.
```
:true

:false
```

## Range
A range of whole integer or character values. Ranges are inclusive on
both sides by default. You may provide a `>` on the left to exclude the
starting value and/or a `<` on the right to exclude the ending value.

If either the left or right side is not provided an "open" range is
created. What values are used will depend on where the range is
consumed.
```
0..100
0..<100
0>..100
0>..<100

..100
...100
0..
0...

..
...

'a'...'z'
```

## Pair
A coupling of two values. Any type of value may be on either side.
```
:name = "Expression Lang"

10 = 100

10..20 = "ten to twenty"
```

## List
A collection of values. The values can be of any type.

Can be created by comma separated values that can span multiple lines or by spaces separated values on the same line. Both may also be used for a single list.

Below, the same list is created three different ways.
```
10, 20, 30,
40, 50, 60

10 20 30 40 50 60

10 20 30,
40 50 60
```

A list can also be created with a single element or no elements at all
by wrapping in parenthesis and a single comma. 
```
(10,)

(,)
```

### Associations
To have a list act like a hash map, items may be give associations by
providing a Pair value with either a hashed value as the left side of
the Pair.

Doing this allows the right side of the Pair to be directly accessed with the left side.
```
:first_name = "John",
:last_name = "Smith"
```

## Expression
While how expressions are organized is largely up to the executing environment, expressions may be nested by using braces.
```
{
    "This is inside an expression"
}
```