(load "/lib/lisp/core.lsp")

(define (factorial-helper n acc)
  (cond
    ((< n 2) acc)
    (true (factorial-helper (- n 1) (* acc n)))))

(define (factorial n)
  (factorial-helper n 1))

(println
  (cond
    ((null? args) "Usage: factorial <num>")
    (true (factorial (string->number (car args))))))
