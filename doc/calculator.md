# MOROS Calculator

A calculator is available in MOROS using double-precision binary floating-point
format (binary64).

It can be invocked from the shell through the command `calc` with an operation
to perform in arguments:

    > calc 2 + 2
    4

And it will open a REPL if no arguments are provided:

    > calc
    MOROS Calc v0.1.0

    > 2 + 2
    4

The following arithmetic operations are supported:

  - `+` addition
  - `-` subtraction
  - `*` multiplication
  - `/` division
  - `%` modulo
  - `^` exponential

Parentheses `()` can change the order of operations:

    > 2 + 3 * 4
    14

    > (2 + 3) * 4
    20
