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

## Seven Primitive Operators
- `quote` (with the `'` syntax)
- `atom` (aliased to `atom?`)
- `eq` (aliased to `eq?`)
- `car` (aliased to `first`)
- `cdr` (aliased to `rest`)
- `cons`
- `cond`

## Two Special Forms
- `label` (aliased to `def`)
- `lambda` (aliased to `fn`)

## Additional Builtins
- `defun` (aliased to `defn`)
- `mapcar` (aliased to `map`)
- `print`
- `read-file`
- `lines`
- `parse`

## Usage

The interpreter can be invoked from the shell:

```
> lisp
MOROS Lisp v0.1.0

> (+ 1 2)
3

> (exit)
```

And it can execute a file. For example a file located in `/tmp/fibonacci.lsp`
with the following content:

```lisp
(defn fib (n)
  (cond
    ((< n 2) n)
    (true (+ (fib (- n 1)) (fib (- n 2))))))

(println (fib 10))
```

Would produce the following output:

```
> lisp /tmp/fibonacci.lsp
55
```
