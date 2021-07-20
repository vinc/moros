# MOROS Lisp

A minimalist Lisp interpreter is available in MOROS to extend the capabilities
of the Shell.

It started from [Risp][https://github.com/stopachka/risp] and was extended to
include the seven primitive operators and the two special forms of John
McCarthy's paper "Recursive Functions of Symbolic Expressions and Their
Computation by Machine" (1960) and "The Roots of Lisp" (2002) by Paul Graham.

MOROS Lisp dialect is also inspired by Scheme and Clojure.

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

## Additional primitives
- `defun` (aliased to `defn`)
- `print`

## Usage

The interpreter can be invoked from the shell:

```
> lisp
MOROS Lisp v0.1.0

> (+ 1 2)
3

> (exit)
```

And it can execute a file.

For example a file located in `/tmp/fib.ls` with the following content:

```lisp
(label fib
  (lambda (n)
    (cond
      ((< n 2) n)
      (true (+ (fib (- n 1)) (fib (- n 2)))))))

(print (fib 6))
```

Would produce the following output:

```
> lisp /tmp/fib.ls
8
```
