| Precedence | Operator                                                           | Group Name       | Associativity |
| :--------: | :----------------------------------------------------------------- | :-----------:    | :-----------: |
| 1          | $? $ : ()                                                          | Value            | left-right    |
| 2          | () {}                                                              | Groupings        | left-right    |
| 3          | .                                                                  | Access           | left-right    |
| 4          | + - ! !! expr`                                                     | Unary Prefix     | right-left    |
| 5          | `expr                                                              | Suffix Apply     | left-right    |
| 6          | #>                                                                 | Type Cast        | left-right    |
| 7          | **                                                                 | Arithmetic 1     | left-right    |
| 8          | * / // %                                                           | Arithmetic 2     | left-right    |
| 9          | + -                                                                | Arithmetic 3     | left-right    |
| 10         | << >>                                                              | Shift            | left-right    |
| 11         | .. >.. ..< >..<                                                    | Create Range     | left-right    |
| 12         | < <= > >=                                                          | Relational       | left-right    |
| 13         | == != #=                                                           | Equality         | left-right    |
| 14         | &                                                                  | Bit And          | left-right    |
| 15         | ^                                                                  | Bit Xor          | left-right    |
| 16         | &#124;                                                             | Bit Or           | left-right    |
| 17         | &&                                                                 | Logical And      | left-right    |
| 18         | ^^                                                                 | Logical Xor      | left-right    |
| 19         | &#124;&#124;                                                       | Logical Or       | left-right    |
| 20         | =                                                                  | Create Pair      | right-left    |
| 21         | ->                                                                 | Create Link      | left-right    |
| 22         | ,                                                                  | Create List      | left-right    |
| 23         | ~~                                                                 | Empty Apply      | left-right    |
| 24         | \`expr\`                                                           | Infix Apply      | left-right    |
| 25         | !> ?>                                                              | Conditional      | left-right    |
| 26         | ~ ~>                                                               | Functional       | left-right    |
