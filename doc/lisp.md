# MOROS Lisp

A minimalist Lisp interpreter is available in MOROS to extend the capabilities
of the Shell.

MOROS Lisp is a Lisp-1 dialect inspired by Scheme and Clojure.

It started from [Risp](https://github.com/stopachka/risp) and was extended to
include the seven primitive operators and the two special forms of John
McCarthy's paper "Recursive Functions of Symbolic Expressions and Their
Computation by Machine" (1960) and "The Roots of Lisp" (2002) by Paul Graham.

In version 0.2.0 the whole implementation was refactored and the parser was
rewritten to use [Nom](https://github.com/Geal/nom). This allowed the addition
of strings to the language and reading from the filesystem.


## Types
- Basics: `bool`, `list`, `symbol`, `string`
- Numbers: `float`, `int`, `bigint`

## Built-in Operators
- `quote` (with the `'` syntax)
- `quasiquote` (with the `` ` ``)
- `unquote` (with the `,` syntax)
- `unquote-splicing` (with the `,@` syntax)
- `atom` (aliased to `atom?`)
- `eq` (aliased to `eq?`)
- `car` (aliased to `first`)
- `cdr` (aliased to `rest`)
- `cons`
- `if`
- `cond`
- `while`
- `set`
- `define` (aliased to `def` and `label`)
- `function` (aliased to `fun` and `lambda`)
- `macro` (aliased to `mac`)
- `define-function` (aliased to `def-fun`)
- `define-macro` (aliased to `def-mac`)
- `apply`
- `eval`
- `expand`
- `do` (aliased to `begin` and `progn`)
- `load`

## Primitive Operators
- `append`
- `type`
- `string`
- `string->number`
- `string->bytes` and `bytes->string`
- `number->bytes` and `bytes->number`
- `regex-find`
- `system`

- Arithmetic operations: `+`, `-`, `*`, `/`, `%`, `^`
- Trigonometric functions: `acos`, `asin`, `atan`, `cos`, `sin`, `tan`
- Comparisons: `>`, `<`, `>=`, `<=`, `=`
- String operations: `lines`
- File IO: `read-file`, `read-file-bytes`, `write-file-bytes`, `append-file-bytes`

## Core Library
- `nil`, `nil?`, `eq?`
- `atom?`, `string?`, `boolean?`, `symbol?`, `number?`, `list?`, `function?`, `macro?`
- `first`, `second`, `third`, `rest`
- `map`, `reduce`, `reverse`, `range`
- `string-join`
- `read-line`, `read-char`
- `print`, `println`
- `write-file`, `append-file`
- `uptime`, `realtime`
- `regex-match?`

- Boolean operations: `not`, `and`, `or`

## Usage

The interpreter can be invoked from the shell:

```
> lisp
MOROS Lisp v0.4.0

> (+ 1 2 3)
6

> (quit)
```

And it can execute a file. For example a file located in `/tmp/lisp/fibonacci.lsp`
with the following content:

```lisp
(load "/lib/lisp/core.lsp")

(define (fibonacci n)
  (if (< n 2) n
    (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(println
  (if (nil? args) "Usage: fibonacci <num>"
    (fibonacci (string->number (car args)))))
```

Would produce the following output:

```
> lisp /tmp/lisp/fibonacci.lsp 20
6755
```

## Examples

```lisp
(load "/lib/lisp/core.lsp")

(define foo 42)                    # Variable definition

(define double (fun (x) (* x 2)))  # Function definition
(define (double x) (* x 2))        # Shortcut

(double foo)                       # => 84

(define (map f ls)
  (if (nil? ls) nil
    (cons
      (f (first ls))
      (map f (rest ls)))))

(define bar (quote (1 2 3)))
(define bar '(1 2 3))              # Shortcut

(map double bar)                   # => (2 4 6)

(map (fun (x) (+ x 1)) '(4 5 6))   # => (5 6 7)

(set foo 0)                        # Variable assignment

(= foo 10)                         # => false

(while (< foo 10)
  (set foo (+ foo 1)))

(= foo 10)                         # => true

(define name "Alice")

(string "Hello, " name)            # => "Hello, Alice"

(^ 2 128)                          # => 340282366920938463463374607431768211456
```
