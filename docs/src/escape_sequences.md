## CharList
Single double quote will skip non-escaped newlines and tabs

"\n" - Newline
"\t" - Tab
"\r" - Carriage return
"\\" - Backslash
"\0" - Null
"\"" - Double Quote (only needed with single quote char list)
"\u{Number}" - Unicode character number (6 digits max). Any number value, ends on first space

## ByteList
Each character is considered a byte and converted to it's ASCII value. All  charlist escape sequences are supported and converted accordingly.

'abc'

'\'' - Single quote literal
''100 200 02_1111'' - Double single quote is parsed as space separated list of numbers

## Numbers
Default is base 10. The following do not support decimals after them

1_000_000 - Underscore as visual spacer

02_1111_0000 - Binary (base 2)

08_77 - Octal (base 8)

016_FF - Hex (base 16)
