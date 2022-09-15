(load "/lib/lisp/core.lsp")

(define (fibonacci n)
  (cond
    ((< n 2) n)
    (true (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))

(println
  (cond
    ((null? args) "Usage: fibonacci <num>")
    (true (fibonacci (string->number (car args))))))
