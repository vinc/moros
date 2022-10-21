(load "/lib/lisp/core.lsp")

(def (sum n acc)
  (if (= n 0) acc (sum (- n 1) (+ n acc))))

(println
  (if (nil? args) "Usage: sum <num>"
    (sum (string->number (car args)) 0)))
