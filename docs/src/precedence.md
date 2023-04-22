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
|     19     |       <>        |   Concatenation   |  left-right   |
|     20     |    < <= > >=    |    Relational     |  left-right   |
|     21     |    == != #=     |     Equality      |  left-right   |
|     22     |       !!        |    Logical Not    |  right-left   |
|     23     |       &&        |    Logical And    |  left-right   |
|     24     |       ^^        |    Logical Xor    |  left-right   |
|     25     |  &#124;&#124;   |    Logical Or     |  left-right   |
|     26     |     expr\`      |   Prefix Apply    |  right-left   |
|     27     |    \`expr\`     |    Infix Apply    |  left-right   |
|     28     |     \`expr      |   Suffix Apply    |  left-right   |
|     29     |      ~ ~>       |    Functional     |  left-right   |
|     30     |    ^~ !> ?>     |    Conditional    |  left-right   |
|     31     |     &#124;>     | Conditional Chain |  left-right   |
|     32     |     10, 20      |    Comma List     |  left-right   |
|     33     |      \n\n       |  Sub-Expression   |  left-right   |
