(load "/lib/lisp/core.lsp")

(defn pi-nth (n)
  (* (^ 16 (- n)) (-
    (/ 4 (+ 1 (* 8 n)))
    (/ 2 (+ 4 (* 8 n)))
    (/ 1 (+ 5 (* 8 n)))
    (/ 1 (+ 6 (* 8 n))))))

(defn pi-digits (n)
  (apply + (map pi-nth (range 0 n))))

(println
  (cond
    ((null? args) "Usage: pi <precision>")
    (true (pi-digits (parse (car args))))))
