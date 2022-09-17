(load "/lib/lisp/core.lsp")

(def (pi-nth n)
  (* (^ 16 (- n)) (-
    (/ 4 (+ 1 (* 8 n)))
    (/ 2 (+ 4 (* 8 n)))
    (/ 1 (+ 5 (* 8 n)))
    (/ 1 (+ 6 (* 8 n))))))

(def (pi-digits n)
  (apply + (map pi-nth (range 0 n))))

(println
  (cond
    ((nil? args) "Usage: pi <precision>")
    (true (pi-digits (string->number (car args))))))
