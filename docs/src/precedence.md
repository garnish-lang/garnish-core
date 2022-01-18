| Precedence |    Operator     |    Group Name     | Associativity |
|:----------:|:---------------:|:-----------------:|:-------------:|
|     1      |  $ : () $? $!   |       Value       |  left-right   |
|     2      |      () {}      |     Groupings     |  left-right   |
|     3      |        .        |      Access       |  left-right   |
|     4      |       ~~        |    Empty Apply    |  left-right   |
|     5      |       _.        |  Internal Prefix  |  right-left   |
|     6      |   ._ .&#124;    |  Internal Suffix  |  left-right   |
|     8      |       ~#        |     Type Cast     |  left-right   |
|     7      |     ++ -- !     | Arithmetic Prefix |  right-left   |
|     9      |       **        |   Arithmetic 1    |  left-right   |
|     10     |    * / // %     |   Arithmetic 2    |  left-right   |
|     11     |       + -       |   Arithmetic 3    |  left-right   |
|     12     |      << >>      |       Shift       |  left-right   |
|     13     |        &        |      Bit And      |  left-right   |
|     14     |        ^        |      Bit Xor      |  left-right   |
|     15     |     &#124;      |      Bit Or       |  left-right   |
|     16     | .. >.. ..< >..< |   Create Range    |  left-right   |
|     17     |        =        |    Create Pair    |  left-right   |
|     18     |      10 20      |    Space List     |  left-right   |
|     19     |       ->        |    Append Link    |  left-right   |
|     20     |       <-        |   Prepend Link    |  right-left   |
|     21     |    < <= > >=    |    Relational     |  left-right   |
|     22     |    == != #=     |     Equality      |  left-right   |
|     23     |       !!        |    Logical Not    |  right-left   |
|     24     |       &&        |    Logical And    |  left-right   |
|     25     |       ^^        |    Logical Xor    |  left-right   |
|     26     |  &#124;&#124;   |    Logical Or     |  left-right   |
|     27     |     expr\`      |   Prefix Apply    |  right-left   |
|     28     |    \`expr\`     |    Infix Apply    |  left-right   |
|     29     |     \`expr      |   Suffix Apply    |  left-right   |
|     30     |      ~ ~>       |    Functional     |  left-right   |
|     31     |    ^~ !> ?>     |    Conditional    |  left-right   |
|     32     |     &#124;>     | Conditional Chain |  left-right   |
|     33     |     10, 20      |    Comma List     |  left-right   |
|     34     |      \n\n       |  Sub-Expression   |  left-right   |
