(load "/lib/lisp/core.lsp")

(def (tak x y z)
  (if (<= x y) z
    (tak
      (tak (- x 1) y z)
      (tak (- y 1) z x)
      (tak (- z 1) x y))))

(print
  (if (not (= (len args) 3)) "Usage: tak <num> <num> <num>"
    (tak
      (str->num (get args 0))
      (str->num (get args 1))
      (str->num (get args 2)))))
