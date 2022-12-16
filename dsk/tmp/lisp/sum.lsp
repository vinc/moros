(load "/lib/lisp/core.lsp")

(define (sum n acc)
  (if (= n 0) acc (sum (- n 1) (+ n acc))))

(println
  (if (nil? args) "Usage: sum <num>"
    (sum (string->number (head args)) 0)))
