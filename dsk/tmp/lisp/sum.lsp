(load "/lib/lisp/core.lsp")

(def (sum n acc)
  (if (= n 0) acc (sum (- n 1) (+ n acc))))

(print
  (if (nil? args) "Usage: sum <num>"
    (sum (str->num (head args)) 0)))
