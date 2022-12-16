(load "/lib/lisp/core.lsp")

(define (factorial-helper n acc)
  (if (< n 2) acc
    (factorial-helper (- n 1) (* acc n))))

(define (factorial n)
  (factorial-helper n 1))

(println
  (if (nil? args) "Usage: factorial <num>"
    (factorial (string->number (head args)))))
