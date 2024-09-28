# MOROS Colors

MOROS operates in 80x25 VGA text mode and supports standard ANSI escape
sequences for color manipulation.

There are 8 dark and 8 bright colors:

    +--------+--------+--------+--------+--------+--------+--------+--------+
    | Black  | Maroon | Green  | Olive  | Navy   | Purple | Teal   | Silver |
    | (30)   | (31)   | (32)   | (33)   | (34)   | (35)   | (36)   | (37)   |
    +--------+--------+--------+--------+--------+--------+--------+--------+
    | Gray   | Red    | Lime   | Yellow | Blue   | Fushia | Aqua   | White  |
    | (90)   | (91)   | (92)   | (93)   | (94)   | (95)   | (96)   | (97)   |
    +--------+--------+--------+--------+--------+--------+--------+--------+

The control sequence `CSI n m` can be used to print a text in color:

    > print "\e[93mYellow\e[m"
    Yellow

The background color can also be changed by adding 10 to the color code. It is
possible to change both at the same time:

    > print "\e[93;46mYellow on teal\e[m"
    Yellow on teal

The values of the colors can be customized.

After its installation MOROS will run `/ini/palettes/gruvbox-dark.sh` at the
end of each boot to load a Gruvbox color palette by default.

It is possible to change this behavior by editing `/ini/boot.sh`.
