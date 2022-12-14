(load "/lib/lisp/core.lsp")

(define (fibonacci n)
  (if (< n 2) n
    (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(println
  (if (nil? args) "Usage: fibonacci <num>"
    (fibonacci (string->number (head args)))))
