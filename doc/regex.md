# MOROS Regular Expression Engine

MOROS include a simplified regular expression engine with the following syntax:

- `\` escape the following character to its literal meaning
- `^` matches the starting position within the string
- `$` matches the ending position within the string
- `*` matches the preceding element zero or more times
- `+` matches the preceding element one or more times
- `?` matches the preceding element zero or one time
- `.` matches any single character
- `\w` matches any alphanumeric character
- `\W` matches any non-alphanumeric character
- `\d` matches any numeric character
- `\D` matches any non-numeric character
- `\w` matches any whitespace character
- `\W` matches any whitespace character

The engine is UTF-8 aware, so for example the unicode character `é` will be
matched by `\w` even if it's not present in the ASCII table and has a size
of two bytes.
