# MOROS Lisp

A minimalist Lisp interpreter is available in MOROS to extend the capabilities
of the Shell.

MOROS Lisp is a Lisp-1 dialect inspired by Scheme and Clojure.

## Changelog

### 0.1.0 (2021-07-21)
MOROS Lisp started from [Risp](https://github.com/stopachka/risp) and was
extended to include the seven primitive operators and the two special forms of
John McCarthy's paper "Recursive Functions of Symbolic Expressions and Their
Computation by Machine" (1960) and "The Roots of Lisp" (2002) by Paul Graham.

### 0.2.0 (2021-12-04)
The whole implementation was refactored and the parser was rewritten to use
[Nom](https://github.com/Geal/nom). This allowed the addition of strings to the
language and reading from the filesystem.

### 0.3.0 (2022-12-12)
Rewrite the evaluation code, add new functions and a core library.

### 0.3.1 (2022-06-06)
Rewrite parts of the code and add new functions and examples.

### 0.3.2 (2022-07-02)
- Add new functions

### 0.3.2 (2022-08-25)
- Add new functions

### 0.4.0 (2022-08-25)
- Rewrite a lot of the code
- Add integer and big integer support
- Add tail call optimization (TCO)
- Add macro support

## Overview

### Types
- Basics: `bool`, `list`, `symbol`, `string`
- Numbers: `float`, `int`, `bigint`

### Built-in Operators
- `quote` (with the `'` syntax)
- `quasiquote` (with the `` ` ``)
- `unquote` (with the `,` syntax)
- `unquote-splice` (with the `,@` syntax)
- `splice` (with the `@` syntax)
- `atom` (aliased to `atom?`)
- `eq` (aliased to `eq?`)
- `head` (aliased to `car`)
- `tail` (aliased to `cdr`)
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

### Primitive Operators
- `append`
- `type`
- `string`
- `string->number`
- `string->bytes` and `bytes->string`
- `number->bytes` and `bytes->number`
- `regex-find`
- `system`

- Arithmetic operations: `+`, `-`, `*`, `/`, `%`, `^`, `abs`
- Trigonometric functions: `acos`, `asin`, `atan`, `cos`, `sin`, `tan`
- Comparisons: `>`, `<`, `>=`, `<=`, `=`
- File IO: `read-file`, `read-file-bytes`, `write-file-bytes`, `append-file-bytes`
- List: `chunks`, `sort`, `unique`, `min`, `max`
- String: `trim`, `split`
- Enumerable: `length`, `nth`, `first`, `second`, `third`, `last`, `rest`, `slice`

### Core Library
- `nil`, `nil?`, `eq?`
- `atom?`, `string?`, `boolean?`, `symbol?`, `number?`, `list?`, `function?`, `macro?`
- `caar`, `cadr`, `cdar`, `cddr`, `first`, `second`, `third`, `rest`
- `map`, `reduce`, `reverse`, `range`, `filter`, `intersection`
- `not`, `and`, `or`
- `let`
- `string-join`, `lines`, `words`, `chars`
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
