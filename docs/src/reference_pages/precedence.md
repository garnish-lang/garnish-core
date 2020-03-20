
| Precedence | Operator                                                           | Group Name       | Associativity |
| :--------: | :----------------------------------------------------------------- | :-----------:    | :-----------: |
| 1          | ? $ : () &#124;>output &#124;>continue &#124;>skip &#124;>complete | Value            | left-right    |
| 2          | () {}                                                              | Groupings        | left-right    |
| 3          | .                                                                  | Access           | left-right    |
| 4          | + - !                                                              | Unary            | left-right    |
| 5          | .. >.. ..< >..<                                                    | Create Range     | left-right    |
| 6          | #>                                                                 | Type Cast        | left-right    |
| 7          | **                                                                 | Arithmetic 1     | left-right    |
| 8          | * / // %                                                           | Arithmetic 2     | left-right    |
| 9          | + -                                                                | Arithmetic 3     | left-right    |
| 10         | << >>                                                              | Shift            | left-right    |
| 11         | < <= > >=                                                          | Relational       | left-right    |
| 12         | == != #=                                                           | Equality         | left-right    |
| 13         | &                                                                  | Bit And          | left-right    |
| 14         | ^                                                                  | Bit Xor          | left-right    |
| 15         | &#124;                                                             | Bit Or           | left-right    |
| 16         | &&                                                                 | Logical And      | left-right    |
| 17         | ^^                                                                 | Logical Xor      | left-right    |
| 18         | &#124;&#124;                                                       | Logical Or       | left-right    |
| 19         | ->                                                                 | Create Link      | left-right    |
| 20         | =                                                                  | Create Pair      | right-left    |
| 21         | ,                                                                  | Create List      | left-right    |
| 22         | ~~                                                                 | Partially Apply  | left-right    |
| 23         | expr`                                                              | Suffix Apply     | right-left    |
| 24         | `expr                                                              | Prefix Apply     | left-right    |
| 25         | \`expr\`                                                           | Infix Apply      | left-right    |
| 26         | => !> =?>                                                          | Conditional      | left-right    |
| 27         | ~ ~>                                                               | Functional       | left-right    |
| 28         | >>> >>&#124; &#124;>> &#124;>&#124; <>>                            | Iteration        | left-right    |