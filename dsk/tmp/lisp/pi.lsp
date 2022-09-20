(load "/lib/lisp/core.lsp")

(define (pi-nth n)
  (* (^ 16 (- n)) (-
    (/ 4 (+ 1 (* 8 n)))
    (/ 2 (+ 4 (* 8 n)))
    (/ 1 (+ 5 (* 8 n)))
    (/ 1 (+ 6 (* 8 n))))))

(define (pi-digits n)
  (apply + (map pi-nth (range 0 n))))

(println
  (cond
    ((nil? args) "Usage: pi <precision>")
    (true (pi-digits (string->number (car args))))))
