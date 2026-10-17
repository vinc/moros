# MOROS Calculator

A calculator is available in MOROS using double-precision binary floating-point
format (float64).

It can be invocked from the shell with the command `calc` and an operation
to perform in arguments:

    > calc 2 + 2
    4

It will open a REPL if no arguments are provided:

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
