# MOROS Lisp

A minimalist Lisp interpreter is available in MOROS to extend the capabilities
of the Shell.

MOROS Lisp is a Lisp-1 dialect inspired by Scheme, Clojure, and Ruby!

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
- `append`
- `type`, `number-type` (aliased to `num-type`)
- `string` (aliased to `str`)
- `string->number` (aliased to to `str->num`)
- `string->binary` and `binary->string` (aliased to `str->bin` and `bin->str`)
- `number->binary` and `binary->number` (aliased to `num->bin` and `bin->num`)
- `regex-find`
- `system`
- Arithmetic operations: `+`, `-`, `*`, `/`, `%`, `^`, `abs`
- Trigonometric functions: `acos`, `asin`, `atan`, `cos`, `sin`, `tan`
- Comparisons: `>`, `<`, `>=`, `<=`, `=`
- File IO: `read-file`, `read-file-binary`, `write-file-binary`, `append-file-binary`
- List: `chunks`, `sort`, `unique` (aliased to `uniq`), `min`, `max`
- String: `trim`, `split`
- Enumerable: `length` (aliased to `len`), `nth`, `first`, `second`, `third`, `last`, `rest`, `slice`

### Core Library
- `nil`, `nil?`, `list?`
- `boolean?` (aliased to `bool?`), `string?` (aliased to `str?`), `symbol?` (aliased to `sym?`), `number?` (aliased to `num?`)
- `function?` (aliased to `fun?`), `macro?` (aliased to `mac?`)
- `first`, `second`, `third`, `rest`
- `map`, `reduce`, `reverse` (aliased to `rev`), `range`, `filter`, `intersection`
- `not`, `and`, `or`
- `let`
- `join-string` (aliased to `join-str`), `lines`, `words`, `chars`
- `read-line`, `read-char`
- `p`, `print`
- `write-file`, `append-file`
- `uptime`, `realtime`
- `regex-match?`

### Compatibility Library

- `atom`, `eq`, `label`, `lambda`, `progn`, `begin`
- `car`, `cdr`, `caar`, `cadr`, `cdar`, `cddr`

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
Rewrite the evaluation code, add new functions and a core library.

### 0.3.1 (2022-06-06)
Rewrite parts of the code and add new functions and examples.

### 0.3.2 (2022-07-02)
- Add new functions

### 0.4.0 (2022-08-25)
- Rewrite a lot of the code
- Add integer and big integer support
- Add tail call optimization (TCO)
- Add macro support

### 0.5.0 (unpublished)
- Rename or add aliases to many functions
- Add full support for line and inline comments
- Add params to function representations
- Add docstring to functions
