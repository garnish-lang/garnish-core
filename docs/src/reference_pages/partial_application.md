# Partial Application

## Single Value

Anytime `expression` is invoked `10` will be the input even if
additional arguments are given

```
expression ~~ 10

expression ~~ 10 () 30

expression ~~ "John"

expression ~~ 1..<10

expression ~~ "John" = 25
```

Using a single pair value with symbol key will actually be considered an
association instead of a value.

```
expression ~~ :name = "John"
```

So `expression` will access this value as `$.name` instead of `$` being
a pair and accessing the left and right with `$.left` and `$.right`.

## Multiple Applications

Applying multiple single values will be equivalent to providing a list
with the applied items in order

```
expression ~~ 10

? ~~ 20

? ~~ 30
```

Is equivalent to

```
expression ~~ 10 20 30
```

Applying a Unit value and then another non unit value will simply
replace the Unit value.

```
expression ~~ ()

? ~~ 20
```

Is equivalent to

```
expression ~~ 20
```

Applying a Unit value after another non Unit value will do nothing and
return the original Partial

```
expression ~~ 20

? ~~ ()
```

Is equivalent to

```
expression ~~ 20
```

Applying again after a list will first fill in any Unit values and then
add on to the end of the list

```
expression ~~ 10 () 30

? ~~ 20 40 
```

Is equivalent to

```
expression ~~ 10 20 30 40
```

Unit values at the beginning of subsequent applications will be used as
a skip marker for where to begin application of new values.

```
expression ~~ () () 30

? ~~ () 20
```

Is equivalent to

```
expression ~~ () 20 30
```

## With associations

Associations in a list are always pushed to the end during partially
application.

```
expression ~~ :name = "John"

? ~~ 25
```

Is equivalent to

```
expression ~~ 25 :name = "John"
```

Also applied associations will never replace non associated Unit values.

```
expression ~~ 10 () 30

? ~~ :name = "John"
```

Is equivalent to

```
expression ~~ 10 () 30 :name = "John"
```

But they will replace any value that has the same association.

```
expression ~~ :name = "John"

? ~~ :name = "Alice"
```

Is equivalent to

```
expression ~~ :name = "Alice"
```