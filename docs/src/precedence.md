| Precedence | Operator            |   Group Name   | Associativity |
|:----------:|:--------------------|:--------------:|:-------------:|
|     1      | $? $ : () $!        |     Value      |  left-right   |
|     2      | () {}               |   Groupings    |  left-right   |
|     3      | .                   |     Access     |  left-right   |
|     4      | ~~                  |  Empty Apply   |  left-right   |
|     5      | ++ -- ! !! expr` _. |  Unary Prefix  |  right-left   |
|     6      | `expr ._ .          |  Unary Suffix  |  left-right   |
|     7      | ~#                  |   Type Cast    |  left-right   |
|     8      | **                  |  Arithmetic 1  |  left-right   |
|     9      | * / // %            |  Arithmetic 2  |  left-right   |
|     10     | + -                 |  Arithmetic 3  |  left-right   |
|     11     | << >>               |     Shift      |  left-right   |
|     12     | .. >.. ..< >..<     |  Create Range  |  left-right   |
|     13     | < <= > >=           |   Relational   |  left-right   |
|     14     | == != #=            |    Equality    |  left-right   |
|     15     | &                   |    Bit And     |  left-right   |
|     16     | ^                   |    Bit Xor     |  left-right   |
|     17     | &#124;              |     Bit Or     |  left-right   |
|     18     | &&                  |  Logical And   |  left-right   |
|     19     | ^^                  |  Logical Xor   |  left-right   |
|     20     | &#124;&#124;        |   Logical Or   |  left-right   |
|     21     | =                   |  Create Pair   |  left-right   |
|     22     | ->                  |  Create Link   |  left-right   |
|     23     | ,                   |  Create List   |  left-right   |
|     24     | \`expr\`            |  Infix Apply   |  left-right   |
|     25     | ~ ~>                |  Functional 1  |  left-right   |
|     26     | ^~                  |  Functional 2  |  left-right   |
|     27     | !> ?>               |  Conditional   |  left-right   |
|     27     | \n\n                | Sub-Expression |  left-right   |
