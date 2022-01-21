## CharList

"\n" - Newline
"\t" - Tab
"\r" - Carriage return
"\\" - Backslash
"\0" - Null
"\"" - Double Quote (only needed with single quote char list)
"\xFF" - Hex (2 digits exact)
"\u{Number}" - Unicode character number (6 digits max). Any number value, ends on first space

## ByteList
Each character is considered a byte and converted to it's ASCII value. All  charlist escape sequences are supported and converted accordingly.

'abc'

'\'' - Single quote literal
'\{100}' - Literal number value (any number value below, binary, octal, hex)

## Numbers
Default is base 10. The following do not support decimals after them

1_000_000 - Underscore as visual spacer

0b1111_0000 - Binary (base 2)

0o77 - Octal (base 8)

0xFF - Hex (base 16)
