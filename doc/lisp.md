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

## Seven Primitive Operators
- `quote` (with the `'` syntax)
- `atom` (aliased to `atom?`)
- `eq` (aliased to `eq?`)
- `car` (aliased to `first`)
- `cdr` (aliased to `rest`)
- `cons`
- `cond`

## Two Special Forms
- `label` (aliased to `define` and `def`)
- `lambda` (aliased to `function`, `fun`, and `fn`)

## Additional Builtins
- `defun` (aliased to `defn`)
- `set`
- `while`
- `apply`
- `type`
- `string`
- `string->number`
- `string->bytes` and `bytes->string`
- `number->bytes` and `bytes->number`
- `regex-find`
- `system`
- `load`

- Arithmetic operations: `+`, `-`, `*`, `/`, `%`, `^`
- Trigonometric functions: `acos`, `asin`, `atan`, `cos`, `sin`, `tan`
- Comparisons: `>`, `<`, `>=`, `<=`, `=`
- Boolean operations: `not`, `and`, `or`
- String operations: `lines`
- File IO: `read-file`, `read-file-bytes`, `write-file-bytes`, `append-file-bytes`

## Core Library
- `nil`, `nil?`, `eq?`
- `atom?`, `string?`, `boolean?`, `symbol?`, `number?`, `list?`, `function?`, `lambda?`
- `first`, `second`, `third`, `rest`
- `map`, `reduce`, `append`, `reverse`
- `string-join`
- `read-line`, `read-char`
- `print`, `println`
- `write-file`, `append-file`
- `uptime`, `realtime`
- `regex-match?`

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

(def (fibonacci n)
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
