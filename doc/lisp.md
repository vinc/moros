# MOROS Lisp

A minimalist Lisp interpreter is available in MOROS to extend the capabilities
of the Shell.

MOROS Lisp is a Lisp-1 dialect inspired by Scheme, Clojure, and Ruby!

## Overview

### Types
- Basics: `bool`, `list`, `symbol`, `string`
- Number: `float`, `int`, `bigint`

### Literals
- String: `"Hello, World!"`
- Number: `2.5`, `-25`, `255`, `0xFF`, `0xDEAD_C0DE`, `0b101010`

### Built-in Operators
- `quote` (abbreviated with `'`)
- `quasiquote` (abbreviated with `` ` ``)
- `unquote` (abbreviated with `,`)
- `unquote-splice` (abbreviated with `,@`)
- `splice` (abbreviated with `@`)
- `atom?`
- `equal?` (aliased to `eq?`)
- `head`
- `tail`
- `cons`
- `if`
- `cond`
- `while`
- `variable` (aliased to `var`)
- `function` (aliased to `fun`)
- `macro` (aliased to `mac`)
- `set`
- `define` (aliased to `def` and equivalent to `define-function`)
- `define-function` (aliased to `def-fun`)
- `define-macro` (aliased to `def-mac`)
- `apply`
- `do`
- `doc`
- `eval`
- `expand`
- `load`

### Primitive Operators
- `type`, `number.type` (aliased to `num.type`), `parse`
- `string` (aliased to `str`)
- `string->number` (aliased to to `str->num`)
- `string->binary` and `binary->string` (aliased to `str->bin` and `bin->str`)
- `number->binary` and `binary->number` (aliased to `num->bin` and `bin->num`)
- `regex.find`
- `shell` (aliased to `sh')
- Arithmetic operations: `+`, `-`, `*`, `/`, `^`, `rem` (aliased to `%`), `trunc`
- Trigonometric functions: `acos`, `asin`, `atan`, `cos`, `sin`, `tan`
- Comparisons: `>`, `<`, `>=`, `<=`, `=`
- Enumerable: `length` (aliased to `len`), `put`, `get`, `slice`, `contains?`
- String: `string.trim` and `string.split` (aliased to `str.trim` and `str.split`)
- List: `list`, `concat`, `chunks`, `sort`, `unique` (aliased to `uniq`)
- Dict: `dict`
- File: `file.size`, `file.open`, `file.close`, `file.read`, `file.write`

### Core Library
- `nil`, `nil?`, `list?`, `empty?`
- `boolean?` (aliased to `bool?`), `string?` (aliased to `str?`), `symbol?` (aliased to `sym?`), `number?` (aliased to `num?`)
- `function?` (aliased to `fun?`), `macro?` (aliased to `mac?`)
- `abs`, `mod`, `min`, `max`
- `first`, `second`, `third`, `last`, `rest`, `push`
- `map`, `reduce`, `reverse` (aliased to `rev`), `range`, `filter`, `reject`, `intersection`
- `not`, `and`, `or`
- `let`
- `string.join` (aliased to `str.join`), `lines`, `words`, `chars`
- `regex.match?`
- `socket.connect`, `socket.listen`, `socket.accept`

### File Library
- `read`, `write`, `append`
- `read-binary`, `write-binary`, `append-binary`
- `read-line`, `read-char`
- `uptime`, `realtime`
- `p`, `print`

### Compatibility Library

- `atom`, `eq`, `label`, `lambda`, `progn`, `begin`
- `car`, `cdr`, `caar`, `cadr`, `cdar`, `cddr`

## Usage

The interpreter can be invoked from the shell:

```
> lisp
MOROS Lisp v0.5.0

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

(print
  (if (nil? args) "Usage: fibonacci <num>"
    (fibonacci (str->num (head args)))))
```

Would produce the following output:

```
> lisp /tmp/lisp/fibonacci.lsp 20
6755
```

## Examples

```lisp
(load "/lib/lisp/core.lsp")

(print "Hello, World!")

(var foo 42)                       # Variable definition
(set foo (+ 40 2))                 # Variable assignement

(var double (fun (x) (* x 2)))     # Function definition
(def (double x) (* x 2))           # Shortcut

(double foo)                       # => 84

(def-mac (++ x)                    # Macro definition
  `(set ,x (+ ,x 1)))

(var i 0)
(while (< i 10)
  (++ i))
(= i 10)                           # => true

(def (map f ls)
  "Apply function to list"
  (if (nil? ls) nil
    (cons
      (f (first ls))
      (map f (rest ls)))))

(doc map)                          # => "Apply function to list"

(var bar (quote (1 2 3)))
(var bar '(1 2 3))                 # Shortcut

(map double bar)                   # => (2 4 6)

(map (fun (x) (+ x 1)) '(4 5 6))   # => (5 6 7)

(var name "Alice")

(str "Hello, " name)               # => "Hello, Alice"

(^ 2 64)                           # => 18446744073709551616
```

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
- Rewrite the evaluation code
- Add new functions
- Add a core library

### 0.3.1 (2022-06-06)
- Rewrite parts of the code
- Add new functions and examples

### 0.3.2 (2022-07-02)
- Add new functions

### 0.4.0 (2022-08-25)
- Rewrite a lot of the code
- Add integer and big integer support
- Add tail call optimization (TCO)
- Add macro support

### 0.5.0 (2023-06-21)
- Rename or add aliases to many functions
- Add full support for line and inline comments
- Add params to function representations
- Add docstring to functions

### 0.6.0 (2023-09-23)
- Add file, number, string, and regex namespaces
- Add socket functions

### Unreleased
- Add binary and hexadecimal number literals
- Rename `nth` to `get`
- Add `empty?`, `reject`, `put`, and `push` functions
- Add `dict` type
