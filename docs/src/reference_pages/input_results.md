# Input & Results

Every expression receives a single input value. This value may be
referenced with the `$` symbol.

Assuming provided input is a number value.

```
$ + 5
```

Each expression can be a set of sub-expression. Each sub-expression
yields a result. The latest result can be referenced with the `?`
symbol. If no sub-expression has been executed by the time the `?`
symbol is used the input value is used.

```
? + 5

? + 10
```
